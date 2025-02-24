
use x86_64::structures::idt::HandlerFunc;
use x86_64::structures::idt::InterruptStackFrame;
use x86_64::instructions::nop;

use crate::idt::set_nmi_handler;
use core::slice;
use core::ptr;

#[repr(C, align(8), u8)]
#[derive(Copy, Clone, Debug)]
pub enum TeeCommand {
    None = 0,
    TeeSend = 1,
    HostSend = 2,
    Unknown (u8),
}


#[repr(C, align(8))]
pub struct SharedMemCommunicator {
    memory : *mut u8,
    size: usize,
    serving_nmi: bool,
}

impl From<u8> for TeeCommand {
    fn from(num: u8) -> TeeCommand {
        match num {
            0 => TeeCommand::None,
            1 => TeeCommand::TeeSend,
            2 => TeeCommand::HostSend,
            x => TeeCommand::Unknown(x),
        }
    }
}


impl SharedMemCommunicator {
    pub unsafe fn from_raw_parts(mem: *mut u8, size: usize) -> Self{
        SharedMemCommunicator {
            memory: mem,
            size,
            serving_nmi: false,
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

    pub unsafe fn write_status(&mut self, status: u8) {
        ptr::write(self.memory, status);
    }

    pub unsafe fn read_mem<'a>(&self, status: u8) -> &'a[u8]{
        slice::from_raw_parts(self.memory.add(1), self.size - 1)
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

    // pub fn toggle_nmi_serving(&self) {
    //     if self.serving_nmi == false {
    //         set_nmi_handler(Self::nmi_handler);
    //     } else {
    //         set_nmi_handler(Self::nmi_trigger_execption);
    //     }
    // }

    pub fn poll(&mut self) {
        loop {
            match self.get_status() {
                TeeCommand::None => log::info!("nothing ot do!"),
                TeeCommand::TeeSend => log::info!("Waiting for response!"),
                TeeCommand::HostSend => log::info!("Received message"),
                TeeCommand::Unknown(x) => log::info!("Found unknown status: {:?}", x)
            }
            for x in 0..(1024u64 * 1024 * 1024 * 10) {
                nop();
            }
        }
    }

    // extern "x86-interrupt" fn nmi_handler(stack_frame: InterruptStackFrame) {
    //     log::info!("Serving NMI...");
    //     // Self::service_nmi();
    //     // loop {}
    // }

    // extern "x86-interrupt" fn nmi_trigger_execption(stack_frame: InterruptStackFrame) {
    //     log::error!("exception: 0x2 debug, stack_frame={stack_frame:#?}");
    //     loop{};
    // }
}
