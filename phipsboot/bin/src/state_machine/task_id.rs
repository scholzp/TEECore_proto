#[repr(u8)]
#[derive(Ord, PartialOrd, Eq, PartialEq, Debug)]
pub enum TaskId {
    Ping = 0x01_u8,
    AttackReadMem = 0x02_u8,
    AttackWriteMem = 0x03_u8,
    AttackNopMem = 0x04_u8,
    Unknown = 0xff_u8,
}

impl From<u8> for TaskId {
    fn from(raw_task: u8) -> Self {
        match raw_task {
            0x01_u8 => TaskId::Ping,
            0x02_u8 => TaskId::AttackReadMem,
            0x03_u8 => TaskId::AttackWriteMem,
            0x04_u8 => TaskId::AttackNopMem,
            _ => TaskId::Unknown,
        }
    }
}

impl From<TaskId> for u8 {
    fn from(task: TaskId) -> Self {
        match task {
           TaskId::Ping => 0x01_u8,
           TaskId::AttackReadMem => 0x02_u8,
           TaskId::AttackWriteMem => 0x03_u8,
           TaskId::AttackNopMem => 0x04_u8,
           TaskId::Unknown => 0xff_u8,
        }
    }
}
