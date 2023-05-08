use std::path::{Path, PathBuf};

use windows::core::{HSTRING, PCWSTR, PWSTR};
use windows::w;
use windows::Win32::Foundation::{CloseHandle, BOOL, FALSE, HANDLE, MAX_PATH, TRUE};
use windows::Win32::Storage::FileSystem::{
    GetFileVersionInfoSizeW, GetFileVersionInfoW, VerQueryValueW,
};
use windows::Win32::System::Threading::{
    OpenProcess, QueryFullProcessImageNameW, PROCESS_NAME_WIN32, PROCESS_QUERY_LIMITED_INFORMATION,
};
use windows::Win32::UI::WindowsAndMessaging::EnumChildWindows;
use windows::Win32::{
    Foundation::{HWND, LPARAM, RECT},
    UI::WindowsAndMessaging::{
        GetForegroundWindow, GetWindowRect, GetWindowTextW, GetWindowThreadProcessId,
    },
};

use crate::{common::platform_api::PlatformApi, ActiveWindow, WindowPosition};

use super::window_position::FromWinRect;

#[derive(Debug)]
struct LangCodePage {
    pub w_language: u16,
    pub w_code_page: u16,
}

#[derive(Debug, Default)]
struct UwpAppWindowInfo {
    pub name: String,
    pub path: PathBuf,
    pub parent_path: PathBuf,
}
impl UwpAppWindowInfo {
    pub fn new(name: String, path: PathBuf, parent_path: PathBuf) -> Self {
        Self {
            name,
            path,
            parent_path,
        }
    }
}

unsafe extern "system" fn enumerate_uwp_app_windows_callback(hwnd: HWND, lparam: LPARAM) -> BOOL {
    let uwp_app_window_info: &mut UwpAppWindowInfo = std::mem::transmute(lparam);

    let mut child_process_id: u32 = 0;
    unsafe { GetWindowThreadProcessId(hwnd, Some(&mut child_process_id as *mut u32)) };
    let child_process_path = get_process_path(child_process_id);

    if child_process_path.is_err() {
        return FALSE;
    }

    if &uwp_app_window_info.parent_path != child_process_path.as_ref().unwrap() {
        let path = child_process_path.unwrap();
        uwp_app_window_info.path = path.clone();
        uwp_app_window_info.name = get_process_name(&path).unwrap_or(String::default());

        return FALSE;
    }

    TRUE
}

pub struct WindowsPlatformApi {}

impl PlatformApi for WindowsPlatformApi {
    fn get_position(&self) -> Result<WindowPosition, ()> {
        let active_window = get_foreground_window();

        if let Ok(win_position) = get_foreground_window_position(active_window) {
            return Ok(WindowPosition::from_win_rect(&win_position));
        }

        Ok(WindowPosition::new(0 as f64, 0 as f64, 0 as f64, 0 as f64))
    }

    fn get_active_window(&self) -> Result<crate::ActiveWindow, ()> {
        let active_window = get_foreground_window();

        let win_position = get_foreground_window_position(active_window)?;
        let active_window_position = WindowPosition::from_win_rect(&win_position);
        let active_window_title = get_window_title(active_window)?;
        let mut lpdw_process_id: u32 = 0;
        unsafe { GetWindowThreadProcessId(active_window, Some(&mut lpdw_process_id as *mut u32)) };

        let mut process_path = get_process_path(lpdw_process_id)?;
        let file_name = process_path.file_name();
        let mut app_name = String::default();

        if file_name.is_some() && file_name.unwrap() == "ApplicationFrameHost.exe" {
            unsafe {
                let mut uwp_window_info = UwpAppWindowInfo::new(
                    String::default(),
                    PathBuf::default(),
                    process_path.clone(),
                );

                EnumChildWindows(
                    active_window,
                    Some(enumerate_uwp_app_windows_callback),
                    LPARAM(&mut uwp_window_info as *mut UwpAppWindowInfo as isize),
                );

                process_path = uwp_window_info.path;
                app_name = uwp_window_info.name;
            };
        } else {
            app_name = get_process_name(&process_path)?;
        }

        let active_window = ActiveWindow {
            title: active_window_title,
            process_path: process_path
                .into_os_string()
                .into_string()
                .unwrap_or(String::default()),
            app_name,
            position: active_window_position,
            process_id: lpdw_process_id as u64,
            window_id: format!("{:?}", active_window),
        };

        Ok(active_window)
    }
}

fn get_foreground_window() -> HWND {
    unsafe { GetForegroundWindow() }
}

fn get_foreground_window_position(hwnd: HWND) -> Result<RECT, ()> {
    unsafe {
        let mut rect: RECT = std::mem::zeroed();

        if GetWindowRect(hwnd, &mut rect).as_bool() {
            Ok(rect)
        } else {
            Err(())
        }
    }
}

fn get_window_title(hwnd: HWND) -> Result<String, ()> {
    let title: String;
    unsafe {
        let mut v: Vec<u16> = vec![0; 255];
        let title_len = GetWindowTextW(hwnd, &mut v);
        title = String::from_utf16_lossy(&v[0..(title_len as usize)]);
    };

    Ok(title)
}

fn get_process_path(process_id: u32) -> Result<PathBuf, ()> {
    let process_handle = get_process_handle(process_id)?;
    let mut lpdw_size: u32 = MAX_PATH;
    let mut process_path_raw = vec![0; MAX_PATH as usize];
    let process_path_pwstr = PWSTR::from_raw(process_path_raw.as_mut_ptr());

    let process_path = unsafe {
        let success = QueryFullProcessImageNameW(
            process_handle,
            PROCESS_NAME_WIN32,
            process_path_pwstr,
            &mut lpdw_size,
        );

        close_process_handle(process_handle);

        if !success.as_bool() {
            return Err(());
        }

        process_path_pwstr.to_string().map_err(|_| ())?
    };

    Ok(Path::new(&process_path).to_path_buf())
}

fn get_process_name(process_path: &Path) -> Result<String, ()> {
    let file_description = get_file_description(process_path);
    if file_description.is_ok() && !file_description.as_ref().unwrap().is_empty() {
        return file_description;
    }

    let process_file_name = process_path
        .file_stem()
        .unwrap_or(std::ffi::OsStr::new(""))
        .to_str()
        .unwrap_or("")
        .to_owned();

    Ok(process_file_name)
}

fn get_file_description(process_path: &Path) -> Result<String, ()> {
    let process_path_hstring: HSTRING = process_path.as_os_str().into();

    let info_size = unsafe { GetFileVersionInfoSizeW(&process_path_hstring, None) };

    if info_size == 0 {
        return Err(());
    }

    let mut file_version_info = vec![0u8; info_size.try_into().unwrap()];

    let file_info_query_success = unsafe {
        GetFileVersionInfoW(
            &process_path_hstring,
            0,
            info_size,
            file_version_info.as_mut_ptr().cast(),
        )
    };
    if !file_info_query_success.as_bool() {
        return Err(());
    }

    let mut lang_ptr = std::ptr::null_mut();
    let mut len = 0;
    let lang_query_success = unsafe {
        VerQueryValueW(
            file_version_info.as_ptr().cast(),
            w!("\\VarFileInfo\\Translation"),
            &mut lang_ptr,
            &mut len,
        )
    };
    if !lang_query_success.as_bool() {
        return Err(());
    }

    let lang: &[LangCodePage] =
        unsafe { std::slice::from_raw_parts(lang_ptr as *const LangCodePage, 1) };

    if lang.is_empty() {
        return Err(());
    }

    let mut query_len: u32 = 0;

    let lang = lang.get(0).unwrap();
    let lang_code = format!(
        "\\StringFileInfo\\{:04x}{:04x}\\FileDescription",
        lang.w_language, lang.w_code_page
    );
    let lang_code = PCWSTR(HSTRING::from(&lang_code).as_wide().as_ptr());

    let mut file_description_ptr = std::ptr::null_mut();

    let file_description_query_success = unsafe {
        VerQueryValueW(
            file_version_info.as_ptr().cast(),
            lang_code,
            &mut file_description_ptr,
            &mut query_len,
        )
    };

    if !file_description_query_success.as_bool() {
        return Err(());
    }

    let file_description =
        unsafe { std::slice::from_raw_parts(file_description_ptr.cast(), query_len as usize) };
    let file_description = String::from_utf16_lossy(file_description);
    let file_description = file_description.trim_matches(char::from(0)).to_owned();

    Ok(file_description)
}

fn get_process_handle(process_id: u32) -> Result<HANDLE, ()> {
    let handle = unsafe { OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, process_id) };

    handle.map_err(|_| ())
}

fn close_process_handle(process_handle: HANDLE) {
    unsafe { CloseHandle(process_handle) };
}
