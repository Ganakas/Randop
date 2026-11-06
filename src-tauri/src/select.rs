use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use std::path::Path;

use windows::Win32::Foundation::HWND;
use windows::Win32::System::Com::{
    CLSCTX_ALL, COINIT_APARTMENTTHREADED, CoCreateInstance, CoInitializeEx, CoTaskMemFree,
    IBindCtx, IServiceProvider,
};
use windows::Win32::System::Variant::{VARIANT, VT_I4};
use windows::Win32::UI::Shell::Common::{ITEMIDLIST, STRRET};
use windows::Win32::UI::Shell::{
    IFolderView, ILClone, ILFindLastID, ILFree, ILRemoveLastID, IPersistFolder2, IShellBrowser,
    IShellFolder, IShellView, IShellWindows, SHGDN_FORPARSING, SHGDN_INFOLDER, SHGDNF,
    SHGetNameFromIDList, SHOpenFolderAndSelectItems, SHParseDisplayName, SID_STopLevelBrowser,
    SIGDN_FILESYSPATH, SVGIO_ALLVIEW, SVSI_DESELECTOTHERS, SVSI_ENSUREVISIBLE, SVSI_FOCUSED,
    SVSI_SELECT, ShellWindows, StrRetToBufW,
};
use windows::Win32::UI::WindowsAndMessaging::{
    GA_ROOT, GetAncestor, GetForegroundWindow, SetForegroundWindow,
};
use windows::core::{Interface, PCWSTR, PWSTR};

use crate::config::same_path;

const SELECT_FLAGS: u32 =
    (SVSI_SELECT.0 | SVSI_DESELECTOTHERS.0 | SVSI_ENSUREVISIBLE.0 | SVSI_FOCUSED.0) as u32;
const ADD_FLAGS: u32 = (SVSI_SELECT.0 | SVSI_ENSUREVISIBLE.0) as u32;

pub fn highlight_many(files: &[std::path::PathBuf]) -> Result<(), String> {
    if files.is_empty() {
        return Ok(());
    }
    match select_in_open_windows(files) {
        Ok(true) => return Ok(()),
        Ok(false) | Err(_) => {}
    }
    reveal(&files[0])
}

fn select_in_open_windows(files: &[std::path::PathBuf]) -> Result<bool, String> {
    let init = unsafe { CoInitializeEx(None, COINIT_APARTMENTTHREADED) };
    if init.is_err() {
        return Err(format!("com: {init}"));
    }

    let shell_windows: IShellWindows =
        unsafe { CoCreateInstance(&ShellWindows, None::<&windows::core::IUnknown>, CLSCTX_ALL) }
            .map_err(|err| err.to_string())?;
    let count = unsafe { shell_windows.Count() }.map_err(|err| err.to_string())?;
    let foreground = unsafe { GetForegroundWindow() };
    let mut selected = false;
    let mut raise: Option<HWND> = None;

    for index in 0..count {
        let Some(view) = shell_view_at(&shell_windows, index) else {
            continue;
        };
        let Some(view_folder) = folder_of_view(&view.folder) else {
            continue;
        };
        let mut indices = Vec::new();
        for file in files {
            let Some(parent) = file.parent() else {
                continue;
            };
            if !same_path(&view_folder, parent) {
                continue;
            }
            if let Some(item) = find_item(&view.view, &view.folder, file) {
                indices.push(item);
            }
        }
        if indices.is_empty() {
            continue;
        }
        let mut picked = false;
        for (nth, item) in indices.iter().enumerate() {
            let flags = if nth == 0 { SELECT_FLAGS } else { ADD_FLAGS };
            if unsafe { view.view.SelectItem(*item, flags) }.is_ok() {
                picked = true;
            }
        }
        if !picked {
            continue;
        }
        selected = true;
        let root = unsafe { GetAncestor(view.hwnd, GA_ROOT) };
        let foreground_root = unsafe { GetAncestor(foreground, GA_ROOT) };
        if root == foreground || root == foreground_root || raise.is_none() {
            raise = Some(if root.0.is_null() { view.hwnd } else { root });
        }
    }

    if selected {
        if let Some(hwnd) = raise {
            unsafe {
                let _ = SetForegroundWindow(hwnd);
            }
        }
    }
    Ok(selected)
}

struct ExplorerView {
    view: IFolderView,
    folder: IShellFolder,
    hwnd: HWND,
}

fn shell_view_at(shell_windows: &IShellWindows, index: i32) -> Option<ExplorerView> {
    let variant = variant_i4(index);
    let dispatch = unsafe { shell_windows.Item(&variant).ok()? };
    let provider: IServiceProvider = dispatch.cast().ok()?;
    let browser: IShellBrowser = unsafe { provider.QueryService(&SID_STopLevelBrowser).ok()? };
    let shell_view: IShellView = unsafe { browser.QueryActiveShellView().ok()? };
    // IShellView starts with IOleWindow. QueryInterface for that base can fail,
    // while the method itself is on the same pointer.
    let hwnd = unsafe { shell_view.GetWindow().ok()? };
    let view: IFolderView = shell_view.cast().ok()?;
    let folder: IShellFolder = unsafe { view.GetFolder().ok()? };
    Some(ExplorerView { view, folder, hwnd })
}

fn folder_of_view(folder: &IShellFolder) -> Option<std::path::PathBuf> {
    let persist: IPersistFolder2 = folder.cast().ok()?;
    let pidl = unsafe { persist.GetCurFolder().ok()? };
    if pidl.is_null() {
        return None;
    }
    let name = unsafe { SHGetNameFromIDList(pidl, SIGDN_FILESYSPATH).ok() };
    unsafe { ILFree(Some(pidl)) };
    let name = name?;
    let text = wide_owned(name);
    Some(std::path::PathBuf::from(text?))
}

fn find_item(view: &IFolderView, folder: &IShellFolder, file: &Path) -> Option<i32> {
    let count = unsafe { view.ItemCount(SVGIO_ALLVIEW).ok()? };
    for index in 0..count {
        let Ok(child) = (unsafe { view.Item(index) }) else {
            continue;
        };
        if child.is_null() {
            continue;
        }
        let name = item_name(folder, child);
        unsafe { ILFree(Some(child)) };
        if name.is_some_and(|name| names_match(&name, file)) {
            return Some(index);
        }
    }
    None
}

fn item_name(folder: &IShellFolder, child: *mut ITEMIDLIST) -> Option<String> {
    let mut value = STRRET::default();
    let flags = SHGDNF(SHGDN_FORPARSING.0 | SHGDN_INFOLDER.0);
    unsafe { folder.GetDisplayNameOf(child, flags, &mut value).ok()? };
    let mut buffer = [0u16; 1024];
    unsafe { StrRetToBufW(&mut value, Some(child), &mut buffer).ok()? };
    let end = buffer
        .iter()
        .position(|unit| *unit == 0)
        .unwrap_or(buffer.len());
    Some(String::from_utf16_lossy(&buffer[..end]))
}

fn names_match(shell_name: &str, file: &Path) -> bool {
    let Some(wanted) = file.file_name() else {
        return false;
    };
    let wanted = wanted.to_string_lossy().to_lowercase();
    let got = shell_name.trim().trim_end_matches(['\\', '/']);
    let got_name = Path::new(got)
        .file_name()
        .map(|name| name.to_string_lossy().to_lowercase())
        .unwrap_or_else(|| got.to_lowercase());
    got_name == wanted || got.to_lowercase() == wanted
}

fn reveal(file: &Path) -> Result<(), String> {
    let init = unsafe { CoInitializeEx(None, COINIT_APARTMENTTHREADED) };
    if init.is_err() {
        return reveal_with_explorer(file);
    }
    if reveal_with_shell(file).is_ok() {
        return Ok(());
    }
    reveal_with_explorer(file)
}

fn reveal_with_shell(file: &Path) -> Result<(), String> {
    let wide = wide_null(file.as_os_str());
    let mut pidl = std::ptr::null_mut();
    unsafe {
        SHParseDisplayName(PCWSTR(wide.as_ptr()), None::<&IBindCtx>, &mut pidl, 0, None)
            .map_err(|err| err.to_string())?;
    }
    if pidl.is_null() {
        return Err("проводник не понял путь".to_owned());
    }
    let child = unsafe { ILClone(ILFindLastID(pidl)) };
    let removed = unsafe { ILRemoveLastID(Some(pidl)) };
    if removed.as_bool() && !child.is_null() {
        let items = [child as *const ITEMIDLIST];
        let opened = unsafe { SHOpenFolderAndSelectItems(pidl, Some(&items), 0) };
        unsafe { ILFree(Some(child)) };
        unsafe { ILFree(Some(pidl)) };
        return opened.map_err(|err| err.to_string());
    }
    if !child.is_null() {
        unsafe { ILFree(Some(child)) };
    }
    unsafe { ILFree(Some(pidl)) };
    Err("не получилось показать файл".to_owned())
}

fn reveal_with_explorer(file: &Path) -> Result<(), String> {
    let mut arg = std::ffi::OsString::from("/select,");
    arg.push(file.as_os_str());
    std::process::Command::new("explorer")
        .arg(arg)
        .spawn()
        .map(|_| ())
        .map_err(|err| format!("не открыть проводник: {err}"))
}

fn variant_i4(value: i32) -> VARIANT {
    let mut variant = VARIANT::default();
    unsafe {
        let head = &mut variant.Anonymous.Anonymous;
        head.vt = VT_I4;
        head.Anonymous.lVal = value;
    }
    variant
}

fn wide_owned(value: PWSTR) -> Option<String> {
    if value.is_null() {
        return None;
    }
    let text = unsafe { value.to_string().ok() };
    unsafe { CoTaskMemFree(Some(value.0.cast())) };
    text
}

fn wide_null(value: &OsStr) -> Vec<u16> {
    value.encode_wide().chain(std::iter::once(0)).collect()
}
