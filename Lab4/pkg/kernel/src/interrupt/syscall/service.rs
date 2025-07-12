use core::alloc::Layout;
use alloc::sync::Arc;
use alloc::string::ToString;

use crate::proc;
use crate::proc::*;
use super::SyscallArgs;

pub fn get_current_pid() -> usize {
    // FIXME: get current pid
    get_process_manager().current().pid().0 as usize
}

pub fn spawn_process(args: &SyscallArgs) -> usize {
    // FIXME: get app name by args
    //       - core::str::from_utf8_unchecked
    //       - core::slice::from_raw_parts
    // FIXME: spawn the process by name
    // FIXME: handle spawn error, return 0 if failed
    // FIXME: return pid as usize
    
    let name_ptr = args.arg0 as *const u8;
    let name_len = args.arg1;
    
    if name_ptr.is_null() || name_len == 0 {
        return 0;
    }
    
    let name_slice = unsafe { core::slice::from_raw_parts(name_ptr, name_len) };
    let name = unsafe { core::str::from_utf8_unchecked(name_slice) };
    
    // Try to find the app in the app list
    if let Some(apps) = get_process_manager().app_list() {
        for app in apps.iter() {
            if app.name.as_str() == name {
                // Get current process as parent
                let current_proc = get_process_manager().current();
                let parent = Some(Arc::downgrade(&current_proc));
                
                // Spawn the process
                let pid = get_process_manager().spawn(&app.elf, name.to_string(), parent, None);
                return pid.0 as usize;
            }
        }
    }
    
    0 // Return 0 if app not found or spawn failed
}

pub fn sys_write(args: &SyscallArgs) -> usize {
    // FIXME: get buffer and fd by args
    //       - core::slice::from_raw_parts
    // FIXME: call proc::write -> isize
    // FIXME: return the result as usize

    let fd = args.arg0 as u8;
    let buf_ptr = args.arg1 as *const u8;
    let buf_len = args.arg2;


    let buf = unsafe { core::slice::from_raw_parts(buf_ptr, buf_len) };

    let result = crate::proc::write(fd, buf);
    
    if result < 0 {
        0  
    } else {
        result as usize
    }
}

pub fn sys_read(args: &SyscallArgs) -> usize {
    // FIXME: just like sys_write

    let fd = args.arg0 as u8;
    let buf_ptr = args.arg1 as *mut u8;
    let buf_len = args.arg2;

    let buf = unsafe { core::slice::from_raw_parts_mut(buf_ptr, buf_len) };


    let result = crate::proc::read(fd, buf);
    
    if result < 0 {
        0  // Return 0 on error or no data available
    } else {
        result as usize
    }
}

pub fn exit_process(args: &SyscallArgs, context: &mut ProcessContext) {
    // FIXME: exit process with retcode
    // Get the exit code from the syscall arguments
    // proc::exit(ret, context);
    let ret = args.arg0 as isize;
    info!("Exiting process with return code: {}", ret);
    
    unsafe { 
        // get_process_manager().current().write().set_status(ProgramStatus::Dead); 
        proc::exit(ret, context);
        *context = get_process_manager().switch_next(context);
    }
}

pub fn stat_process() {
    // List all processes using ProcessManager's print_process_list method
    get_process_manager().print_process_list();
}

pub fn list_apps() {
    // List all available applications
    if let Some(apps) = get_process_manager().app_list() {
        println!("Available applications:");
        for (index, app) in apps.iter().enumerate() {
            println!("  [{}] {}", index, app.name);
        }
    } else {
        println!("No applications available.");
    }
}

pub fn sys_allocate(args: &SyscallArgs) -> usize {
    let layout = unsafe { (args.arg0 as *const Layout).as_ref().unwrap() };

    if layout.size() == 0 {
        return 0;
    }

    let ret = crate::memory::user::USER_ALLOCATOR
        .lock()
        .allocate_first_fit(*layout);

    match ret {
        Ok(ptr) => ptr.as_ptr() as usize,
        Err(_) => 0,
    }
}

pub fn sys_deallocate(args: &SyscallArgs) {
    let layout = unsafe { (args.arg1 as *const Layout).as_ref().unwrap() };

    if args.arg0 == 0 || layout.size() == 0 {
        return;
    }

    let ptr = args.arg0 as *mut u8;

    unsafe {
        crate::memory::user::USER_ALLOCATOR
            .lock()
            .deallocate(core::ptr::NonNull::new_unchecked(ptr), *layout);
    }
}

pub fn wait_pid(args: &SyscallArgs) -> usize {
    // FIXME: check if the process is running or get retcode
    let pid = ProcessId(args.arg0 as u16);
    
    // Get the process
    if let Some(proc) = get_process_manager().get_proc(&pid) {
        let proc_inner = proc.read();
        
        // Check if the process is dead and return exit code
        if proc_inner.status() == ProgramStatus::Dead {
            if let Some(exit_code) = proc_inner.exit_code() {
                return exit_code as usize;
            }
        }
        
        // Process is still running or blocked, return a special value
        // In a real implementation, this would block until the process exits
        return usize::MAX; // Indicates process is still running
    }
    
    0 // Process not found
}
