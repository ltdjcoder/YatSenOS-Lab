#![no_std]
#![no_main]

use log::info;
use ysos::*;
use ysos_kernel::{self as ysos, proc::ProcessId};

extern crate alloc;

boot::entry_point!(kernel_main);

pub fn kernel_main(boot_info: &'static boot::BootInfo) -> ! {
    ysos::init(boot_info);
    ysos::wait(spawn_init());
    ysos::shutdown();
}

pub fn spawn_init() -> proc::ProcessId {
    // NOTE: you may want to clear the screen before starting the shell
    // print!("\x1b[1;1H\x1b[2J");

    proc::list_app();
    proc::spawn("sh").unwrap()
    // ProcessId(0)
}
