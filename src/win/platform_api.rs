use crate::{
    common::{
        platform_api::PlatformApi,
        window_position::WindowPosition,
        active_window_error::ActiveWindowError
    },
    ActiveWindow
};
use winapi::shared::windef::HWND__;
use winapi::um::winuser::{GetWindowThreadProcessId};
use winapi::{
    shared::windef::{ RECT },
    um::{winuser::{ GetForegroundWindow, GetWindowRect }}
};
use super::window_position::FromWinRect;

pub struct WindowsPlatformApi {
    
}

impl PlatformApi for WindowsPlatformApi {
    fn get_position(&self) -> Result<WindowPosition, ActiveWindowError> {
        let active_window = get_foreground_window()?;

        if let Ok(win_position) = get_foreground_window_position(active_window) {
            return Ok(WindowPosition::from_win_rect(&win_position));
        }

        Ok(WindowPosition::new(0 as f64, 0 as f64, 0 as f64, 0 as f64))
    }

    fn get_active_window(&self) -> Result<ActiveWindow, ActiveWindowError> {
        let active_window = get_foreground_window()?;
        let active_window_position = self.get_position()?;

        let lpdw_process_id = unsafe {
            let pid_ptr: *mut u32 = std::mem::zeroed();
            GetWindowThreadProcessId(active_window, pid_ptr)
        };

        let active_window = ActiveWindow {
            position: active_window_position,
            process_id: lpdw_process_id as u64,
            window_id: format!("{:?}", active_window),
        };

        Ok(active_window)
    }
}

fn get_foreground_window() -> Result<*mut HWND__, ActiveWindowError> {
    let active_window = unsafe { GetForegroundWindow() };

    if active_window.is_null() {
        return Err(ActiveWindowError::GetActiveWindowFailed);
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
