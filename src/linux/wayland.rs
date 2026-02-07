use std::fs::read_link;
use std::process::Command;

use serde::Deserialize;

use crate::{ActiveWindow, WindowPosition};

#[derive(Debug, Deserialize)]
struct SwayNode {
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    app_id: Option<String>,
    #[serde(default)]
    pid: Option<u32>,
    #[serde(default)]
    id: Option<u64>,
    #[serde(default)]
    focused: bool,
    #[serde(default)]
    rect: Option<SwayRect>,
    #[serde(default)]
    nodes: Vec<SwayNode>,
    #[serde(default)]
    floating_nodes: Vec<SwayNode>,
}

#[derive(Debug, Deserialize)]
struct SwayRect {
    x: i32,
    y: i32,
    width: i32,
    height: i32,
}

fn find_focused_sway_node(node: &SwayNode) -> Option<&SwayNode> {
    if node.focused {
        return Some(node);
    }

    for child in &node.nodes {
        if let Some(focused) = find_focused_sway_node(child) {
            return Some(focused);
        }
    }

    for floating in &node.floating_nodes {
        if let Some(focused) = find_focused_sway_node(floating) {
            return Some(focused);
        }
    }

    None
}

fn try_sway() -> Option<ActiveWindow> {
    let output = Command::new("swaymsg")
        .args(["-t", "get_tree"])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let tree: SwayNode = serde_json::from_slice(&output.stdout).ok()?;
    let focused = find_focused_sway_node(&tree)?;

    let pid = focused.pid?;
    let title = focused.name.clone().unwrap_or_default();
    let app_name = focused.app_id.clone().unwrap_or_default();
    let window_id = focused.id.map(|id| id.to_string()).unwrap_or_default();
    let rect = focused.rect.as_ref()?;

    let process_path = read_link(format!("/proc/{}/exe", pid)).unwrap_or_default();

    Some(ActiveWindow {
        title,
        app_name,
        window_id,
        process_id: pid as u64,
        process_path,
        position: WindowPosition {
            x: rect.x as f64,
            y: rect.y as f64,
            width: rect.width as f64,
            height: rect.height as f64,
        },
    })
}

#[derive(Debug, Deserialize)]
struct HyprlandWindow {
    #[serde(default)]
    title: String,
    #[serde(default)]
    class: String,
    #[serde(default)]
    pid: i32,
    #[serde(default)]
    address: String,
    #[serde(default)]
    at: Vec<i32>,
    #[serde(default)]
    size: Vec<i32>,
}

fn try_hyprland() -> Option<ActiveWindow> {
    let output = Command::new("hyprctl")
        .args(["activewindow", "-j"])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let window: HyprlandWindow = serde_json::from_slice(&output.stdout).ok()?;

    let pid = window.pid;
    if pid <= 0 {
        return None;
    }

    let process_path = read_link(format!("/proc/{}/exe", pid)).unwrap_or_default();

    // Validate that we have position data
    if window.at.len() < 2 || window.size.len() < 2 {
        return None;
    }

    Some(ActiveWindow {
        title: window.title,
        app_name: window.class,
        window_id: window.address,
        process_id: pid as u64,
        process_path,
        position: WindowPosition {
            x: window.at[0] as f64,
            y: window.at[1] as f64,
            width: window.size[0] as f64,
            height: window.size[1] as f64,
        },
    })
}

fn try_kwin() -> Option<ActiveWindow> {
    // Get the active window ID from KWin
    let output = Command::new("qdbus")
        .args([
            "org.kde.KWin",
            "/KWin",
            "org.kde.KWin.activeWindow",
        ])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let window_id = String::from_utf8(output.stdout).ok()?.trim().to_string();
    if window_id.is_empty() {
        return None;
    }

    // Get window info using dbus-send
    let output = Command::new("dbus-send")
        .args([
            "--session",
            "--print-reply",
            "--dest=org.kde.KWin",
            &format!("/{}", window_id.replace('_', "/")),
            "org.freedesktop.DBus.Properties.GetAll",
            "string:org.kde.KWin.Window",
        ])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    // Parse dbus-send output (this is a simplified parser)
    let output_str = String::from_utf8(output.stdout).ok()?;
    
    // Try to extract basic info from dbus output
    // This is a best-effort approach as dbus-send output is complex
    // Note: We look for "caption" or "\"title\"" for the window title.
    // If multiple matches exist, the first one is used. Similarly for resourceClass/resourceName.
    let mut title = String::new();
    let mut app_name = String::new();
    let mut pid = 0u32;
    let mut title_found = false;
    let mut app_found = false;
    let mut pid_found = false;
    
    for line in output_str.lines() {
        let line = line.trim();
        if !title_found && (line.contains("caption") || line.contains("\"title\"")) {
            if let Some(value) = line.split('"').nth(1) {
                if !value.is_empty() {
                    title = value.to_string();
                    title_found = true;
                }
            }
        }
        if !app_found && (line.contains("resourceClass") || line.contains("resourceName")) {
            if let Some(value) = line.split('"').nth(1) {
                if !value.is_empty() {
                    app_name = value.to_string();
                    app_found = true;
                }
            }
        }
        if !pid_found && line.contains("\"pid\"") {
            // Look for the pid value in the next line or same line
            if let Some(num_part) = line.split_whitespace().last() {
                if let Ok(parsed_pid) = num_part.parse::<u32>() {
                    pid = parsed_pid;
                    pid_found = true;
                }
            }
        }
    }

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

#[derive(Debug, Deserialize)]
struct GnomeWindow {
    #[serde(default)]
    wm_class: String,
    #[serde(default)]
    pid: i32,
    #[serde(default)]
    title: String,
    #[serde(default)]
    id: u64,
    #[serde(default)]
    x: i32,
    #[serde(default)]
    y: i32,
    #[serde(default)]
    width: i32,
    #[serde(default)]
    height: i32,
}

fn try_gnome() -> Option<ActiveWindow> {
    let output = Command::new("gdbus")
        .args([
            "call",
            "--session",
            "--dest",
            "org.gnome.Shell",
            "--object-path",
            "/org/gnome/Shell/Extensions/Windows",
            "--method",
            "org.gnome.Shell.Extensions.Windows.List",
        ])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let output_str = String::from_utf8(output.stdout).ok()?;
    
    // Strip the gdbus variant tuple wrapper
    // Handles both formats: (<'[{...}]'>,) and ('[{...}]',)
    let json_str = output_str
        .trim()
        .strip_prefix("(<'")
        .or_else(|| output_str.trim().strip_prefix("('"))
        .and_then(|s| s.strip_suffix("'>,)").or_else(|| s.strip_suffix("',)")))
        .unwrap_or(output_str.trim())
        .replace("\\'", "'");

    let windows: Vec<GnomeWindow> = serde_json::from_str(&json_str).ok()?;
    
    // Note: We assume the first window is the focused one. The GNOME Shell Extensions API
    // typically returns windows in focus order, with the active window first, but this is
    // not guaranteed by the API specification. This is a best-effort approach for GNOME.
    let window = windows.first()?;

    let pid = window.pid;
    if pid <= 0 {
        return None;
    }

    let process_path = read_link(format!("/proc/{}/exe", pid)).unwrap_or_default();

    Some(ActiveWindow {
        title: window.title.clone(),
        app_name: window.wm_class.clone(),
        window_id: window.id.to_string(),
        process_id: pid as u64,
        process_path,
        position: WindowPosition {
            x: window.x as f64,
            y: window.y as f64,
            width: window.width as f64,
            height: window.height as f64,
        },
    })
}

pub fn get_active_window_wayland() -> Option<ActiveWindow> {
    try_sway()
        .or_else(try_hyprland)
        .or_else(try_kwin)
        .or_else(try_gnome)
}
