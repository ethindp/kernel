extern crate lazy_static;
extern crate uart_16550;
extern crate volatile;
use core::fmt;
use lazy_static::lazy_static;
use spin::Mutex;
use uart_16550::SerialPort;
use volatile::Volatile;

const VGA_HEIGHT: usize = 25;
const VGA_WIDTH: usize = 80;

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Color {
    Black = 0,
    Blue = 1,
    Green = 2,
    Cyan = 3,
    Red = 4,
    Magenta = 5,
    Brown = 6,
    LightGray = 7,
    DarkGray = 8,
    LightBlue = 9,
    LightGreen = 10,
    LightCyan = 11,
    LightRed = 12,
    Pink = 13,
    Yellow = 14,
    White = 15,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
struct ColorCode(u8);

impl ColorCode {
    fn new(fg: Color, bg: Color) -> ColorCode {
        ColorCode((bg as u8) << 4 | (fg as u8))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
struct DrawableChar {
    ascii: u8,
    color: ColorCode,
}

#[repr(transparent)]
struct VgaBuffer {
    characters: [[Volatile<DrawableChar>; VGA_WIDTH]; VGA_HEIGHT],
}

pub struct ScreenWriter {
    pub column: usize,
    color: ColorCode,
    buffer: &'static mut VgaBuffer,
}

impl ScreenWriter {
    pub fn write_char(&mut self, character: u8) {
        match character {
            b'\n' => self.write_newline(),
            character => {
                if self.column >= VGA_WIDTH {
                    self.write_newline();
                }
                let row = VGA_HEIGHT - 1;
                let col = self.column;
                let color = self.color;
                self.buffer.characters[row][col].write(DrawableChar {
                    ascii: character,
                    color,
                });
                self.column += 1;
            }
        }
    }

    pub fn write_newline(&mut self) {
        for row in 1..VGA_HEIGHT {
            for col in 0..VGA_WIDTH {
                let character = self.buffer.characters[row][col].read();
                self.buffer.characters[row - 1][col].write(character);
            }
        }
        self.clear_row(VGA_HEIGHT - 1);
        self.column = 0;
    }

    pub fn clear_row(&mut self, row: usize) {
        let nothing = DrawableChar {
            ascii: b' ',
            color: self.color,
        };
        for col in 0..VGA_WIDTH {
            self.buffer.characters[row][col].write(nothing);
        }
    }

    pub fn write(&mut self, what: &str) {
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
    pub static ref WRITER: Mutex<ScreenWriter> = Mutex::new(ScreenWriter {
        column: 0,
        color: ColorCode::new(Color::White, Color::Black),
        buffer: unsafe { &mut *(0xb8000 as *mut VgaBuffer) },
    });
    pub static ref SERIAL_WRITER: Mutex<SerialPort> = Mutex::new({
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
pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;
    use x86_64::instructions::interrupts;
    interrupts::without_interrupts(|| {
        WRITER.lock().write_fmt(args).unwrap();
    });
}

#[doc(hidden)]
pub fn _sprint(args: ::core::fmt::Arguments) {
    use core::fmt::Write;
    use x86_64::instructions::interrupts;
    interrupts::without_interrupts(|| {
        SERIAL_WRITER
            .lock()
            .write_fmt(args)
            .expect("Could not write to serial device!");
    });
}