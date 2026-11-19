//! Everything that talks to Windows: listing windows, closing processes, closing a browser tab,
//! reading app icons, writing the hosts file with elevation, and autostart.
//! Replaces the PowerShell scripts of the Electron version with direct Win32 calls.

#![allow(unsafe_code)]

use std::collections::HashMap;
use std::ffi::OsString;
use std::os::windows::ffi::OsStringExt;
use std::path::PathBuf;

use base64::Engine as _;
use windows::Win32::Foundation::{CloseHandle, FALSE, HWND, LPARAM, TRUE, WAIT_OBJECT_0};
use windows::Win32::Graphics::Dwm::{DWMWA_CLOAKED, DwmGetWindowAttribute};
use windows::Win32::Graphics::Gdi::{
    BI_RGB, BITMAP, BITMAPINFO, BITMAPINFOHEADER, DIB_RGB_COLORS, DeleteObject, GetDC, GetDIBits,
    GetObjectW, HGDIOBJ, ReleaseDC,
};
use windows::Win32::Storage::FileSystem::{
    FILE_ATTRIBUTE_NORMAL, GetFileVersionInfoSizeW, GetFileVersionInfoW, VerQueryValueW,
};
use windows::Win32::System::Diagnostics::ToolHelp::{
    CreateToolhelp32Snapshot, PROCESSENTRY32W, Process32FirstW, Process32NextW, TH32CS_SNAPPROCESS,
};
use windows::Win32::System::Threading::{
    GetExitCodeProcess, OpenProcess, PROCESS_NAME_WIN32, PROCESS_QUERY_LIMITED_INFORMATION,
    PROCESS_TERMINATE, QueryFullProcessImageNameW, TerminateProcess, WaitForSingleObject,
};
use windows::Win32::UI::Input::KeyboardAndMouse::{
    INPUT, INPUT_0, INPUT_KEYBOARD, KEYBD_EVENT_FLAGS, KEYBDINPUT, KEYEVENTF_KEYUP, SendInput,
    VIRTUAL_KEY, VK_CONTROL,
};
use windows::Win32::UI::Shell::{
    SEE_MASK_NOCLOSEPROCESS, SHELLEXECUTEINFOW, SHFILEINFOW, SHGFI_ICON, SHGFI_LARGEICON,
    SHGetFileInfoW, ShellExecuteExW,
};
use windows::Win32::UI::WindowsAndMessaging::{
    DestroyIcon, EnumWindows, GW_OWNER, GetForegroundWindow, GetIconInfo, GetWindow,
    GetWindowTextLengthW, GetWindowTextW, GetWindowThreadProcessId, ICONINFO, IsWindowVisible,
    PostMessageW, SW_HIDE, SetForegroundWindow, WM_CLOSE,
};
use windows::core::{BOOL, PCWSTR, PWSTR, w};

#[derive(Clone, Debug, serde::Serialize)]
pub struct WinInfo {
    pub pid: u32,
    pub name: String,
    pub title: String,
    pub path: String,
}

fn wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

fn from_wide(buf: &[u16]) -> String {
    let end = buf.iter().position(|c| *c == 0).unwrap_or(buf.len());
    OsString::from_wide(&buf[..end])
        .to_string_lossy()
        .into_owned()
}

/// pid → process name without ".exe" (what PowerShell calls `ProcessName`), plus parent pids.
fn process_table() -> (HashMap<u32, String>, HashMap<u32, u32>) {
    let (mut names, mut parents) = (HashMap::new(), HashMap::new());
    unsafe {
        let Ok(snap) = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) else {
            return (names, parents);
        };
        let mut e = PROCESSENTRY32W {
            dwSize: std::mem::size_of::<PROCESSENTRY32W>() as u32,
            ..Default::default()
        };
        if Process32FirstW(snap, &mut e).is_ok() {
            loop {
                let full = from_wide(&e.szExeFile);
                let name = full
                    .strip_suffix(".exe")
                    .or_else(|| full.strip_suffix(".EXE"))
                    .unwrap_or(&full)
                    .to_string();
                names.insert(e.th32ProcessID, name);
                parents.insert(e.th32ProcessID, e.th32ParentProcessID);
                if Process32NextW(snap, &mut e).is_err() {
                    break;
                }
            }
        }
        let _ = CloseHandle(snap);
    }
    (names, parents)
}

fn process_path(pid: u32) -> String {
    unsafe {
        let Ok(h) = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid) else {
            return String::new();
        };
        let mut buf = vec![0u16; 1024];
        let mut len = buf.len() as u32;
        let ok =
            QueryFullProcessImageNameW(h, PROCESS_NAME_WIN32, PWSTR(buf.as_mut_ptr()), &mut len)
                .is_ok();
        let _ = CloseHandle(h);
        if ok {
            String::from_utf16_lossy(&buf[..len as usize])
        } else {
            String::new()
        }
    }
}

fn title_of(hwnd: HWND) -> String {
    unsafe {
        let len = GetWindowTextLengthW(hwnd);
        if len <= 0 {
            return String::new();
        }
        let mut buf = vec![0u16; len as usize + 1];
        let n = GetWindowTextW(hwnd, &mut buf);
        String::from_utf16_lossy(&buf[..n.max(0) as usize])
    }
}

fn pid_of(hwnd: HWND) -> u32 {
    let mut pid = 0u32;
    unsafe { GetWindowThreadProcessId(hwnd, Some(&mut pid)) };
    pid
}

unsafe extern "system" fn collect_main_windows(hwnd: HWND, lparam: LPARAM) -> BOOL {
    unsafe {
        let found = &mut *(lparam.0 as *mut Vec<(u32, String)>);
        if !IsWindowVisible(hwnd).as_bool() {
            return TRUE;
        }
        if GetWindow(hwnd, GW_OWNER).is_ok_and(|h| !h.is_invalid()) {
            return TRUE; // owned windows (dialogs) are not a program's main window
        }
        let mut cloaked = 0u32;
        let _ = DwmGetWindowAttribute(hwnd, DWMWA_CLOAKED, (&mut cloaked as *mut u32).cast(), 4);
        if cloaked != 0 {
            return TRUE; // hidden UWP shells and windows on other desktops
        }
        let title = title_of(hwnd);
        if title.is_empty() {
            return TRUE;
        }
        found.push((pid_of(hwnd), title));
        TRUE
    }
}

/// One entry per process that has a visible main window with a title.
pub fn list_windows() -> Vec<WinInfo> {
    let mut found: Vec<(u32, String)> = Vec::new();
    unsafe {
        let _ = EnumWindows(
            Some(collect_main_windows),
            LPARAM((&mut found as *mut Vec<(u32, String)>) as isize),
        );
    }
    let (names, _) = process_table();
    let mut seen = std::collections::HashSet::new();
    let mut out = Vec::new();
    for (pid, title) in found {
        if pid == 0 || !seen.insert(pid) {
            continue;
        }
        let path = process_path(pid);
        let name = names.get(&pid).cloned().unwrap_or_else(|| {
            PathBuf::from(&path)
                .file_stem()
                .map(|s| s.to_string_lossy().into_owned())
                .unwrap_or_default()
        });
        out.push(WinInfo {
            pid,
            name,
            title,
            path,
        });
    }
    out
}

/// The active window: pid, process name, full title.
pub fn foreground() -> Option<WinInfo> {
    let hwnd = unsafe { GetForegroundWindow() };
    if hwnd.is_invalid() {
        return None;
    }
    let pid = pid_of(hwnd);
    let (names, _) = process_table();
    let path = process_path(pid);
    let name = names.get(&pid).cloned().unwrap_or_default();
    Some(WinInfo {
        pid,
        name,
        title: title_of(hwnd),
        path,
    })
}

unsafe extern "system" fn post_close(hwnd: HWND, lparam: LPARAM) -> BOOL {
    unsafe {
        if pid_of(hwnd) == lparam.0 as u32 {
            let _ = PostMessageW(Some(hwnd), WM_CLOSE, Default::default(), Default::default());
        }
        TRUE
    }
}

/// Ask the program to close (WM_CLOSE to its windows), or end it and its children right away.
pub fn close_process(pid: u32, force: bool) -> bool {
    if pid == 0 || pid == std::process::id() {
        return false;
    }
    if !force {
        unsafe {
            let _ = EnumWindows(Some(post_close), LPARAM(pid as isize));
        }
        return true;
    }
    let (_, parents) = process_table();
    let mut order = vec![pid];
    let mut i = 0;
    while i < order.len() {
        let cur = order[i];
        let kids: Vec<u32> = parents
            .iter()
            .filter(|(c, p)| **p == cur && !order.contains(c))
            .map(|(c, _)| *c)
            .collect();
        order.extend(kids);
        i += 1;
    }
    let mut ok = false;
    for p in order.into_iter().rev() {
        if p == std::process::id() {
            continue;
        }
        unsafe {
            if let Ok(h) = OpenProcess(PROCESS_TERMINATE, false, p) {
                ok |= TerminateProcess(h, 1).is_ok();
                let _ = CloseHandle(h);
            }
        }
    }
    ok
}

/// Close the tab being viewed with Ctrl+W, but only if the active window is still that browser with the
/// very same title (so a different tab is never closed by mistake).
pub fn close_active_tab(pid: u32, title: &str) -> &'static str {
    let Some(fg) = foreground() else {
        return "skipped";
    };
    if fg.pid != pid || fg.title != title {
        return "skipped";
    }
    let key = |vk: u16, up: bool| INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 {
            ki: KEYBDINPUT {
                wVk: VIRTUAL_KEY(vk),
                wScan: 0,
                dwFlags: if up {
                    KEYEVENTF_KEYUP
                } else {
                    KEYBD_EVENT_FLAGS(0)
                },
                time: 0,
                dwExtraInfo: 0,
            },
        },
    };
    let ctrl = VK_CONTROL.0;
    let inputs = [
        key(ctrl, false),
        key(b'W' as u16, false),
        key(b'W' as u16, true),
        key(ctrl, true),
    ];
    let sent = unsafe { SendInput(&inputs, std::mem::size_of::<INPUT>() as i32) };
    if sent == inputs.len() as u32 {
        "closed"
    } else {
        "error"
    }
}

// ───────────────────────── app list: readable name and real icon ─────────────────────────

fn file_description(path: &str) -> String {
    unsafe {
        let p = wide(path);
        let size = GetFileVersionInfoSizeW(PCWSTR(p.as_ptr()), None);
        if size == 0 {
            return String::new();
        }
        let mut buf = vec![0u8; size as usize];
        if GetFileVersionInfoW(PCWSTR(p.as_ptr()), None, size, buf.as_mut_ptr().cast()).is_err() {
            return String::new();
        }
        let mut ptr = std::ptr::null_mut();
        let mut len = 0u32;
        let mut lang_cp: Option<(u16, u16)> = None;
        if VerQueryValueW(
            buf.as_ptr().cast(),
            w!("\\VarFileInfo\\Translation"),
            &mut ptr,
            &mut len,
        )
        .as_bool()
            && len >= 4
        {
            let pair = ptr as *const u16;
            lang_cp = Some((*pair, *pair.add(1)));
        }
        let (lang, cp) = lang_cp.unwrap_or((0x0409, 0x04B0));
        let sub = wide(&format!(
            "\\StringFileInfo\\{lang:04x}{cp:04x}\\FileDescription"
        ));
        if VerQueryValueW(
            buf.as_ptr().cast(),
            PCWSTR(sub.as_ptr()),
            &mut ptr,
            &mut len,
        )
        .as_bool()
            && len > 0
        {
            let s = std::slice::from_raw_parts(ptr as *const u16, len as usize);
            return from_wide(s).trim().to_string();
        }
        String::new()
    }
}

fn encode_png(rgba: &[u8], width: u32, height: u32) -> Option<Vec<u8>> {
    let mut bytes = Vec::new();
    {
        let mut enc = png::Encoder::new(&mut bytes, width, height);
        enc.set_color(png::ColorType::Rgba);
        enc.set_depth(png::BitDepth::Eight);
        let mut writer = enc.write_header().ok()?;
        writer.write_image_data(rgba).ok()?;
    }
    Some(bytes)
}

fn icon_png_base64(path: &str) -> String {
    unsafe {
        let p = wide(path);
        let mut sfi = SHFILEINFOW::default();
        let r = SHGetFileInfoW(
            PCWSTR(p.as_ptr()),
            FILE_ATTRIBUTE_NORMAL,
            Some(&mut sfi),
            std::mem::size_of::<SHFILEINFOW>() as u32,
            SHGFI_ICON | SHGFI_LARGEICON,
        );
        if r == 0 || sfi.hIcon.is_invalid() {
            return String::new();
        }
        let mut out = String::new();
        let mut info = ICONINFO::default();
        if GetIconInfo(sfi.hIcon, &mut info).is_ok() {
            if !info.hbmColor.is_invalid() {
                let mut bm = BITMAP::default();
                GetObjectW(
                    HGDIOBJ(info.hbmColor.0),
                    std::mem::size_of::<BITMAP>() as i32,
                    Some((&mut bm as *mut BITMAP).cast()),
                );
                let (width, height) = (bm.bmWidth.max(1), bm.bmHeight.max(1));
                let mut bi = BITMAPINFO {
                    bmiHeader: BITMAPINFOHEADER {
                        biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                        biWidth: width,
                        biHeight: -height, // top-down
                        biPlanes: 1,
                        biBitCount: 32,
                        biCompression: BI_RGB.0,
                        ..Default::default()
                    },
                    ..Default::default()
                };
                let mut px = vec![0u8; (width * height * 4) as usize];
                let dc = GetDC(None);
                let lines = GetDIBits(
                    dc,
                    info.hbmColor,
                    0,
                    height as u32,
                    Some(px.as_mut_ptr().cast()),
                    &mut bi,
                    DIB_RGB_COLORS,
                );
                ReleaseDC(None, dc);
                if lines != 0 {
                    let no_alpha = px.as_chunks::<4>().0.iter().all(|c| c[3] == 0);
                    for c in px.as_chunks_mut::<4>().0 {
                        c.swap(0, 2); // BGRA → RGBA
                        if no_alpha {
                            c[3] = 255;
                        }
                    }
                    if let Some(bytes) = encode_png(&px, width as u32, height as u32) {
                        out = base64::engine::general_purpose::STANDARD.encode(bytes);
                    }
                }
                let _ = DeleteObject(HGDIOBJ(info.hbmColor.0));
            }
            if !info.hbmMask.is_invalid() {
                let _ = DeleteObject(HGDIOBJ(info.hbmMask.0));
            }
        }
        let _ = DestroyIcon(sfi.hIcon);
        out
    }
}

struct FocusCtx {
    pid: u32,
    found: Option<HWND>,
}

unsafe extern "system" fn find_main_window(hwnd: HWND, lparam: LPARAM) -> BOOL {
    unsafe {
        let ctx = &mut *(lparam.0 as *mut FocusCtx);
        let owned = GetWindow(hwnd, GW_OWNER).is_ok_and(|h| !h.is_invalid());
        if pid_of(hwnd) == ctx.pid
            && IsWindowVisible(hwnd).as_bool()
            && !owned
            && !title_of(hwnd).is_empty()
        {
            ctx.found = Some(hwnd);
            return FALSE;
        }
        TRUE
    }
}

/// Bring a program's main window to the front (allowed right after the user clicked one of our windows).
pub fn focus_process(pid: u32) -> bool {
    let mut ctx = FocusCtx { pid, found: None };
    unsafe {
        let _ = EnumWindows(
            Some(find_main_window),
            LPARAM((&mut ctx as *mut FocusCtx) as isize),
        );
        ctx.found.is_some_and(|h| SetForegroundWindow(h).as_bool())
    }
}

/// A readable name for a program: its file description ("Microsoft Edge"), else its capitalised process name.
pub fn display_name(path: &str, name: &str) -> String {
    let desc = if path.is_empty() {
        String::new()
    } else {
        file_description(path)
    };
    if !desc.is_empty() {
        return desc;
    }
    let mut c = name.chars();
    c.next().map_or_else(String::new, |f| {
        f.to_uppercase().collect::<String>() + c.as_str()
    })
}

#[derive(Clone, Debug)]
pub struct RichApp {
    pub info: WinInfo,
    pub desc: String,
    pub icon: String,
}

/// Open apps (one per program name) with their readable name and icon.
pub fn list_apps_rich() -> Vec<RichApp> {
    let mut seen = std::collections::HashSet::new();
    list_windows()
        .into_iter()
        .filter(|w| seen.insert(w.name.to_lowercase()))
        .map(|w| {
            let (desc, icon) = if w.path.is_empty() {
                (String::new(), String::new())
            } else {
                (file_description(&w.path), icon_png_base64(&w.path))
            };
            RichApp {
                info: w,
                desc,
                icon,
            }
        })
        .collect()
}

// ───────────────────────── hosts file ─────────────────────────

pub fn hosts_path() -> PathBuf {
    let root =
        std::env::var_os("SystemRoot").map_or_else(|| PathBuf::from(r"C:\Windows"), PathBuf::from);
    root.join(r"System32\drivers\etc\hosts")
}

pub fn read_hosts() -> String {
    std::fs::read(hosts_path())
        .map(|b| String::from_utf8_lossy(&b).into_owned())
        .unwrap_or_default()
}

/// Write the hosts file through a hidden elevated `cmd` (Windows asks the user for permission).
pub fn write_hosts_elevated(content: &str) -> Result<(), String> {
    let tmp = std::env::temp_dir().join(format!(
        "disturbless-hosts-{}-{}.txt",
        std::process::id(),
        now_ms()
    ));
    std::fs::write(&tmp, content).map_err(|e| e.to_string())?;
    let params = format!(
        "/c copy /y \"{}\" \"{}\" >nul && ipconfig /flushdns >nul",
        tmp.display(),
        hosts_path().display()
    );
    let result = run_elevated("cmd.exe", &params, 120_000);
    let _ = std::fs::remove_file(&tmp);
    result
}

fn now_ms() -> u128 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_millis())
}

fn run_elevated(exe: &str, params: &str, timeout_ms: u32) -> Result<(), String> {
    unsafe {
        let (verb, file, args) = (wide("runas"), wide(exe), wide(params));
        let mut info = SHELLEXECUTEINFOW {
            cbSize: std::mem::size_of::<SHELLEXECUTEINFOW>() as u32,
            fMask: SEE_MASK_NOCLOSEPROCESS,
            lpVerb: PCWSTR(verb.as_ptr()),
            lpFile: PCWSTR(file.as_ptr()),
            lpParameters: PCWSTR(args.as_ptr()),
            nShow: SW_HIDE.0,
            ..Default::default()
        };
        ShellExecuteExW(&mut info).map_err(|_| "uac-declined-or-failed".to_string())?;
        if info.hProcess.is_invalid() {
            return Err("uac-declined-or-failed".into());
        }
        let waited = WaitForSingleObject(info.hProcess, timeout_ms);
        let mut code = 1u32;
        let _ = GetExitCodeProcess(info.hProcess, &mut code);
        let _ = CloseHandle(info.hProcess);
        if waited == WAIT_OBJECT_0 && code == 0 {
            Ok(())
        } else {
            Err("uac-declined-or-failed".into())
        }
    }
}

// ───────────────────────── autostart ─────────────────────────

const RUN_KEY: &str = r"Software\Microsoft\Windows\CurrentVersion\Run";

/// Add or remove this app from the user's startup list (current user only, no admin needed).
pub fn set_autostart(on: bool) -> Result<(), String> {
    use winreg::RegKey;
    use winreg::enums::{HKEY_CURRENT_USER, KEY_SET_VALUE};
    let run = RegKey::predef(HKEY_CURRENT_USER)
        .open_subkey_with_flags(RUN_KEY, KEY_SET_VALUE)
        .map_err(|e| e.to_string())?;
    if on {
        let exe = std::env::current_exe().map_err(|e| e.to_string())?;
        run.set_value("Disturbless", &format!("\"{}\" --hidden", exe.display()))
            .map_err(|e| e.to_string())
    } else {
        match run.delete_value("Disturbless") {
            Ok(()) => Ok(()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(e) => Err(e.to_string()),
        }
    }
}
