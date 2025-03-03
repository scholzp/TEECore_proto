use alloc::collections::BTreeMap;
use alloc::sync::{Arc};
use alloc::boxed::Box;
use log::info;

use lib::safe::Safe;

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

