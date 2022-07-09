use active_win_pos_rs::get_active_window;

fn main() {
    match get_active_window() {
        Ok(active_window) => {
            println!("active window: {:#?}", active_window);
        },
        Err(()) => {
            println!("error occurred while getting the active window");
        }
    }
}
