use std::env;
use std::fs::read_link;

use hyprland::data::Client;
use hyprland::prelude::HyprDataActiveOptional;

use crate::{ActiveWindow, WindowPosition};

fn try_hyprland() -> Option<ActiveWindow> {
    env::var_os("HYPRLAND_INSTANCE_SIGNATURE")?;

    let info = Client::get_active().ok()??;
    let process_id = info.pid.try_into().ok()?;
    let process_path = read_link(format!("/proc/{}/exe", info.pid)).unwrap_or_default();

    Some(ActiveWindow {
        title: info.title,
        app_name: info.class,
        window_id: info.address.to_string(),
        process_id,
        process_path,
        position: WindowPosition {
            x: f64::from(info.at.0),
            y: f64::from(info.at.1),
            width: f64::from(info.size.0),
            height: f64::from(info.size.1),
        },
    })
}

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
    try_kwin().or_else(try_hyprland)
}
