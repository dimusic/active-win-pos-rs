use std::fs::read_link;

use xcb::{x, Xid};

use crate::{common::platform_api::PlatformApi, ActiveWindow, WindowPosition};

fn get_xcb_window_pid(conn: &xcb::Connection, window: x::Window) -> xcb::Result<u32> {
    let window_pid = conn.send_request(&x::InternAtom {
        only_if_exists: true,
        name: b"_NET_WM_PID",
    });
    let window_pid = conn.wait_for_reply(window_pid)?.atom();

    let window_pid = conn.send_request(&x::GetProperty {
        delete: false,
        window,
        property: window_pid,
        r#type: x::ATOM_ANY,
        long_offset: 0,
        long_length: 1,
    });
    let window_pid = conn.wait_for_reply(window_pid)?;

    Ok(window_pid.value::<u32>().first().unwrap_or(&0).to_owned())
}

fn get_xcb_window_title(conn: &xcb::Connection, window: x::Window) -> xcb::Result<String> {
    let window_title = conn.send_request(&x::GetProperty {
        delete: false,
        window,
        property: x::ATOM_WM_NAME,
        r#type: x::ATOM_ANY,
        long_offset: 0,
        long_length: 1024,
    });
    let window_title = conn.wait_for_reply(window_title)?;
    let window_title = window_title.value();
    let window_title = std::str::from_utf8(window_title);
    Ok(window_title.unwrap_or("").to_owned())
}

fn get_xcb_window_class(conn: &xcb::Connection, window: x::Window) -> xcb::Result<String> {
    let window_class = conn.send_request(&x::GetProperty {
        delete: false,
        window,
        property: x::ATOM_WM_CLASS,
        r#type: x::ATOM_STRING,
        long_offset: 0,
        long_length: 1024,
    });
    let window_class = conn.wait_for_reply(window_class)?;
    let window_class = window_class.value();
    let window_class = std::str::from_utf8(window_class);
    Ok(window_class.unwrap_or("").to_owned())
}

fn get_xcb_active_window_atom(conn: &xcb::Connection) -> xcb::Result<x::Atom> {
    let active_window_id = conn.send_request(&x::InternAtom {
        only_if_exists: true,
        name: b"_NET_ACTIVE_WINDOW",
    });

    Ok(conn.wait_for_reply(active_window_id)?.atom())
}

fn get_xcb_translated_position(
    conn: &xcb::Connection,
    active_window: x::Window,
) -> xcb::Result<WindowPosition> {
    let window_geometry = conn.send_request(&x::GetGeometry {
        drawable: x::Drawable::Window(active_window),
    });
    let window_geometry = conn.wait_for_reply(window_geometry)?;
    let window_geometry_x = window_geometry.x();
    let window_geometry_y = window_geometry.y();

    let translated_position = conn.send_request(&x::TranslateCoordinates {
        dst_window: window_geometry.root(),
        src_window: active_window,
        src_x: window_geometry_x,
        src_y: window_geometry_y,
    });
    let translated_position = conn.wait_for_reply(translated_position)?;

    Ok(WindowPosition {
        x: (translated_position.dst_x() - window_geometry_x)
            .try_into()
            .unwrap(),
        y: (translated_position.dst_y() - window_geometry_y)
            .try_into()
            .unwrap(),
        width: window_geometry.width().try_into().unwrap(),
        height: window_geometry.height().try_into().unwrap(),
    })
}

pub struct LinuxPlatformApi {}

impl PlatformApi for LinuxPlatformApi {
    fn get_position(&self) -> Result<WindowPosition, ()> {
        let active_winow = self.get_active_window()?;
        Ok(active_winow.position)
    }

    fn get_active_window(&self) -> Result<ActiveWindow, ()> {
        let (conn, _) = xcb::Connection::connect(None).map_err(|_| ())?;
        let setup = conn.get_setup();

        let xcb_active_window_atom = get_xcb_active_window_atom(&conn).map_err(|_| ())?;
        if xcb_active_window_atom == x::ATOM_NONE {
            // EWMH not supported
            return Err(());
        }

        let root_window = setup.roots().next();
        if root_window.is_none() {
            return Err(());
        }
        let root_window = root_window.unwrap().root();

        let active_window = conn.send_request(&x::GetProperty {
            delete: false,
            window: root_window,
            property: xcb_active_window_atom,
            r#type: x::ATOM_WINDOW,
            long_offset: 0,
            long_length: 1,
        });
        let active_window = conn.wait_for_reply(active_window).map_err(|_| ())?;
        let active_window = active_window.value::<x::Window>().get(0);
        if active_window.is_none() {
            return Err(());
        }
        let active_window = active_window.unwrap();

        let window_pid: u32 = get_xcb_window_pid(&conn, *active_window).map_err(|_| ())?;
        let position = get_xcb_translated_position(&conn, *active_window).map_err(|_| ())?;
        let title = get_xcb_window_title(&conn, *active_window).map_err(|_| ())?;
        let window_class = get_xcb_window_class(&conn, *active_window).map_err(|_| ())?;

        let mut process_name = window_class
            .split('\u{0}')
            .filter(|str| !str.is_empty())
            .collect::<Vec<&str>>();
        let process_name = process_name.pop().unwrap_or("").to_owned();

        let process_path = read_link(format!("/proc/{}/exe", window_pid));

        Ok(ActiveWindow {
            process_id: window_pid.try_into().unwrap(),
            window_id: active_window.resource_id().to_string(),
            app_name: process_name,
            position,
            title,
            process_path: process_path.unwrap_or_default(),
        })
    }
}
