
use log::info;
use lib::pmc_utils::vendor;
use lib::pmc_utils::intel;
use lib::pmc_utils::architectural;

const COUNTER_NUM: usize = 4;
const COUNTER_NUM_P: usize = 3 + 1;

pub fn setup_pmcs() {
	use vendor::{check_vendor, CpuVendor};

	if false == check_vendor(CpuVendor::Intel) {
		return;
	}
	setup_architecturial();
	// setup_offcore();
}

#[allow(dead_code)]
fn setup_offcore() {
	use intel::MsrOffcoreRspEventCounter;

	let mut counter = MsrOffcoreRspEventCounter::new(0, 3);
	counter.set_offcore_configuration(
		0x10002
	);
	counter.activate_counter(0x0_u64);
}

fn setup_architecturial() {
	use architectural::{
		ArchitecturalEventCounter,
		EVENT_ICELAKE_L1D_REPLACEMENT,
		IA32_PERFEVTSEL_USR,
		IA32_PERFEVTSEL_OS,
		IA32_PERFEVTSEL_INT,
	};

	let mut counters: [ArchitecturalEventCounter; COUNTER_NUM] = [ArchitecturalEventCounter::new(0); COUNTER_NUM];
	for x in 0..COUNTER_NUM {
		counters[x].set_index(x as u8);
	}

	let event_l1i_stalls: u64 = 0xC6_u64 | 0x01_u64 << 8;
	let event_l2_hit: u64 = 0xd1_u64 | 0x02_u64 << 8;
	let event_l3_hit: u64 = 0xd1_u64 | 0x04_u64 << 8;

	counters[0].set_configuration(event_l1i_stalls | IA32_PERFEVTSEL_OS | IA32_PERFEVTSEL_USR);
	// We have to program MSR_PEBS_FRONTEND for the L1I Miss counter to work
	unsafe {
		use x86::msr::{wrmsr};
		wrmsr(0x3f7, 0x12_u64);
	}
	counters[1].set_configuration(EVENT_ICELAKE_L1D_REPLACEMENT | IA32_PERFEVTSEL_OS | IA32_PERFEVTSEL_USR);
	counters[2].set_configuration(event_l2_hit | IA32_PERFEVTSEL_OS | IA32_PERFEVTSEL_USR);
	counters[3].set_configuration(event_l3_hit | IA32_PERFEVTSEL_OS | IA32_PERFEVTSEL_USR);

	for x in 0..COUNTER_NUM {
		counters[x].activate_counter(0);
	}
}


pub fn read_and_print_pmcs() {
	use architectural::{ArchitecturalEventCounter};
	use vendor::{check_vendor, CpuVendor};

	if false == check_vendor(CpuVendor::Intel) {
		return;
	}

	let mut counters: [ArchitecturalEventCounter; COUNTER_NUM_P] = [ArchitecturalEventCounter::new(0); COUNTER_NUM_P];
	for x in 0..COUNTER_NUM_P {
		counters[x].set_index(x as u8);
	}

	info!("IA_PMC1 (L1I Stalls)       = {:#018x?}", counters[0].read_pcm_val());
	info!("IA_PMC0 (L1D Replacements) = {:#018x?}", counters[1].read_pcm_val());
	info!("IA_PMC2 (L2 Hits)          = {:#018x?}", counters[2].read_pcm_val());
	info!("IA_PMC3 (L3 Hits)          = {:#018x?}", counters[3].read_pcm_val());
}
