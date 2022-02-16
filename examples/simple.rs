use active_win_pos_rs::get_position;

fn main() {
    match get_position() {
        Ok(window_position) => {
            println!("window position: {:?}", window_position);
        },
        Err(()) => {
            println!("error occurred while getting window position");
        }
    }
}