mod platform_api;
mod wayland;

use crate::common::platform_api::PlatformApi;
use platform_api::FreeBsdPlatformApi;

pub fn init_platform_api() -> impl PlatformApi {
    FreeBsdPlatformApi {}
}
