use windows::Win32::Foundation::RECT;

use crate::common::window_position::WindowPosition;

pub trait FromWinRect {
    fn from_win_rect(rect: &RECT) -> WindowPosition;
}

impl FromWinRect for WindowPosition {
    fn from_win_rect(rect: &RECT) -> Self {
        WindowPosition {
            x: rect.left as f64,
            y: rect.top as f64,
            width: (rect.right - rect.left) as f64,
            height: (rect.bottom - rect.top) as f64,
        }
    }
}
