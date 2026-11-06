use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Config {
    #[serde(default = "yes")]
    pub menu_pick: bool,
    #[serde(default)]
    pub menu_open: bool,
    #[serde(default)]
    pub menu_multi: bool,
    #[serde(default)]
    pub recursive: bool,
    #[serde(default)]
    pub last_pick: BTreeMap<String, String>,
}

fn yes() -> bool {
    true
}

impl Default for Config {
    fn default() -> Self {
        Self {
            menu_pick: true,
            menu_open: false,
            menu_multi: false,
            recursive: false,
            last_pick: BTreeMap::new(),
        }
    }
}

pub fn config_path() -> PathBuf {
    dirs::data_local_dir()
        .unwrap_or_else(std::env::temp_dir)
        .join("Randop")
        .join("config.json")
}

pub fn load() -> Config {
    let Ok(text) = std::fs::read_to_string(config_path()) else {
        return Config::default();
    };
    serde_json::from_str(&text).unwrap_or_default()
}

pub fn save(config: &Config) -> Result<(), String> {
    let path = config_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|err| format!("не сохранить настройки: {err}"))?;
    }
    let text = serde_json::to_string_pretty(config).map_err(|err| err.to_string())?;
    std::fs::write(path, text).map_err(|err| format!("не сохранить настройки: {err}"))
}

pub fn norm_key(path: &Path) -> String {
    let text = path.to_string_lossy();
    let text = text.trim().trim_end_matches(['\\', '/']);
    let text = text.strip_prefix(r"\\?\").unwrap_or(text);
    let text = text.strip_prefix(r"//?\").unwrap_or(text);
    text.replace('/', "\\").to_lowercase()
}

pub fn same_path(left: &Path, right: &Path) -> bool {
    norm_key(left) == norm_key(right)
}

#[cfg(test)]
mod tests {
    use super::{norm_key, same_path};
    use std::path::Path;

    #[test]
    fn paths_match_despite_slash_and_case() {
        let left = Path::new(r"C:\Music\FL");
        let right = Path::new(r"c:\music\fl\");
        assert!(same_path(left, right));
        assert_eq!(
            norm_key(Path::new(r"\\?\D:\Songs")),
            norm_key(Path::new(r"D:\Songs"))
        );
    }
}
