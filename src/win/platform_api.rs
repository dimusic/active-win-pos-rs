use crate::{common::platform_api::PlatformApi, ActiveWindow};
use crate::common::window_position::WindowPosition;
use winapi::shared::windef::HWND__;
use winapi::um::winuser::{GetWindowThreadProcessId};
use winapi::{
    shared::windef::{ RECT },
    um::winuser::{GetForegroundWindow, GetWindowRect, GetWindowTextW},
};
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
        let active_window_position = self.get_position()?;
        let active_window_title = get_window_title(active_window)?;
        let lpdw_process_id = unsafe {
            let pid_ptr: *mut u32 = std::mem::zeroed();
            GetWindowThreadProcessId(active_window, pid_ptr)
        };

        let active_window = ActiveWindow {
            title: active_window_title,
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
        let mut v: [u16; 255] = std::mem::zeroed();
        let title_len = GetWindowTextW(hwnd, v.as_mut_ptr(), 255) as usize;
        if title_len == 0 {
            return Err(());
        }
        title = String::from_utf16_lossy(&v[0..title_len]);
    };
    Ok(title)
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
