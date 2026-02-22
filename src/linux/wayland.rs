use std::fs::read_link;

use crate::{ActiveWindow, WindowPosition};

fn try_kwin() -> Option<ActiveWindow> {
    // Use kdotool library to get active window info
    let info = kdotool::get_active_window_info().ok()?;

    let process_path = read_link(format!("/proc/{}/exe", info.pid)).unwrap_or_default();

    Some(ActiveWindow {
        title: info.title,
        app_name: info.class_name,
        window_id: info.id,
        process_id: info.pid as u64,
        process_path,
        position: WindowPosition {
            x: info.x,
            y: info.y,
            width: info.width,
            height: info.height,
        },
    })
}

pub fn get_active_window_wayland() -> Option<ActiveWindow> {
    try_kwin()
}
