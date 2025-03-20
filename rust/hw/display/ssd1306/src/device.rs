use core::ptr::NonNull;
use std::{
    ffi::{c_void, CStr},
    ptr::{addr_of, addr_of_mut},
};

use qemu_api::{
    bindings::{
        error_fatal, qdev_new, qdev_prop_set_chr, qemu_console_surface, qemu_irq,
        sysbus_connect_irq, sysbus_mmio_map, sysbus_realize_and_unref, Chardev, DisplaySurface,
        Error, I2CSlave,
    },
    c_str,
    i2cslave::I2CSlaveImpl,
    qdev::{DeviceImpl, DeviceState, Property, ResetType, ResettablePhasesImpl},
    qom::{IsA, Object, ObjectImpl, ObjectType, ParentField},
    qom_isa,
    vmstate::VMStateDescription,
};
use qemu_api_macros::Object;

const WIDTH: usize = 128;
const HEIGHT: usize = 64;
const MAGNIFY: usize = 1;
const I2C_ADDRESS: u8 = 0x3d;

#[inline(always)]
/// TODO: document this, and check if bindgen supports exporting C macros
const fn _pixman_format_reshift(val: u32, ofs: u32, num: u32) -> u32 {
    ((val >> ofs) & ((1 << num) - 1)) << ((val >> 22) & 3)
}

#[derive(Debug)]
/// The addressing mode used for the GDDRAM.
pub enum MemoryAddressingMode {
    /// After the display RAM is read/written, the column address pointer is
    /// increased automatically by 1. **If the column address pointer
    /// reaches column end address, the column address pointer is
    /// reset to column start address and page address pointer is increased by
    /// 1**
    Horizontal,
    /// After the display RAM is read/written, the page address pointer is
    /// increased automatically by 1. If the page address pointer reaches
    /// the page end address, the page address pointer is reset
    /// to page start address and column address pointer is increased by 1
    Vertical,
    /// After the display RAM is read/written, the column address pointer is
    /// increased automatically by 1. **If the column address pointer
    /// reaches column end address, the column address pointer is
    /// reset to column start address and page address pointer is not changed**
    Page,
}

#[derive(Debug)]
/// The control byte (D/C# bit).
///
/// This determines whether the current I2C transfer contains a command or
/// GDDRAM data.
pub enum DataMode {
    /// The following byte(s) are interpreted as commands.
    /// Corresponds to D/C# = 0 in the control byte.
    Command,
    /// The following byte(s) are stored in the GDDRAM.
    /// Corresponds to D/C# = 1 in the control byte.
    Data,
}

#[repr(C)]
#[derive(Debug, Object, qemu_api_macros::offsets)]
pub struct SSD1306State {
    pub parent_obj: ParentField<I2CSlave>,
    pub i2c: I2CSlave,
    pub console: *mut qemu_api::bindings::QemuConsole,
    pub someprop: char,

    /// Indicates whether the command is still waiting for additional
    /// parameters.
    ///
    /// Commands may receive additional bytes via I2C. The command itself and
    /// its subsequent "parameters" are sent together. The `in_command` flag
    /// is `true` when the system is still waiting for more parameters, and
    /// it is `false` once all parameters have been received or the I2C
    /// transaction ends.
    ///
    /// Note: in the datasheet, these additional bytes are referred to as
    /// A/B/C/D/E/F
    pub in_command: bool,

    /// This holds the parameters for the current command
    pub parameters: Vec<u8>,

    pub command: u8,
    pub params_number: usize,
    /// Graphic Display Data RAM (GDDRAM).
    ///
    /// The GDDRAM is a bit mapped static RAM holding the bit pattern to be
    /// displayed. The size of the RAM is **128 x 64 bits** and the RAM is
    /// divided into eight pages, from PAGE0 to PAGE7.
    ///
    /// Each byte in the array directly corresponds to a column in a specific
    /// page.
    pub gddram: [u8; 128 * 8],
    pub memory_addressing_mode: MemoryAddressingMode,
    pub column_start_address: u8,
    pub column_end_address: u8,
    pub column_address_pointer: u8,
    pub page_start_address: u8,
    pub page_end_address: u8,
    pub page_address_pointer: u8,
    /// Whether we have received D/C yet for the current transfer
    /// Note: each transfer beings with a D/C
    pub recieved_dc: bool,
    pub display_enabled: bool,
    pub multiplex_ratio: u8,
    pub display_offset: u8,
    pub display_start_line: u8,
    pub segment_remap_enabled: bool,
    pub com_remap_enabled: bool,
    pub force_display_on: bool,
    /// An inverted display will treat 1 in the RAM as OFF, and 0 as ON.
    pub display_inverted: bool,
    /// The data mode for the current transfer
    pub data_mode: DataMode,
}

unsafe impl ObjectType for SSD1306State {
    type Class = SSD1306Class;
    const TYPE_NAME: &'static CStr = crate::TYPE_SSD1306;
}

impl ObjectImpl for SSD1306State {
    type ParentType = I2CSlave;

    const INSTANCE_INIT: Option<unsafe fn(&mut Self)> = Some(Self::init);
    const INSTANCE_POST_INIT: Option<fn(&Self)> = None;
    const CLASS_INIT: fn(&mut Self::Class) = Self::Class::class_init::<Self>;
}

trait SSD1306Impl: I2CSlaveImpl + IsA<SSD1306State> {}
impl SSD1306Impl for SSD1306State {}
impl I2CSlaveImpl for SSD1306State {}
impl DeviceImpl for SSD1306State {
    fn properties() -> &'static [Property] {
        &crate::device_class::SSD1306_PROPERTIES
    }
    fn vmsd() -> Option<&'static VMStateDescription> {
        Some(&crate::device_class::VMSTATE_SSD1306)
    }
    const REALIZE: Option<fn(&Self)> = Some(Self::realize);
}

impl ResettablePhasesImpl for SSD1306State {
    const HOLD: Option<fn(&Self, ResetType)> = Some(Self::reset_hold);
}

pub extern "C" fn ssd1306_update_display(opaque: *mut std::os::raw::c_void) {
    unsafe {
        let mut state = NonNull::new_unchecked(opaque.cast::<SSD1306State>());
        state.as_mut().update_display();
    }
}

static mut SSD1306_OPS: qemu_api::bindings::GraphicHwOps = qemu_api::bindings::GraphicHwOps {
    get_flags: None,
    invalidate: None,
    gfx_update: Some(ssd1306_update_display),
    gfx_update_async: true,
    text_update: None,
    ui_info: None,
    gl_block: None,
};

impl SSD1306State {
    /// Initializes a pre-allocated, unitialized instance of `SSD1306State`.
    ///
    /// # Safety
    ///
    /// `self` must point to a correctly sized and aligned location for the
    /// `SSD1306State` type. It must not be called more than once on the same
    /// location/instance. All its fields are expected to hold unitialized
    /// values with the sole exception of `parent_obj`.
    pub unsafe fn init(&mut self) {
        println!("init ssd1306");
        // TODO: move to reset
        self.parameters = Vec::with_capacity(10);
        self.gddram = [0; HEIGHT * (WIDTH / 8)];
        unsafe {
            println!("setting ssd1306 address");
            qemu_api::bindings::i2c_slave_set_address(&mut self.i2c as *mut _, I2C_ADDRESS);
        }

        unsafe {
            // TODO: this was done quickly, make sure it is safe later
            self.console = qemu_api::bindings::graphic_console_init(
                (self as *mut Self).cast::<qemu_api::bindings::DeviceState>(),
                0,
                addr_of_mut!(SSD1306_OPS),
                (self as *mut Self).cast::<c_void>(),
            );
            qemu_api::bindings::qemu_console_resize(
                self.console,
                (WIDTH * MAGNIFY) as i32,
                (HEIGHT * MAGNIFY) as i32,
            );
        }
    }
    pub fn realize(&self) {
        println!("realize ssd1306");
    }

    pub fn reset_hold(&self, _type: ResetType) {
        println!("reset ssd1306");
        // TODO: implement all comments with "RESET"
        // TODO: USE CELL/OR CHECK WHY RESET DOES NOT TAKE A MUTABLE REF
        //unsafe {
        //    let a = addr_of!(self) as *mut Self;
        //    let a = &mut *a as &mut Self;
        //    a.in_command = false;
        //    a.parameters = Vec::with_capacity(10);
        //    a.memory_addressing_mode = MemoryAddressingMode::Page;
        //}
    }

    pub fn update_display(&mut self) {
        let surface: &DisplaySurface = unsafe {
            NonNull::new(qemu_console_surface(self.console))
                .expect("display surface pointer is null")
                .as_ref()
        };
        // Calculate dest width
        //let _dest_width = unsafe {
        //    // PIXMAN_FORMAT_BPP
        //    // TODO: understand how this function works later.
        //    let surface_bits_per_pixel = pixman_format_reshift(
        //        qemu_api::bindings::pixman_image_get_format(surface.image),
        //        24,
        //        8,
        //    );
        //    dbg!(surface_bits_per_pixel);
        //    match surface_bits_per_pixel {
        //        0 => return,
        //        15 | 16 => 2,
        //        24 => 3,
        //        32 => 4,
        //        _ => {
        //            // TODO: Error handling
        //            eprintln!("BAD COLOR DIPTH: {}", surface_bits_per_pixel);
        //            return;
        //        }
        //    }
        //};
        //let _dest_width = dest_width * MAGNIFY;

        let data = unsafe { qemu_api::bindings::pixman_image_get_data(surface.image) };
        for y in 0..HEIGHT {
            for x in 0..WIDTH {
                let page = y / 8;
                let byte_index = page * WIDTH + x;
                let bit_position = y % 8;
                let pixel_val = (self.gddram[byte_index] >> bit_position) & 1;

                let pixel_offset = y * WIDTH + x; // Map (x, y) to framebuffer index
                unsafe {
                    *data.add(pixel_offset) = if pixel_val == 0 {
                        0 // Black pixel
                    } else {
                        0xffffffff // Color in ARGB (Blue pixel) 
                    };
                }
            }
        }
    }

    /// # Safety
    pub unsafe fn i2c_recv(&mut self) -> u8 {
        println!("I2C RECV");
        b'a'
    }

    /// Returns the number of parameters a command expects.
    ///
    /// For example, if the command 0x26 is recieved, we expect 6 bytes to be
    /// sent over I2C.
    const fn get_number_of_parameters(&self) -> usize {
        match self.command {
            0x81 | 0x1 | 0x20 | 0xd3 | 0xda | 0xd5 | 0xd9 | 0xdb | 0xa8 | 0x8d => 1,
            0x26 | 0x27 => 6,
            0x29 | 0x2a => 5,
            0xa3 | 0x21 | 0x22 => 2,
            _ => 0,
        }
    }

    // When this is called, the command parameters are in `self.parameters`
    //
    // Note: it is possible that the I2C transaction was ended before recieving all
    // parameters, so the vector may be incomplete.
    fn command(&mut self) {
        dbg!(self.display_inverted);
        dbg!(self.com_remap_enabled);
        dbg!(self.segment_remap_enabled);
        dbg!(self.multiplex_ratio);
        dbg!(self.column_start_address);
        dbg!(self.column_end_address);
        dbg!(self.page_start_address);
        dbg!(self.page_end_address);

        //println!("command: {}", self.command);
        match self.command {
            // --- Fundamental Commands ---
            //
            // Set Contrast Control
            //
            // Contrast increases as the value increases.
            // Reset: 0x7f
            0x81 => {
                // TODO: expects an A[7:0] byte
                if let Some(_contrast) = self.parameters.first() {
                    // TODO: Should i emulate contrast?
                } else {
                    eprintln!("Expected parameter `contrast`");
                }
            }
            // Entire Display ON
            0xa4 => {
                // resume display to GDDRAM content
                self.force_display_on = false;
            }
            0xa5 => {
                // force entire display on, lit every pixel
                self.force_display_on = true;
            }
            // Set Normal/Inverse Display
            0xa6 => {
                self.display_inverted = false;
                println!("display is NOT inverted");
            }
            0xa7 => {
                self.display_inverted = true;
                println!("display inverted");
            }
            0xae => {
                self.display_enabled = false;
            }
            0xaf => {
                self.display_enabled = true;
            }
            // --- Scrolling Commands ---
            //
            // TODO TODO TODO
            //
            // --- Addressing Setting Commands ---
            //
            // Set the lower nibble of the column start address
            // register for Page Addressing Mode using X[3:0]
            // as data bits. The initial display line register is
            // reset to 0000b after RESET.
            0x00..=0x0f => {
                todo!();
            }
            // Set the higher nibble of the column start address
            // register for Page Addressing Mode using X[3:0]
            // as data bits. The initial display line register is
            // reset to 0000b after RESET.
            0x10..=0x1f => {
                let high_nibble = self.command & 0x0f;
                println!(
                    "setting high nibble for colummn start address: {:b}",
                    high_nibble
                );
                println!("{:#?}", self.memory_addressing_mode);
                todo!();
            }
            // Set Memory Addressing Mode
            0x20 => {
                // the 2 LSBs are the mode.
                if let Some(data) = self.parameters.first() {
                    match data & 0x3 {
                        // Horizontal Addressing Mode
                        0x0 => {
                            self.memory_addressing_mode = MemoryAddressingMode::Horizontal;
                        }
                        // Vertical Addressing Mode
                        0x1 => {
                            self.memory_addressing_mode = MemoryAddressingMode::Vertical;
                        }
                        // Page Addressing Mode (RESET)
                        0x2 => {
                            self.memory_addressing_mode = MemoryAddressingMode::Page;
                        }
                        // Invalid
                        _ => {
                            eprintln!("Invalid memory addressing mode")
                        }
                    }
                }
            }
            // Set Column Address
            0x21 => {
                if matches!(self.memory_addressing_mode, MemoryAddressingMode::Page) {
                    eprintln!("Setting the column address is not allowed in page addressing mode");
                    return;
                }

                if let Some(address) = self.parameters.first() {
                    self.column_start_address = address & 0b01111111;
                    println!("set column start address: {}", self.column_start_address);
                } else {
                    eprintln!("expected column start address");
                };
                if let Some(address) = self.parameters.get(1) {
                    self.column_end_address = address & 0b01111111;
                    println!("set column end address: {}", self.column_end_address);
                } else {
                    eprintln!("expected column end address");
                };
            }
            // Set Page Address
            //
            // Note: This command is only for horizontal or vertical addressing mode
            0x22 => {
                if matches!(self.memory_addressing_mode, MemoryAddressingMode::Page) {
                    eprintln!(
                        "setting the page start and end address is not supported in Page memory \
                         addressing."
                    );
                    return;
                }
                if let Some(start) = self.parameters.first() {
                    self.page_start_address = start & 0b00000111;
                } else {
                    eprintln!("expected parameter `page_start_address`");
                }
                if let Some(end) = self.parameters.get(1) {
                    self.page_end_address = end & 0b00000111;
                } else {
                    eprintln!("expected parameter `page_start_address`");
                }
            }
            // Set Page Start Address for Page Addressing Mode.
            //
            // Set GDDRAM Page Start Address (PAGE0~PAGE7) for Page Addressing Mode using X[2:0].
            //
            // Note: This command is only for page addressing mode
            0xb0..=0xb7 => {
                todo!();
            }
            // --- Hardware Configuration (Panel resolution & layout related) Commands ---
            //
            // Set Display Start Line using
            0x40..=0x7f => {
                // TODO: using X[5:0]
                // TODO: RESET = 0
                self.display_start_line = self.command & 0b00111111;
                println!("setting display start line: {}", self.display_start_line);
            }
            // Set Segment Re-map
            0xa0 => {
                self.segment_remap_enabled = false;
                println!("segment remap disabled")
            }
            0xa1 => {
                self.segment_remap_enabled = true;
                println!("segment remap enabled")
            }
            // Set Multiplex Ratio.
            //
            // Set MUX ratio to N+1 MUX
            0xa8 => {
                // TODO: RESET = 0b111111 (63d)
                if let Some(ratio) = self.parameters.first() {
                    let ratio = ratio & 0b00111111;
                    if !(16..=63).contains(&ratio) {
                        eprintln!("unsupported multiplex ratio: {}", ratio);
                    } else {
                        println!("setting mux ratio to: {}", ratio);
                        self.multiplex_ratio = ratio;
                    }
                } else {
                    eprintln!("expected a parameter");
                }
            }
            // Set COM Output Scan Direction
            0xc0 => {
                self.com_remap_enabled = false;
                println!("COM remap disabled");
            }
            0xc8 => {
                self.com_remap_enabled = true;
                println!("COM remap enabled");
            }
            // Set Display Offset
            0xd3 => {
                // TODO: expects A[5:0]
                // Set vertical shift by COM from 0d~63d The value is reset to
                // 00h after RESET.
                if let Some(offset) = self.parameters.first() {
                    let offset = offset & 0b00111111;
                    if !(0..=63).contains(&offset) {
                        eprintln!("unsupported display offset: {}", offset);
                    } else {
                        println!("setting display offset to: {}", offset);
                        self.display_offset = offset;
                    }
                } else {
                    eprintln!("Expected a parameter");
                }
            }
            // Set COM Pins Hardware Configuration
            0xda => {
                // TODO: expects A[5:4]
                // A[4]=0b, Sequential COM pin configuration
                // A[4]=1b(RESET), Alternative COM pin
                // configuration
                // A[5]=0b(RESET), Disable COM Left/Right
                // remap
                // A[5]=1b, Enable COM Left/Right remap
            }
            // --- Timing & Driving Scheme Setting Commands ---
            //
            // Set Display Clock Divide Ratio/Oscillator Frequency
            0xd5 => {
                // TODO: expects A[7:0]
                if let Some(_ratio) = self.parameters.first() {
                    // NOOP
                    // TODO: remove this print
                    println!("Setting Clock Divide Ratio")
                } else {
                    eprintln!("Expected parameter `ratio`");
                }
            }
            // Set Pre-charge Period
            0xd9 => {
                if let Some(_period) = self.parameters.first() {
                    // NOOP
                } else {
                    eprintln!("Expected parameter `period`");
                }
            }
            // Set V COMH Deselect Level
            0xdb => {
                // NOOP
            }
            0x8d => {
                if let Some(charge_pump) = self.parameters.first() {
                    println!("setting charge pump: {}", charge_pump);
                } else {
                    eprintln!("Expected parameter `charge pump`");
                }
            }
            // NOP
            0xe3 => {}
            _ => {
                eprintln!("Unknown command: {:x}", self.command);
            }
        }
    }

    pub fn write_gddram(&mut self, data: u8) {
        match self.memory_addressing_mode {
            MemoryAddressingMode::Horizontal => {
                // 128, not column_end_address
                let index = self.page_address_pointer as usize * 128_usize
                    + self.column_address_pointer as usize;
                self.gddram[index] = data;

                self.column_address_pointer += 1;
                if self.column_address_pointer > self.column_end_address {
                    self.column_address_pointer = self.column_start_address;
                    self.page_address_pointer += 1;
                    if self.page_address_pointer > self.page_end_address {
                        self.page_address_pointer = self.page_start_address;
                    }
                }
            }
            MemoryAddressingMode::Vertical => {
                todo!()
            }
            MemoryAddressingMode::Page => {
                todo!()
            }
        }
    }

    /// Data is **sent** to the device.
    /// # Safety
    pub unsafe fn i2c_send(&mut self, data: u8) -> i32 {
        if !self.recieved_dc {
            self.recieved_dc = true;
            self.data_mode = if data & 0b01000000 == 0 {
                DataMode::Command
            } else {
                DataMode::Data
            };
            return 0; // drop the first data which is D/C
        }

        if matches!(self.data_mode, DataMode::Data) {
            self.write_gddram(data);
            return 0;
        }

        if self.in_command {
            // `data` is a parameter for the current command
            self.parameters.push(data);
            if self.parameters.len() == self.params_number {
                self.in_command = false;
                // all parameters are recieved, run the command.
                self.command();
            }
        } else {
            self.command = data;
            self.in_command = true;
            self.parameters.clear();
            self.params_number = self.get_number_of_parameters();
            if self.params_number == 0 {
                self.in_command = false;
                self.command();
            }
        }

        0 // success
    }
    pub fn i2c_event(&mut self, event: u32) -> i32 {
        if event == 1 {
            // new transfer, wait for D/C
            self.recieved_dc = false;
        }
        0
    }
}

/// # Safety
///
/// We expect the FFI user of this function to pass a valid pointer.
pub unsafe extern "C" fn ssd1306_i2c_recv(dev: *mut I2CSlave) -> u8 {
    unsafe {
        assert!(!dev.is_null());
        let mut state = NonNull::new_unchecked(dev.cast::<SSD1306State>());
        state.as_mut().i2c_recv();
    }
    1
}

/// # Safety
///
/// We expect the FFI user of this function to pass a valid pointer.
pub unsafe extern "C" fn ssd1306_i2c_send(dev: *mut I2CSlave, data: u8) -> i32 {
    unsafe {
        assert!(!dev.is_null());
        let mut state = NonNull::new_unchecked(dev.cast::<SSD1306State>());
        state.as_mut().i2c_send(data)
    }
}

/// # Safety
///
/// We expect the FFI user of this function to pass a valid pointer.
pub unsafe extern "C" fn ssd1306_i2c_event(dev: *mut I2CSlave, event: u32) -> i32 {
    unsafe {
        let mut state = NonNull::new(dev.cast::<SSD1306State>()).expect("I2Cslave pointer is null");
        state.as_mut().i2c_event(event)
    }
}

qom_isa!(SSD1306State: I2CSlave, DeviceState, Object);

#[repr(C)]
pub struct SSD1306Class {
    parent_class: <I2CSlave as ObjectType>::Class,
}

impl SSD1306Class {
    fn class_init<T: SSD1306Impl>(&mut self) {
        self.parent_class.class_init::<T>();
        println!("ssd1306 class init");
        unsafe {
            let a = self as *mut SSD1306Class;
            let mut i2c = NonNull::new_unchecked(a.cast::<qemu_api::bindings::I2CSlaveClass>());
            i2c.as_mut().recv = Some(ssd1306_i2c_recv);
            i2c.as_mut().send = Some(ssd1306_i2c_send);
            i2c.as_mut().event = Some(ssd1306_i2c_event);
        }
    }
}

/// # Safety
///
/// We expect the FFI user of this function to pass a valid pointer, that has
/// the same size as [`SSD1306State`]. We also expect the device is
/// readable/writeable from one thread at any time.
pub unsafe extern "C" fn ssd1306_init(obj: *mut Object) {
    unsafe {
        debug_assert!(!obj.is_null());
        let mut state = NonNull::new_unchecked(obj.cast::<SSD1306State>());
        state.as_mut().init();
    }
}

/// # Safety
///
/// We expect the FFI user of this function to pass a valid pointer for `chr`.
#[no_mangle]
pub unsafe extern "C" fn ssd1306_create(
    addr: u64,
    irq: qemu_irq,
    chr: *mut Chardev,
) -> *mut DeviceState {
    unsafe {
        let dev: *mut qemu_api::bindings::DeviceState = qdev_new(SSD1306State::TYPE_INFO.name);
        let sysbus = dev.cast::<qemu_api::bindings::SysBusDevice>();

        qdev_prop_set_chr(dev, c_str!("chardev").as_ptr(), chr);
        sysbus_realize_and_unref(sysbus, addr_of!(error_fatal) as *mut *mut Error);
        sysbus_mmio_map(sysbus, 0, addr);
        sysbus_connect_irq(sysbus, 0, irq);
        dev as *mut DeviceState
    }
}
