use std::path::Path;
use winapi::{
    shared::{
        minwindef::MAX_PATH,
        windef::{HWND__, RECT},
    },
    um::{
        handleapi::CloseHandle,
        processthreadsapi::OpenProcess,
        winbase::QueryFullProcessImageNameW,
        winnt::PROCESS_QUERY_LIMITED_INFORMATION,
        winuser::{GetWindowThreadProcessId, GetForegroundWindow, GetWindowRect, GetWindowTextW},
    },
};

use crate::{common::platform_api::PlatformApi, ActiveWindow};
use crate::common::window_position::WindowPosition;
use super::window_position::FromWinRect;

pub struct WindowsPlatformApi {
    
}

impl PlatformApi for WindowsPlatformApi {
    fn get_position(&self) -> Result<WindowPosition, ()> {
        let active_window = get_foreground_window()?;

        if let Ok(win_position) = get_foreground_window_position(active_window) {
            return Ok(WindowPosition::from_win_rect(&win_position));
        }

        Ok(WindowPosition::new(0 as f64, 0 as f64, 0 as f64, 0 as f64))
    }

    fn get_active_window(&self) -> Result<ActiveWindow, ()> {
        let active_window = get_foreground_window()?;
        let win_position = get_foreground_window_position(active_window)?;
        let active_window_position = WindowPosition::from_win_rect(&win_position);
        let active_window_title = get_window_title(active_window)?;
        let mut lpdw_process_id: u32 = 0;
        unsafe {
            GetWindowThreadProcessId(active_window, &mut lpdw_process_id)
        };
        let process_name = get_window_process_name(lpdw_process_id)?;

        let active_window = ActiveWindow {
            title: active_window_title,
            name: process_name,
            position: active_window_position,
            process_id: lpdw_process_id as u64,
            window_id: format!("{:?}", active_window),
        };

        Ok(active_window)
    }
}

fn get_window_title(hwnd: *mut HWND__) -> Result<String, ()> {
    let title: String;
    unsafe {
        let mut v = vec![0; 255];
        let title_len = GetWindowTextW(hwnd, v.as_mut_ptr(), 255) as usize;
        if title_len == 0 {
            return Err(());
        }
        title = String::from_utf16_lossy(&v[0..title_len]);
    };
    Ok(title)
}

fn get_window_process_name(process_id: u32) -> Result<String, ()>{
    let mut lpdw_size: u32 = MAX_PATH.try_into().unwrap();
    let mut process_path = vec![0; MAX_PATH];

    let process_handle: *mut winapi::ctypes::c_void = get_process_handle(process_id);
    let process_name_result = unsafe {
        QueryFullProcessImageNameW(process_handle, 0, process_path.as_mut_ptr(), &mut lpdw_size)
    };
    if process_name_result == 0 {
        return Err(());
    }
    let process_path = String::from_utf16_lossy(&process_path[0..(lpdw_size as usize)]);
    
    let process_name = Path::new(&process_path)
        .file_stem()
        .unwrap_or(std::ffi::OsStr::new(""))
        .to_str()
        .unwrap_or("");

    close_process_handle(process_handle);

    Ok(process_name.into())
}

fn get_process_handle(process_id: u32) -> *mut winapi::ctypes::c_void {
    unsafe {
        OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, process_id)
    }
}

fn close_process_handle(process_handle: *mut winapi::ctypes::c_void) -> () {
    unsafe { CloseHandle(process_handle) };
}

fn get_foreground_window() -> Result<*mut HWND__, ()> {
    let active_window = unsafe { GetForegroundWindow() };

    if active_window.is_null() {
        return Err(());
    }

    Ok(active_window)
}

fn get_foreground_window_position(hwnd: *mut HWND__) -> Result<RECT, ()> {
    unsafe {
        let mut rect: RECT = std::mem::zeroed();

        if GetWindowRect( hwnd, &mut rect as *mut RECT) == 1 {
            return Ok(rect);
        }
        else {
            return Err(());
        }
    };
}
