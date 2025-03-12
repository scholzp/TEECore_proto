
use log::info;
use lib::pmc_utils::vendor;
use lib::pmc_utils::intel;
use lib::pmc_utils::architectural;

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

    let mut counter = MsrOffcoreRspEventCounter::new(0, 0);
    counter.set_offcore_configuration(
        // 0x0_u64 | intel::SUPPLIER_ANY
        // | intel::REQUEST_DMND_DATA_RD
        // | intel::REQUEST_DMND_RFO
        // | intel::REQUEST_DMND_IFETCH
        // | intel::REQUEST_DMND_HWPF_L2_DATA_RD
        // | intel::REQUEST_DMND_HWPF_L2_RFO
        // | intel::REQUEST_DMND_HWPF_L3
        // | intel::REQUEST_DMND_HWPF_L1D_AND_SWPF
        // | intel::REQUEST_DMND_STREAMING_WR
        // | intel::REQUEST_OTHER
        0x184000001_u64
    );
    counter.activate_counter(0x0_u64);
}

fn setup_architecturial() {
	use architectural::{
		ArchitecturalEventCounter, EVENT_SKYLAKE_L2_LINES_IN_ALL,
		EVENT_PREDEFINED_LLC_REFERENCES, EVENT_SKYLAKE_L2_REQUEST_MISS,
        EVENT_ICELAKE_OFFCORE_ALL_EVENTS,
        EVENT_ICELAKE_MEM_LOAD_MISC_RETIRED_UC,
        EVENT_ICELAKE_MEM_LOAD_L3_HIT_RETIRED_XSNP_HITM,
        EVENT_ICELAKE_L2_ALL_DEMAND_MISS,
        EVENT_ICELAKE_L2_ALL_DEMAND_DATA_RD,
		IA32_PERFEVTSEL_OS, IA32_PERFEVTSEL_USR
	};

    let mut pmc_0 = ArchitecturalEventCounter::new(0);
	let mut pmc_1 = ArchitecturalEventCounter::new(1);
	let mut pmc_2 = ArchitecturalEventCounter::new(2);
	let mut pmc_3 = ArchitecturalEventCounter::new(3);

    pmc_0.set_configuration(EVENT_ICELAKE_MEM_LOAD_MISC_RETIRED_UC | IA32_PERFEVTSEL_OS | IA32_PERFEVTSEL_USR);
	pmc_1.set_configuration(EVENT_ICELAKE_OFFCORE_ALL_EVENTS | IA32_PERFEVTSEL_OS | IA32_PERFEVTSEL_USR);
	pmc_2.set_configuration(EVENT_ICELAKE_L2_ALL_DEMAND_DATA_RD | IA32_PERFEVTSEL_OS | IA32_PERFEVTSEL_USR);
	pmc_3.set_configuration(EVENT_ICELAKE_L2_ALL_DEMAND_MISS | IA32_PERFEVTSEL_OS | IA32_PERFEVTSEL_USR);

    pmc_0.activate_counter(0);
	pmc_1.activate_counter(0);
	pmc_2.activate_counter(0);
	pmc_3.activate_counter(0);
}


pub fn read_and_print_pmcs() {
	use architectural::{ArchitecturalEventCounter};
    use vendor::{check_vendor, CpuVendor};

    if false == check_vendor(CpuVendor::Intel) {
        return;
    }

	let pmc_0 = ArchitecturalEventCounter::new(0);
	let pmc_1 = ArchitecturalEventCounter::new(1);
	let pmc_2 = ArchitecturalEventCounter::new(2);
	let pmc_3 = ArchitecturalEventCounter::new(3);

	info!("IA_PMC1 (UC LOADS)   = {:#016x?}", pmc_0.read_pcm_val());
	info!("IA_PMC0 (ALL OFC)    = {:#016x?}", pmc_1.read_pcm_val());
	info!("IA_PMC2 (L2 ALL DRD) = {:#016x?}", pmc_2.read_pcm_val());
	info!("IA_PMC3 (L2 ALL MSS) = {:#016x?}", pmc_3.read_pcm_val());
}
