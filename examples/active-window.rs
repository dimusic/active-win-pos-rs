use std::time::Duration;

use active_win_pos_rs::{get_active_window, ActiveWindow};

fn main() {
    let mut window_state = ActiveWindow::default();

    loop {
        match get_active_window() {
            Ok(active_window) => {
                if active_window != window_state {
                    println!("active window: {:#?}", active_window);
                    window_state = active_window
                }
            }
            Err(()) => {
                println!("error occurred while getting the active window");
                window_state = ActiveWindow::default();
            }
        };
        std::thread::sleep(Duration::from_millis(500));
    }
}
