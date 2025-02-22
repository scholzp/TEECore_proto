//! Types and helpers for x86_64 4-level paging.


pub const PAGE_TABLE_ENTRY_SIZE: u64 = core::mem::size_of::<u64>() as u64;

/// 9 bits select the entry of the given page table.
pub const INDEX_BITMASK: u64 = 0x1ff;

/// Address of the last used L1 page table
static LAST_L1: u64 = 0;

/// Index into the last used L1 table to use for next mapping
/// This uses the assumption that the last 128 entries are free
static mut LAST_L1_INDEX: usize = 384-1;

#[derive(Copy, Clone, Debug, PartialOrd, PartialEq, Hash, Eq, Ord)]
pub enum Level {
    One = 1,
    Two = 2,
    Three = 3,
    Four = 4,
}

impl Level {
    pub fn val(self) -> u64 {
        self as u64
    }
}

/// Helper for common impls of phys and virt addresses.
macro_rules! impl_addr {
    ($typ:ty) => {
        impl $typ {
            /// Constructor.
            pub fn new(val: u64) -> Self {
                Self(val)
            }

            /// Returns the inner value.
            pub fn val(self) -> u64 {
                self.0
            }
        }

        impl From<u64> for $typ {
            fn from(val: u64) -> Self {
                Self::new(val)
            }
        }

        impl From<$typ> for u64 {
            fn from(val: $typ) -> Self {
                val.0
            }
        }

        impl From<*const u8> for $typ {
            fn from(val: *const u8) -> Self {
                Self::new(val as u64)
            }
        }

        impl From<$typ> for *const u8 {
            fn from(val: $typ) -> Self {
                (val.0 as u64) as *const u8
            }
        }

        impl From<i64> for $typ {
            fn from(val: i64) -> Self {
                Self::new(val as u64)
            }
        }

        impl From<$typ> for i64 {
            fn from(val: $typ) -> Self {
                val.0 as i64
            }
        }
    };
}

#[derive(Copy, Clone, Debug, PartialOrd, PartialEq, Hash, Eq, Ord, Default)]
#[repr(transparent)]
pub struct VirtAddr(u64);

#[derive(Copy, Clone, Debug, PartialOrd, PartialEq, Hash, Eq, Ord, Default)]
#[repr(transparent)]
pub struct PhysAddr(u64);

impl_addr!(PhysAddr);
impl_addr!(VirtAddr);

impl VirtAddr {
    /// Returns the index into the page table of the given level.
    /// The returned value is in range `0..512`.
    pub fn pt_index(&self, level: Level) -> u64 {
        let level = level.val();
        let bits = self.val() >> ((level - 1) * 9) + 12;
        bits & INDEX_BITMASK
    }

    /// Returns the byte offset into the page table of the given level.
    /// The returned value is in range `0..4096`.
    pub fn pt_offset(&self, level: Level) -> u64 {
        self.pt_index(level) * PAGE_TABLE_ENTRY_SIZE
    }
}

/// Creates one single 1 GiB mapping with rwx permissions.
fn _map_single_entry(_src: VirtAddr, _dest: PhysAddr, _flags: u64) {}

/// This function maps the given physical page to the same frame as the given base address. Base address is expected
/// to be 2 MiB aligned
pub unsafe fn map_phys_rel_base_addr(src: PhysAddr, pml1: VirtAddr) -> VirtAddr {
    use x86::controlregs::cr3;
    use core::ptr;
    use crate::logger;
    // Page walk to find L1 table
    let mut result = 0x0_u64;
    let mut mapped = false;

    let pml1_addr : u64 = pml1.into();
    while (LAST_L1_INDEX < 512) && (false == mapped) {
        let pm_entry = ptr::read((pml1_addr as *mut u64).add(LAST_L1_INDEX));
        // Check present bit
        if 0 == (pm_entry & 0x1) {
            result = ((pml1_addr & (!0x1FFFFFu64)) + ((LAST_L1_INDEX as u64) << 12)) as u64;
            ptr::write(
                (pml1_addr as *mut u64).add(LAST_L1_INDEX) as *mut u64,
                (Into::<u64>::into(src) & ( & (!0xFFFu64))) | 0x3,
            );
            mapped = true;
        }
        LAST_L1_INDEX += 1;
    }
    VirtAddr::from(result)
}


#[cfg(test)]
mod tests {
    use super::*;

    /// Tests that the indices and offsets into page tables are properly
    /// calculated. I used the "paging-calculator" facility to verify those
    /// results.
    #[test]
    fn page_table_index_and_offset() {
        let addr = VirtAddr::from(0xdead_beef_1337_1337_u64);
        assert_eq!(addr.pt_index(Level::One), 369);
        assert_eq!(addr.pt_index(Level::Two), 153);
        assert_eq!(addr.pt_index(Level::Three), 444);
        assert_eq!(addr.pt_index(Level::Four), 381);
        assert_eq!(addr.pt_offset(Level::One), 0xb88);
        assert_eq!(addr.pt_offset(Level::Two), 0x4c8);
        assert_eq!(addr.pt_offset(Level::Three), 0xde0);
        assert_eq!(addr.pt_offset(Level::Four), 0xbe8);
    }
}
