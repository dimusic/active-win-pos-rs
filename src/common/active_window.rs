use super::window_position::WindowPosition;

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ActiveWindow {
    pub title: String,
    pub process_path: String,
    pub app_name: String,
    pub window_id: String,
    pub process_id: u64,
    pub position: WindowPosition,
}
