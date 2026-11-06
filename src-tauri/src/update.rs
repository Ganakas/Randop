use serde::Deserialize;
use std::ffi::c_void;

use windows::Win32::Networking::WinHttp::{
    WinHttpAddRequestHeaders, WinHttpCloseHandle, WinHttpConnect, WinHttpOpen, WinHttpOpenRequest,
    WinHttpQueryDataAvailable, WinHttpQueryHeaders, WinHttpQueryOption, WinHttpReadData,
    WinHttpReceiveResponse, WinHttpSendRequest, WinHttpSetTimeouts,
    WINHTTP_ACCESS_TYPE_AUTOMATIC_PROXY, WINHTTP_ADDREQ_FLAG_ADD, WINHTTP_FLAG_SECURE,
    WINHTTP_OPTION_URL, WINHTTP_QUERY_FLAG_NUMBER, WINHTTP_QUERY_STATUS_CODE,
};
use windows::core::PCWSTR;

const REPO: &str = "Ganakas/Randop";
const RELEASES: &str = "https://api.github.com/repos/Ganakas/Randop/releases/latest";
const ASSET_NAME: &str = "Randop.exe";
const MAX_BYTES: usize = 40 * 1024 * 1024;

#[derive(Debug, Deserialize)]
struct Release {
    tag_name: String,
    assets: Vec<Asset>,
}

#[derive(Debug, Deserialize)]
struct Asset {
    name: String,
    browser_download_url: String,
}

pub struct Offer {
    pub available: bool,
    pub version: String,
}

pub fn check() -> Result<Offer, String> {
    let release = latest()?;
    let version = release.tag_name.trim().trim_start_matches(['v', 'V']).to_string();
    Ok(Offer {
        available: newer(&version, env!("CARGO_PKG_VERSION")),
        version,
    })
}

pub fn install() -> Result<(), String> {
    let release = latest()?;
    let version = release.tag_name.trim().trim_start_matches(['v', 'V']);
    if !newer(version, env!("CARGO_PKG_VERSION")) {
        return Err("Это уже последняя версия".to_string());
    }
    let asset = release
        .assets
        .iter()
        .find(|asset| asset.name.eq_ignore_ascii_case(ASSET_NAME))
        .ok_or_else(|| format!("в релизе нет файла {ASSET_NAME}"))?;
    expect_download_url(&asset.browser_download_url)?;
    let bytes = http_get(&asset.browser_download_url, false)?;
    replace_and_restart(&bytes)
}

fn latest() -> Result<Release, String> {
    let bytes = http_get(RELEASES, true)?;
    serde_json::from_slice(&bytes).map_err(|_| "GitHub ответил непонятно".to_string())
}

fn expect_download_url(url: &str) -> Result<(), String> {
    let prefix = format!("https://github.com/{REPO}/releases/download/");
    if url.starts_with(&prefix) && !url.contains("..") {
        Ok(())
    } else {
        Err("ссылка на файл обновления выглядит неверно".to_string())
    }
}

pub fn newer(remote: &str, current: &str) -> bool {
    let remote = numbers(remote);
    let current = numbers(current);
    let len = remote.len().max(current.len());
    for index in 0..len {
        let left = remote.get(index).copied().unwrap_or(0);
        let right = current.get(index).copied().unwrap_or(0);
        if left != right {
            return left > right;
        }
    }
    false
}

fn numbers(version: &str) -> Vec<u64> {
    version
        .trim()
        .trim_start_matches(['v', 'V'])
        .split(|ch: char| !ch.is_ascii_digit())
        .filter(|part| !part.is_empty())
        .filter_map(|part| part.parse().ok())
        .collect()
}

fn http_get(url: &str, json: bool) -> Result<Vec<u8>, String> {
    let (host, path) = split_https(url)?;
    let offline = || "Не получилось связаться с GitHub".to_string();
    unsafe {
        let agent = wide("Randop");
        let session = Internet(WinHttpOpen(
            PCWSTR(agent.as_ptr()),
            WINHTTP_ACCESS_TYPE_AUTOMATIC_PROXY,
            PCWSTR::null(),
            PCWSTR::null(),
            0,
        ));
        if session.0.is_null() {
            return Err(offline());
        }
        WinHttpSetTimeouts(session.0, 8_000, 8_000, 20_000, 120_000).map_err(|_| offline())?;
        let host = wide(&host);
        let connection = Internet(WinHttpConnect(session.0, PCWSTR(host.as_ptr()), 443, 0));
        if connection.0.is_null() {
            return Err(offline());
        }
        let verb = wide("GET");
        let object = wide(&path);
        let request = Internet(WinHttpOpenRequest(
            connection.0,
            PCWSTR(verb.as_ptr()),
            PCWSTR(object.as_ptr()),
            PCWSTR::null(),
            PCWSTR::null(),
            std::ptr::null(),
            WINHTTP_FLAG_SECURE,
        ));
        if request.0.is_null() {
            return Err(offline());
        }
        if json {
            let headers: Vec<u16> = "Accept: application/vnd.github+json\r\n"
                .encode_utf16()
                .collect();
            WinHttpAddRequestHeaders(request.0, &headers, WINHTTP_ADDREQ_FLAG_ADD)
                .map_err(|_| offline())?;
        }
        WinHttpSendRequest(request.0, None, None, 0, 0, 0).map_err(|_| offline())?;
        WinHttpReceiveResponse(request.0, std::ptr::null_mut()).map_err(|_| offline())?;
        let status = status_code(request.0)?;
        let landed = current_url(request.0)?;
        if !url_allowed(&landed) {
            return Err("Файл пришёл не с GitHub".to_string());
        }
        let body = read_body(request.0)?;
        if status == 404 {
            return Err("Пока нет опубликованной версии".to_string());
        }
        if !(200..300).contains(&status) {
            return Err("GitHub не отдал обновление".to_string());
        }
        Ok(body)
    }
}

fn wide(text: &str) -> Vec<u16> {
    text.encode_utf16().chain(std::iter::once(0)).collect()
}

fn status_code(request: *mut c_void) -> Result<u32, String> {
    let mut status = 0u32;
    let mut size = std::mem::size_of::<u32>() as u32;
    let mut index = 0u32;
    unsafe {
        WinHttpQueryHeaders(
            request,
            WINHTTP_QUERY_STATUS_CODE | WINHTTP_QUERY_FLAG_NUMBER,
            PCWSTR::null(),
            Some(&mut status as *mut u32 as *mut c_void),
            &mut size,
            &mut index,
        )
        .map_err(|_| "GitHub ответил непонятно".to_string())?;
    }
    Ok(status)
}

fn read_body(request: *mut c_void) -> Result<Vec<u8>, String> {
    let mut body = Vec::new();
    loop {
        let mut available = 0u32;
        unsafe {
            WinHttpQueryDataAvailable(request, &mut available)
                .map_err(|_| "Скачивание прервалось".to_string())?;
        }
        if available == 0 {
            break;
        }
        if body.len() + available as usize > MAX_BYTES {
            return Err("Файл обновления слишком большой".to_string());
        }
        let mut chunk = vec![0u8; available as usize];
        let mut read = 0u32;
        unsafe {
            WinHttpReadData(
                request,
                chunk.as_mut_ptr() as *mut c_void,
                available,
                &mut read,
            )
            .map_err(|_| "Скачивание прервалось".to_string())?;
        }
        chunk.truncate(read as usize);
        body.extend_from_slice(&chunk);
        if read == 0 {
            break;
        }
    }
    Ok(body)
}

fn current_url(request: *mut c_void) -> Result<String, String> {
    let mut buf = vec![0u16; 2048];
    let mut size = (buf.len() * 2) as u32;
    unsafe {
        WinHttpQueryOption(
            request,
            WINHTTP_OPTION_URL,
            Some(buf.as_mut_ptr() as *mut c_void),
            &mut size,
        )
        .map_err(|_| "Не вижу адрес скачивания".to_string())?;
    }
    let chars = (size as usize / 2).min(buf.len());
    let end = buf[..chars].iter().position(|unit| *unit == 0).unwrap_or(chars);
    Ok(String::from_utf16_lossy(&buf[..end]))
}

fn url_allowed(url: &str) -> bool {
    let Ok((host_port, path)) = split_https(url) else {
        return false;
    };
    let host = host_port.split(':').next().unwrap_or("").to_ascii_lowercase();
    let ours = path.to_ascii_lowercase().contains("/ganakas/randop");
    match host.as_str() {
        "api.github.com" | "github.com" => ours,
        "release-assets.githubusercontent.com"
        | "objects.githubusercontent.com"
        | "github-releases.githubusercontent.com" => true,
        _ => false,
    }
}

fn replace_and_restart(bytes: &[u8]) -> Result<(), String> {
    if bytes.len() < 100_000 || !bytes.starts_with(b"MZ") {
        return Err("Скачанный файл не похож на программу".to_string());
    }
    let current = std::env::current_exe().map_err(|_| "Не вижу свой файл".to_string())?;
    let dir = current
        .parent()
        .ok_or_else(|| "Не вижу папку программы".to_string())?;
    let name = current
        .file_name()
        .and_then(|item| item.to_str())
        .unwrap_or(ASSET_NAME);
    let incoming = dir.join(format!("{name}.new"));
    let previous = dir.join(format!("{name}.old"));
    let parked = dir.join(format!("{name}.bad"));
    let marker = dir.join(format!("{name}.updated"));
    std::fs::write(&incoming, bytes).map_err(|_| {
        "Не могу записать обновление рядом с программой. Положите её в свою папку.".to_string()
    })?;
    if previous.exists() && std::fs::remove_file(&previous).is_err() {
        let _ = std::fs::remove_file(&incoming);
        return Err("Прежняя копия ещё занята. Закройте Randop и попробуйте снова.".to_string());
    }
    if let Err(err) = std::fs::rename(&current, &previous) {
        let _ = std::fs::remove_file(&incoming);
        return Err(format!("Не получилось заменить программу: {err}"));
    }
    if let Err(err) = std::fs::rename(&incoming, &current) {
        if std::fs::rename(&previous, &current).is_err() {
            return Err(
                "Не получилось заменить программу. Рабочая копия осталась в файле .old".to_string(),
            );
        }
        let _ = std::fs::remove_file(&incoming);
        return Err(format!("Не получилось заменить программу: {err}"));
    }
    match std::process::Command::new(&current).spawn() {
        Ok(_) => {
            let _ = std::fs::write(&marker, b"ok");
            Ok(())
        }
        Err(err) => {
            let _ = std::fs::remove_file(&parked);
            if std::fs::rename(&current, &parked).is_ok() && std::fs::rename(&previous, &current).is_ok()
            {
                let _ = std::fs::remove_file(&parked);
                Err(format!("Новая версия не запустилась: {err}"))
            } else {
                Err(
                    "Новая версия не запустилась. Если окно не открывается, переименуйте файл .old обратно."
                        .to_string(),
                )
            }
        }
    }
}

pub fn cleanup_previous() {
    let Ok(current) = std::env::current_exe() else {
        return;
    };
    let Some(dir) = current.parent() else {
        return;
    };
    let Some(name) = current.file_name().and_then(|item| item.to_str()) else {
        return;
    };
    let marker = dir.join(format!("{name}.updated"));
    if marker.exists() {
        let _ = std::fs::remove_file(dir.join(format!("{name}.old")));
        let _ = std::fs::remove_file(&marker);
    }
    let _ = std::fs::remove_file(dir.join(format!("{name}.new")));
    let _ = std::fs::remove_file(dir.join(format!("{name}.bad")));
}

fn split_https(url: &str) -> Result<(String, String), String> {
    let rest = url
        .strip_prefix("https://")
        .ok_or_else(|| "Ссылка обновления должна быть https".to_string())?;
    let (host, path) = rest
        .split_once('/')
        .ok_or_else(|| "Ссылка обновления выглядит неверно".to_string())?;
    if host.is_empty() || path.is_empty() || host.contains('@') {
        return Err("Ссылка обновления выглядит неверно".to_string());
    }
    Ok((host.to_string(), format!("/{path}")))
}

struct Internet(*mut c_void);

impl Drop for Internet {
    fn drop(&mut self) {
        unsafe {
            let _ = WinHttpCloseHandle(self.0);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{expect_download_url, newer};

    #[test]
    fn version_order() {
        assert!(newer("0.2.0", "0.1.0"));
        assert!(newer("v0.1.1", "0.1.0"));
        assert!(!newer("0.1.0", "0.1.0"));
        assert!(!newer("v0.1.0", "0.1.9"));
    }

    #[test]
    fn final_url_must_stay_on_github() {
        assert!(super::url_allowed(
            "https://api.github.com/repos/Ganakas/Randop/releases/latest"
        ));
        assert!(super::url_allowed(
            "https://github.com/Ganakas/Randop/releases/download/v0.2.0/Randop.exe"
        ));
        assert!(super::url_allowed(
            "https://release-assets.githubusercontent.com/github-production-release-asset/1/2"
        ));
        assert!(!super::url_allowed("https://example.com/Randop.exe"));
        assert!(!super::url_allowed("https://github.com/other/Randop/releases/download/v1/Randop.exe"));
    }

    #[test]
    fn download_url_stays_on_our_releases() {
        assert!(expect_download_url(
            "https://github.com/Ganakas/Randop/releases/download/v0.2.0/Randop.exe"
        )
        .is_ok());
        assert!(expect_download_url("https://example.com/Randop.exe").is_err());
        assert!(expect_download_url(
            "https://github.com/Ganakas/Randop/releases/download/../secret"
        )
        .is_err());
    }
}
