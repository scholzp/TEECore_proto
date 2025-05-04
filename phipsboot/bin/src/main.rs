#![feature(abi_x86_interrupt)]
#![no_main]
#![no_std]

// #![feature(error_in_core)]

extern crate alloc;

mod asm;
mod driver;
mod env;
mod extern_symbols;
mod idt;
mod mem;
mod xen_pvh;
mod shared_mem_com;
mod state_machine;

use crate::mem::stack;
use alloc::alloc::{dealloc, alloc, Layout};
use core::fmt::Write;
use core::hint::black_box;
use core::ops::Deref;
use core::panic::PanicInfo;
use lib::logger;
use lib::mem::paging;
use lib::mem::paging::{PhysAddr, VirtAddr};
use x86::{msr, apic};
use x86::cpuid::CpuId;
use multiboot2::{BootInformation, BootInformationHeader, MemoryAreaTypeId};
use crate::state_machine::*;
use crate::state_machine::task::init_task_map;
use crate::state_machine::pmc;

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
    x86_64::instructions::interrupts::enable();
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
    let l1_addr = crate::extern_symbols::boot_symbol_to_high_address(crate::extern_symbols::boot_mem_pt_l1_hi());
    // make the L! page table available for translation
    unsafe { paging::use_l1_page_table(l1_addr as u64) };
    unsafe {
        log::info!("{:?} {:?}", PhysAddr::from(apic_page), VirtAddr::from(crate::extern_symbols::link_addr_high_base() as u64));
        let virt_lapic = unsafe {
            paging::map_phys_rel_base_addr(
                PhysAddr::from(apic_page),
                1,
                VirtAddr::from(l1_addr),
                0x3 | (0x1 << 5)
            )
        };
        log::info!("Virtual lapic after map_phys_rel_base_addr(): {:#016x?}", Into::<u64>::into(virt_lapic));
        let lvt_pcr_offset: u64 = 0x340;
        let lvt_pcr_ptr: *mut u32 = (Into::<u64>::into(virt_lapic) + lvt_pcr_offset) as *mut u32;
        let lvt_pcr_content = core::ptr::read(lvt_pcr_ptr);
        log::info!("Content of LVT PCR: {:#010x}", lvt_pcr_content);
        // Check if LVT PCR is supported. If so, the read should not return 0 because Mask bit is either set or
        // the vector field is not zero.
        if 0 != lvt_pcr_content {
            // Content to write to LVT PCR: Masked | Delivery Mode | Vector
            let lvt_pcr_new : u32 = 0x0_u32 | 0b0 << 16 | 0b100 << 8 | 0x40;
            core::ptr::write(lvt_pcr_ptr, lvt_pcr_new);
            log::info!("Updated LVT PCR to: {:#010x}", core::ptr::read(lvt_pcr_ptr));
        }
    }

    let mbi_addr : u64 = *(env::BOOT_INFO_PTR.get().unwrap());
    let mbi_virt = unsafe { Into::<u64>::into(
        paging::map_phys_rel_base_addr(
            PhysAddr::from(mbi_addr),
            1,
            VirtAddr::from(l1_addr),
            0x3
        ))
    };
    log::info!("MBI virtual address: {:#016x?}", mbi_virt);
    let boot_info = unsafe { BootInformation::load( mbi_virt as *const BootInformationHeader) };
    unsafe {
        log::info!("MBI header size: {:?}", (*(mbi_virt as *const BootInformationHeader)).total_size());
    }

    let binding = boot_info.unwrap();
    let mmap_shared_entry = binding
            .memory_map_tag()
            .expect("This setup contains a memory map")
            .memory_areas()
            .iter()
            .find(|area| area.typ() == MemoryAreaTypeId::from(7))
            .expect("Setup should contain shared memory");

    let shared_mem_virt = unsafe { Into::<u64>::into(
        paging::map_phys_rel_base_addr(
            PhysAddr::from(mmap_shared_entry.start_address()),
            (mmap_shared_entry.size() / 4096) as usize,
            VirtAddr::from(l1_addr),
            0x3 | (0x1 << 4) // present, RW, CD
        ))
    };

    log::info!("Virt addr of shared mem: {:#016x?}", shared_mem_virt);
    log::info!("Phys addr of shared mem: {:#016x?}", unsafe { paging::get_physical_address(shared_mem_virt) });
    unsafe {core::ptr::write(shared_mem_virt as *mut u8, 1); }
    log::info!("Set message byte to: {:?}", unsafe {core::ptr::read(shared_mem_virt as *mut u8)});

    let mut shared_mem_communicator = unsafe {
        shared_mem_com::SharedMemCommunicator::from_raw_parts(
            shared_mem_virt as *mut u8,
            mmap_shared_entry.size() as usize,
        )
    };
    init_task_map();

    // unsafe {
    //     use alloc::alloc::{alloc, Layout};
    //     use core::ptr;
    //     use core::arch::asm;

    //     let data_ptr = unsafe {
    //         alloc(Layout::from_size_align(1024 * 64, 4096).unwrap()) as *mut u32
    //     };
    //     let mut items = 1024;
    //     for a in 0..6 {
    //         log::info!("Num 32 bit elements: {:?}", items);
    //         for b in 0..5 {
    //             pmc::setup_pmcs();
    //             for c in 0..(items / 4)
    //             {
    //                 asm!(
    //                     "clflush [{tpm}]",
    //                     tpm = in(reg) (data_ptr.add(c) as *const u8),
    //                 );
    //             }
    //             asm!("mfence");
    //             for c in 0..(items / 4)
    //             {
    //                 ptr::write_volatile(data_ptr.add(c), ptr::read_volatile(data_ptr.add(c)) as u32);
    //             }
    //             pmc::read_and_print_pmcs();
    //         }
    //         items = items * 2;
    //     }
    // }
    // log::info!("Number of supported counters: {:?}",
    //     CpuId::new().get_performance_monitoring_info().unwrap().number_of_counters());
    let mut state_machine = state_machine::StateMachine::<state_machine::StateInitialized>::new(shared_mem_communicator);
    unsafe { log::info!("Hash of memory: {:#016x?}", paging::touch_all_present_pages() )};
    loop {
        pmc::setup_pmcs();
        state_machine = state_machine::run_state_machine(state_machine);
        pmc::read_and_print_pmcs();
    }
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
