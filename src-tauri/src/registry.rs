use std::io;
use std::path::{Path, PathBuf};

use winreg::RegKey;
use winreg::enums::HKEY_CURRENT_USER;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MenuState {
    pub present: bool,
    pub current: bool,
}

#[derive(Clone, Copy, Debug)]
pub struct MenuFlags {
    pub pick: bool,
    pub open: bool,
    pub multi: bool,
}

impl MenuFlags {
    pub fn from_config(config: &crate::config::Config) -> Self {
        Self {
            pick: config.menu_pick,
            open: config.menu_open,
            multi: config.menu_multi,
        }
    }
}

const VERBS: [(&str, &str); 3] = [
    ("Randop1", "highlight"),
    ("Randop2", "open"),
    ("Randop3", "multi"),
];

const ROOTS: [&str; 2] = [
    r"Software\Classes\Directory\Background\shell",
    r"Software\Classes\Drive\Background\shell",
];

pub fn menu_state() -> MenuState {
    let Ok(exe) = exe_path() else {
        return MenuState {
            present: false,
            current: false,
        };
    };
    let exe_text = exe.display().to_string();
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let mut present = false;
    let mut current = false;
    for root in ROOTS {
        for name in verb_names() {
            let path = format!(r"{root}\{name}");
            if hkcu.open_subkey(&path).is_err() {
                continue;
            }
            present = true;
            let command = read_command(&hkcu, &path);
            if command.contains(&exe_text) {
                current = true;
            }
        }
    }
    MenuState { present, current }
}

pub fn apply(flags: MenuFlags) -> Result<(), String> {
    uninstall()?;
    if !flags.pick && !flags.open && !flags.multi {
        return Ok(());
    }
    let exe = exe_path().map_err(|err| err.to_string())?;
    let exe_text = exe.display().to_string();
    let enabled = [flags.pick, flags.open, flags.multi];
    for root in ROOTS {
        for (verb, on) in VERBS.iter().zip(enabled) {
            if !on {
                continue;
            }
            let path = format!(r"{root}\{0}", verb.0);
            let label = crate::i18n::verb(crate::i18n::active(), verb.1);
            write_verb(&path, label, &command_quote(&exe, verb.1), &exe_text)
                .map_err(|err| crate::i18n::write_menu(&err.to_string()))?;
        }
    }
    Ok(())
}

pub fn uninstall() -> Result<(), String> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    for root in ROOTS {
        for name in verb_names() {
            let path = format!(r"{root}\{name}");
            match hkcu.delete_subkey_all(path) {
                Ok(()) => {}
                Err(err) if err.kind() == io::ErrorKind::NotFound => {}
                Err(err) => return Err(crate::i18n::remove_menu(&err.to_string())),
            }
        }
    }
    Ok(())
}

fn verb_names() -> [&'static str; 4] {
    ["Randop", "Randop1", "Randop2", "Randop3"]
}

fn read_command(hkcu: &RegKey, verb: &str) -> String {
    let direct = format!(r"{verb}\command");
    if let Ok(key) = hkcu.open_subkey(&direct)
        && let Ok(command) = key.get_value::<String, _>("")
    {
        return command;
    }
    let nested = format!(r"{verb}\shell\1highlight\command");
    if let Ok(key) = hkcu.open_subkey(nested)
        && let Ok(command) = key.get_value::<String, _>("")
    {
        return command;
    }
    String::new()
}

fn write_verb(location: &str, label: &str, command: &str, exe: &str) -> io::Result<()> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let (key, _) = hkcu.create_subkey(location)?;
    key.set_value("MUIVerb", &label)?;
    key.set_value("Icon", &exe)?;
    let (command_key, _) = key.create_subkey("command")?;
    command_key.set_value("", &command)?;
    Ok(())
}

pub fn exe_path() -> io::Result<PathBuf> {
    let path = std::env::current_exe()?;
    Ok(plain_path(path))
}

fn plain_path(path: PathBuf) -> PathBuf {
    let text = path.to_string_lossy();
    if let Some(rest) = text.strip_prefix(r"\\?\UNC\") {
        return PathBuf::from(format!(r"\\{rest}"));
    }
    if let Some(rest) = text.strip_prefix(r"\\?\") {
        return PathBuf::from(rest);
    }
    path
}

pub fn command_quote(exe: &Path, action: &str) -> String {
    format!("\"{}\" {action} \"%V\"", exe.display())
}

#[cfg(test)]
mod tests {
    use super::command_quote;
    use std::path::Path;

    #[test]
    fn command_keeps_spaces_inside_quotes() {
        let command = command_quote(Path::new(r"C:\My Apps\Randop.exe"), "highlight");
        assert_eq!(command, r#""C:\My Apps\Randop.exe" highlight "%V""#);
    }
}
