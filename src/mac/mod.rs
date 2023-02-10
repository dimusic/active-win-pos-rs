mod core_graphics_patch;
mod platform_api;
mod window_position;

use crate::common::platform_api::PlatformApi;
use platform_api::MacPlatformApi;

pub fn init_platform_api() -> impl PlatformApi {
    MacPlatformApi {}
}
