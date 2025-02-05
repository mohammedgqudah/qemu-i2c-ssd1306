use core::ptr::NonNull;
use std::os::raw::{c_uint, c_void};

use qemu_api::{bindings::*, zeroable::Zeroable};

use crate::device::TWI_I2CState;

pub static TWI_I2C_OPS: MemoryRegionOps = MemoryRegionOps {
    read: Some(twi_i2c_read),
    write: Some(twi_i2c_write),
    read_with_attrs: None,
    write_with_attrs: None,
    endianness: device_endian::DEVICE_NATIVE_ENDIAN,
    valid: Zeroable::ZERO,
    impl_: MemoryRegionOps__bindgen_ty_2 {
        min_access_size: 4,
        max_access_size: 4,
        ..Zeroable::ZERO
    },
};

unsafe extern "C" fn twi_i2c_read(opaque: *mut c_void, addr: hwaddr, size: c_uint) -> u64 {
    assert!(!opaque.is_null());
    let mut state = unsafe { NonNull::new_unchecked(opaque.cast::<TWI_I2CState>()) };
    let val = unsafe { state.as_mut().read(addr, size) };
    val
}

unsafe extern "C" fn twi_i2c_write(opaque: *mut c_void, addr: hwaddr, data: u64, _size: c_uint) {
    unsafe {
        assert!(!opaque.is_null());
        let mut state = NonNull::new_unchecked(opaque.cast::<TWI_I2CState>());
        state.as_mut().write(addr, data as u8)
    }
}
