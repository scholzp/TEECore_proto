
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

fn setup_offcore() {
	use intel::MsrOffcoreRspEventCounter;

	let mut counter = MsrOffcoreRspEventCounter::new(0, 3);
	counter.set_offcore_configuration(
		// 0x0_u64 | intel::SUPPLIER_ANY
		// | intel::REQUEST_DMND_DATA_RD
		// | intel::REQUEST_DMND_RFO
		// | intel::REQUEST_DMND_CODE_RD
		// | intel::REQUEST_DMND_HWPF_L2_DATA_RD
		// | intel::REQUEST_DMND_HWPF_L2_RFO
		// | intel::REQUEST_DMND_HWPF_L3
		// | intel::REQUEST_DMND_HWPF_L1D_AND_SWPF
		// | intel::REQUEST_DMND_STREAMING_WR
		// | intel::REQUEST_OTHER
		// 0x184000001_u64
        // 0x0_u64 | (0x1 << 18) | (0x1 << 19) | (0x1 << 20) // Spulier = L3
        // | (0x1 << 34) | (0x1 << 36) // SNOOP_HITM & SNOOP_HIT_NO_FW
        // | (0x1 << 0) | (0x1 << 1) | (0x1 << 2) // DATA_RD, RFO, CODE_RD
        // 0b0000_0000_0000_0000_0000_0000_0000_0001_0000_0000_0001_1100_0010_1111_1011_0111
        //63   59   55   51   47   43   39   35   31   27   23   19   15   11   7    3
        // 0x3FBFC00002// L3 MISS -> 1
        // 0x8003C0001 // L3 snoop hit -> 1
        // 0x10003C0002 // L3 HITM -> 1
        0x184000001
	);
	counter.activate_counter(0x0_u64);
}

fn setup_architecturial() {
	use architectural::{
		ArchitecturalEventCounter, EVENT_SKYLAKE_L2_LINES_IN_ALL,
		EVENT_PREDEFINED_LLC_REFERENCES, EVENT_SKYLAKE_L2_REQUEST_MISS,
		EVENT_ICELAKE_OFFCORE_ALL_REQUESTS,
		EVENT_ICELAKE_MEM_LOAD_MISC_RETIRED_UC,
		EVENT_ICELAKE_MEM_LOAD_L3_HIT_RETIRED_XSNP_HITM,
		EVENT_ICELAKE_L2_ALL_DEMAND_MISS,
		EVENT_ICELAKE_L2_ALL_DEMAND_DATA_RD,
		EVENT_ICELAKE_MEM_LOAD_RETIRED_L1_HIT,
		EVENT_ICELAKE_L1D_REPLACEMENT,
		EVENT_ICELAKE_MEM_LOAD_RETIRED_L1_MISS_ANY,
		EVENT_ICELAKE_MEM_LOAD_RETIRED_L1_MISS,
		IA32_PERFEVTSEL_OS, IA32_PERFEVTSEL_USR, IA32_PERFEVTSEL_INT
	};

    let mut counters: [ArchitecturalEventCounter; COUNTER_NUM] = [ArchitecturalEventCounter::new(0); COUNTER_NUM];
    for x in 0..COUNTER_NUM {
        counters[x].set_index(x as u8);
    }

    let TEST_EVENT: u64 = 0xd1_u64 | 0x10_u64 << 8;
    let TEST_EVENT2: u64 = 0xd2_u64 | 0x08_u64 << 8; // L3 Hits without snoops

	counters[0].set_configuration(EVENT_ICELAKE_L1D_REPLACEMENT | IA32_PERFEVTSEL_OS | IA32_PERFEVTSEL_USR | IA32_PERFEVTSEL_INT);
	counters[1].set_configuration(EVENT_ICELAKE_MEM_LOAD_RETIRED_L1_HIT | IA32_PERFEVTSEL_OS | IA32_PERFEVTSEL_USR | IA32_PERFEVTSEL_INT);
	counters[2].set_configuration(TEST_EVENT2 | IA32_PERFEVTSEL_OS | IA32_PERFEVTSEL_USR | IA32_PERFEVTSEL_INT);
	counters[3].set_configuration(TEST_EVENT | IA32_PERFEVTSEL_OS | IA32_PERFEVTSEL_USR | IA32_PERFEVTSEL_INT);

    for x in 0..COUNTER_NUM {
        counters[x].activate_counter(u64::MAX);
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

	info!("IA_PMC1 (Replacement) = {:#016x?}", counters[0].read_pcm_val());
	info!("IA_PMC0 (L1 Hit)      = {:#016x?}", counters[1].read_pcm_val());
	info!("IA_PMC2 (All OffCore) = {:#016x?}", counters[2].read_pcm_val());
	info!("IA_PMC3 (Own OffCore) = {:#016x?}", counters[3].read_pcm_val());
    // info!("IA_PMC4 (UC MEM Accs) = {:#016x?}", counters[4].read_pcm_val());
	// info!("IA_PMC5 (L3 HitM Snp) = {:#016x?}", counters[5].read_pcm_val());
	// info!("IA_PMC6 (L2 ALL Miss) = {:#016x?}", counters[6].read_pcm_val());
}
