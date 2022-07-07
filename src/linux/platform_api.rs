use xcb::x;

use crate::{common::platform_api::PlatformApi, WindowPosition, ActiveWindow};
// use xcb::x::GetGeo // {get_geometry, translate_coordinates};
// use xcb_util::ewmh::{get_active_window as xcb_get_active_window, get_wm_pid};

pub struct LinuxPlatformApi {

}

impl PlatformApi for LinuxPlatformApi {
    fn get_position(&self) -> Result<WindowPosition, ()> {
        let active_winow = self.get_active_window()?;
        Ok(active_winow.position)
    }

    fn get_active_window(&self) -> Result<ActiveWindow, ()> {
        let (conn, sceen_num) = xcb::Connection::connect(None)
            .map_err(|_| ())?;
        let setup = conn.get_setup();

        let active_window_id = conn.send_request(&x::InternAtom {
            only_if_exists: true,
            name: b"_NET_ACTIVE_WINDOW",
        });
        let active_window_id = conn.wait_for_reply(active_window_id)
            .map_err(|_| ())?.atom();
        
        assert!(active_window_id != x::ATOM_NONE, "EWMH not supported");

        let root_window = setup.roots().next().unwrap().root();
        let active_window = conn.send_request(&x::GetProperty {
            delete: false,
            window: root_window,
            // property: x::ATOM_WINDOW,
            property: active_window_id,
            // r#type: x::ATOM_CARDINAL,
            r#type: x::ATOM_WINDOW,
            long_offset: 0,
            long_length: 32,
        });
        let active_window = conn.wait_for_reply(active_window).map_err(|_| ())?;

        let window_obj = active_window.value::<x::Window>();
        println!("window_obj getProperty: {:?}", window_obj);
        
        

        let win_geometry = conn.send_request(&x::GetGeometry {
            drawable: x::Drawable::Window(*window_obj.get(0).unwrap()),
        });
        let win_geometry = conn.wait_for_reply(win_geometry)
            .map_err(|_| ())?;

        println!("window_geometry: {:?}", win_geometry);

        

        let window_pid: u64 = 1;
        let window_id: String = String::from("123");
        let position = WindowPosition {
            height: 0.,
            width: 0.,
            x: 0.,
            y: 0.,
        };

        get_xcb_window_position(&conn, 0);

        // let (xcb_connection, default_screen) = xcb::Connection::connect(None)
        //     .map_err(|_| ())?;
        // let xcb_connection = xcb_util::ewmh::Connection::connect(xcb_connection)
        //     .map_err(|(_a, _b)| ())?;
        
        // let xcb_active_window = xcb_get_active_window(&xcb_connection, default_screen)
        //     .get_reply()
        //     .map_err(|_| ())?;
        
        // let window_position = get_xcb_window_position(&xcb_connection, xcb_active_window)
        //     .map_err(|_| ())?;
        
        // let window_pid  = get_wm_pid(&xcb_connection, xcb_active_window)
        //     .get_reply()
        //     .map_err(|_| ())?;
        
        Ok(ActiveWindow {
            process_id: window_pid,
            window_id: window_id,
            position: position
        })

        // Ok(ActiveWindow {
        //     process_id: window_pid as u64,
        //     window_id: xcb_active_window.to_string(),
        //     position: window_position
        // })
    }
}

fn get_xcb_window_position(xcb_connection: &xcb::Connection, xcb_window: u32) {
    println!("test");
}

// fn get_xcb_window_position(xcb_connection: &Connection, xcb_window: u32) -> Result<WindowPosition, Box<dyn std::error::Error>> {
//     let xcb_window_geometry = get_geometry(&xcb_connection, xcb_window)
//         .get_reply()?;

//     let xcb_coordinates = translate_coordinates(
//         &xcb_connection,
//         xcb_window,
//         xcb_window_geometry.root(),
//         xcb_window_geometry.x(),
//         xcb_window_geometry.y()
//     ).get_reply()?;

//     Ok(WindowPosition::new(
//         xcb_coordinates.dst_x() as f64,
//         xcb_coordinates.dst_y() as f64,
//         xcb_window_geometry.width() as f64,
//         xcb_window_geometry.height() as f64
//     ))
// }
