use super::window_position::WindowPosition;

#[derive(Debug)]
pub struct ActiveWindow {
    pub title: String,
    pub process_name: String,
    pub window_id: String,
    pub process_id: u64,
    pub position: WindowPosition,
}
