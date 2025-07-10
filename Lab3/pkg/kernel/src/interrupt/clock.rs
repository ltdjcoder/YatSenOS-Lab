use crate::interrupt::consts::Interrupts;
use x86_64::structures::{gdt, idt::{InterruptDescriptorTable, InterruptStackFrame}};
use core::sync::atomic::{AtomicUsize, Ordering};
use crate::interrupt::consts::Irq;

//FIX-ME
pub unsafe fn register_idt(idt: &mut InterruptDescriptorTable) {
    idt[Interrupts::IrqBase as u8 + Irq::Timer as u8]
        .set_handler_fn(clock_interrupt_handler);
}


// 使用原子操作
static COUNTER: AtomicUsize = AtomicUsize::new(0);
extern "x86-interrupt" fn clock_interrupt_handler(_stack_frame: InterruptStackFrame) {
    x86_64::instructions::interrupts::without_interrupts(|| {
        if inc_counter() % 0x10 == 0 {
            // info!("Tick! @{}", read_counter());
        }
        super::ack();
    });
}


// FIX-ME
#[inline]
pub fn read_counter() -> usize {
    COUNTER.load(Ordering::Relaxed)
}

#[inline]
pub fn inc_counter() -> usize {
    COUNTER.fetch_add(1, Ordering::Relaxed)
}