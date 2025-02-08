use std::ffi::CStr;

use crate::{
    bindings::{self, I2CSlave, I2CSlaveClass},
    prelude::*,
    qdev::{DeviceClass, DeviceState},
    qom::ClassInitImpl,
};

unsafe impl ObjectType for I2CSlave {
    type Class = I2CSlaveClass;
    const TYPE_NAME: &'static CStr =
        unsafe { CStr::from_bytes_with_nul_unchecked(bindings::TYPE_I2C_SLAVE) };
}
qom_isa!(I2CSlave: DeviceState, Object);

impl<T> ClassInitImpl<I2CSlaveClass> for T
where
    T: ClassInitImpl<DeviceClass>,
{
    fn class_init(sdc: &mut I2CSlaveClass) {
        <T as ClassInitImpl<DeviceClass>>::class_init(&mut sdc.parent_class);
    }
}

pub trait I2CSlaveMethods: ObjectDeref
where
    Self::Target: IsA<I2CSlave>,
{
}

impl<R: ObjectDeref> I2CSlaveMethods for R where R::Target: IsA<I2CSlave> {}
