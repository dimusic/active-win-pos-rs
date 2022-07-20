use core_foundation::{
    string::{
        CFString, kCFStringEncodingUTF8, CFStringGetCStringPtr, CFStringGetTypeID
    },
    base::{
        ToVoid, CFGetTypeID
    },
    number::{
        CFNumberRef, CFNumberGetType, CFNumberGetValue, CFBooleanGetValue, CFNumberGetTypeID, CFNumberType
    },
    mach_port::CFTypeID,
    boolean::CFBooleanGetTypeID, dictionary::CFDictionaryGetTypeID,
};
use core_graphics::display::*;
use std::{ffi::{ CStr, c_void }};
use appkit_nsworkspace_bindings::{NSWorkspace, INSWorkspace, INSRunningApplication};
use crate::common::{window_position::WindowPosition, platform_api::PlatformApi, active_window::ActiveWindow};
use super::core_graphics_patch::CGRectMakeWithDictionaryRepresentation;
use super::window_position::FromCgRect;

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

pub struct MacPlatformApi {
    
}

impl PlatformApi for MacPlatformApi {
    fn get_position(&self) -> Result<WindowPosition, ()> {
        if let Ok(active_window) = self.get_active_window() {
            return Ok(active_window.position);
        }

        return Err(());
    }

    fn get_active_window(&self) -> Result<ActiveWindow, ()> {
        const OPTIONS: CGWindowListOption = kCGWindowListOptionOnScreenOnly | kCGWindowListExcludeDesktopElements;
        let window_list_info = unsafe { CGWindowListCopyWindowInfo(OPTIONS, kCGNullWindowID) };
        
        let count: isize = unsafe { CFArrayGetCount(window_list_info) };

        let active_window_pid = unsafe {
            let workspace = NSWorkspace::sharedWorkspace();
            let active_app = workspace.frontmostApplication();
            active_app.processIdentifier() as i64
        };

        let mut win_pos = WindowPosition::new(0., 0., 0., 0.);
        
        for i in 0..count-1 {
            let dic_ref = unsafe { CFArrayGetValueAtIndex(window_list_info, i) as CFDictionaryRef };

            let window_pid = get_from_dict(dic_ref, "kCGWindowOwnerPID");

            if let DictEntryValue::_Number(win_pid) = window_pid {
                if win_pid != active_window_pid {
                    continue;
                }

                if let DictEntryValue::_Rect(window_bounds) = get_from_dict(dic_ref, "kCGWindowBounds") {
                    if window_bounds.width < 50. || window_bounds.height < 50. {
                        continue;
                    }

                    win_pos = window_bounds;
                }

                if let DictEntryValue::_Number(window_id) = get_from_dict(dic_ref, "kCGWindowNumber") {
                    let active_window = ActiveWindow {
                        window_id: window_id.to_string(),
                        process_id: active_window_pid as u64,
                        position: win_pos,
                        title: String::from(""),
                    };

                    return Ok(active_window)
                }
            }
        }

        unsafe {
            CFRelease(window_list_info as CFTypeRef)
        }

        return Err(());
    }
}

// Taken from https://github.com/sassman/t-rec-rs/blob/v0.7.0/src/macos/window_id.rs#L73
// Modified to support dictionary type id for kCGWindowBounds
fn get_from_dict(dict: CFDictionaryRef, key: &str) -> DictEntryValue {
    let cf_key: CFString = key.into();
    let mut value: *const c_void = std::ptr::null();
    if unsafe { CFDictionaryGetValueIfPresent(dict, cf_key.to_void(), &mut value) != 0 } {
        let type_id: CFTypeID = unsafe { CFGetTypeID(value) };
        if type_id == unsafe { CFNumberGetTypeID() } {
            let value = value as CFNumberRef;
            
            #[allow(non_upper_case_globals)]
            match unsafe { CFNumberGetType(value) } {
                kCFNumberSInt64Type => {
                    let mut value_i64 = 0_i64;
                    let out_value: *mut i64 = &mut value_i64;
                    let converted = unsafe { CFNumberGetValue(value, kCFNumberSInt64Type, out_value.cast()) };
                    if converted {
                        return DictEntryValue::_Number(value_i64);
                    }
                }
                kCFNumberSInt32Type => {
                    let mut value_i32 = 0_i32;
                    let out_value: *mut i32 = &mut value_i32;
                    let converted = unsafe { CFNumberGetValue(value, kCFNumberSInt32Type, out_value.cast()) };
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
            let c_ptr = unsafe { CFStringGetCStringPtr(value.cast(), kCFStringEncodingUTF8) };
            return if !c_ptr.is_null() {
                let c_result = unsafe { CStr::from_ptr(c_ptr) };
                let result = String::from(c_result.to_str().unwrap());
                DictEntryValue::_String(result)
            } else {
                DictEntryValue::_Unknown
            };
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
