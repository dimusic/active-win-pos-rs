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

    // Get window geometry (position and size)
    let geometry_output = Command::new("kdotool")
        .args(["getwindowgeometry", &window_id])
        .output()
        .ok()?;
    
    let mut position = WindowPosition::default();
    
    if geometry_output.status.success() {
        let geometry_str = String::from_utf8(geometry_output.stdout).ok()?;
        
        // Parse output format:
        //   Position: x,y
        //   Geometry: widthxheight
        for line in geometry_str.lines() {
            let line = line.trim();
            if line.starts_with("Position:") {
                if let Some(pos_str) = line.strip_prefix("Position:").map(|s| s.trim()) {
                    let coords: Vec<&str> = pos_str.split(',').collect();
                    if coords.len() == 2 {
                        if let (Ok(x), Ok(y)) = (coords[0].parse::<f64>(), coords[1].parse::<f64>()) {
                            position.x = x;
                            position.y = y;
                        }
                    }
                }
            } else if line.starts_with("Geometry:") {
                if let Some(geom_str) = line.strip_prefix("Geometry:").map(|s| s.trim()) {
                    let dims: Vec<&str> = geom_str.split('x').collect();
                    if dims.len() == 2 {
                        if let (Ok(width), Ok(height)) = (dims[0].parse::<f64>(), dims[1].parse::<f64>()) {
                            position.width = width;
                            position.height = height;
                        }
                    }
                }
            }
        }
    }

    Some(ActiveWindow {
        title,
        app_name,
        window_id,
        process_id: pid as u64,
        process_path,
        position,
    })
}

pub fn get_active_window_wayland() -> Option<ActiveWindow> {
    try_kwin()
}
