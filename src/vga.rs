// SPDX-License-Identifier: MPL-2.0
use core::fmt;
use lazy_static::lazy_static;
use spin::Mutex;
use uart_16550::SerialPort;
use vga::colors::{Color16, TextModeColor};
use vga::writers::{ScreenCharacter, Text80x25, TextWriter};

const VGA_HEIGHT: usize = 25;
const VGA_WIDTH: usize = 80;

#[allow(missing_debug_implementations)]
pub(crate) struct ScreenWriter {
    pub(crate) column: usize,
    pub(crate) color: (Color16, Color16),
    pub(crate) buffer: Text80x25,
}

impl ScreenWriter {
    pub(crate) fn write_char(&mut self, character: u8) {
        match character {
            b'\n' => self.write_newline(),
            character => {
                if self.column >= VGA_WIDTH {
                    self.write_newline();
                }
                let row = VGA_HEIGHT - 1;
                let col = self.column;
                let color = self.color;
                let color = TextModeColor::new(color.0, color.1);
                let schar = ScreenCharacter::new(character, color);
                self.buffer.write_character(row, col, schar);
                self.column += 1;
            }
        }
    }

    pub(crate) fn write_newline(&mut self) {
        for row in 1..VGA_HEIGHT {
            for col in 0..VGA_WIDTH {
                let character = self.buffer.read_character(row, col);
                self.buffer.write_character(row - 1, col, character);
            }
        }
        self.clear_row(VGA_HEIGHT - 1);
        self.column = 0;
    }

    pub(crate) fn clear_row(&mut self, row: usize) {
        let color = TextModeColor::new(Color16::Black, Color16::Black);
        let schar = ScreenCharacter::new(b' ', color);
        for col in 0..VGA_WIDTH {
            self.buffer.write_character(row, col, schar);
        }
    }

    pub(crate) fn write(&mut self, what: &str) {
        for byte in what.bytes() {
            match byte {
                0x20..=0x7e | b'\n' => self.write_char(byte),
                // Do nothing if we get an unprintable char
                _ => (),
            }
        }
    }
}

lazy_static! {
    pub(crate) static ref WRITER: Mutex<ScreenWriter> = Mutex::new({
        let text_mode = Text80x25::new();
        text_mode.set_mode();
        text_mode.clear_screen();
        ScreenWriter {
            column: 0,
            color: (Color16::White, Color16::Black),
            buffer: text_mode,
        }
    });
    pub(crate) static ref SERIAL_WRITER: Mutex<SerialPort> = Mutex::new({
        let mut serial_port = unsafe { SerialPort::new(0x3F8) };
        serial_port.init();
        serial_port
    });
}

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

#[macro_export]
macro_rules! sprint {
($($arg:tt)*) => ($crate::vga::_sprint(format_args!($($arg)*)));
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
$crate::vga::_print(format_args!($($arg)*));
$crate::vga::_sprint(format_args!($($arg)*));
}
}
}

#[macro_export]
macro_rules! printkln {
() => ($crate::printk!("\n"));
($($arg:tt)*) => ($crate::printk!("{}\n", format_args!($($arg)*)));
}

#[doc(hidden)]
pub(crate) fn _print(args: fmt::Arguments) {
    use core::fmt::Write;
    use x86_64::instructions::interrupts;
    interrupts::without_interrupts(|| {
        WRITER.lock().write_fmt(args).unwrap();
    });
}

#[doc(hidden)]
pub(crate) fn _sprint(args: ::core::fmt::Arguments) {
    use core::fmt::Write;
    use x86_64::instructions::interrupts;
    interrupts::without_interrupts(|| {
        SERIAL_WRITER
            .lock()
            .write_fmt(args)
            .expect("Could not write to serial device!");
    });
}
