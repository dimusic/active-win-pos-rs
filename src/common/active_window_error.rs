use std::fmt;

#[derive(Debug, PartialEq, Clone)]
pub enum ActiveWindowError {
    GetActiveWindowFailed
}

impl std::error::Error for ActiveWindowError {}

impl fmt::Display for ActiveWindowError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ActiveWindowError::GetActiveWindowFailed => write!(f, "couldn't get active window"),
        }
    }
}
