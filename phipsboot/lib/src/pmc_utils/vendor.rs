use core::arch::x86_64::__cpuid_count;
use log::info;

#[derive(Debug, Eq, PartialEq)]
pub enum CpuVendor {
	Intel,
	Amd,
	Unknown,
}

/// Returns the vendor of the CPU
pub fn get_cpu_vendor() -> CpuVendor{
	let cpuid_result = unsafe {
		__cpuid_count(0x0, 0x0)
	};
	let mut bytes = [0x0u8; 12];
	bytes[0..4].copy_from_slice(&cpuid_result.ebx.to_le_bytes());
	bytes[4..8].copy_from_slice(&cpuid_result.edx.to_le_bytes());
	bytes[8..].copy_from_slice(&cpuid_result.ecx.to_le_bytes());

	match bytes {
		[0x41, 0x75, 0x74, 0x68, 0x65, 0x6E, 0x74, 0x69, 0x63, 0x41, 0x4D, 0x44] => {
			CpuVendor::Amd
		},
		[0x47, 0x65, 0x6E, 0x75, 0x69, 0x6E, 0x65, 0x49, 0x6E, 0x74, 0x65, 0x6C] => {
			CpuVendor::Intel

		},
		_ => CpuVendor::Unknown
	}

}

pub fn check_vendor(vendor: CpuVendor) -> bool {
	if get_cpu_vendor() != vendor {
		info!("Function not supported on this machine!");
		return false;
	}
	return true;
}