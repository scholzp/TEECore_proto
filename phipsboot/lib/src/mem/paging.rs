//! Types and helpers for x86_64 4-level paging.
use core::ptr;

pub const PAGE_TABLE_ENTRY_SIZE: u64 = core::mem::size_of::<u64>() as u64;

/// Mask that only contains the 40 address bits used to address a 4KiB page
const ADDRESS_BITS_MASK: u64 = 0x000F_FFFF_FFFF_F000_u64;
const L1_ADDRESS_BITS_MASK: u64 = 0xFFFF_FFFF_FFE0_0000_u64;

/// 9 bits select the entry of the given page table.
pub const INDEX_BITMASK: u64 = 0x1ff;

/// Address of the last used L1 page table
static mut LAST_L1: u64 = 0;

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

pub unsafe fn use_l1_page_table(table_addr: u64) {
    LAST_L1 = table_addr
}

/// returns a non cryptographic 64 bit hash so this doesn't get optimized away
pub unsafe fn touch_all_present_pages() -> u64 {
    use crate::logger;
    let mut result: u64 = 0;
    let l1_address_bits = LAST_L1 & L1_ADDRESS_BITS_MASK;
    for x in 0..512 {
        // read the x-th page table entry of L1 pt
        let pt_entry = ptr::read((LAST_L1 as *mut u64).add(x));
        // check present bit
        if 1 == (pt_entry & 0x1_u64) {
            // generate virtual address of first byte in the mapped page from L1
            // PT and the index
            let first_qword = (l1_address_bits | ((x as u64) << 12)) as *mut u32;
            for i in 0..1024 {
                let target_address = first_qword.add(i);
                result = result.wrapping_add(ptr::read_volatile(target_address) as u64);
            }
            result += 1;
        }
    }
    result
}

/// Return the physical address a given virtual address is given. If the entry
/// is not present return 0.
pub unsafe fn get_physical_address(virt_addr: u64) -> u64 {
    // generate the L1 index, that is bit 12...21
    let l1_index: usize = ((virt_addr >> 12) & 0x1FF).try_into().unwrap();
    // Check the present bit (bit 0)
    if 0 == (ptr::read((LAST_L1 as *mut u64).add(l1_index)) & (0x1_u64)) {
        0_u64
    }
    else {
        // offset into phys page equals the last 12 bits of the virtual address
        let offset_within_page: u64 = (virt_addr & 0xFFF_u64).try_into().unwrap();
        // We read the entry in the page table, mask the last 12 bits and add the physical offset
        ((ptr::read((LAST_L1 as *mut u64).add(l1_index)) & (!0xFFF_u64))
            | offset_within_page )// Add offset for non page aligned addresses
            & (!(0x1_u64 << 63)) // Mask NX bit
    }
}


/// Return the virtual address if a mapping in the L1 page exists. Else return 0
pub unsafe fn get_virtual_address(phys_addr: u64) -> u64 {
    // offset into phys page equals the last 12 bits of the virtual address
    let offset_within_page: u64 = (phys_addr & 0xFFF).try_into().unwrap();
    for x in 0..512 {
        let pt_entry = (ptr::read((LAST_L1 as *mut u64).add(x)) & !(0xFFF)) & !(0x1_u64 << 63);
        if (phys_addr & !(0xFFF)) == pt_entry {
            return pt_entry | offset_within_page;
        }
    }
    return 0;
}

/// This function maps the given physical page to the same frame as the given base address. Base address is expected
/// to be 2 MiB aligned
pub unsafe fn map_phys_rel_base_addr(src: PhysAddr, size: usize, pml1: VirtAddr, flags: u64) -> VirtAddr {
    use x86::controlregs::cr3;
    use crate::logger;
    // Page walk to find L1 table
    let mut result = 0x0_u64;
    let mut mapped = false;

    let pml1_addr : u64 = pml1.into();
    let mut pages_to_map = size;
    while (LAST_L1_INDEX < 512) && (false == mapped) {
        let pm_entry = ptr::read((pml1_addr as *mut u64).add(LAST_L1_INDEX));
        // Check present bit
        if 0 == (pm_entry & 0x1) {
            // Create virtual address from start of the contiguous page block
            result = ((pml1_addr & (!0x1FFFFFu64)) + ((LAST_L1_INDEX as u64) << 12)) as u64;
            while (0 < pages_to_map) && (LAST_L1_INDEX < 512) {
                ptr::write(
                    (pml1_addr as *mut u64).add(LAST_L1_INDEX) as *mut u64,
                    (Into::<u64>::into(src) & ( & (!0xFFFu64))) | flags,
                );
                LAST_L1_INDEX += 1;
                pages_to_map -= 1;
            }
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
