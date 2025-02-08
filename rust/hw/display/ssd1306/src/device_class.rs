use core::ptr::NonNull;

use qemu_api::{
    bindings::*, c_str, vmstate_fields, vmstate_i2c_slave, vmstate_unused, zeroable::Zeroable,
};

use crate::device::SSD1306State;

qemu_api::declare_properties! {
    SSD1306_PROPERTIES,
    qemu_api::define_property!(
        c_str!("someprop"),
        SSD1306State,
        someprop,
        unsafe { &qdev_prop_chr },
        CharBackend
    ),
}

pub static VMSTATE_SSD1306: VMStateDescription = VMStateDescription {
    name: c_str!("ssd1306").as_ptr(),
    // TODO: whats the version?
    version_id: 2,
    minimum_version_id: 2,
    //post_load: Some(pl011_post_load),
    post_load: None,
    fields: vmstate_fields! {
        vmstate_unused!(core::mem::size_of::<u32>()),
        vmstate_i2c_slave!(i2c, SSD1306State),
        //vmstate_uint32!(flags, SSD1306State),
    },
    ..Zeroable::ZERO
};

/// # Safety
///
/// We expect the FFI user of this function to pass a valid pointer, that has
/// the same size as [`SSD1306State`]. We also expect the device is
/// readable/writeable from one thread at any time.
pub unsafe extern "C" fn ssd1306_realize(dev: *mut DeviceState, _errp: *mut *mut Error) {
    unsafe {
        assert!(!dev.is_null());
        let mut state = NonNull::new_unchecked(dev.cast::<SSD1306State>());
        state.as_mut().realize();
    }
}

/// # Safety
///
/// We expect the FFI user of this function to pass a valid pointer, that has
/// the same size as [`SSD1306State`]. We also expect the device is
/// readable/writeable from one thread at any time.
pub unsafe extern "C" fn ssd1306_reset(dev: *mut DeviceState) {
    unsafe {
        assert!(!dev.is_null());
        let mut state = NonNull::new_unchecked(dev.cast::<SSD1306State>());
        state.as_mut().reset();
    }
}
