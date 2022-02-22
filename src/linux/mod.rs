mod platform_api;

use crate::common::platform_api::PlatformApi;
use platform_api::LinuxPlatformApi;

pub fn init_platform_api() -> impl PlatformApi {
    LinuxPlatformApi { }
}
