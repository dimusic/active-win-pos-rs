use super::window_position::WindowPosition;

#[derive(Debug)]
pub struct ActiveWindow {
    pub window_id: String,
    pub position: WindowPosition
}
