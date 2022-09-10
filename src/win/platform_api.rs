use std::path::{Path, PathBuf};

use windows::core::PWSTR;
use windows::Win32::Foundation::{CloseHandle, HANDLE, MAX_PATH};
use windows::Win32::System::Threading::{
    OpenProcess, QueryFullProcessImageNameW, PROCESS_NAME_WIN32, PROCESS_QUERY_LIMITED_INFORMATION,
};
use windows::Win32::{
    Foundation::{HWND, RECT},
    UI::WindowsAndMessaging::{
        GetForegroundWindow, GetWindowRect, GetWindowTextW, GetWindowThreadProcessId,
    },
};

use crate::{common::platform_api::PlatformApi, ActiveWindow, WindowPosition};

use super::window_position::FromWinRect;

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
        unsafe { GetWindowThreadProcessId(active_window, &mut lpdw_process_id) };
        let process_name = get_window_process_name(lpdw_process_id)?;

        let active_window = ActiveWindow {
            title: active_window_title,
            process_name,
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
            return Ok(rect);
        } else {
            return Err(());
        }
    };
}

fn get_window_title(hwnd: HWND) -> Result<String, ()> {
    let title: String;
    unsafe {
        let mut v: Vec<u16> = vec![0; 255];
        let title_len = GetWindowTextW(hwnd, &mut v);
        if title_len == 0 {
            return Err(());
        }
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

fn get_window_process_name(process_id: u32) -> Result<String, ()> {
    let process_handle = get_process_handle(process_id)?;

    let process_path = get_process_path(process_handle)?;
    let process_file_name = process_path
        .file_stem()
        .unwrap_or(std::ffi::OsStr::new(""))
        .to_str()
        .unwrap_or("");

    close_process_handle(process_handle);

    Ok(process_file_name.into())
}

fn get_process_handle(process_id: u32) -> Result<HANDLE, ()> {
    let handle = unsafe { OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, process_id) };

    Ok(handle.map_err(|_| ())?)
}

fn close_process_handle(process_handle: HANDLE) -> () {
    unsafe { CloseHandle(process_handle) };
}
