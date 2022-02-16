use super::window_position::WindowPosition;

pub trait PlatformApi {
    fn get_position(&self) -> Result<WindowPosition, ()>;
}
