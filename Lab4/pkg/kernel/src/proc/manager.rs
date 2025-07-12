use super::*;
use crate::{memory::{
    self,
    allocator::{ALLOCATOR, HEAP_SIZE},
    get_frame_alloc_for_sure, PAGE_SIZE,
}, proc::vm::stack};
use alloc::{collections::*, format, sync::Weak};
use arrayvec::ArrayVec;
use boot::{App, AppList, ElfFile};
use spin::{Mutex, RwLock};
use x86::current;
use x86_64::{structures::idt::InterruptStackFrame, VirtAddr};
use alloc::sync::Arc;

pub static PROCESS_MANAGER: spin::Once<ProcessManager> = spin::Once::new();

pub fn init(init: Arc<Process>, app_list:Option<&'static ArrayVec<App<'static>, 16>>) {

    // FIX-ME: set init process as Running
    init.write().set_status(ProgramStatus::Running);

    // FIX-ME: set processor's current pid to init's pid
    processor::set_pid(init.pid());

    PROCESS_MANAGER.call_once(|| ProcessManager::new(init, app_list));
}

pub fn get_process_manager() -> &'static ProcessManager {
    PROCESS_MANAGER
        .get()
        .expect("Process Manager has not been initialized")
}

pub struct ProcessManager {
    processes: RwLock<BTreeMap<ProcessId, Arc<Process>>>,
    ready_queue: Mutex<VecDeque<ProcessId>>,
    app_list: Option<&'static ArrayVec<App<'static>, 16>>,//FIXME
}

impl ProcessManager {
    pub fn new(init: Arc<Process>, app_list: Option<&'static ArrayVec<App<'static>, 16>>) -> Self {
        let mut processes = BTreeMap::new();
        let ready_queue = VecDeque::new();
        let pid = init.pid();

        trace!("Init {:#?}", init);

        processes.insert(pid, init);
        Self {
            processes: RwLock::new(processes),
            ready_queue: Mutex::new(ready_queue),
            app_list,
        }
    }

    #[inline]
    pub fn push_ready(&self, pid: ProcessId) {
        self.ready_queue.lock().push_back(pid);
    }

    #[inline]
    fn add_proc(&self, pid: ProcessId, proc: Arc<Process>) {
        self.processes.write().insert(pid, proc);
    }

    #[inline]
    pub fn get_proc(&self, pid: &ProcessId) -> Option<Arc<Process>> {
        self.processes.read().get(pid).cloned()
    }

    pub fn current(&self) -> Arc<Process> {
        self.get_proc(&processor::get_pid())
            .expect("No current process")
    }

    pub fn save_current(&self, context: &ProcessContext) {
        // FIX-ME: update current process's tick count
        // self.current().write().tick(); // 在这里加1 ？ 不干

        // FIX-ME: save current process's context
        self.current().write().save(context);

    }

    // FIX-ME: switch to the next process
        //      - save current process's context
        //      - handle ready queue update
        //      - restore next process's context
        // 我直接大搬迁
        // 换下一个：先存这个，在找下个，加载下个，全都考虑队列
    pub unsafe fn switch_next(&self, context: &mut ProcessContext) -> ProcessContext {

        // self.current().write().set_status(ProgramStatus::Ready);
        if(self.current().read().status() == ProgramStatus::Running) {
            self.current().write().pause();
            self.save_current(context);
            self.ready_queue.lock().push_back(processor::get_pid());
        }
        

        // FIX-ME: fetch the next process from ready queue
        let mut ready_queue = self.ready_queue.lock();
        let mut next_pid_result = ready_queue.pop_front();
        let mut next_pid = ProcessId(0);


        // FIX-ME: check if the next process is ready,
        //        continue to fetch if not ready
        while let Some(pid) = next_pid_result {
            if let Some(proc) = self.get_proc(&pid) {
                if proc.read().status() == ProgramStatus::Ready {
                    next_pid = pid;
                    break;
                }else{
                    if proc.read().status() != ProgramStatus::Dead {
                        self.ready_queue.lock().push_back(pid);
                    }
                }
            }
            // If not ready, continue to fetch the next one
            next_pid_result = ready_queue.pop_front();
        }

        let mut context = context.clone();

        // FIX-ME: restore next process's context
        if let Some(proc) = self.get_proc(&next_pid) {
            context = unsafe { proc.as_ref().write().restore() };
            // proc.as_ref().write().set_status(ProgramStatus::Running);
            proc.as_ref().write().resume();
        } else {
            warn!("No process found for pid: {}", next_pid);
            return context; 
        }

        // FIX-ME: update processor's current pid
        processor::set_pid(next_pid);

        // print_process_list();

        // FIX-ME: return next process's pid
        context
    }

    pub fn spawn_kernel_thread(
        &self,
        entry: VirtAddr,
        name: String,
        proc_data: Option<ProcessData>,
    ) -> ProcessId {
        let kproc = self.get_proc(&KERNEL_PID).unwrap();
        let page_table = kproc.read().clone_page_table();
        let proc_vm = Some(ProcessVm::new(page_table));
        let proc = Process::new(name, Some(Arc::downgrade(&kproc)), proc_vm, proc_data);
        let pid = proc.pid();

        // alloc stack for the new process base on pid
        let stack_top = proc.alloc_init_stack();

        // FIXME: set the stack frame
        proc.as_ref().write().get_proc_context().init_stack_frame(entry, stack_top);

        // FIXME: add to process map
        self.add_proc(pid, proc);

        // FIXME: push to ready queue
        self.push_ready(pid);

        // FIXME: return new process pid
        pid
    }

    

    pub fn kill_current(&self, ret: isize) {
        self.kill(processor::get_pid(), ret);
    }

    pub fn handle_page_fault(&self, addr: VirtAddr, err_code: PageFaultErrorCode) -> bool {
        // FIXME: handle page fault
        let current_proc = self.current();
        // let mut proc_writer = current_proc.write();
        // 检查缺页异常是否是由于写入一个不存在的页面引起的
        // info!("Handling page fault at {:#?} with error code: {:?}", addr, err_code);
        if !err_code.contains(PageFaultErrorCode::CAUSED_BY_WRITE)
        {
            // 我们只处理由写入操作引起的缺页异常
            info!("Page fault not caused by write operation, ignoring.");
            return true;
        }

        if(err_code.contains(PageFaultErrorCode::PROTECTION_VIOLATION)) {
            info!("Page fault caused by protection violation, ignoring.");
            return false;
        }

        if let vm = current_proc.as_ref().write().vm_mut() {
            return vm.handle_page_fault(addr);
        }

        false
    }

    pub fn kill(&self, pid: ProcessId, ret: isize) {
        let proc = self.get_proc(&pid);

        if proc.is_none() {
            warn!("Process #{} not found.", pid);
            return;
        }

        let proc = proc.unwrap();

        if proc.read().status() == ProgramStatus::Dead {
            warn!("Process #{} is already dead.", pid);
            return;
        }

        trace!("Kill {:#?}", &proc);

        proc.kill(ret);
    }

    pub fn print_process_list(&self) {
        let mut output = String::from("  PID | PPID | Process Name |  Ticks  | Status\n");

        self.processes
            .read()
            .values()
            .filter(|p| p.read().status() != ProgramStatus::Dead)
            .for_each(|p| output += format!("{}\n", p).as_str());

        // TODO: print memory usage of kernel heap

        output += format!("Queue  : {:?}\n", self.ready_queue.lock()).as_str();

        output += &processor::print_processors();

        print!("{}", output);
    }

    pub fn app_list(&self) -> Option<&'static ArrayVec<App<'static>, 16>> {
        self.app_list
    }


    pub fn spawn(
        &self,
        elf: &ElfFile,
        name: String,
        parent: Option<Weak<Process>>,
        proc_data: Option<ProcessData>,
    ) -> ProcessId {
        let kproc = self.get_proc(&KERNEL_PID).unwrap();
        let page_table = kproc.read().clone_page_table();
        let proc_vm = Some(ProcessVm::new(page_table));
        let proc = Process::new(name, parent, proc_vm, proc_data);

        // let mut inner = proc.write();
        
        // FIXME: load elf to process pagetable
        proc.write().load_elf(elf);
        
        // FIXME: alloc new stack for process
        let stack_top = proc.alloc_init_stack();
        
        // Get ELF entry point
        let entry_point = VirtAddr::new(elf.header.pt2.entry_point());
        
        // Set up the process context with entry point and stack for user process
        proc.write().get_proc_context().init_user_stack_frame(entry_point, stack_top);
        
        // FIXME: mark process as ready
        proc.write().set_status(ProgramStatus::Ready);
        
        drop(proc.write());

        trace!("New {:#?}", &proc);

        let pid = proc.pid();
        
        // FIXME: something like kernel thread
        // Add to process map
        self.add_proc(pid, proc);
        
        // Push to ready queue
        self.push_ready(pid);

        pid
    }

    pub fn read(&self, fd: u8, buf: &mut [u8]) -> isize {
        self.current().read().read(fd, buf)
    }

    pub fn write(&self, fd: u8, buf: &[u8]) -> isize {
        self.current().read().write(fd, buf)
    }

    

}


