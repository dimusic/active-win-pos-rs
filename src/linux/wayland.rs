use std::fs::read_link;
use std::process::Command;

use crate::{ActiveWindow, WindowPosition};

fn try_kwin() -> Option<ActiveWindow> {

    // Check if kdotool is installed
    if Command::new("kdotool").arg("--version").output().is_err() {
        return None;
    }

    // Get the active window ID from KWin using kdotool
    let output = Command::new("kdotool")
        .arg("getactivewindow")
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let window_id = String::from_utf8(output.stdout).ok()?.trim().to_string();
    if window_id.is_empty() {
        return None;
    }

    // Get window title
    let title_output = Command::new("kdotool")
        .args(["getwindowname", &window_id])
        .output()
        .ok()?;
    let title = if title_output.status.success() {
        String::from_utf8(title_output.stdout)
            .ok()?
            .trim()
            .to_string()
    } else {
        String::new()
    };

    // Get window class
    let class_output = Command::new("kdotool")
        .args(["getwindowclassname", &window_id])
        .output()
        .ok()?;
    let app_name = if class_output.status.success() {
        String::from_utf8(class_output.stdout)
            .ok()?
            .trim()
            .to_string()
    } else {
        String::new()
    };

    // Get window PID
    let pid_output = Command::new("kdotool")
        .args(["getwindowpid", &window_id])
        .output()
        .ok()?;
    
    if !pid_output.status.success() {
        return None;
    }

    let pid_str = String::from_utf8(pid_output.stdout).ok()?;
    let pid = pid_str.trim().parse::<u32>().ok()?;

    if pid == 0 {
        return None;
    }

    let process_path = read_link(format!("/proc/{}/exe", pid)).unwrap_or_default();

    Some(ActiveWindow {
        title,
        app_name,
        window_id,
        process_id: pid as u64,
        process_path,
        position: WindowPosition::default(),
    })
}

pub fn get_active_window_wayland() -> Option<ActiveWindow> {
    try_kwin()
    //try_sway()
        //.or_else(try_hyprland)
        //.or_else(try_kwin)
        //.or_else(try_gnome)
}
