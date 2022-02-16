mod core_graphics_patch;
mod window_position;
mod platform_api;

use crate::common::{platform_api::PlatformApi};
use platform_api::MacPlatformApi;

pub fn init_platform_api() -> impl PlatformApi {
    MacPlatformApi { }
}
