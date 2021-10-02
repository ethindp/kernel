// SPDX-License-Identifier: MPL-2.0
use core::fmt::Arguments as FormatArguments;
use spin::{mutex::ticket::TicketMutex, Lazy};
use uart_16550::SerialPort;

/*
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub(crate) struct ScreenWriter {
    pub(crate) position: Point,
    pub(crate) color: Rgb888,
    pub(crate) buffer_info: FrameBufferInfo,
    pub(crate) buffer: FrameBuffer,
}

impl DrawTarget for ScreenWriter {

pub(crate) static WRITER: Once<ScreenWriter> = Once::initialized({
        let text_mode = Text80x25::new();
        text_mode.set_mode();
        text_mode.clear_screen();
        ScreenWriter {
            column: 0,
            color: (Color16::White, Color16::Black),
            buffer: text_mode,
        }
    });
*/
pub(crate) static SERIAL_WRITER: Lazy<TicketMutex<SerialPort>> = Lazy::new(|| {
    TicketMutex::new({
        let mut serial_port = unsafe { SerialPort::new(0x3F8) };
        serial_port.init();
        serial_port
    })
});

/*
impl fmt::Write for ScreenWriter {
    fn write_str(&mut self, what: &str) -> fmt::Result {
        self.write(what);
        Ok(())
    }
}


#[macro_export]
macro_rules! print {
($($arg:tt)*) => ($crate::vga::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
() => ($crate::print!("\n"));
($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}
*/

#[macro_export]
macro_rules! sprint {
($($arg:tt)*) => ($crate::graphics::_sprint(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! sprintln {
() => ($crate::sprint!("\n"));
($fmt:expr) => ($crate::sprint!(concat!($fmt, "\n")));
($fmt:expr, $($arg:tt)*) => ($crate::sprint!(concat!($fmt, "\n"), $($arg)*));
}

#[macro_export]
macro_rules! printk {
($($arg:tt)*) => {
{
//$crate::graphics::_print(format_args!($($arg)*));
$crate::graphics::_sprint(format_args!($($arg)*));
}
}
}

#[macro_export]
macro_rules! printkln {
() => ($crate::printk!("\n"));
($($arg:tt)*) => ($crate::printk!("{}\n", format_args!($($arg)*)));
}

/*
#[doc(hidden)]
pub(crate) fn _print(args: fmt::Arguments) {
    use core::fmt::Write;
    use x86_64::instructions::interrupts;
    interrupts::without_interrupts(|| {
        WRITER.lock().write_fmt(args).unwrap();
    });
}
*/

#[doc(hidden)]
pub(crate) fn _sprint(args: FormatArguments) {
    use core::fmt::Write;
    use x86_64::instructions::interrupts;
    interrupts::without_interrupts(|| {
        SERIAL_WRITER
            .lock()
            .write_fmt(args)
            .expect("Could not write to serial device!");
    });
}
