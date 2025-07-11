use crate::interrupt::consts::Interrupts;
use crate::proc::{self, ProcessContext};
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};
use core::sync::atomic::{AtomicUsize, Ordering};
use crate::interrupt::consts::Irq;
use crate::memory::gdt::CLOCK_INTERRUPT_IST_INDEX;
use crate::utils::regs;
use crate::proc::get_process_manager;

//FIX-ME
pub unsafe fn register_idt(idt: &mut InterruptDescriptorTable) {
    idt[Interrupts::IrqBase as u8 + Irq::Timer as u8]
        .set_handler_fn(clock_interrupt_handler)
        .set_stack_index(CLOCK_INTERRUPT_IST_INDEX); 
}



// extern "x86-interrupt" fn clock_interrupt(_stack_frame: InterruptStackFrame) {
//     x86_64::instructions::interrupts::without_interrupts(|| {
//         let reg_value = regs::get_current_registers();
//         let mut context = ProcessContext::new(_stack_frame, reg_value);
//         proc::switch();
//         super::ack();
//     });
// }

// as_handler!(clock_interrupt);


pub unsafe extern "C" fn clock_interrupt(mut context: ProcessContext) {
    x86_64::instructions::interrupts::without_interrupts(|| {
        info!("Clock interrupt happen.");
        // proc::switch(&mut context);
        context = get_process_manager().switch_next(&mut context);
        super::ack();
    });
}

as_handler!(clock_interrupt);