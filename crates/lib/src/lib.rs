use bitfield_struct::bitfield;
use once_cell::sync::OnceCell;
use serde::{Deserialize, Serialize};
use std::sync::Once;
use thiserror::Error;

// Use the OS specific implementation
#[cfg(target_os = "linux")]
mod os_linux;
#[cfg(target_os = "windows")]
mod os_windows;

// Import the OS specific functions
#[cfg(target_os = "linux")]
use self::os_linux::{rapl_init, read_msr};
#[cfg(target_os = "windows")]
use self::os_windows::{rapl_init, read_msr};

pub struct IntelRaplReaderBuilder {
    pub pp0: bool,
    pub pp1: bool,
    pub pkg: bool,
    pub dram: bool,
    enable_joule_conversion: bool,
}

pub struct IntelRaplReader {
    pp0: bool,
    pp1: bool,
    pkg: bool,
    dram: bool,
}

impl IntelRaplReader {
    pub fn read_measurement() -> RaplMeasurement {
        read_rapl_msr_registers()
    }
}

impl IntelRaplReaderBuilder {
    pub fn new() -> Self {
        #[cfg(amd)]
        panic!("This is an Intel only feature");

        #[allow(unreachable_code)]
        Self {
            pp0: false,
            pp1: false,
            pkg: false,
            dram: false,
            enable_joule_conversion: false,
        }
    }

    pub fn with_pp0(&mut self, pp0: bool) -> &mut Self {
        self.pp0 = pp0;
        self
    }

    pub fn with_pp1(&mut self, pp1: bool) -> &mut Self {
        self.pp1 = pp1;
        self
    }

    pub fn with_pkg(&mut self, pkg: bool) -> &mut Self {
        self.pkg = pkg;
        self
    }

    pub fn with_dram(&mut self, dram: bool) -> &mut Self {
        self.dram = dram;
        self
    }

    pub fn enable_joule_conversion(&mut self, enable: bool) -> &mut Self {
        self.enable_joule_conversion = enable;
        self
    }

    pub fn build(&self) -> IntelRaplReader {
        IntelRaplReader {
            pp0: self.pp0,
            pp1: self.pp1,
            pkg: self.pkg,
            dram: self.dram,
        }
    }
}

#[derive(Error, Debug)]
pub enum RaplError {
    #[error("io error")]
    Io(#[from] std::io::Error),
    #[cfg(target_os = "windows")]
    #[error("windows error")]
    Windows(#[from] windows::core::Error),
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct IntelRaplRegisters {
    pp0: u64,
    pp1: u64,
    pkg: u64,
    dram: u64,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct AmdRaplRegisters {
    core: u64,
    pkg: u64,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub enum RaplMeasurement {
    Intel(IntelRaplRegisters),
    AMD(AmdRaplRegisters),
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct IntelRaplRegistersJoules {
    pp0: f64,
    pp1: f64,
    pkg: f64,
    dram: f64,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct AmdRaplRegistersJoules {
    core: f64,
    pkg: f64,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub enum RaplMeasurementJoules {
    Intel(IntelRaplRegistersJoules),
    AMD(AmdRaplRegistersJoules),
}

#[bitfield(u64)]
#[derive(PartialEq, Eq)] // <- Attributes after `bitfield` are carried over
struct IntelRaplPowerUnits {
    #[bits(4)]
    power_units: usize,

    #[bits(4)]
    reserved_1: usize,

    #[bits(4)]
    energy_status_units: usize,

    #[bits(3)]
    reserved_2: usize,

    #[bits(4)]
    time_units: usize,

    #[bits(45)]
    reserved_3: u64,
}

static RAPL_INIT: Once = Once::new();

// TOOD: Bitfield here, use the "bitfield-struct" crate or so. Just check it out at least. Utilize OS specific ver for it
#[cfg(amd)]
static RAPL_POWER_UNITS: OnceCell<u64> = OnceCell::new();

#[cfg(intel)]
static RAPL_POWER_UNITS: OnceCell<u64> = OnceCell::new();

/// Read the RAPL MSR registers. This gets all the registers except for the power unit.
pub fn read_rapl_msr_registers() -> RaplMeasurement {
    RAPL_INIT.call_once(|| {
        // Run the OS specific rapl_init function, to enable reading MSR registers
        rapl_init();
    });

    // Read and return the RAPL measurement
    read_rapl_measurement()
}

pub fn read_rapl_msr_registers_as_joules(
    prev_rapl_measurement: Option<RaplMeasurementJoules>,
) -> RaplMeasurementJoules {
    let rapl_registers = read_rapl_msr_registers();

    let power_unit = *RAPL_POWER_UNITS.get_or_init(|| read_rapl_msr_power_unit());

    // Shift the power unit by 8 bits and then AND it with 0x1f
    let joule_unit = (power_unit >> 8) & 0x1f;

    // do mod pow 0.5 ^ joule_unit
    let energy_unit = 0.5f64.powi(joule_unit as i32);

    // TODO: Overflow check, cba rn
    if let Some(prev_rapl_measurement) = prev_rapl_measurement {}

    match rapl_registers {
        RaplMeasurement::Intel(registers) => {
            let pp0 = registers.pp0 as f64 * energy_unit;
            let pp1 = registers.pp1 as f64 * energy_unit;
            let pkg = registers.pkg as f64 * energy_unit;
            let dram = registers.dram as f64 * energy_unit;

            RaplMeasurementJoules::Intel(IntelRaplRegistersJoules {
                pp0,
                pp1,
                pkg,
                dram,
            })
        }
        RaplMeasurement::AMD(registers) => {
            let core = registers.core as f64 * energy_unit;
            let pkg = registers.pkg as f64 * energy_unit;

            RaplMeasurementJoules::AMD(AmdRaplRegistersJoules { core, pkg })
        }
    }
}

/// Read the RAPL MSR power unit register. This is a separate function because it is only needed once.
pub fn read_rapl_msr_power_unit() -> u64 {
    RAPL_INIT.call_once(|| {
        // Run the OS specific rapl_init function, to enable reading MSR registers
        rapl_init();
    });

    // Import the MSR RAPL power unit constants per CPU type
    #[cfg(amd)]
    use crate::amd::MSR_RAPL_POWER_UNIT;
    #[cfg(intel)]
    use crate::intel::MSR_RAPL_POWER_UNIT;

    // Return the power unit
    let power_unit = IntelRaplPowerUnits::from_bits(
        read_msr(MSR_RAPL_POWER_UNIT).expect("failed to read RAPL power unit"),
    );

    power_unit.into_bits()
}

pub fn convert_joule_thing() {}

#[cfg(amd)]
fn read_rapl_measurement() -> RaplMeasurement {
    use self::amd::{AMD_MSR_CORE_ENERGY, MSR_RAPL_PKG_ENERGY_STAT};

    RaplMeasurement::AMD(AmdRaplRegisters {
        core: read_msr(AMD_MSR_CORE_ENERGY).expect("failed to read CORE_ENERGY"),
        pkg: read_msr(MSR_RAPL_PKG_ENERGY_STAT).expect("failed to read RAPL_PKG_ENERGY_STAT"),
    })
}

#[cfg(intel)]
fn read_rapl_measurement() -> RaplMeasurement {
    use self::intel::{
        INTEL_MSR_RAPL_DRAM, INTEL_MSR_RAPL_PP0, INTEL_MSR_RAPL_PP1, MSR_RAPL_PKG_ENERGY_STAT,
    };

    RaplMeasurement::Intel(IntelRaplRegisters {
        pp0: read_msr(INTEL_MSR_RAPL_PP0).expect("failed to read PP0"),
        pp1: read_msr(INTEL_MSR_RAPL_PP1).expect("failed to read PP1"),
        pkg: read_msr(MSR_RAPL_PKG_ENERGY_STAT).expect("failed to read RAPL_PKG_ENERGY_STAT"),
        dram: read_msr(INTEL_MSR_RAPL_DRAM).expect("failed to read DRAM"),
    })
}

#[cfg(amd)]
pub mod amd {
    pub const MSR_RAPL_POWER_UNIT: u64 = 0xC0010299; // Similar to Intel MSR_RAPL_POWER_UNIT
    pub const MSR_RAPL_PKG_ENERGY_STAT: u64 = 0xC001029B; // Similar to Intel PKG_ENERGY_STATUS (This is for the whole socket)

    pub const AMD_MSR_CORE_ENERGY: u64 = 0xC001029A; // Similar to Intel PP0_ENERGY_STATUS (PP1 is for the GPU)
}

#[cfg(intel)]
pub mod intel {
    pub const MSR_RAPL_POWER_UNIT: u64 = 0x606;
    pub const MSR_RAPL_PKG_ENERGY_STAT: u64 = 0x611;

    pub const INTEL_MSR_RAPL_PP0: u64 = 0x639;
    pub const INTEL_MSR_RAPL_PP1: u64 = 0x641;
    pub const INTEL_MSR_RAPL_DRAM: u64 = 0x619;
}
