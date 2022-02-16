use core_graphics::display::CGRect;
use core_graphics::display::CFDictionaryRef;
use core_graphics::base::boolean_t;

#[link(name = "CoreGraphics", kind = "framework")]
extern {
    pub fn CGRectMakeWithDictionaryRepresentation(dict: CFDictionaryRef,
        rect: *mut CGRect) -> boolean_t;
}