use std::ffi::CStr;

use crate::{
    bindings::{self, I2CSlave, I2CSlaveClass},
    prelude::*,
    qdev::{DeviceImpl, DeviceState},
};

unsafe impl ObjectType for I2CSlave {
    type Class = I2CSlaveClass;
    const TYPE_NAME: &'static CStr =
        unsafe { CStr::from_bytes_with_nul_unchecked(bindings::TYPE_I2C_SLAVE) };
}
qom_isa!(I2CSlave: DeviceState, Object);

pub trait I2CSlaveImpl: DeviceImpl + IsA<I2CSlave> {}

impl I2CSlaveClass {
    pub fn class_init<T: I2CSlaveImpl>(self: &mut I2CSlaveClass) {
        self.parent_class.class_init::<T>();
    }
}

pub trait I2CSlaveMethods: ObjectDeref
where
    Self::Target: IsA<I2CSlave>,
{
}

impl<R: ObjectDeref> I2CSlaveMethods for R where R::Target: IsA<I2CSlave> {}
