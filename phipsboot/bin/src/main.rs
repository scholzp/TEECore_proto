#![feature(abi_x86_interrupt)]
#![no_main]
#![no_std]

// #![feature(error_in_core)]

// extern crate alloc;

extern crate alloc;

mod asm;
mod driver;
mod env;
mod extern_symbols;
mod idt;
mod mem;
mod xen_pvh;

use crate::mem::stack;
use alloc::alloc::{dealloc, alloc, Layout};
use core::fmt::Write;
use core::hint::black_box;
use core::panic::PanicInfo;
use lib::logger;
use x86::{msr, apic};
use x86_64::addr::VirtAddr;

/// Entry into the high-level code of the loader.
///
/// # Machine State
/// - 64-bit long mode with 4-level paging
/// - `CR0` has the following bits set: PE (0), WP (1), PG (31)
/// - `CR3` holds the physical address of the root page table
/// - `CR4` has the following bits set: PAE (5)
///
/// # Paging
/// The hole loader is reachable via its link address (2 MiB mapping) and via
/// an identity mapping of the physical location in memory.
#[no_mangle]
extern "C" fn rust_entry64(
    bootloader_magic: u64,
    bootloader_info_ptr: u64,
    load_addr_offset: i64,
) -> ! {
    // The order of the init functions mostly reflect actual dependencies!
    idt::init();
    mem::init(load_addr_offset);
    logger::init(); // after mem init; logger depends on heap!
    logger::add_backend(driver::DebugconLogger::default()).unwrap();
    logger::add_backend(driver::SerialLogger::default()).unwrap();
    logger::flush(); // flush all buffered messages
    log::info!("Logging works");

    env::init(bootloader_magic, bootloader_info_ptr);

    env::print();
    stack::assert_sanity_checks();

    log::info!("Now loading your kernel into 64-bit mode...");
    log::info!("Not implemented yet! =(");

    // break_stack();
    //create_pagefault();

    // Read the APIC's address from the respective MSR
    let apic_base_content = unsafe {
        msr::rdmsr(msr::APIC_BASE)
    };
    log::info!("APIC Base content: {:#016x}", apic_base_content);
    let apic_page = apic_base_content & (!0x0FFFu64);
    log::info!("APIC page: {:#016x}", apic_page);
    // Map the APIC page
    let layout = Layout::from_size_align(4096, 4096).expect("Layout should work");
    let ptr = unsafe {alloc(layout) as u64 };
    log::info!("Allocated memory at {:016x?}", ptr);
    let mut lapic_v_address : u64 = 0xffffffff88200000;
    let mut l0 : *const u8 = crate::extern_symbols::boot_mem_pt_l1_hi().cast::<u8>();
    let mut l1 : *const u8 = unsafe { l0.add(load_addr_offset as usize) };
    // l1 += 4;
    unsafe {
        log::info!("Link address high: {:?}", crate::extern_symbols::link_addr_high_base());
        log::info!("L1 entry address: {:#016x?}", l0);
        log::info!("L1 entry address: {:#016x?}", l1);
        log::info!("offset {:x}", load_addr_offset);
        let mut l1_ptr : * mut u64 = l1 as *mut u64;
        let l4_index : usize = 1;
        lapic_v_address += 0x1000 * (l4_index as u64);
        log::info!("LAPIC vaddr: {:016x?}", lapic_v_address);
        log::info!("L1 entry address: {:?}", l1_ptr.add(l4_index));
        core::ptr::write(l1_ptr.add(l4_index), apic_page | (0x1b));
        x86_64::instructions::tlb::flush_all();
        log::info!("Entry in L1 0x{:x?} at address 0x{:x?}", core::ptr::read(l1_ptr.add(l4_index)), l1_ptr.add(l4_index));
        log::info!("LAPIC version register address 0x{:x?}", (lapic_v_address + 0x30));
        // let apic = apic::xapic::XAPIC::new(lapic_v_address);
        log::info!("LAPIC version 0x{:x?}", core::ptr::read((lapic_v_address + 0x30) as *const u32));
        log::info!("Sending IPI:");
        let icr_l : u64= 0x300;
        core::ptr::write((lapic_v_address + icr_l) as *mut u32, 0xc0400);

    }
    log::info!("Still alive");
    loop {}
}

/// Sometimes useful to test the stack + stack canary.
#[allow(unused, unconditional_recursion)]
#[inline(never)]
fn break_stack() {
    log::debug!("Breaking stack ...");
    stack::assert_sanity_checks();
    log::debug!("stack usage: {:#.2?}", stack::usage());
    break_stack();
}

/// Sometimes useful to test the binary.
#[allow(unused)]
fn create_pagefault() {
    log::debug!("Creating page fault ...");
    let ptr = core::ptr::null::<u8>();
    unsafe {
        black_box(core::ptr::read_volatile(ptr));
    }
}

#[panic_handler]
fn panic_handler(info: &PanicInfo) -> ! {
    // If a panic happens, we are screwed anyways. We do some additional
    // emergency logging without the whole log-stack
    let _ = writeln!(&mut driver::DebugconLogger, "PANIC: {info:#?}");

    // log::error!("PANIC: {info:#?}");

    unsafe {
        // TODO only do this when no logging is initialized?!
        core::arch::asm!("ud2", in("rax") 0xbadb001);
    }
    loop {}
}
