use qemu_api::{bindings::*, c_str, vmstate_fields, vmstate_unused, zeroable::Zeroable};

use crate::device::TWIState;

// TODO: Find a way to not define properties
qemu_api::declare_properties! {
    TWI_I2C_PROPERTIES,
    qemu_api::define_property!(
        c_str!("someprop"),
        TWIState,
        someprop,
        unsafe { &qdev_prop_chr },
        CharBackend
    ),
}

pub static VMSTATE_TWI_I2C: VMStateDescription = VMStateDescription {
    name: c_str!("twi_i2c").as_ptr(),
    version_id: 1,
    minimum_version_id: 1,
    post_load: None,
    fields: vmstate_fields! {
        vmstate_unused!(core::mem::size_of::<u32>()),
        //vmstate_uint32!(flags, TWI_I2CState),
    },
    ..Zeroable::ZERO
};

// # Safety
//
// We expect the FFI user of this function to pass a valid pointer, that has
// the same size as [`TWI_I2CState`]. We also expect the device is
// readable/writeable from one thread at any time.
//pub unsafe extern "C" fn twi_i2c_realize(dev: *mut DeviceState, _errp: *mut *mut Error) {
//    unsafe {
//        assert!(!dev.is_null());
//        let mut state = NonNull::new_unchecked(dev.cast::<TWIState>());
//        state.as_mut().realize();
//    }
//}

// # Safety
//
// We expect the FFI user of this function to pass a valid pointer, that has
// the same size as [`TWI_I2CState`]. We also expect the device is
// readable/writeable from one thread at any time.
//pub unsafe extern "C" fn twi_i2c_reset(dev: *mut DeviceState) {
//    unsafe {
//        assert!(!dev.is_null());
//        let mut state = NonNull::new_unchecked(dev.cast::<TWIState>());
//        state.as_mut().reset_hold();
//    }
//}
