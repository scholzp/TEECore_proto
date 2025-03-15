use x86::msr::{
	rdmsr, wrmsr,
	IA32_PERFEVTSEL0, IA32_PMC0,
	IA32_PERFEVTSEL1, IA32_PMC1,
	IA32_PERFEVTSEL2, IA32_PMC2,
	IA32_PERFEVTSEL3, IA32_PMC3,
	IA32_PERFEVTSEL4, IA32_PMC4,
	IA32_PERFEVTSEL5, IA32_PMC5,
	IA32_PERFEVTSEL6, IA32_PMC6,
	IA32_PERFEVTSEL7, IA32_PMC7,
};

use log::info;

/// USR bit in PERFEVTSEL. When set, counter is incremented when logical core is
/// in privilege level 1,2 or 3.index
pub const  IA32_PERFEVTSEL_USR: u64 = 0x1 << 16;
/// OS bit of PERFEVTSEL. When set, counter is incremented when logical core is
/// in privilege level 0.
pub const  IA32_PERFEVTSEL_OS: u64 = 0x1 << 17;
/// E bit in PERFEVTSEL. Enables (when set) edge detection of the selected
/// microarchitectural condition.
pub const  IA32_PERFEVTSEL_E: u64 = 0x1 << 18;
/// PC bit in PERFEVTSEL. Not supported since Sandy Bridge (Core 2xxx). When set
/// processor toggles PMi pins and increments the PMC. When clear, processor
/// toggles PMi pins on counter overflow
pub const  IA32_PERFEVTSEL_PC: u64 = 0x1 << 19;
/// When set, the logical processor generates an exception through its local
/// APIC on counter overflow
pub const  IA32_PERFEVTSEL_INT: u64 = 0x1 << 20;
/// When set the corresponding PMC counts the event. When clear, the counting
/// stops and the corresponding PMC can be written
pub const  IA32_PERFEVTSEL_EN: u64 = 0x1 << 22;
/// Invert flag. Inverts counter mask when set.
pub const  IA32_PERFEVTSEL_INV: u64 = 0x1 << 23;

/*
*  IA32_PERFEVTSELx MSRs:
*  | reserved | cmask | flags | UMASK | EventSelect |
*   63      32 31   24 23   16 15    8 7           0
*/
// First operand of or is event selection, second is UMASK
/// PMC Event for Skylake that counts all requests that miss L2
pub const EVENT_SKYLAKE_L2_REQUEST_MISS: u64 = 0x24_u64 | 0x3f_u64 << 8;
/// Counts th number of Cache lines filling the L2 cache
pub const EVENT_SKYLAKE_L2_LINES_IN_ALL: u64 = 0xf1_u64 | 0x1f_u64 << 8;
/// Predefined events that counts references to on-die LLC
pub const EVENT_PREDEFINED_LLC_REFERENCES: u64 = 0x2e_u64 | 0x4f_u64 << 8;

pub const EVENT_ICELAKE_OFFCORE_ALL_REQUESTS: u64 = 0x21_u64 | 0x80_u64 << 8;
pub const EVENT_ICELAKE_MEM_LOAD_MISC_RETIRED_UC: u64 = 0xd4_u64 | 0x04_u64 << 8;
pub const EVENT_ICELAKE_MEM_LOAD_L3_HIT_RETIRED_XSNP_HITM: u64 = 0xd2_u64 | 0x04_u64 << 8;
pub const EVENT_ICELAKE_L2_ALL_DEMAND_DATA_RD: u64 = 0x24_u64 | 0xe1_u64 << 8;
pub const EVENT_ICELAKE_L2_ALL_DEMAND_MISS: u64 = 0x24_u64 | 0x27_u64 << 8;
pub const EVENT_ICELAKE_MEM_LOAD_RETIRED_L1_HIT: u64 = 0xd1_u64 | 0x01_u64 << 8;
pub const EVENT_ICELAKE_MEM_LOAD_RETIRED_L1_MISS: u64 = 0xd1_u64 | 0x08_u64 << 8;
pub const EVENT_ICELAKE_MEM_LOAD_RETIRED_L1_MISS_ANY: u64 = 0x43_u64 | 0xfd_u64 << 8;
pub const EVENT_ICELAKE_L1D_REPLACEMENT: u64 = 0x51_u64 | 0x01_u64 << 8;
// Retired all stores: pub const EVENT_ICELAKE_L2_ALL_DEMAND_MISS: u64 = 0xd0_u64 | 0x82_u64 << 8;
// All L1 hits pub const EVENT_ICELAKE_L2_ALL_DEMAND_MISS: u64 = 0xd1_u64 | 0x01_u64 << 8;
// L2 Hits as source
// pub const EVENT_ICELAKE_L2_ALL_DEMAND_MISS: u64 = 0xd1_u64 | 0x02_u64 << 8;

#[derive(Copy, Debug, Clone)]
pub struct ArchitecturalEventCounter {
	pmc_index: u8,
	event_config: u64,
}

impl Default for ArchitecturalEventCounter {
	fn default() -> Self {
		Self {
			pmc_index: 0x0_u8,
			event_config: 0x0_u64,
		}
	}
}

impl ArchitecturalEventCounter {
	/// Creates new ArchitecturalEventCounter with given id.
	///
	/// A processor can implement multiple architectural PMC registers. In this
	/// case they are denoted IA32_PMCx with corresponding IA32_PERFEVTSELx in
	/// the Intel SDM .
	///
	/// * `index`	- Index of the MSR_OFFCORE_RSP to use
	pub fn new(index: u8) -> Self {
		Self {
			pmc_index: index,
			event_config: 0x0_u64,
		}
	}

	/// Updates the configuration stored in this struct.
	///
	/// This does not automatically write to the respective IA32_PERFEVTSELx.
	///
	/// * `event_config`- Bitvector to use for later operations
	pub fn set_configuration(&mut self, event_config: u64) {
		self.event_config = event_config;
	}

	/// Sets index.
	///
	/// * `x`- Index of the IA32_PMCx to use
	pub fn set_index(&mut self, x: u8) {
		self.pmc_index = x;
	}

	/// Initialize and activate the counter facility.
	///
	/// Write the configuration to the MSR_OFFCORE_RSP and activate the
	/// respective GP PMC to count events using this configuration. Reset the
	/// counter to the given value.
	///
	///
	/// * `init_v`: Value to reset the counter to
	pub fn activate_counter(&self, init_v: u64) {
		/* To activate a PMC, we need to do the following things:
		*  1) Stop IA32_PMCx.
		*  2) Configure the IA32_PERFEVTSELx with the behavior we wish for
		*  3) Event and UMASK in IA32_PERFEVTSELx are chosen so that the
		*	 PMC uses the configuration from MSR_OFFCORE_RSPx
		*  4) Initialize IA32_PMCx (do we increment, do we decrement...?)
		*  5) Start the counter by setting the bit in IA32_PERFEVTSELx
		*/
		let mut msr_pmc: u32 = 0;
		let mut msr_pmc_eventsel: u32 = 0;
		match self.pmc_index {
			0 => {
				msr_pmc = IA32_PMC0;
				msr_pmc_eventsel = IA32_PERFEVTSEL0;
			},
			1 => {
				// We operate on PMC1 and IA32_PERFEVTSEL1
				msr_pmc = IA32_PMC1;
				msr_pmc_eventsel = IA32_PERFEVTSEL1;
			},
			2 => {
				// We operate on PMC2 and IA32_PERFEVTSEL2
				msr_pmc = IA32_PMC2;
				msr_pmc_eventsel = IA32_PERFEVTSEL2;

			},
			3 => {
				// We operate on PMC3 and IA32_PERFEVTSEL3
				msr_pmc = IA32_PMC3;
				msr_pmc_eventsel = IA32_PERFEVTSEL3;
			},
			4 => {
				// We operate on PMC3 and IA32_PERFEVTSEL3
				msr_pmc = IA32_PMC4;
				msr_pmc_eventsel = IA32_PERFEVTSEL4;
			},
			5 => {
				// We operate on PMC3 and IA32_PERFEVTSEL3
				msr_pmc = IA32_PMC5;
				msr_pmc_eventsel = IA32_PERFEVTSEL5;
			},
			6 => {
				// We operate on PMC3 and IA32_PERFEVTSEL3
				msr_pmc = IA32_PMC6;
				msr_pmc_eventsel = IA32_PERFEVTSEL6;
			},
			7 => {
				// We operate on PMC3 and IA32_PERFEVTSEL3
				msr_pmc = IA32_PMC7;
				msr_pmc_eventsel = IA32_PERFEVTSEL7;
			},
			_ => {
				info!("No CPU known to implement 8 or more GP PMCs!");
				return;  //TODO: We want, at some point, return an error
			},
		}
		Self::init_and_conf_pmc(
			msr_pmc_eventsel, msr_pmc, init_v, self.event_config
		);
	}

	fn init_and_conf_pmc(perfevtsel_register: u32, pmc_register: u32, init_v: u64, perfsel_content: u64) {
		unsafe {
			// Cancel any running performance measurements
			wrmsr(perfevtsel_register, 0x0_u64);
			// Reset the counter to zero
			wrmsr(pmc_register, init_v);
			// MSR_OFFCOREx was configured before
			// Activate the counter
			wrmsr(perfevtsel_register, perfsel_content | IA32_PERFEVTSEL_EN | IA32_PERFEVTSEL_E);
		}
	}

	pub fn read_pcm_val(&self) -> u64 {
		match self.pmc_index {
			0 => unsafe { rdmsr(IA32_PMC0) },
			1 => unsafe { rdmsr(IA32_PMC1) },
			2 => unsafe { rdmsr(IA32_PMC2) },
			3 => unsafe { rdmsr(IA32_PMC3) },
			4 => unsafe { rdmsr(IA32_PMC4) },
			5 => unsafe { rdmsr(IA32_PMC5) },
			6 => unsafe { rdmsr(IA32_PMC6) },
			7 => unsafe { rdmsr(IA32_PMC7) },
			_ => {
				info!("No CPU known to implement 8 or more GP PMCs!");
				//return;  //TODO: We want, at some point, return an error
				0
			},
		}
	}
}
