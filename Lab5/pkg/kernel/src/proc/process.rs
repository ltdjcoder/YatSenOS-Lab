use core::mem::take;

use super::*;
use crate::memory::*;
use alloc::sync::{Arc, Weak};
use alloc::vec::Vec;
use spin::*;
use x86_64::structures::paging::mapper::MapToError;
use x86_64::structures::paging::page::PageRange;
use x86_64::structures::paging::*;
use x86_64::VirtAddr;
use x86_64::registers::control::Cr3;
use crate::proc::paging::PageTableContext;
use crate::proc::ProcessContext;
use x86_64::instructions::tlb;


#[derive(Clone)]
pub struct Process {
    pid: ProcessId,
    inner: Arc<RwLock<ProcessInner>>,
}

pub struct ProcessInner {
    name: String,
    parent: Option<Weak<Process>>,
    children: Vec<Arc<Process>>,
    ticks_passed: usize,
    status: ProgramStatus,
    context: ProcessContext,
    exit_code: Option<isize>,
    proc_data: Option<ProcessData>,
    proc_vm: Option<ProcessVm>,
}

impl Process {
    #[inline]
    pub fn pid(&self) -> ProcessId {
        self.pid
    }

    #[inline]
    pub fn write(&self) -> RwLockWriteGuard<ProcessInner> {
        self.inner.write()
    }

    #[inline]
    pub fn read(&self) -> RwLockReadGuard<ProcessInner> {
        self.inner.read()
    }

    pub fn new(
        name: String,
        parent: Option<Weak<Process>>,
        proc_vm: Option<ProcessVm>,
        proc_data: Option<ProcessData>,
    ) -> Arc<Self> {
        let name = name.to_ascii_lowercase();

        // create context
        let pid = ProcessId::new();
        let proc_vm = proc_vm.unwrap_or_else(|| ProcessVm::new(PageTableContext::new()));

        let inner = ProcessInner {
            name,
            parent,
            status: ProgramStatus::Ready,
            context: ProcessContext::default(),
            ticks_passed: 0,
            exit_code: None,
            children: Vec::new(),
            proc_vm: Some(proc_vm),
            proc_data: Some(proc_data.unwrap_or_default()),
        };

        trace!("New process {}#{} created.", &inner.name, pid);

        // create process struct
        Arc::new(Self {
            pid,
            inner: Arc::new(RwLock::new(inner)),
        })
    }

    pub fn kill(&self, ret: isize) {
        // let mut inner = self.inner.write();

        debug!(
            "Killing process {}#{} with ret code: {}",
            self.inner.write().name(),
            self.pid,
            ret
        );

        self.inner.write().kill(ret);
    }

    pub fn alloc_init_stack(&self) -> VirtAddr {
        self.write().vm_mut().init_proc_stack(self.pid)
    }
}

impl ProcessInner {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn tick(&mut self) {
        self.ticks_passed += 1;
    }

    pub fn status(&self) -> ProgramStatus {
        self.status
    }

    pub fn set_status(&mut self, status: ProgramStatus) {
        self.status = status;
    }

    pub fn pause(&mut self) {
        self.status = ProgramStatus::Ready;
    }

    pub fn resume(&mut self) {
        self.status = ProgramStatus::Running;
    }

    pub fn exit_code(&self) -> Option<isize> {
        self.exit_code
    }

    pub fn clone_page_table(&self) -> PageTableContext {
        // FIX-ME
        self.proc_vm.as_ref().unwrap().page_table.clone_level_4()
    }

    pub fn is_ready(&self) -> bool {
        self.status == ProgramStatus::Ready
    }

    pub fn vm(&self) -> &ProcessVm {
        self.proc_vm.as_ref().unwrap()
    }

    pub fn vm_mut(&mut self) -> &mut ProcessVm {
        self.proc_vm.as_mut().unwrap()
    }

    pub fn handle_page_fault(&mut self, addr: VirtAddr) -> bool {
        self.vm_mut().handle_page_fault(addr)
    }

    pub fn load_elf(&mut self, elf: &boot::ElfFile) {
        self.vm_mut().load_elf(elf);
    }

    /// Save the process's context
    /// mark the process as ready 为什么？
    pub(super) fn save(&mut self, context: &ProcessContext) {
        // FIXME: save the process's context
        self.context = context.clone();
    }

    /// Restore the process's context
    /// mark the process as running
    /// 这里的“存储”应该是指加载到运行环境，那为何要传入参数？
    // pub(super) fn restore(&mut self, context: &mut ProcessContext) {
    //pub(super) fn restore(&mut self){
        // FIX-ME: restore the process's context
        // FIX-ME: restore the process's page table
    /// Restores the process context.
    ///
    /// This function is unsafe because it directly manipulates registers and memory to switch contexts.

    pub unsafe fn restore(&self) -> ProcessContext {
        // 1. Switch Page Table
        let page_table_addr = self.vm().page_table.page_table_addr();
        let (frame, _) = Cr3::read();
        if frame.start_address().as_u64() != page_table_addr {
            unsafe {
                Cr3::write(
                    x86_64::structures::paging::PhysFrame::from_start_address(
                        x86_64::PhysAddr::new(page_table_addr),
                    )
                    .unwrap(),
                    x86_64::registers::control::Cr3Flags::empty(),
                );
                // When cr3 is switched, the tlb should be refreshed
                tlb::flush_all();
            }
        }

        // 2. Restore registers by overwriting the context on the interrupt stack.
        // The `as_handler` macro will then pop these values into the registers
        // and `iretq` will use the restored stack frame.
        self.context
    }

    pub fn parent(&self) -> Option<Arc<Process>> {
        self.parent.as_ref().and_then(|p| p.upgrade())
    }

    pub fn kill(&mut self, ret: isize) {
        self.exit_code = Some(ret);
        self.status = ProgramStatus::Dead;

        // take(&mut self.proc_vm);
        // self.proc_data.take();
        self.proc_vm = None;
        self.proc_data = None;
    }

    pub fn get_proc_context(& mut self) -> & mut ProcessContext {
        & mut self.context
    }
}

impl core::ops::Deref for Process {
    type Target = Arc<RwLock<ProcessInner>>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl core::ops::Deref for ProcessInner {
    type Target = ProcessData;

    fn deref(&self) -> &Self::Target {
        self.proc_data
            .as_ref()
            .expect("Process data empty. The process may be killed.")
    }
}

impl core::ops::DerefMut for ProcessInner {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.proc_data
            .as_mut()
            .expect("Process data empty. The process may be killed.")
    }
}


impl core::fmt::Debug for Process {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        let inner = self.inner.read();
        f.debug_struct("Process")
            .field("pid", &self.pid)
            .field("name", &inner.name)
            .field("parent", &inner.parent().map(|p| p.pid))
            .field("status", &inner.status)
            .field("ticks_passed", &inner.ticks_passed)
            .field("children", &inner.children.iter().map(|c| c.pid.0))
            .field("status", &inner.status)
            .field("context", &inner.context)
            .field("vm", &inner.proc_vm)
            .finish()
    }
}

impl core::fmt::Display for Process {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        let inner = self.inner.read();
        write!(
            f,
            " #{:-3} | #{:-3} | {:12} | {:7} | {:?}",
            self.pid.0,
            inner.parent().map(|p| p.pid.0).unwrap_or(0),
            inner.name,
            inner.ticks_passed,
            inner.status
        )?;
        Ok(())
    }
}
