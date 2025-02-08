use core::ptr::NonNull;

use ssd1306::device::SSD1306State;

#[test]
fn it_collects_command_parameters() {
    let ssd1306: &mut SSD1306State = unsafe {
        let state =
            std::alloc::alloc(std::alloc::Layout::new::<SSD1306State>()).cast::<SSD1306State>();
        NonNull::new_unchecked(state).as_mut()
    };

    unsafe {
        ssd1306.i2c_send(0x26); // A command that expects 6 parameters
        assert_eq!(ssd1306.command, 0x26);
        assert_eq!(ssd1306.parameters.len(), 0);

        ssd1306.i2c_send(2);
        ssd1306.i2c_send(3);
        ssd1306.i2c_send(4);
        assert_eq!(ssd1306.in_command, true); // we haven't sent all 6 params yet
        ssd1306.i2c_send(5);
        ssd1306.i2c_send(6);
        ssd1306.i2c_send(7);
        assert_eq!(ssd1306.in_command, false); // all command parameters were sent.
        assert_eq!(ssd1306.parameters.len(), 6);
        assert_eq!(ssd1306.parameters, [2, 3, 4, 5, 6]);

        ssd1306.i2c_send(0x81);
        assert_eq!(ssd1306.in_command, true); // we haven't sent all 6 params yet
        ssd1306.i2c_send(1);
        assert_eq!(ssd1306.in_command, false);
        assert_eq!(ssd1306.parameters.len(), 1);
        assert_eq!(ssd1306.parameters, [1]);
    }
}

#[test]
/// If the transmission ends before all params are sent, then the command is
/// considered recieved and should run with what we have.
fn it_collects_command_parameters_even_if_transmission_ends() {
    let ssd1306: &mut SSD1306State = unsafe {
        let state =
            std::alloc::alloc(std::alloc::Layout::new::<SSD1306State>()).cast::<SSD1306State>();
        NonNull::new_unchecked(state).as_mut()
    };
    unsafe {
        ssd1306.i2c_send(0x26); // one of the commands that expect 6 parameters
        assert_eq!(ssd1306.command, 0x26);
        assert_eq!(ssd1306.parameters.len(), 0);

        ssd1306.i2c_send(2);
        ssd1306.i2c_send(3);
        ssd1306.i2c_send(4);
        assert_eq!(ssd1306.in_command, true); // we haven't sent all 6 params yet
                                              // TODO: ssd1306.i2c_event(ENDED);

        assert_eq!(ssd1306.in_command, false);
        assert_eq!(ssd1306.parameters.len(), 3);
        assert_eq!(ssd1306.parameters, [2, 3, 4]);
    };
}
