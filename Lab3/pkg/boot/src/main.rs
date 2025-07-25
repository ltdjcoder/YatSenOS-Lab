#![no_std]
#![no_main]
#![feature(alloc_error_handler)]

#[macro_use]
extern crate log;
extern crate alloc;

use alloc::boxed::Box;
use alloc::vec;
use uefi::mem::memory_map::MemoryMap;
use uefi::{entry, Status};
use x86_64::registers::control::*;
use ysos_boot::*;
use fs::open_file;
use fs::load_file;

mod config;

const CONFIG_PATH: &str = "\\EFI\\BOOT\\boot.conf";

#[entry]
fn efi_main() -> Status {
    uefi::helpers::init().expect("Failed to initialize utilities");

    log::set_max_level(log::LevelFilter::Info);
    info!("Running UEFI bootloader...");

    // 1. Load config fix-me
    let config = { 
        let mut file = open_file(CONFIG_PATH);
        let buf = load_file(&mut file);
        config::Config::parse(buf)
     };

    info!("Config: {:#x?}", config);

    // Elf::parse(buf);
    // 2. Load ELF files fix-me
    let mut kernel_file = open_file(config.kernel_path);
    let kernel_buf: &'static mut [u8] = load_file(&mut kernel_file);
    let elf = { 
        xmas_elf::ElfFile::new(&kernel_buf).unwrap()
    };

    unsafe {
        set_entry(elf.header.pt2.entry_point() as usize);
    }

    // 3. Load MemoryMap fix-me
    let mmap = uefi::boot::memory_map(uefi::boot::MemoryType::LOADER_DATA).expect("Failed to get memory map");

    let max_phys_addr = mmap
        .entries()
        .map(|m| m.phys_start + m.page_count * 0x1000)
        .max()
        .unwrap()
        .max(0x1_0000_0000); // include IOAPIC MMIO area

    // 4. Map ELF segments, kernel stack and physical memory to virtual memory
    let mut page_table = current_page_table();
    let mut frame_allocator = allocator::UEFIFrameAllocator;

    // FIX-ME: root page table is readonly, disable write protect (Cr0)
    unsafe {
        let mut cr0 = Cr0::read();
        cr0.remove(Cr0Flags::WRITE_PROTECT);
        Cr0::write(cr0);
    }

    // FIX-ME: map physical memory to specific virtual address offset
    elf::map_physical_memory(
        config.physical_memory_offset,
        max_phys_addr,
        &mut page_table,
        &mut frame_allocator,
    );

    // FIX-ME: load and map the kernel elf file
    elf::load_elf(&elf, config.physical_memory_offset, &mut page_table, &mut frame_allocator)
        .expect("Failed to load ELF");

    // FIX-ME: map kernel stack
    elf::map_range(
        config.kernel_stack_address,
        config.kernel_stack_size,
        &mut page_table,
        &mut frame_allocator,
    ).expect("Failed to map kernel stack");

    // FIX-ME: recover write protect (Cr0)
    unsafe {
        let mut cr0 = Cr0::read();
        cr0.insert(Cr0Flags::WRITE_PROTECT);
        Cr0::write(cr0);
    }

    free_elf(elf);

    // 5. Pass system table to kernel
    let ptr = uefi::table::system_table_raw().expect("Failed to get system table");
    let system_table = ptr.cast::<core::ffi::c_void>();


    // 6. Exit boot and jump to ELF entry
    info!("Exiting boot services...");

    let mmap = unsafe { uefi::boot::exit_boot_services(MemoryType::LOADER_DATA) };
    // NOTE: alloc & log are no longer available

    // construct BootInfo
    let bootinfo = BootInfo {
        memory_map: mmap.entries().copied().collect(),
        physical_memory_offset: config.physical_memory_offset,
        system_table,
    };

    // align stack to 8 bytes
    let stacktop = config.kernel_stack_address + config.kernel_stack_size * 0x1000 - 8;

    jump_to_entry(&bootinfo, stacktop);
}
