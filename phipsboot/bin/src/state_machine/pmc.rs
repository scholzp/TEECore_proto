
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
    setup_offcore();
}

fn setup_offcore() {
    use intel::MsrOffcoreRspEventCounter;

    let mut counter = MsrOffcoreRspEventCounter::new(0, 0);
    counter.set_offcore_configuration(
        0x0_u64 | intel::SUPPLIER_ANY | intel::SUPPLIER_DRAM | intel::SUPPLIER_NO_SUPP
        | intel::SUPPLIER_L3_HIT_E_SATE | intel::SUPPLIER_L3_HIT_S_SATE
        | intel::SUPPLIER_L3_HITM_SATE
        | intel::REQUEST_DMND_DATA_RD | intel::REQUEST_DMND_IFETCH
        | intel::REQUEST_DMND_RFO | intel::REQUEST_OTHER | intel::SNOOP_NOT_NEEDED | intel::SNOOP_HITM
        | intel::SNOOP_HIT_NO_FWD | intel::SNOOP_HIT_WITH_FWD | intel::SNOOP_MISS | intel::SNOOP_NONE
    );
    counter.activate_counter(0x0_u64);
}

fn setup_architecturial() {
	use architectural::{
		ArchitecturalEventCounter, EVENT_SKYLAKE_L2_LINES_IN_ALL,
		EVENT_PREDEFINED_LLC_REFERENCES, EVENT_SKYLAKE_L2_REQUEST_MISS,
		IA32_PERFEVTSEL_OS, IA32_PERFEVTSEL_USR
	};

	let mut pmc_1 = ArchitecturalEventCounter::new(1);
	let mut pmc_2 = ArchitecturalEventCounter::new(2);
	let mut pmc_3 = ArchitecturalEventCounter::new(3);

	pmc_1.set_configuration(EVENT_SKYLAKE_L2_REQUEST_MISS | IA32_PERFEVTSEL_OS | IA32_PERFEVTSEL_USR);
	pmc_2.set_configuration(EVENT_SKYLAKE_L2_LINES_IN_ALL | IA32_PERFEVTSEL_OS | IA32_PERFEVTSEL_USR);
	pmc_3.set_configuration(EVENT_PREDEFINED_LLC_REFERENCES | IA32_PERFEVTSEL_OS | IA32_PERFEVTSEL_USR);

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

	info!("IA_PMC0 (OFFCORE)  = {:#016x?}", pmc_0.read_pcm_val());
	info!("IA_PMC1 (L2 Req M) = {:#016x?}", pmc_1.read_pcm_val());
	info!("IA_PMC2 (L2 Fills) = {:#016x?}", pmc_2.read_pcm_val());
	info!("IA_PMC3 (LLC Refs) = {:#016x?}", pmc_3.read_pcm_val());
}
