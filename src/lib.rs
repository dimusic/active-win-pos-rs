#[cfg(target_os = "macos")]
#[macro_use]
extern crate objc;

mod common;
#[cfg(target_os = "macos")]
mod mac;
#[cfg(target_os = "windows")]
mod win;
#[cfg(target_os = "linux")]
mod linux;

#[cfg(target_os = "macos")]
use mac::init_platform_api;
#[cfg(target_os = "windows")]
use win::init_platform_api;
#[cfg(target_os = "linux")]
use linux::init_platform_api;

pub use common::window_position::WindowPosition;
pub use common::active_window::ActiveWindow;
use common::platform_api::PlatformApi;

pub fn get_position() -> Result<WindowPosition, ()> {
    let api = init_platform_api();
    api.get_position()
}

pub fn get_active_window() -> Result<ActiveWindow, ()> {
    let api = init_platform_api();
    api.get_active_window()
}
