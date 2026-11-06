#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[cfg(not(windows))]
compile_error!("Randop работает только на Windows");

mod config;
#[allow(dead_code)]
mod icon_raster;
mod launch;
mod notify;
mod registry;
mod scan;
mod select;
mod update;

use std::ffi::{OsStr, OsString};
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Settings {
    enabled: bool,
    pick: bool,
    open: bool,
    multi: bool,
    recursive: bool,
}

fn current_settings() -> Settings {
    let config = config::load();
    Settings {
        enabled: registry::menu_state().present,
        pick: config.menu_pick,
        open: config.menu_open,
        multi: config.menu_multi,
        recursive: config.recursive,
    }
}

fn store_settings(state: &Settings) -> Result<(), String> {
    let mut config = config::load();
    config.menu_pick = state.pick;
    config.menu_open = state.open;
    config.menu_multi = state.multi;
    config.recursive = state.recursive;
    config::save(&config)?;
    if state.enabled {
        registry::apply(registry::MenuFlags {
            pick: state.pick,
            open: state.open,
            multi: state.multi,
        })
    } else {
        registry::uninstall()
    }
}

#[tauri::command]
fn settings() -> Settings {
    current_settings()
}

#[tauri::command]
fn save_settings(state: Settings) -> Result<Settings, String> {
    store_settings(&state)?;
    Ok(current_settings())
}

const GITHUB_URL: &str = "https://github.com/Ganakas/Randop";

#[tauri::command]
fn open_github() -> Result<(), String> {
    launch::open_url(GITHUB_URL)
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct UpdateOffer {
    available: bool,
    version: String,
}

#[tauri::command]
fn check_update() -> Result<UpdateOffer, String> {
    let offer = update::check()?;
    Ok(UpdateOffer {
        available: offer.available,
        version: offer.version,
    })
}

#[tauri::command]
fn install_update(app: tauri::AppHandle) -> Result<(), String> {
    update::install()?;
    app.exit(0);
    Ok(())
}

fn main() {
    if let Err(err) = run() {
        notify::message_box(&err);
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    update::cleanup_previous();
    let args: Vec<OsString> = std::env::args_os().skip(1).collect();
    if args.is_empty() {
        tauri::Builder::default()
            .invoke_handler(tauri::generate_handler![
                settings,
                save_settings,
                open_github,
                check_update,
                install_update
            ])
            .run(tauri::generate_context!())
            .map_err(|err| err.to_string())?;
        return Ok(());
    }

    let action = args[0].to_string_lossy();
    match action.as_ref() {
        "highlight" => act(Action::Highlight, args.get(1).map(OsString::as_os_str)),
        "open" => act(Action::Open, args.get(1).map(OsString::as_os_str)),
        "multi" => act(Action::Multi, args.get(1).map(OsString::as_os_str)),
        "--install" => registry::apply(registry::MenuFlags::from_config(&config::load())),
        "--uninstall" => registry::uninstall(),
        other => {
            let path = PathBuf::from(other);
            if path.is_dir() {
                act(Action::Highlight, Some(path.as_os_str()))
            } else {
                Err("не понял команду".to_owned())
            }
        }
    }
}

enum Action {
    Highlight,
    Open,
    Multi,
}

fn act(action: Action, folder: Option<&OsStr>) -> Result<(), String> {
    let folder = resolve_folder(folder)?;
    let files = match action {
        Action::Highlight | Action::Open => vec![scan::choose(&folder)?],
        Action::Multi => scan::choose_many(&folder, 3)?,
    };
    match action {
        Action::Open => launch::open(&files[0])?,
        Action::Highlight | Action::Multi => {
            std::thread::sleep(std::time::Duration::from_millis(160));
            select::highlight_many(&files)?;
        }
    }
    Ok(())
}

fn resolve_folder(folder: Option<&OsStr>) -> Result<PathBuf, String> {
    if let Some(folder) = folder {
        let text = folder.to_string_lossy();
        let text = text.trim().trim_matches('"');
        if !text.is_empty() {
            let path = PathBuf::from(text);
            if path.is_dir() {
                return Ok(path);
            }
            return Err(format!("папка не найдена:\n{}", path.display()));
        }
    }
    let current = std::env::current_dir().map_err(|err| err.to_string())?;
    if current.is_dir() {
        Ok(current)
    } else {
        Err("проводник не передал папку".to_owned())
    }
}
