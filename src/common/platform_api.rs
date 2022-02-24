use super::active_window_error::ActiveWindowError;
use super::window_position::WindowPosition;
use super::active_window::ActiveWindow;

pub trait PlatformApi {
    fn get_position(&self) -> Result<WindowPosition, ActiveWindowError>;
    fn get_active_window(&self) -> Result<ActiveWindow, ActiveWindowError>;
}
