use crate::common::platform_api::PlatformApi;
use crate::common::window_position::WindowPosition;
use winapi::{
    shared::windef::{ RECT },
    um::{winuser::{ GetForegroundWindow, GetWindowRect }}
};
use std::{io::Error};
use super::window_position::FromWinRect;

pub struct WindowsPlatformApi {
    
}

impl PlatformApi for WindowsPlatformApi {
    fn get_position(&self) -> Result<WindowPosition, ()> {
        if let Ok(win_position) = get_foreground_window_position() {
            return Ok(WindowPosition::from_win_rect(&win_position));
        }

        Ok(WindowPosition::new(0 as f64, 0 as f64, 0 as f64, 0 as f64))
    }
}

fn get_foreground_window_position() -> Result<RECT, Error> {
    let active_window = unsafe {
        GetForegroundWindow()
    };

    if active_window.is_null() {
        return Err(Error::last_os_error());
    }
    else {
        unsafe {
            let mut rect: RECT = std::mem::zeroed();

            if GetWindowRect( active_window, &mut rect as *mut RECT) == 1 {
                return Ok(rect);
            }
            else {
                return Err(Error::last_os_error());
            }
        };
    }
}
