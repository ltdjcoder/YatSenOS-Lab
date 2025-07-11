use super::LocalApic;
use bit_field::BitField;
use core::fmt::{Debug, Error, Formatter};
use core::ptr::{read_volatile, write_volatile};
use x86::cpuid::CpuId;
use crate::interrupt::consts::{Interrupts, Irq};

/// Default physical address of xAPIC
pub const LAPIC_ADDR: u64 = 0xFEE00000;

// FIX-ME
// xAPIC寄存器偏移地址
const APIC_ID: u32 = 0x0020;           // APIC ID寄存器
const APIC_VERSION: u32 = 0x0030;      // 版本寄存器
const APIC_TPR: u32 = 0x0080;          // Task Priority寄存器
const APIC_EOI: u32 = 0x00B0;          // End of Interrupt寄存器
const APIC_SVR: u32 = 0x00F0;          // Spurious Interrupt Vector寄存器
const APIC_ESR: u32 = 0x0280;          // Error Status寄存器
const APIC_ICR_LOW: u32 = 0x0300;      // Interrupt Command寄存器低32位
const APIC_ICR_HIGH: u32 = 0x0310;     // Interrupt Command寄存器高32位
const APIC_LVT_TIMER: u32 = 0x0320;    // LVT Timer寄存器
const APIC_LVT_LINT0: u32 = 0x0350;    // LVT LINT0寄存器
const APIC_LVT_LINT1: u32 = 0x0360;    // LVT LINT1寄存器
const APIC_LVT_ERROR: u32 = 0x0370;    // LVT Error寄存器
const APIC_LVT_PCINT: u32 = 0x0340;    // LVT Performance Counter寄存器
const APIC_TIMER_INIT: u32 = 0x0380;   // Timer Initial Count寄存器
const APIC_TIMER_DIV: u32 = 0x03E0;    // Timer Divide寄存器

// 中断向量号
const SPURIOUS_INTERRUPT_VECTOR: u32 = 0xFF;
const ERROR_INTERRUPT_VECTOR: u32 = 0xFE;

bitflags! {
    /// Spurious Interrupt Vector Register flags
    struct SvrFlags: u32 {
        const APIC_ENABLE = 1 << 8;  // APIC软件使能位
    }
}

bitflags! {
    /// LVT (Local Vector Table) flags
    struct LvtFlags: u32 {
        const MASKED = 1 << 16;      // 中断屏蔽位
        const LEVEL_TRIGGERED = 1 << 15;  // 电平触发(vs边沿触发)
        const ACTIVE_LOW = 1 << 13;  // 低电平有效(vs高电平)
        const DELIVERY_STATUS = 1 << 12;  // 传送状态
        const VECTOR = 0xFF;  // 中断向量号
    }
}


//FIX-ME
pub struct LVT_Editor {
    pub data: u32,
}

impl LVT_Editor {
    pub fn new() -> Self {
        LVT_Editor { data: 0 }
    }

    pub fn with_data(data: u32) -> Self {
        LVT_Editor { data }
    }

    pub fn bits(&self) -> u32 {
        self.data
    }

    // 终端屏蔽
    pub fn write_masked(mut self, masked: bool) -> Self {
        if masked {
            self.data |= LvtFlags::MASKED.bits();
        } else {
            self.data &= !LvtFlags::MASKED.bits();
        }
        self
    }

    // 设置电平触发
    pub fn write_level_triggered(mut self, level_triggered: bool) -> Self {
        if level_triggered {
            self.data |= LvtFlags::LEVEL_TRIGGERED.bits();
        } else {
            self.data &= !LvtFlags::LEVEL_TRIGGERED.bits();
        }        
        self
    }

    // 设置低电平有效
    pub fn write_active_low(mut self, active_low: bool) -> Self {
        if active_low {
            self.data |= LvtFlags::ACTIVE_LOW.bits();
        } else {
            self.data &= !LvtFlags::ACTIVE_LOW.bits();      
        }
        self 
    }

    // 设置中断向量号
    pub fn write_vector(mut self, vector: u32) -> Self {
        self.data &= !LvtFlags::VECTOR.bits(); // 清除原有向量
        self.data |= vector & LvtFlags::VECTOR.bits(); // 设置新向量
        self        
    }

    pub fn write_bit(mut self, index:u32, value:bool) -> Self {
        self.data |= if value { 1 << index } else { 0 };
        self
    }

}



pub struct XApic {
    addr: u64,
}

impl XApic {
    pub unsafe fn new(addr: u64) -> Self {
        XApic { addr }
    }

    unsafe fn read(&self, reg: u32) -> u32 {
        unsafe {
            read_volatile((self.addr + reg as u64) as *const u32)
        }
    }

    unsafe fn write(&mut self, reg: u32, value: u32) {
        unsafe {
            write_volatile((self.addr + reg as u64) as *mut u32, value);
            self.read(APIC_ID);
        }
    }
}

impl LocalApic for XApic {
    /// If this type APIC is supported
    fn support() -> bool {
        // 检查CPUID是否支持APIC
        CpuId::new().get_feature_info().map(
            |f| f.has_apic()
        ).unwrap_or(false)
    }

    /// Initialize the xAPIC for the current CPU.
    fn cpu_init(&mut self) {
        unsafe {
            // FIX-ME: Enable local APIC; set spurious interrupt vector.
            // 启用Local APIC并设置伪中断向量
            // 用 bitflags 语义更加清晰
            let mut svr = self.read(APIC_SVR);
            svr |= SvrFlags::APIC_ENABLE.bits() | SPURIOUS_INTERRUPT_VECTOR;
            self.write(APIC_SVR, svr);

            // FIX-ME: The timer repeatedly counts down at bus frequency
            self.write(APIC_LVT_TIMER, LVT_Editor::new()
                .write_masked(false) // 禁用计时器中断
                .write_level_triggered(false) // 边沿触发
                .write_active_low(false) // 高电平有效
                .write_vector(Interrupts::IrqBase as u32 + Irq::Timer as u32)// 设置处理 set Timer Periodic Mode
                .write_bit(17, true)
                .bits());
            
            // FIX-ME: Disable logical interrupt lines (LINT0, LINT1)
            // 3. 禁用逻辑中断线 (LINT0, LINT1) FIX-ME 不能这样做（
            self.write(APIC_LVT_LINT0, LVT_Editor::new().write_masked(true).bits());
            self.write(APIC_LVT_LINT1, LvtFlags::MASKED.bits());
            self.write(0x350, 1 << 16); // set Mask

            // FIX-ME: Disable performance counter overflow interrupts (PCINT)
            // 4. 禁用性能计数器溢出中断 (PCINT)
            self.write(APIC_LVT_PCINT, LvtFlags::MASKED.bits());

            // FIX-ME: Map error interrupt to IRQ_ERROR.
            // 5. 映射错误中断到ERROR向量
            self.write(APIC_LVT_ERROR, ERROR_INTERRUPT_VECTOR);

            // FIX-ME: Clear error status register (requires back-to-back writes).
            // 6. 清除错误状态寄存器 (需要连续两次写入)
            self.write(APIC_ESR, 0);
            self.write(APIC_ESR, 0);

            // 7. 确认任何未处理的中断
            self.write(APIC_EOI, 0);

            // 8. 发送Init Level De-Assert来同步仲裁ID
            // 这里发送到所有处理器(不包括自己)
            self.set_icr((0x0 << 32) | (0x5 << 18) | (0x1 << 14) | 0x0);

            // 9. 设置任务优先级为0 (允许所有中断)
            self.write(APIC_TPR, 0);
        }
    }
    
    //FIX-ME
    fn id(&self) -> u32 {
        unsafe { self.read(APIC_ID) >> 24 }
    }

    //FIX-ME
    fn version(&self) -> u32 {
        unsafe { self.read(APIC_VERSION) }
    }

    //FIX-ME
    fn icr(&self) -> u64 {
        unsafe { 
            (self.read(APIC_ICR_HIGH) as u64) << 32 | self.read(APIC_ICR_LOW) as u64 
        }
    }

    //FIX-ME
    fn set_icr(&mut self, value: u64) {
        unsafe {
            // 等待直到ICR可用 (Delivery Status位为0)
            while self.read(APIC_ICR_LOW).get_bit(12) {}
            
            // 先写高32位，再写低32位
            self.write(APIC_ICR_HIGH, (value >> 32) as u32);
            self.write(APIC_ICR_LOW, value as u32);
            
            // 等待发送完成
            while self.read(APIC_ICR_LOW).get_bit(12) {}
        }
    }

    //FIX-ME
    fn eoi(&mut self) {
        unsafe {
            self.write(APIC_EOI, 0);
        }
    }
}

impl Debug for XApic {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        f.debug_struct("Xapic")
            .field("id", &self.id())
            .field("version", &self.version())
            .field("icr", &self.icr())
            .finish()
    }
}
