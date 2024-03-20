use crate::{read_rapl_msr_registers, RaplMeasurement};

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
