use core::ptr::NonNull;
use std::{
    ffi::{c_uint, c_void, CStr},
    ptr::{addr_of, addr_of_mut},
};

use qemu_api::{
    bindings::{
        hwaddr, i2c_end_transfer, i2c_init_bus, i2c_send, memory_region_init_io, qemu_irq,
        qemu_set_irq, sysbus_init_irq, sysbus_init_mmio, DeviceState, I2CBus, MemoryRegion, Object,
        SysBusDevice,
    },
    c_str,
    qdev::{DeviceImpl, Property},
    qom::{ClassInitImpl, ObjectImpl, ObjectType, ParentField},
    sysbus::SysBusDeviceClass,
    vmstate::VMStateDescription,
};
use qemu_api_macros::Object;

use crate::{memory_ops::TWI_I2C_OPS, registers};

#[derive(Debug, Object, qemu_api_macros::offsets)]
#[repr(C)]
pub struct TWI_I2CState {
    pub parent_obj: ParentField<SysBusDevice>,
    pub iomem: MemoryRegion,
    pub someprop: char,
    pub bus: *mut I2CBus,
    pub irq: qemu_irq,

    pub twbr: registers::TWBR,
    pub twsr: registers::TWSR,
    pub twar: registers::TWAR,
    pub twdr: registers::TWDR,
    pub twcr: registers::TWCR,

    pub enabled: bool,
    // TODO: is there a way to track this using only the registers?
    pub in_transaction: bool,
}

unsafe impl ObjectType for TWI_I2CState {
    type Class = TWI_I2CClass;
    const TYPE_NAME: &'static CStr = crate::TYPE_TWI_I2C;
}

impl ObjectImpl for TWI_I2CState {
    type ParentType = SysBusDevice;

    const INSTANCE_INIT: Option<unsafe fn(&mut Self)> = Some(Self::init);
    const INSTANCE_POST_INIT: Option<fn(&Self)> = None;
}

impl DeviceImpl for TWI_I2CState {
    fn properties() -> &'static [Property] {
        &crate::device_class::TWI_I2C_PROPERTIES
    }
    fn vmsd() -> Option<&'static VMStateDescription> {
        Some(&crate::device_class::VMSTATE_TWI_I2C)
    }
    const REALIZE: Option<fn(&Self)> = Some(Self::realize);
    const RESET: Option<fn(&Self)> = Some(Self::reset);
}

impl TWI_I2CState {
    /// Initializes a pre-allocated, unitialized instance of `TWI_I2CState`.
    ///
    /// # Safety
    ///
    /// `self` must point to a correctly sized and aligned location for the
    /// `TWI_I2CState` type. It must not be called more than once on the same
    /// location/instance. All its fields are expected to hold unitialized
    /// values with the sole exception of `parent_obj`.
    pub fn init(&mut self) {
        println!("init twi");

        let device = addr_of_mut!(*self).cast::<DeviceState>();
        let sbd = addr_of_mut!(*self).cast::<SysBusDevice>();
        unsafe {
            sysbus_init_mmio(sbd, addr_of_mut!(self.iomem));
            sysbus_init_irq(sbd, &mut self.irq);
            self.bus = i2c_init_bus(device, c_str!("i2c-bus").as_ptr());
            qemu_api::bindings::i2c_slave_create_simple(
                self.bus,
                c_str!("ssd1306").as_ptr(),
                0x3d_u8,
            );
        }
    }

    pub fn realize(&self) {
        println!("realize twi");
        unsafe {
            memory_region_init_io(
                addr_of!(self.iomem) as *mut _,
                addr_of!(*self).cast::<Object>() as *mut _,
                &TWI_I2C_OPS,
                addr_of!(*self).cast::<c_void>() as *mut _,
                Self::TYPE_INFO.name,
                0x6,
            );
        }
    }

    /// Reset the TWI controller registers.
    ///
    /// Based on "Register Description" in the ATmega640/1280/1281/2560/2561
    /// Datasheet.
    pub fn reset(&self) {
        println!("reset");
        unsafe {
            // TODO: ONLY UNTIL I START USING THE BGL OR CELL.
            let s = addr_of!(self) as *mut Self;
            let s = &mut *s as &mut Self;
            s.twbr = 0.into();
            s.twsr = 0b11111000.into();
            s.twar = 0b11111110.into();
            s.twdr = 0xFF.into();
            s.twcr = 0.into();
            s.in_transaction = false;
        }
    }

    pub fn read(&mut self, offset: hwaddr, _size: c_uint) -> u64 {
        //println!("read address: {}: size: {}", offset, size);
        match offset {
            0 => {
                //println!("reading: reg 0");
            }
            1 => {
                let x: u8 = self.twsr.into();
                return u64::from(x);
            }
            2 => {
                //println!("reading: reg 1");
            }
            3 => {
                //println!("reading: reg 3");
            }
            4 => {
                //println!("reading: reg 4");
                let x: u8 = self.twcr.into();
                return u64::from(x);
            }
            5 => {
                //println!("reading: reg 5");
            }
            _ => {
                //eprintln!("reading: bad offset");
            }
        }
        0xFF
    }

    pub fn write(&mut self, address: hwaddr, data: u8) {
        //println!("write address: {}: data: {}", address, data);
        match address {
            0 => {
                // set the bit rate
                println!("setting TWI bit rate");
                let _r = registers::TWBR::from(data);
            }
            1 => {
                // TODO: handle the first two bits.
                let _r = registers::TWSR::from(data);
            }
            2 => {
                // set address
                self.twar = registers::TWAR::from(data);
                // TODO: handle the LSB
                println!("slave address set: {}", u8::from(self.twar) >> 1);
            }
            3 => {
                // set data
                self.twdr = registers::TWDR::from(data);
                //println!(
                //    "char: {} ; hex: {:X}",
                //    u8::from(self.twdr) as char,
                //    u8::from(self.twdr)
                //);
                self.write_data();
            }
            4 => {
                let r = registers::TWCR::from(data);
                //println!("{:#?}", r);
                // TODO: if this bit is reset, terminate all on going trasmissions
                self.enabled = r.twen();
                self.twcr.set_twint(r.twint());

                if r.twsta() {
                    self.twcr.set_twsta(true);
                }

                if r.twsto() {
                    //println!("Going to stop");
                    self.stop();
                } else if r.twint() && r.twen() {
                    // TODO: The global interrupt must be enabled SREG (I)

                    // This is a new transaction
                    if !self.in_transaction {
                        // TODO: don't touch the first two bits when setting the status
                        self.in_transaction = true;
                        self.twsr = registers::TW_START.into();
                        self.twcr.set_twen(true);
                    }

                    unsafe {
                        // TODO: use the new InterruptSource API
                        qemu_set_irq(self.irq, 1);
                    };
                    self.twcr.set_twint(true); // Fake TWI
                }
            }
            _ => {
                // TODO: better error reporting
                eprintln!("bad offset");
            }
        }
    }

    /// Handle a START condition.
    const fn _start(&self) {}

    /// Handle a STOP condition.
    fn stop(&mut self) {
        unsafe {
            i2c_end_transfer(self.bus);
            //println!("ended transfer");
        };
        self.in_transaction = false;
        self.twcr.set_twsto(false); // report that STOP has executed on the bus.
        self.twcr.set_twint(false); // TODO: according to twi_stop
                                    // in the arduino library, confirm
                                    // in
                                    // the data sheet
    }
    fn write_data(&mut self) {
        // report that you cannot write when twint is low
        if !self.twcr.twint() {
            self.twcr.set_twwc(true);
            return;
        }

        if self.twcr.twsta() {
            // If the START bit was set, then this is the first data in the transaction
            // and it contains the sla+w (slave address + w bit)
            //
            // TODO: THIS IS NOT ALWAYS TRUE, in repeated start, the address wont be sent
            // again. (not sure)
            self.twcr.set_twsta(false);
            let start_read = self.twdr.twd0(); // write bit
                                               // TODO: do i use start_transfer when attempting to read?

            match i2c_start_transfer(self.bus, u8::from(self.twdr) >> 1, false) {
                Ok(()) => {
                    self.set_status(if start_read {
                        registers::TW_MR_SLA_ACK
                    } else {
                        registers::TW_MT_SLA_ACK
                    });
                }
                Err(()) => {
                    eprintln!("start transfer failed");
                    // TODO: im faking ACK for testing
                    //
                    //self.set_status(if start_read {
                    //    registers::TW_MR_SLA_NACK
                    //} else {
                    //    registers::TW_MT_SLA_NACK
                    //});
                    self.set_status(if start_read {
                        registers::TW_MR_SLA_ACK
                    } else {
                        registers::TW_MT_SLA_ACK
                    });
                }
            }
        } else {
            // TODO: refactor to a Result<(), ()>
            let resp = unsafe { i2c_send(self.bus, self.twdr.into()) };
            if resp == 0 {
                self.set_status(registers::TW_MT_DATA_ACK);
            } else {
                self.set_status(registers::TW_MT_DATA_NACK);
            }
        }
    }

    /// Set the status bits in TWSR.
    fn set_status(&mut self, status: u8) {
        // TODO: only modify the last 5 bits.
        self.twsr = status.into();
    }
}

// TODO: move somewhere else
// TODO: Should this be safe or unsafe?
fn i2c_start_transfer(bus: *mut I2CBus, address: u8, is_recv: bool) -> Result<(), ()> {
    //println!("Starting a transfer @ address: {}", address);
    let result = unsafe { qemu_api::bindings::i2c_start_transfer(bus, address, is_recv) };
    if result > 0 {
        Err(())
    } else {
        Ok(())
    }
}

#[repr(C)]
pub struct TWI_I2CClass {
    parent_class: <SysBusDevice as ObjectType>::Class,
}

impl ClassInitImpl<TWI_I2CClass> for TWI_I2CState {
    fn class_init(klass: &mut TWI_I2CClass) {
        <Self as ClassInitImpl<SysBusDeviceClass>>::class_init(&mut klass.parent_class);
    }
}

/// # Safety
///
/// We expect the FFI user of this function to pass a valid pointer, that has
/// the same size as [`TWI_I2CState`]. We also expect the device is
/// readable/writeable from one thread at any time.
pub unsafe extern "C" fn twi_i2c_init(obj: *mut Object) {
    unsafe {
        // TODO: replace debug_assert with new instead of using new_unchecked
        debug_assert!(!obj.is_null());
        let mut state = NonNull::new_unchecked(obj.cast::<TWI_I2CState>());
        state.as_mut().init();
    }
}
