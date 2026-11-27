use std::path::{Path, PathBuf};

use crate::config::{self, Config, same_path};

const FILE_ATTRIBUTE_HIDDEN: u32 = 0x2;
const FILE_ATTRIBUTE_SYSTEM: u32 = 0x4;
const FILE_ATTRIBUTE_REPARSE_POINT: u32 = 0x400;
const MAX_ITEMS: usize = 20_000;
const MAX_DEPTH: usize = 8;

pub fn choose(folder: &Path) -> Result<PathBuf, String> {
    let mut config = config::load();
    let items = collect(folder, &config)?;
    if items.is_empty() {
        return Err(empty_message());
    }
    let key = config::norm_key(folder);
    let previous = config.last_pick.get(&key).map(PathBuf::from);
    let picked = pick(&items, previous.as_deref()).ok_or_else(empty_message)?;
    remember(&mut config, key, &picked);
    let _ = config::save(&config);
    Ok(picked)
}

pub fn choose_many(folder: &Path, count: usize) -> Result<Vec<PathBuf>, String> {
    let mut config = config::load();
    let mut items = collect(folder, &config)?;
    if items.is_empty() {
        return Err(empty_message());
    }
    let count = count.clamp(1, items.len());
    let mut picked = Vec::with_capacity(count);
    for _ in 0..count {
        let index = rand::random_range(0..items.len());
        picked.push(items.swap_remove(index));
    }
    let key = config::norm_key(folder);
    remember(&mut config, key, &picked[0]);
    let _ = config::save(&config);
    Ok(picked)
}

pub fn collect(root: &Path, config: &Config) -> Result<Vec<PathBuf>, String> {
    if !root.is_dir() {
        return Err(crate::i18n::folder_missing(&root.display().to_string()));
    }
    let mut items = Vec::new();
    walk(root, config, 0, &mut items)?;
    Ok(items)
}

fn remember(config: &mut Config, key: String, picked: &Path) {
    if config.last_pick.len() > 80 {
        config.last_pick.clear();
    }
    config
        .last_pick
        .insert(key, picked.to_string_lossy().into_owned());
}

fn empty_message() -> String {
    crate::i18n::empty()
}

pub fn pick(items: &[PathBuf], previous: Option<&Path>) -> Option<PathBuf> {
    if items.is_empty() {
        return None;
    }
    let pool: Vec<&PathBuf> = if items.len() > 1 {
        let filtered: Vec<&PathBuf> = items
            .iter()
            .filter(|item| previous.is_none_or(|previous| !same_path(item, previous)))
            .collect();
        if filtered.is_empty() {
            items.iter().collect()
        } else {
            filtered
        }
    } else {
        items.iter().collect()
    };
    let index = rand::random_range(0..pool.len());
    Some(pool[index].clone())
}

fn walk(dir: &Path, config: &Config, depth: usize, out: &mut Vec<PathBuf>) -> Result<(), String> {
    if out.len() >= MAX_ITEMS {
        return Ok(());
    }
    let entries = match std::fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(err) if depth == 0 => return Err(crate::i18n::open_folder(&err.to_string())),
        Err(_) => return Ok(()),
    };

    for entry in entries.flatten() {
        if out.len() >= MAX_ITEMS {
            break;
        }
        let path = entry.path();
        let Ok(meta) = entry.metadata() else {
            continue;
        };
        let attributes = std::os::windows::fs::MetadataExt::file_attributes(&meta);
        if attributes & (FILE_ATTRIBUTE_HIDDEN | FILE_ATTRIBUTE_SYSTEM) != 0 {
            continue;
        }
        if is_junk_name(&path) {
            continue;
        }

        if meta.is_dir() {
            let can_descend = config.recursive
                && depth < MAX_DEPTH
                && attributes & FILE_ATTRIBUTE_REPARSE_POINT == 0;
            if can_descend {
                walk(&path, config, depth + 1, out)?;
            }
        } else {
            out.push(path);
        }
    }
    Ok(())
}

fn is_junk_name(path: &Path) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| name.starts_with("~$"))
}

#[cfg(test)]
mod tests {
    use super::{collect, pick};
    use crate::config::Config;
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn pick_skips_the_previous_file_when_it_can() {
        let items = vec![PathBuf::from("a.flp"), PathBuf::from("b.flp")];
        let previous = PathBuf::from("a.flp");
        let picked = pick(&items, Some(&previous)).unwrap();
        assert_eq!(picked, PathBuf::from("b.flp"));
    }

    #[test]
    fn collect_reads_files_and_skips_hidden() {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis();
        let dir = std::env::temp_dir().join(format!("randop-scan-{stamp}"));
        let nested = dir.join("inner");
        fs::create_dir_all(&nested).unwrap();
        fs::write(dir.join("visible.flp"), b"a").unwrap();
        fs::write(dir.join("note.txt"), b"b").unwrap();
        fs::write(dir.join("secret.flp"), b"c").unwrap();
        fs::write(nested.join("deep.flp"), b"d").unwrap();
        let _ = std::process::Command::new("attrib")
            .args(["+h", "secret.flp"])
            .current_dir(&dir)
            .status();

        let mut config = Config::default();
        let top = collect(&dir, &config).unwrap();
        assert!(top.iter().any(|path| path.ends_with("visible.flp")));
        assert!(top.iter().any(|path| path.ends_with("note.txt")));
        assert!(!top.iter().any(|path| path.ends_with("secret.flp")));
        assert!(!top.iter().any(|path| path.ends_with("deep.flp")));

        config.recursive = true;
        let deep = collect(&dir, &config).unwrap();
        assert!(deep.iter().any(|path| path.ends_with("deep.flp")));

        let _ = std::process::Command::new("attrib")
            .args(["-h", "secret.flp"])
            .current_dir(&dir)
            .status();
        let _ = fs::remove_dir_all(&dir);
    }
}
