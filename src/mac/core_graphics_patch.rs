use core_graphics::base::boolean_t;
use core_graphics::display::CFDictionaryRef;
use core_graphics::display::CGRect;

#[link(name = "CoreGraphics", kind = "framework")]
extern "C" {
    pub fn CGRectMakeWithDictionaryRepresentation(
        dict: CFDictionaryRef,
        rect: *mut CGRect,
    ) -> boolean_t;
}
