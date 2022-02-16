use crate::common::window_position::WindowPosition;
use core_graphics::display::CGRect;

pub trait FromCgRect {
    fn from_cg_rect(cgrect: &CGRect) -> WindowPosition;
}

impl FromCgRect for WindowPosition {
    fn from_cg_rect(cgrect: &CGRect) -> Self {
        Self::new(cgrect.origin.x, cgrect.origin.y, cgrect.size.width, cgrect.size.height)
    }
}
