use core::fmt;

// 静态变量存储当前的寄存器值
static mut CURRENT_REGISTERS: RegistersValue = RegistersValue {
    r15: 0,
    r14: 0,
    r13: 0,
    r12: 0,
    r11: 0,
    r10: 0,
    r9: 0,
    r8: 0,
    rdi: 0,
    rsi: 0,
    rdx: 0,
    rcx: 0,
    rbx: 0,
    rax: 0,
    rbp: 0,
};

/// 获取当前保存的寄存器值
pub fn get_current_registers() -> RegistersValue {
    unsafe { CURRENT_REGISTERS }
}

#[repr(align(8), C)]
#[derive(Clone, Default, Copy)]
pub struct RegistersValue {
    pub r15: usize,
    pub r14: usize,
    pub r13: usize,
    pub r12: usize,
    pub r11: usize,
    pub r10: usize,
    pub r9: usize,
    pub r8: usize,
    pub rdi: usize,
    pub rsi: usize,
    pub rdx: usize,
    pub rcx: usize,
    pub rbx: usize,
    pub rax: usize,
    pub rbp: usize,
}

impl fmt::Debug for RegistersValue {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "Registers")?;
        write!(f, "r15: 0x{:016x}, ", self.r15)?;
        write!(f, "r14: 0x{:016x}, ", self.r14)?;
        writeln!(f, "r13: 0x{:016x},", self.r13)?;
        write!(f, "r12: 0x{:016x}, ", self.r12)?;
        write!(f, "r11: 0x{:016x}, ", self.r11)?;
        writeln!(f, "r10: 0x{:016x},", self.r10)?;
        write!(f, "r9 : 0x{:016x}, ", self.r9)?;
        write!(f, "r8 : 0x{:016x}, ", self.r8)?;
        writeln!(f, "rdi: 0x{:016x},", self.rdi)?;
        write!(f, "rsi: 0x{:016x}, ", self.rsi)?;
        write!(f, "rdx: 0x{:016x}, ", self.rdx)?;
        writeln!(f, "rcx: 0x{:016x},", self.rcx)?;
        write!(f, "rbx: 0x{:016x}, ", self.rbx)?;
        write!(f, "rax: 0x{:016x}, ", self.rax)?;
        write!(f, "rbp: 0x{:016x}", self.rbp)?;
        Ok(())
    }
}

#[macro_export]
macro_rules! as_handler {
    ($fn: ident) => {
        paste::item! {
            #[unsafe(naked)]
            pub extern "x86-interrupt" fn [<$fn _handler>](_sf: InterruptStackFrame) {
                unsafe{
                core::arch::naked_asm!("
                push rbp
                push rax
                push rbx
                push rcx
                push rdx
                push rsi
                push rdi
                push r8
                push r9
                push r10
                push r11
                push r12
                push r13
                push r14
                push r15
                call {}
                pop r15
                pop r14
                pop r13
                pop r12
                pop r11
                pop r10
                pop r9
                pop r8
                pop rdi
                pop rsi
                pop rdx
                pop rcx
                pop rbx
                pop rax
                pop rbp
                iretq",
                sym $fn);
            }}
        }
    };
}

/// 使用 Rust 内联汇编的输出功能
#[macro_export]
macro_rules! as_handler_with_output {
    ($fn: ident) => {
        paste::item! {
            #[unsafe(naked)]
            pub extern "x86-interrupt" fn [<$fn _handler_out>](_sf: InterruptStackFrame) {
                core::arch::naked_asm!("
                // 直接将寄存器值输出到静态变量
                mov {}, r15
                mov {}, r14  
                mov {}, r13
                mov {}, r12
                mov {}, r11
                mov {}, r10
                mov {}, r9
                mov {}, r8
                mov {}, rdi
                mov {}, rsi
                mov {}, rdx
                mov {}, rcx
                mov {}, rbx
                mov {}, rax
                mov {}, rbp
                
                // 然后正常保存到栈
                push rbp
                push rax
                push rbx
                push rcx
                push rdx
                push rsi
                push rdi
                push r8
                push r9
                push r10
                push r11
                push r12
                push r13
                push r14
                push r15
                
                call {}
                
                // 恢复寄存器
                pop r15
                pop r14
                pop r13
                pop r12
                pop r11
                pop r10
                pop r9
                pop r8
                pop rdi
                pop rsi
                pop rdx
                pop rcx
                pop rbx
                pop rax
                pop rbp
                iretq",
                out("r15") unsafe { CURRENT_REGISTERS.r15 },
                out("r14") unsafe { CURRENT_REGISTERS.r14 },
                out("r13") unsafe { CURRENT_REGISTERS.r13 },
                out("r12") unsafe { CURRENT_REGISTERS.r12 },
                out("r11") unsafe { CURRENT_REGISTERS.r11 },
                out("r10") unsafe { CURRENT_REGISTERS.r10 },
                out("r9") unsafe { CURRENT_REGISTERS.r9 },
                out("r8") unsafe { CURRENT_REGISTERS.r8 },
                out("rdi") unsafe { CURRENT_REGISTERS.rdi },
                out("rsi") unsafe { CURRENT_REGISTERS.rsi },
                out("rdx") unsafe { CURRENT_REGISTERS.rdx },
                out("rcx") unsafe { CURRENT_REGISTERS.rcx },
                out("rbx") unsafe { CURRENT_REGISTERS.rbx },
                out("rax") unsafe { CURRENT_REGISTERS.rax },
                out("rbp") unsafe { CURRENT_REGISTERS.rbp },
                sym $fn);
            }
        }
    };
}
