#![no_std]
#![no_main]

#[macro_use]
extern crate log;
use ysos::*;
use core::arch::asm;
use ysos_kernel as ysos;


boot::entry_point!(kernel_main);

pub fn kernel_main(boot_info: &'static boot::BootInfo) -> ! {
    ysos::init(boot_info);

    loop {
        print!("> ");
        let input = input::get_line();

        match input.trim() {
            "exit" => break,
            _ => {
                println!("You said: {}", input);
            }
        }
    }

    ysos::shutdown();
}
 