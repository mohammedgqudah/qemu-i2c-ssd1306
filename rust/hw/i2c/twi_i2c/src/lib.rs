pub mod device_class;
pub mod device;
pub mod memory_ops;
pub mod registers;

use qemu_api::c_str;

pub const TYPE_TWI_I2C: &::std::ffi::CStr = c_str!("TWI_I2C");

