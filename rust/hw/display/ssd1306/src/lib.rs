pub mod device;
pub mod device_class;

use qemu_api::c_str;

pub const TYPE_SSD1306: &::std::ffi::CStr = c_str!("ssd1306");
