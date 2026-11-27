#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[cfg(not(windows))]
compile_error!("Randop only runs on Windows");

mod config;
mod i18n;
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
    lang: String,
}

fn current_settings() -> Settings {
    let config = config::load();
    Settings {
        enabled: registry::menu_state().present,
        pick: config.menu_pick,
        open: config.menu_open,
        multi: config.menu_multi,
        recursive: config.recursive,
        lang: config
            .lang
            .as_deref()
            .and_then(i18n::known)
            .unwrap_or_else(i18n::system)
            .to_string(),
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
fn set_language(lang: String) -> Result<Settings, String> {
    let code = i18n::known(&lang).ok_or_else(|| "Unknown language.".to_string())?;
    let mut config = config::load();
    config.lang = Some(code.to_string());
    config::save(&config)?;
    if registry::menu_state().present {
        registry::apply(registry::MenuFlags::from_config(&config))?;
    }
    Ok(current_settings())
}

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

fn settle_language() {
    let mut config = config::load();
    if let Some(code) = config.lang.as_deref().and_then(i18n::known) {
        if config.lang.as_deref() != Some(code) {
            config.lang = Some(code.to_string());
            let _ = config::save(&config);
        }
        return;
    }
    config.lang = Some(i18n::system().to_string());
    let _ = config::save(&config);
    if registry::menu_state().present {
        let _ = registry::apply(registry::MenuFlags::from_config(&config));
    }
}

fn run() -> Result<(), String> {
    update::cleanup_previous();
    settle_language();
    let args: Vec<OsString> = std::env::args_os().skip(1).collect();
    if args.is_empty() {
        tauri::Builder::default()
            .invoke_handler(tauri::generate_handler![
                settings,
                save_settings,
                set_language,
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
                Err(i18n::bad_command())
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
            return Err(i18n::folder_missing(&path.display().to_string()));
        }
    }
    let current = std::env::current_dir().map_err(|err| err.to_string())?;
    if current.is_dir() {
        Ok(current)
    } else {
        Err(i18n::no_folder())
    }
}
