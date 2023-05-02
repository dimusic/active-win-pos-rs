use std::mem::size_of_val;
use std::os::raw::c_void;
use std::path::{Path, PathBuf};

use windows::core::{HSTRING, PCWSTR, PWSTR};
use windows::Win32::UI::WindowsAndMessaging::GUITHREADINFO;

use windows::w;
use windows::Win32::Foundation::{CloseHandle, HANDLE, MAX_PATH};
use windows::Win32::Storage::FileSystem::{
    GetFileVersionInfoSizeW, GetFileVersionInfoW, VerQueryValueW,
};
use windows::Win32::System::Threading::{
    OpenProcess, QueryFullProcessImageNameW, PROCESS_NAME_WIN32, PROCESS_QUERY_LIMITED_INFORMATION,
};
use windows::Win32::{
    Foundation::{HWND, RECT},
    UI::WindowsAndMessaging::{
        GetForegroundWindow, GetGUIThreadInfo, GetWindowRect, GetWindowTextW,
        GetWindowThreadProcessId,
    },
};

use crate::{common::platform_api::PlatformApi, ActiveWindow, WindowPosition};

use super::window_position::FromWinRect;

#[derive(Debug)]
struct LangCodePage {
    pub w_language: u16,
    pub w_code_page: u16,
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
        let active_window = get_new_hwid();
        // let active_window = get_foreground_window();

        let win_position = get_foreground_window_position(active_window)?;
        let active_window_position = WindowPosition::from_win_rect(&win_position);
        let active_window_title = get_window_title(active_window)?;
        let mut lpdw_process_id: u32 = 0;
        unsafe { GetWindowThreadProcessId(active_window, Some(&mut lpdw_process_id)) };
        let process_path = get_window_path_name(lpdw_process_id).unwrap_or(String::default());
        let app_name = get_window_process_name(lpdw_process_id)?;

        let active_window = ActiveWindow {
            title: active_window_title,
            process_path,
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

fn get_process_path(process_handle: HANDLE) -> Result<PathBuf, ()> {
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

        if !success.as_bool() {
            return Err(());
        }

        process_path_pwstr.to_string().map_err(|_| ())?
    };

    Ok(Path::new(&process_path).to_path_buf())
}

fn get_window_path_name(process_id: u32) -> Result<String, ()> {
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

        if !success.as_bool() {
            return Err(());
        }

        process_path_pwstr.to_string().map_err(|_| ())?
    };
    close_process_handle(process_handle);

    Ok(Path::new(&process_path)
        .to_path_buf()
        .to_str()
        .unwrap()
        .to_string())
}

fn get_new_hwid() -> HWND {
    let null_ptr: *const c_void = std::ptr::null();

    let mut gui_thread = GUITHREADINFO::default();
    gui_thread.cbSize = size_of_val(&gui_thread) as u32;

    unsafe { GetGUIThreadInfo(0, &mut gui_thread) };

    let mut focus_hwnd = gui_thread.hwndFocus;

    if focus_hwnd.0 == null_ptr as isize {
        println!("supposed error, matching number");
        focus_hwnd = gui_thread.hwndActive;
    }

    focus_hwnd
}

fn get_window_process_name(process_id: u32) -> Result<String, ()> {
    let process_handle = get_process_handle(process_id)?;

    let process_path = get_process_path(process_handle)?;

    close_process_handle(process_handle);

    if let Ok(file_description) = get_file_description(&process_path) {
        return Ok(file_description);
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

    let info_size = unsafe { GetFileVersionInfoSizeW(&process_path_hstring, Some(std::ptr::null_mut())) };

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
    let hstring = HSTRING::from(lang_code);
    let mut lang_code_ptr = unsafe{*hstring.as_ptr()}; // Get the pointer to the HSTRING data
    let lang_code = PCWSTR(&mut lang_code_ptr);

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
