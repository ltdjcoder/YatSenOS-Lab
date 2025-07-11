use crate::{input, interrupt::{self, consts::Interrupts}, serial::get_serial_for_sure};
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};

//FIX-ME

pub unsafe fn register_idt(idt: &mut InterruptDescriptorTable) {
    idt[Interrupts::IrqBase as u8 + interrupt::Irq::Serial0 as u8]
        .set_handler_fn(serial_interrupt_handler);
}


pub extern "x86-interrupt" fn serial_interrupt_handler(_st: InterruptStackFrame) {
    receive();
    super::ack();
}

/// Receive character from uart 16550
/// Should be called on every interrupt
fn receive() {
    // FIX-ME: receive character from uart 16550, put it into INPUT_BUFFER
    let result;
    {
        let mut serial = get_serial_for_sure();
        result = serial.receive();
    }
    if let Some(key) = result {
        input::push_key(key);
        // print!("{}", key as char);
    } else {
    }
}