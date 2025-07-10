mod apic;
mod consts;
// mod clock;
// mod serial;
mod exceptions;
mod clock;
mod serial;

use apic::*;
use x86_64::structures::idt::InterruptDescriptorTable;
use crate::{interrupt::consts::Irq, memory::physical_to_virtual};

lazy_static! {
    // FIX-ME
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        unsafe {
            exceptions::register_idt(&mut idt);
            // TODO: clock::register_idt(&mut idt);
            clock::register_idt(&mut idt);
            // TODO: serial::register_idt(&mut idt);
            serial::register_idt(&mut idt);
        }
        idt
    };
}

/// init interrupts system
pub fn init() {
    IDT.load();

    // FIXME: check and init APIC
    if XApic::support() {
        let mut lapic = unsafe { XApic::new(physical_to_virtual(LAPIC_ADDR)) };
        lapic.cpu_init();
    } else {
        panic!("Local APIC not supported!");
    }

    // FIXME: enable serial irq with IO APIC (use enable_irq)
    let mut ioapic = unsafe { IoApic::new(physical_to_virtual(IOAPIC_ADDR)) };
    ioapic.enable(consts::Irq::Serial0 as u8, 0);
    ioapic.enable(consts::Irq::Serial1 as u8, 0);

    enable_irq(Irq::Serial0 as u8, 0);


    info!("Interrupts Initialized.");
}

#[inline(always)]
pub fn enable_irq(irq: u8, cpuid: u8) {
    let mut ioapic = unsafe { IoApic::new(physical_to_virtual(IOAPIC_ADDR)) };
    ioapic.enable(irq, cpuid);
}

#[inline(always)]
pub fn ack() {
    let mut lapic = unsafe { XApic::new(physical_to_virtual(LAPIC_ADDR)) };
    lapic.eoi();
}
