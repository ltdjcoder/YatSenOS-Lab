use core::alloc::Layout;

use crate::proc;
use crate::proc::*;
use crate::utils::*;
use super::SyscallArgs;

pub fn spawn_process(args: &SyscallArgs) -> usize {
    // FIXME: get app name by args
    //       - core::str::from_utf8_unchecked
    //       - core::slice::from_raw_parts
    // FIXME: spawn the process by name
    // FIXME: handle spawn error, return 0 if failed
    // FIXME: return pid as usize

    0
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
    let ret = args.arg0 as isize;
    info!("Exiting process with return code: {}", ret);
    proc::exit(ret, context);
}

pub fn list_process() {
    // FIXME: list all processes
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
