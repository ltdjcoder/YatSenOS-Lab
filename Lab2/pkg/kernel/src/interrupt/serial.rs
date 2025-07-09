use crate::{interrupt::{self, consts::Interrupts}, serial::get_serial_for_sure};
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};

//FIX-ME

pub unsafe fn register_idt(idt: &mut InterruptDescriptorTable) {
    idt[Interrupts::IrqBase as u8 + interrupt::Irq::Serial0 as u8]
        .set_handler_fn(serial_interrupt_handler);
}


extern "x86-interrupt" fn serial_interrupt_handler(_stack_frame: InterruptStackFrame) {
    // 处理串口中断
    x86_64::instructions::interrupts::without_interrupts(|| {
        
        print!("Received data from serial:[ ");
        loop
        {
            let result;
            {
                let mut serial = get_serial_for_sure();
                result = serial.receive();
            }
            if let Some(byte) = result {
                print!("{}", byte as char);
            } else {
                break;
            }
        }
        println!("]");
        super::ack();
    });
}