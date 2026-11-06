use std::path::Path;

use windows::Win32::UI::Shell::ShellExecuteW;
use windows::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL;
use windows::core::{HSTRING, PCWSTR};

pub fn open_url(url: &str) -> Result<(), String> {
    let target = HSTRING::from(url);
    let operation = HSTRING::from("open");
    let code = unsafe {
        ShellExecuteW(
            None,
            PCWSTR(operation.as_ptr()),
            PCWSTR(target.as_ptr()),
            PCWSTR::null(),
            PCWSTR::null(),
            SW_SHOWNORMAL,
        )
    };
    if (code.0 as isize) <= 32 {
        Err("не получилось открыть ссылку".to_string())
    } else {
        Ok(())
    }
}

pub fn open(path: &Path) -> Result<(), String> {
    let file = HSTRING::from(path);
    let operation = HSTRING::from("open");
    let code = unsafe {
        ShellExecuteW(
            None,
            PCWSTR(operation.as_ptr()),
            PCWSTR(file.as_ptr()),
            PCWSTR::null(),
            PCWSTR::null(),
            SW_SHOWNORMAL,
        )
    };
    if (code.0 as isize) <= 32 {
        Err(format!(
            "не получилось открыть «{}» (код {})",
            file_label(path),
            code.0 as isize
        ))
    } else {
        Ok(())
    }
}

fn file_label(path: &Path) -> String {
    path.file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_else(|| path.display().to_string())
}
