#[derive(Debug, Clone, PartialEq)]
pub struct WindowPosition {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

impl WindowPosition {
    pub fn new(x: f64, y: f64, w: f64, h: f64) -> Self {
        Self {
            x,
            y,
            width: w,
            height: h,
        }
    }
}

impl Default for WindowPosition {
    fn default() -> Self {
        Self::new(0.0, 0.0, 0.0, 0.0)
    }
}
