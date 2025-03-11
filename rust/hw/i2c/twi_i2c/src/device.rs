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
    cell::BqlRefCell,
    qdev::{DeviceImpl, Property, ResetType, ResettablePhasesImpl},
    qom::{ClassInitImpl, ObjectImpl, ObjectType, ParentField},
    sysbus::SysBusDeviceClass,
    vmstate::VMStateDescription,
};
use qemu_api_macros::Object;

use crate::{memory_ops::TWI_I2C_OPS, registers};

#[derive(Object, qemu_api_macros::offsets)]
#[repr(C)]
pub struct TWIState {
    pub parent_obj: ParentField<SysBusDevice>,
    pub iomem: MemoryRegion,
    pub someprop: char,
    pub bus: *mut I2CBus,
    pub irq: qemu_irq,

    pub registers: BqlRefCell<registers::TWIRegisters>,
    pub enabled: bool,
}

unsafe impl ObjectType for TWIState {
    type Class = TWI_I2CClass;
    const TYPE_NAME: &'static CStr = crate::TYPE_TWI_I2C;
}

impl ObjectImpl for TWIState {
    type ParentType = SysBusDevice;

    const INSTANCE_INIT: Option<unsafe fn(&mut Self)> = Some(Self::init);
    const INSTANCE_POST_INIT: Option<fn(&Self)> = None;
}

impl DeviceImpl for TWIState {
    fn properties() -> &'static [Property] {
        &crate::device_class::TWI_I2C_PROPERTIES
    }
    fn vmsd() -> Option<&'static VMStateDescription> {
        Some(&crate::device_class::VMSTATE_TWI_I2C)
    }
    const REALIZE: Option<fn(&Self)> = Some(Self::realize);
}

impl ResettablePhasesImpl for TWIState {
    const HOLD: Option<fn(&Self, ResetType)> = Some(Self::reset_hold);
}

impl TWIState {
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

    /// Reset the TWI controller.
    pub fn reset_hold(&self, _type: ResetType) {
        println!("reset");
        self.registers.borrow_mut().reset();
        //unsafe {
        //    s.in_transaction = false;
        //}
    }

    pub fn read(&mut self, offset: hwaddr, _size: c_uint) -> u64 {
        //println!("read address: {}: size: {}", offset, size);
        match offset {
            0 => {
                //println!("reading: reg 0");
            }
            1 => {
                let x: u8 = self.registers.borrow().twsr.into();
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
                let x: u8 = self.registers.borrow().twcr.into();
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
        let mut registers = self.registers.borrow_mut();
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
                registers.twar = registers::TWAR::from(data);
                // TODO: handle the LSB
                println!("slave address set: {}", u8::from(registers.twar) >> 1);
            }
            3 => {
                // set data
                registers.twdr = registers::TWDR::from(data);
                //println!(
                //    "char: {} ; hex: {:X}",
                //    u8::from(self.twdr) as char,
                //    u8::from(self.twdr)
                //);

                // Release the BqlRefCell before calling write_data
                drop(registers);

                self.write_data();
            }
            4 => {
                let r = registers::TWCR::from(data);
                //println!("{:#?}", r);
                // TODO: if this bit is reset, terminate all on going trasmissions
                self.enabled = r.twen();
                registers.twcr.set_twint(r.twint());

                if r.twsta() {
                    registers.twcr.set_twsta(true);
                }

                if r.twsto() {
                    //println!("Going to stop");
                    drop(registers);
                    self.stop();
                } else if r.twint() && r.twen() {
                    // TODO: The global interrupt must be enabled SREG (I)

                    // This is a new transaction
                    if !registers.in_transaction {
                        // TODO: don't touch the first two bits when setting the status
                        registers.in_transaction = true;
                        registers.twsr = registers::TW_START.into();
                        registers.twcr.set_twen(true);
                    }

                    unsafe {
                        // TODO: use the new InterruptSource API
                        qemu_set_irq(self.irq, 1);
                    };
                    registers.twcr.set_twint(true); // Fake TWI
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
    fn stop(&self) {
        let mut registers = self.registers.borrow_mut();
        unsafe {
            i2c_end_transfer(self.bus);
            //println!("ended transfer");
        };
        registers.in_transaction = false;
        registers.twcr.set_twsto(false); // report that STOP has executed on the bus.
        registers.twcr.set_twint(false); // TODO: according to twi_stop
                                         // in the arduino library, confirm
                                         // in
                                         // the data sheet
    }
    fn write_data(&self) {
        let mut registers = self.registers.borrow_mut();
        // report that you cannot write when twint is low
        if !registers.twcr.twint() {
            registers.twcr.set_twwc(true);
            return;
        }

        if registers.twcr.twsta() {
            // If the START bit was set, then this is the first data in the transaction
            // and it contains the sla+w (slave address + w bit)
            //
            // TODO: THIS IS NOT ALWAYS TRUE, in repeated start, the address wont be sent
            // again. (not sure)
            registers.twcr.set_twsta(false);
            let start_read = registers.twdr.twd0(); // write bit
                                                    // TODO: do i use start_transfer when attempting to read?

            let address = u8::from(registers.twdr) >> 1;

            // Release the BqlRefCell before calling set_status
            drop(registers);

            match i2c_start_transfer(self.bus, address, false) {
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
            let resp = unsafe { i2c_send(self.bus, registers.twdr.into()) };

            // Release the BqlRefCell before calling set_status
            drop(registers);

            if resp == 0 {
                self.set_status(registers::TW_MT_DATA_ACK);
            } else {
                self.set_status(registers::TW_MT_DATA_NACK);
            }
        }
    }

    /// Set the status bits in TWSR.
    fn set_status(&self, status: u8) {
        // TODO: only modify the last 5 bits.
        self.registers.borrow_mut().twsr = status.into();
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

impl ClassInitImpl<TWI_I2CClass> for TWIState {
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
        let mut state = NonNull::new_unchecked(obj.cast::<TWIState>());
        state.as_mut().init();
    }
}
