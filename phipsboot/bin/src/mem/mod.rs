//! Abstraction for managing memory of the system and the loader.

use core::cell::OnceCell;
use lib::mem::paging::{PhysAddr, VirtAddr};
use lib::safe::Safe;

mod heap;
pub mod stack;

/// Stores the load offset of the loader in physical memory.
static ONCE: Safe<OnceCell<i64>> = Safe::new(OnceCell::new());

pub fn init(load_offset: i64, l1_addr: u64) {
    use lib::mem::paging::get_physical_address;

    let _ = ONCE.get_or_init(|| load_offset);
    stack::init();
    unsafe {
        heap::init(l1_addr, get_physical_address(l1_addr));
    }
}

/// Returns the load offset of the loader in physical memory.
pub fn load_offset() -> i64 {
    *ONCE.get().expect("should have been configured")
}

/// Translates the virtual link address to a physical address in memory.
pub fn virt_to_phys(virt: VirtAddr) -> PhysAddr {
    // assert_eq!(virt >= );
    (virt.val() as i64 + load_offset()).into()
}

/// Returns the current heap size
pub fn get_current_heap_size() -> usize {
    heap::get_curr_heap_size()
}
