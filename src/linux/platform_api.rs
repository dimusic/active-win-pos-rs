use crate::{common::platform_api::PlatformApi, WindowPosition, ActiveWindow};
use crate::common::active_window_error::ActiveWindowError;

use xcb::{get_geometry, translate_coordinates};
use xcb_util::ewmh::{Connection, get_active_window as xcb_get_active_window, get_wm_pid};

pub struct LinuxPlatformApi {

}

impl PlatformApi for LinuxPlatformApi {
    fn get_position(&self) -> Result<WindowPosition, ActiveWindowError> {
        let active_winow = self.get_active_window()?;
        Ok(active_winow.position)
    }

    fn get_active_window(&self) -> Result<ActiveWindow, ActiveWindowError> {
        let (xcb_connection, default_screen) = xcb::Connection::connect(None)
            .map_err(|_| ActiveWindowError::GetActiveWindowFailed)?;
        let xcb_connection = xcb_util::ewmh::Connection::connect(xcb_connection)
            .map_err(|(_a, _b)| ActiveWindowError::GetActiveWindowFailed)?;
        
        let xcb_active_window = xcb_get_active_window(&xcb_connection, default_screen)
            .get_reply()
            .map_err(|_| ActiveWindowError::GetActiveWindowFailed)?;
        
        let window_position = get_xcb_window_position(&xcb_connection, xcb_active_window)
            .map_err(|_| ActiveWindowError::GetActiveWindowFailed)?;
        
        let window_pid  = get_wm_pid(&xcb_connection, xcb_active_window)
            .get_reply()
            .map_err(|_| ActiveWindowError::GetActiveWindowFailed)?;
        
        Ok(ActiveWindow {
            process_id: window_pid as u64,
            window_id: xcb_active_window.to_string(),
            position: window_position
        })
    }
}

fn get_xcb_window_position(xcb_connection: &Connection, xcb_window: u32) -> Result<WindowPosition, Box<dyn std::error::Error>> {
    let xcb_window_geometry = get_geometry(&xcb_connection, xcb_window)
        .get_reply()?;

    let xcb_coordinates = translate_coordinates(
        &xcb_connection,
        xcb_window,
        xcb_window_geometry.root(),
        xcb_window_geometry.x(),
        xcb_window_geometry.y()
    ).get_reply()?;

    Ok(WindowPosition::new(
        xcb_coordinates.dst_x() as f64,
        xcb_coordinates.dst_y() as f64,
        xcb_window_geometry.width() as f64,
        xcb_window_geometry.height() as f64
    ))
}
