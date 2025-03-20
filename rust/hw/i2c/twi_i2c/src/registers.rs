extern crate bilge;
extern crate bilge_impl;

use bilge::prelude::*;

/// ATmega640/1280/1281/2560/2561 Datasheet
///
///
/// Datasheet: <https://ww1.microchip.com/downloads/aemDocuments/documents/OTH/ProductDocuments/DataSheets/ATmega640-1280-1281-2560-2561-Datasheet-DS40002211A.pdf>
#[allow(non_camel_case_types)]
pub enum RegisterOffset {
    TWBR = 0x0,
    TWSR = 0x1,
    TWAR = 0x2,
    TWDR = 0x3,
    TWCR = 0x4,
}

pub struct TWIRegisters {
    pub twbr: TWBR,
    pub twsr: TWSR,
    pub twar: TWAR,
    pub twdr: TWDR,
    pub twcr: TWCR,
    pub in_transaction: bool,
}

impl TWIRegisters {
    /// Reset the TWI controller registers.
    ///
    /// Based on "Register Description" in the ATmega640/1280/1281/2560/2561
    /// Datasheet.
    pub fn reset(&mut self) {
        self.twbr = 0.into();
        self.twsr = 0b11111000.into();
        self.twar = 0b11111110.into();
        self.twdr = 0xFF.into();
        self.twcr = 0.into();
    }
}

#[bitsize(8)]
#[derive(Clone, Copy, DebugBits, FromBits)]
/// TWBR – TWI Bit Rate Register.
///
/// Selects the division factor for the bit rate generator.
/// See "Bit Rate Generator Unit" for more details.
pub struct TWBR {
    pub twbr0: bool,
    pub twbr1: bool,
    pub twbr2: bool,
    pub twbr3: bool,
    pub twbr4: bool,
    pub twbr5: bool,
    pub twbr6: bool,
    pub twbr7: bool,
}

#[bitsize(8)]
#[derive(Clone, Copy, DebugBits, FromBits)]
/// TWSR – TWI Status Register.
pub struct TWSR {
    /// twps0 and twps1 bits can be read and written, and control the bit rate
    /// prescaler.
    ///
    /// | TWPS1 | TWPS0 | Prescaler Value |
    /// |-------|-------|-----------------|
    /// |   0   |   0   |        1        |
    /// |   0   |   1   |        4        |
    /// |   1   |   0   |       16        |
    /// |   1   |   1   |       64        |
    pub twps0: bool,
    pub twps1: bool,
    _reserved: bool,
    /// These five bits reflect the status of the TWI logic and the 2-wire
    /// Serial Bus (Possible statuses are defined below as TW_\<STATUS\>)
    pub status: u5,
}

#[bitsize(8)]
#[derive(Clone, Copy, DebugBits, FromBits)]
/// TWAR – TWI (Slave) Address Register.
///
/// The TWAR should be loaded with the 7-bit Slave address (in the seven most
/// significant bits of TWAR) to which the TWI will respond when programmed as a
/// Slave Transmitter or Receiver, and not needed in the Master modes. In
/// multimaster systems, TWAR must be set in masters which can be addressed as
/// Slaves by other Masters. The LSB of TWAR is used to enable recognition of
/// the general call address (0x00). There is an associated address comparator
/// that looks for the slave address (or general call address if enabled) in the
/// received serial address. If a match is found, an interrupt request is
/// generated.
pub struct TWAR {
    pub twgce: bool,
    pub twa0: bool,
    pub twa1: bool,
    pub twa2: bool,
    pub twa3: bool,
    pub twa4: bool,
    pub twa5: bool,
    pub twa6: bool,
}

#[bitsize(8)]
#[derive(Clone, Copy, DebugBits, FromBits)]
/// TWDR – TWI Data Register.
///
/// In Transmit mode, TWDR contains the next byte to be transmitted. In Receive
/// mode, the TWDR contains the last byte received. It is writable while the TWI
/// is not in the process of shifting a byte. This occurs when the TWI Interrupt
/// Flag (TWINT) is set by hardware. Note that the Data Register cannot be
/// initialized by the user before the first interrupt occurs. The data in TWDR
/// remains stable as long as TWINT is set. While data is shifted out, data on
/// the bus is simultaneously shifted in. TWDR always contains the last byte
/// present on the bus, except after a wake up from a sleep mode by the TWI
/// interrupt. In this case, the contents of TWDR is undefined. In the case of a
/// lost bus arbitration, no data is lost in the transition from Master to
/// Slave. Handling of the ACK bit is controlled automatically by the TWI logic,
/// the CPU cannot access the ACK bit directly.
///
/// 24.6 Using the TWI
///
/// The application software should now examine the value of TWSR, to make sure
/// that the START condition was successfully transmitted. If TWSR indicates
/// otherwise, the application software might take some special action, like
/// calling an error routine. Assuming that the status code is as expected,
/// **the application must load SLA+W into TWDR.** Remember that TWDR is used
/// both for address and data.
pub struct TWDR {
    pub twd0: bool,
    pub twd1: bool,
    pub twd2: bool,
    pub twd3: bool,
    pub twd4: bool,
    pub twd5: bool,
    pub twd6: bool,
    pub twd7: bool,
}

#[bitsize(8)]
#[derive(Clone, Copy, DebugBits, FromBits)]
/// TWCR – TWI Control Register.
///
/// The TWCR is used to control the operation of the TWI. It is used to enable
/// the TWI, to initiate a Master access by applying a START condition to the
/// bus, to generate a Receiver acknowledge, to generate a stop condition, and
/// to control halting of the bus while the data to be written to the bus are
/// written to the TWDR. It also indicates a write collision if data is
/// attempted written to TWDR while the register is inaccessible.
pub struct TWCR {
    /// TWI Interrupt Enable.
    ///
    /// When this bit is written to one, and the I-bit in SREG is set, the TWI
    /// interrupt request will be activated for as long as the TWINT Flag is
    /// high.
    pub twie: bool,
    /// This bit is a reserved bit and will always read as zero.
    _reserved: bool,
    /// TWI Enable Bit.
    ///
    /// The TWEN bit enables TWI operation and activates the TWI interface. When
    /// TWEN is written to one, the TWI takes control over the I/O pins
    /// connected to the SCL and SDA pins, enabling the slew-rate limiters and
    /// spike filters. If this bit is written to zero, the TWI is switched
    /// off and all TWI transmissions are terminated, regardless of any
    /// ongoing operation.
    pub twen: bool,
    /// TWI Write Collision Flag.
    ///
    /// The TWWC bit is set when attempting to write to TWDR when TWINT is low.
    /// This flag is cleared by writing the TWDR Register when TWINT is
    /// high.
    pub twwc: bool,
    /// TWI STOP Condition Bit.
    ///
    /// Writing the TWSTO bit to one in Master mode will generate a STOP
    /// condition on the 2-wire Serial Bus. When the STOP condition is
    /// executed on the bus, the TWSTO bit is cleared automatically. In Slave
    /// mode, setting the TWSTO bit can be used to recover from an error
    /// condition. This will not generate a STOP condition, but the TWI
    /// returns to a well-defined unaddressed Slave mode and releases the SCL
    /// and SDA lines to a high impedance state
    pub twsto: bool,
    /// TWI START Condition Bit.
    ///
    /// The application writes the TWSTA bit to one when it desires to become a
    /// Master on the 2-wire Serial Bus. The TWI hardware checks if the bus
    /// is available, and generates a START condition on the bus if it is free.
    /// However, if the bus is not free, the TWI waits until a STOP
    /// condition is detected, and then generates a new START condition to
    /// claim the bus Master status. TWSTA must be cleared by software when the
    /// START condition has been transmitted.
    pub twsta: bool,
    /// TWI Enable Acknowledge Bit.
    ///
    /// The TWEA bit controls the generation of the acknowledge pulse. If the
    /// TWEA bit is written to one, the ACK pulse is generated on the TWI
    /// bus if the following conditions are met:
    /// 1. The device's own slave address has been received.
    /// 2. A general call has been received, while the TWGCE bit in the TWAR is
    ///    set.
    /// 3. A data byte has been received in Master Receiver or Slave Receiver
    ///    mode.
    /// By writing the TWEA bit to zero, the device can be virtually
    /// disconnected from the 2-wire Serial Bus temporarily.
    /// Address recognition can then be resumed by writing the TWEA bit to one
    /// again.
    pub twea: bool,
    /// TWI Interrupt Flag.
    ///
    /// This bit is set by hardware when the TWI has finished its current job
    /// and expects application software response. If the I-bit in SREG and
    /// TWIE in TWCR are set, the MCU will jump to the TWI Interrupt Vector.
    /// While the TWINT Flag is set, the SCL low period is stretched. The
    /// TWINT Flag must be cleared by software by writing a logic one to it.
    /// Note that this flag is not automatically cleared by hardware when
    /// executing the interrupt routine. Also note that clearing this flag
    /// starts the operation of the TWI, so all accesses to the TWI Address
    /// Register (TWAR), TWI Status Register (TWSR), and TWI Data Register
    /// (TWDR) must be complete before clearing this flag.
    pub twint: bool,
}

// Start condition transmitted
pub const TW_START: u8 = 0x08;

// Repeated start condition transmitted
pub const TW_REP_START: u8 = 0x10;

// Master Transmitter
//
// SLA+W transmitted, ACK received
pub const TW_MT_SLA_ACK: u8 = 0x18;

// SLA+W transmitted, NACK received
pub const TW_MT_SLA_NACK: u8 = 0x20;

// Data transmitted, ACK received
pub const TW_MT_DATA_ACK: u8 = 0x28;

// Data transmitted, NACK received
pub const TW_MT_DATA_NACK: u8 = 0x30;

// Arbitration lost in SLA+W or data
pub const TW_MT_ARB_LOST: u8 = 0x38;
//
// ----------------------
//
// Master Receiver
//
// Arbitration lost in SLA+R or NACK
pub const TW_MR_ARB_LOST: u8 = 0x38;

// SLA+R transmitted, ACK received
pub const TW_MR_SLA_ACK: u8 = 0x40;

// SLA+R transmitted, NACK received
pub const TW_MR_SLA_NACK: u8 = 0x48;

// Data received, ACK returned
pub const TW_MR_DATA_ACK: u8 = 0x50;

// Data received, NACK returned
pub const TW_MR_DATA_NACK: u8 = 0x58;

// Slave Transmitter
// SLA+R received, ACK returned
pub const TW_ST_SLA_ACK: u8 = 0xA8;

// Arbitration lost in SLA+RW, SLA+R received, ACK returned
pub const TW_ST_ARB_LOST_SLA_ACK: u8 = 0xB0;

// Data transmitted, ACK received
pub const TW_ST_DATA_ACK: u8 = 0xB8;

// Data transmitted, NACK received
pub const TW_ST_DATA_NACK: u8 = 0xC0;

// Last data byte transmitted, ACK received
pub const TW_ST_LAST_DATA: u8 = 0xC8;

// Slave Receiver
// SLA+W received, ACK returned
pub const TW_SR_SLA_ACK: u8 = 0x60;

// Arbitration lost in SLA+RW, SLA+W received, ACK returned
pub const TW_SR_ARB_LOST_SLA_ACK: u8 = 0x68;

// General call received, ACK returned
pub const TW_SR_GCALL_ACK: u8 = 0x70;

// Arbitration lost in SLA+RW, general call received, ACK returned
pub const TW_SR_ARB_LOST_GCALL_ACK: u8 = 0x78;

// Data received, ACK returned
pub const TW_SR_DATA_ACK: u8 = 0x80;

// Data received, NACK returned
pub const TW_SR_DATA_NACK: u8 = 0x88;

// General call data received, ACK returned
pub const TW_SR_GCALL_DATA_ACK: u8 = 0x90;

// General call data received, NACK returned
pub const TW_SR_GCALL_DATA_NACK: u8 = 0x98;

// Stop or repeated start condition received while selected
pub const TW_SR_STOP: u8 = 0xA0;

// Misc
// No state information available
pub const TW_NO_INFO: u8 = 0xF8;

// Illegal start or stop condition
pub const TW_BUS_ERROR: u8 = 0x00;
