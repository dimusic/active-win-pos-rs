use super::core_graphics_patch::CGRectMakeWithDictionaryRepresentation;
use super::window_position::FromCgRect;
use crate::common::{
    active_window::ActiveWindow, platform_api::PlatformApi, window_position::WindowPosition,
};
use appkit_nsworkspace_bindings::{INSRunningApplication, INSWorkspace, NSWorkspace, INSURL};
use core_foundation::{
    base::{CFGetTypeID, ToVoid},
    boolean::CFBooleanGetTypeID,
    dictionary::CFDictionaryGetTypeID,
    mach_port::CFTypeID,
    number::{
        CFBooleanGetValue, CFNumberGetType, CFNumberGetTypeID, CFNumberGetValue, CFNumberRef,
        CFNumberType,
    },
    string::{CFString, CFStringGetTypeID},
};
use core_graphics::display::*;
use objc::runtime::Object;
use std::{ffi::c_void, path::PathBuf};

#[allow(non_upper_case_globals)]
pub const kCFNumberSInt32Type: CFNumberType = 3;
#[allow(non_upper_case_globals)]
pub const kCFNumberSInt64Type: CFNumberType = 4;

#[derive(Debug)]
enum DictEntryValue {
    _Number(i64),
    _Bool(bool),
    _String(String),
    _Rect(WindowPosition),
    _Unknown,
}

pub struct MacPlatformApi {}

impl PlatformApi for MacPlatformApi {
    fn get_position(&self) -> Result<WindowPosition, ()> {
        if let Ok(active_window) = self.get_active_window() {
            return Ok(active_window.position);
        }

        Err(())
    }

    fn get_active_window(&self) -> Result<ActiveWindow, ()> {
        const OPTIONS: CGWindowListOption =
            kCGWindowListOptionOnScreenOnly | kCGWindowListExcludeDesktopElements;
        let window_list_info = unsafe { CGWindowListCopyWindowInfo(OPTIONS, kCGNullWindowID) };

        let windows_count: isize = unsafe { CFArrayGetCount(window_list_info) };

        let active_app = unsafe {
            let workspace = NSWorkspace::sharedWorkspace();
            workspace.frontmostApplication()
        };

        let active_window_pid = unsafe { active_app.processIdentifier() as i64 };

        let mut win_pos = WindowPosition::new(0., 0., 0., 0.);
        let mut win_title = String::from("");
        let mut app_name = String::from("");

        for i in 0..windows_count {
            let dic_ref = unsafe { CFArrayGetValueAtIndex(window_list_info, i) as CFDictionaryRef };

            if dic_ref.is_null() {
                continue;
            }

            let window_pid = get_from_dict(dic_ref, "kCGWindowOwnerPID");

            if let DictEntryValue::_Number(win_pid) = window_pid {
                if win_pid != active_window_pid {
                    continue;
                }

                if let DictEntryValue::_Rect(window_bounds) =
                    get_from_dict(dic_ref, "kCGWindowBounds")
                {
                    if window_bounds.width < 50. || window_bounds.height < 50. {
                        continue;
                    }

                    win_pos = window_bounds;
                }

                if let DictEntryValue::_String(window_title) =
                    get_from_dict(dic_ref, "kCGWindowName")
                {
                    win_title = window_title;
                }

                if let DictEntryValue::_String(owner_name) =
                    get_from_dict(dic_ref, "kCGWindowOwnerName")
                {
                    app_name = owner_name;
                }

                let process_path: PathBuf = unsafe {
                    let bundle_url = active_app.bundleURL();
                    if bundle_url.0.is_null() {
                        PathBuf::new()
                    } else {
                        let path = bundle_url.path();
                        PathBuf::from(nsstring_to_rust_string(path.0))
                    }
                };

                if let DictEntryValue::_Number(window_id) =
                    get_from_dict(dic_ref, "kCGWindowNumber")
                {
                    let active_window = ActiveWindow {
                        window_id: window_id.to_string(),
                        process_id: active_window_pid as u64,
                        app_name,
                        position: win_pos,
                        title: win_title,
                        process_path,
                    };

                    unsafe { CFRelease(window_list_info as CFTypeRef) }

                    return Ok(active_window);
                }
            }
        }

        unsafe { CFRelease(window_list_info as CFTypeRef) }

        Err(())
    }
}

// Taken from https://github.com/sassman/t-rec-rs/blob/v0.7.0/src/macos/window_id.rs#L73
// Modified to support dictionary type id for kCGWindowBounds
fn get_from_dict(dict: CFDictionaryRef, key: &str) -> DictEntryValue {
    let cf_key: CFString = key.into();
    let mut value: *const c_void = std::ptr::null();
    if unsafe { CFDictionaryGetValueIfPresent(dict, cf_key.to_void(), &mut value) } != 0 {
        let type_id: CFTypeID = unsafe { CFGetTypeID(value) };
        if type_id == unsafe { CFNumberGetTypeID() } {
            let value = value as CFNumberRef;

            #[allow(non_upper_case_globals)]
            match unsafe { CFNumberGetType(value) } {
                kCFNumberSInt64Type => {
                    let mut value_i64 = 0_i64;
                    let out_value: *mut i64 = &mut value_i64;
                    let converted =
                        unsafe { CFNumberGetValue(value, kCFNumberSInt64Type, out_value.cast()) };
                    if converted {
                        return DictEntryValue::_Number(value_i64);
                    }
                }
                kCFNumberSInt32Type => {
                    let mut value_i32 = 0_i32;
                    let out_value: *mut i32 = &mut value_i32;
                    let converted =
                        unsafe { CFNumberGetValue(value, kCFNumberSInt32Type, out_value.cast()) };
                    if converted {
                        return DictEntryValue::_Number(value_i32 as i64);
                    }
                }
                n => {
                    eprintln!("Unsupported Number of typeId: {}", n);
                }
            }
        } else if type_id == unsafe { CFBooleanGetTypeID() } {
            return DictEntryValue::_Bool(unsafe { CFBooleanGetValue(value.cast()) });
        } else if type_id == unsafe { CFStringGetTypeID() } {
            let str = nsstring_to_rust_string(value as *mut Object);
            return DictEntryValue::_String(str);
        } else if type_id == unsafe { CFDictionaryGetTypeID() } && key == "kCGWindowBounds" {
            let rect: CGRect = unsafe {
                let mut rect = std::mem::zeroed();
                CGRectMakeWithDictionaryRepresentation(value.cast(), &mut rect);
                rect
            };

            return DictEntryValue::_Rect(WindowPosition::from_cg_rect(&rect));
        } else {
            eprintln!("Unexpected type: {}", type_id);
        }
    }

    DictEntryValue::_Unknown
}

pub fn nsstring_to_rust_string(nsstring: *mut Object) -> String {
    unsafe {
        let cstr: *const i8 = msg_send![nsstring, UTF8String];
        if !cstr.is_null() {
            std::ffi::CStr::from_ptr(cstr)
                .to_string_lossy()
                .into_owned()
        } else {
            "".into()
        }
    }
}
