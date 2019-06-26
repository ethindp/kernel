use crate::vga::WRITER;
use crate::{printk, printkln};
use alloc::string::String;
use kernel::drivers::hid::keyboard::read;
use pc_keyboard::KeyCode;
use x86_64::instructions::hlt;

/// Initialize the text user interface
pub fn init() {
    printkln!("Kernel control and command console (KC3)");
    printkln!("Type 'help' for a list of commands");
    printkln!("*** WARNING ***\nYou are currently operating in kernel mode. Remember that with power comes\nresponsibility, and you are responsible for any damages that arise out of your\nmisuse of this console, your mishandling of devices and ports, your failure to\nfollow documentation, etc.");
    cmd_loop();
}

fn cmd_loop() {
    printk!("KC3> ");
    let mut command_str = String::new();
    loop {
        let (character, keycode) = match read() {
            Some(c) => c,
            None => (None, None),
        };
        if character.is_some() {
            if character.unwrap() != '\n' {
                command_str.push(character.unwrap());
                printk!("{}", character.unwrap());
            } else {
                printk!("\n");
                process_cmd(command_str.as_str());
                command_str.clear();
                printk!("KC3> ");
            }
        }
        if keycode.is_some() {
            if keycode.unwrap() == KeyCode::Backspace || keycode.unwrap() == KeyCode::Delete {
                if command_str.len() > 0 {
                    command_str.pop();
                    // This is dangerous but required
                    WRITER.lock().column -= 1;
                    printk!(" ");
                    WRITER.lock().column -= 1;
                }
            }
        }
        hlt();
    }
}

fn process_cmd(command: &str) {
    match command {
        "help" => {
            printkln!("This console has no commands quite yet.");
        }
        _ => {
            printkln!("Invalid command");
        }
    }
}
