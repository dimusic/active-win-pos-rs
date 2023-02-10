mod platform_api;
mod window_position;

use crate::common::platform_api::PlatformApi;
use platform_api::WindowsPlatformApi;

pub fn init_platform_api() -> impl PlatformApi {
    WindowsPlatformApi {}
}
