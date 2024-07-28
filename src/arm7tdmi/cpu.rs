pub struct CPU {
    registers: [u32; 31]
}

impl CPU {
    pub fn initialize() -> CPU {
        CPU {
            registers: [0; 31]
        }
    }
    pub fn get_pc(&self) -> u32 {
        self.registers[15]
    }

    fn get_sp(&self) {

    }
}
