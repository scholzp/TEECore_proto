
use x86_64::structures::idt::HandlerFunc;
use x86_64::structures::idt::InterruptStackFrame;
use x86_64::instructions::nop;

use crate::idt::set_nmi_handler;
use crate::state_machine::task_id::TaskId;

use core::slice;
use core::ptr;

#[repr(C, align(8), u8)]
#[derive(Copy, Clone, Debug)]
pub enum TeeCommand {
    None = 0,
    TeeReady = 0x01,
    TeeSend = 0x02,
    HostSend = 0x11,
    Unknown (u8),
}


#[derive(Debug)]
#[repr(C, align(8))]
pub struct SharedMemCommunicator {
    memory : *mut u8,
    size: usize,
    still_waiting: bool,
}

impl From<u8> for TeeCommand {
    fn from(num: u8) -> TeeCommand {
        match num {
            0 => TeeCommand::None,
            0x01 => TeeCommand::TeeReady,
            0x02 => TeeCommand::TeeSend,
            0x11 => TeeCommand::HostSend,
            x => TeeCommand::Unknown(x),
        }
    }
}

impl From<TeeCommand> for u8 {
    fn from(command: TeeCommand) -> u8 {
        match command {
            TeeCommand::None => 0,
            TeeCommand::TeeReady => 0x01,
            TeeCommand::TeeSend => 0x02,
            TeeCommand::HostSend => 0x11,
            TeeCommand::Unknown(x) => x,
        }
    }
}


impl SharedMemCommunicator {
    pub unsafe fn from_raw_parts(mem: *mut u8, size: usize) -> Self{
        SharedMemCommunicator {
            memory: mem,
            size,
            still_waiting: false,
        }
    }

    pub fn get_status(&self) -> TeeCommand {
        if true == self.memory.is_null() {
            return TeeCommand::Unknown(0xff);
        }
        unsafe {
            Into::<TeeCommand>::into(ptr::read(self.memory))
        }
    }

    pub fn get_task(&self) -> TaskId {
        if true == self.memory.is_null() {
            return TaskId::Unknown;
        }
        unsafe {
            Into::<TaskId>::into(ptr::read(self.memory.add(1)))
        }
    }

    pub fn set_status(&self, command: TeeCommand) {
        unsafe {
            (ptr::write(self.memory, Into::<u8>::into(command)));
        }
    }

    pub fn set_task(&self, task: TaskId) {
        unsafe {
            (ptr::write(self.memory.add(1), Into::<u8>::into(task)));
        }
    }

    pub unsafe fn write_status(&mut self, status: u8) {
        ptr::write(self.memory, status);
    }

    pub unsafe fn get_slice<'a>(&mut self) -> &'a mut [u8]{
        slice::from_raw_parts_mut(self.memory.add(2), self.size - 2)
    }

    /// Write bytes from source to the shared memory with offset `offset`
    pub unsafe fn write_mem(&self, src: &[u8], offset: usize) {
        let bytes_to_copy = if offset < self.size {
            if src.len() < (self.size - offset) {
                src.len()
            } else {
                self.size - offset
            }
        } else {
            0
        };
        ptr::copy_nonoverlapping(src.as_ptr(), self.memory.add(offset), bytes_to_copy);
    }

    pub fn poll(&mut self) {
        loop {
            match self.get_status() {
                TeeCommand::None =>{
                    if  false == self.still_waiting {
                        log::info!("nothing ot do!");
                    }
                    self.still_waiting = true;
                }
                TeeCommand::TeeSend | TeeCommand::TeeReady => {
                    log::info!("Waiting for response!");
                },
                TeeCommand::HostSend => {
                    log::info!("Received message");
                    self.still_waiting = false;
                    return;
                },
                TeeCommand::Unknown(x) => log::info!("Found unknown status: {:#02x?}", x)
            }
            for x in 0..(1024u64 * 1024 * 1024 * 10) {
                nop();
            }
        }
    }
}
