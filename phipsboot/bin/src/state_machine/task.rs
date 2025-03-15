use alloc::collections::BTreeMap;
use alloc::sync::{Arc};
use alloc::boxed::Box;
use log::info;

use lib::safe::Safe;
use core::ptr;
use lib::mem::paging;

use crate::shared_mem_com::SharedMemCommunicator;
use crate::state_machine::task_id::TaskId;
use crate::state_machine::TeeCommand;

static mut TaskMap: Safe<BTreeMap<TaskId, Box<dyn Fn(&mut SharedMemCommunicator)>>> = Safe::new(
    BTreeMap::new());

static mut COMMUNICATOR: Option<Box<SharedMemCommunicator>> = None;

pub fn init_task_map() {
    unsafe {
        // if true == COMMUNICATOR.is_none() {
        //     // TODO: proper error handling for singleton
        //     return;
        //  }
        // COMMUNICATOR = Some(Box::new(communicator));
        TaskMap.insert(TaskId::Ping, Box::new(task_ping));
        TaskMap.insert(TaskId::AttackReadMem, Box::new(task_attack_read_mem));
        TaskMap.insert(TaskId::AttackWriteMem, Box::new(task_attack_write_mem));
        TaskMap.insert(TaskId::AttackNopMem, Box::new(task_attack_nop_mem));
    }
}

fn task_ping(communicator: &mut SharedMemCommunicator) {
    let count = 0;

    let mut payload_mem = unsafe { communicator.get_slice() };
    payload_mem[0] += 1;


    communicator.set_task(TaskId::Ping);
    communicator.set_status(TeeCommand::TeeSend);
    info!("Ping");
}


fn task_attack_write_mem(communicator: &mut SharedMemCommunicator) {
    task_mem_helper(communicator);
    communicator.set_task(TaskId::AttackWriteMem);
    communicator.set_status(TeeCommand::TeeSend);
}

fn task_attack_read_mem(communicator: &mut SharedMemCommunicator) {
    task_mem_helper(communicator);
    communicator.set_task(TaskId::AttackReadMem);
    communicator.set_status(TeeCommand::TeeSend);
}

fn task_attack_nop_mem(communicator: &mut SharedMemCommunicator) {
    task_mem_helper(communicator);
    communicator.set_task(TaskId::AttackNopMem);
    communicator.set_status(TeeCommand::TeeSend);
}

fn task_mem_helper(communicator: &mut SharedMemCommunicator) {
    use alloc::alloc::{alloc, Layout};
    // We have one byte status field and 8 byte physical address that are
    // stored in the shared memory.

    // first get memory and the respective physical address
    let mut payload_mem = unsafe { communicator.get_slice() };
    // We want to fill 4 KiB of memory
    let num_elements : usize = 0x1 << 12;
    let address_offset = 2;

    // First byte denote if vector was initialized
    if 0 == payload_mem[0] {
        // Create a vecotr with capacity to make sure that all is done with one
        //allocation
        let data_ptr = unsafe {
            alloc(Layout::from_size_align(num_elements, 4096).unwrap()) as *mut u32
        };
        for x in 0..(num_elements / 4) {
            unsafe {
                ptr::write_volatile(data_ptr.add(x), 0x1_u32);
            }
        }
        unsafe {
            ptr::write_volatile(
                payload_mem.as_mut_ptr().add(address_offset) as *mut u64,
                paging::get_physical_address(data_ptr as u64)
            );
        }
        // info!("Initialized vector: {:#016x?} -> {:#016x?}", data_ptr as u64, unsafe{ paging::get_physical_address(data_ptr as u64) });
        payload_mem[0] = 1;
    } else {
        unsafe {
            use crate::state_machine::pmc;
            use core::arch::asm;

            let mut hash: u32 = 0;
            let data_ptr = paging::get_virtual_address(
                ptr::read_volatile(payload_mem.as_mut_ptr().add(address_offset) as *mut u64)
            ) as *mut u32;
            for x in 0..(num_elements / 4) {
                // let value = ptr::read_volatile(data_ptr.add(x));
                // hash += value + 1;
                // ptr::write_volatile(data_ptr.add(x), value + 1);
                ptr::write_volatile(data_ptr.add(x), x as u32);
            }
        }
    }
}

pub fn execute_task(task_id: TaskId, communicator: &mut SharedMemCommunicator) {
    unsafe {
        match TaskMap.get(&task_id) {
            Some(func) => func(communicator),
            None =>{
                info!("No task");
            },
        };
    };

}

