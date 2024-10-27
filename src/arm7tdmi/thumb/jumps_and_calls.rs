use num_traits::sign;

use crate::{arm7tdmi::cpu::{FlagsRegister, CPU, LINK_REGISTER}, types::CYCLES, utils::bits::{sign_extend, Bits}};

impl CPU {
    pub fn thumb_conditional_branch(&mut self, instruction: u32) -> CYCLES {
        let mut cycles = 1;
        let condition = (instruction & 0x0F00) >> 8;
        let offset = (instruction & 0x00FF) << 1;

        let condition_passed = match condition {
            0b0000 => self.get_flag(FlagsRegister::Z) == 1, //EQ
            0b0001 => self.get_flag(FlagsRegister::Z) == 0, //NE
            0b0010 => self.get_flag(FlagsRegister::C) == 1, //CS
            0b0011 => self.get_flag(FlagsRegister::C) == 0, //CC
            0b0100 => self.get_flag(FlagsRegister::N) == 1, //MI
            0b0101 => self.get_flag(FlagsRegister::N) == 0, //PL
            0b0110 => self.get_flag(FlagsRegister::V) == 1, //VS
            0b0111 => self.get_flag(FlagsRegister::V) == 0, //VC
            0b1000 => self.get_flag(FlagsRegister::C) == 1 && self.get_flag(FlagsRegister::Z) == 0, //HI
            0b1001 => self.get_flag(FlagsRegister::C) == 0 || self.get_flag(FlagsRegister::Z) == 1, //LS
            0b1010 => self.get_flag(FlagsRegister::N) == self.get_flag(FlagsRegister::V), //GE
            0b1011 => self.get_flag(FlagsRegister::N) != self.get_flag(FlagsRegister::V), //LT
            0b1100 => {
                self.get_flag(FlagsRegister::Z) == 0
                    && self.get_flag(FlagsRegister::N) == self.get_flag(FlagsRegister::V)
            } //GT
            0b1101 => {
                self.get_flag(FlagsRegister::Z) == 1
                    || self.get_flag(FlagsRegister::N) != self.get_flag(FlagsRegister::V)
            } //LE
            _ => panic!("Impossible/Undefined condition code")
        };

        if condition_passed {
            self.set_pc(self.get_pc() + sign_extend(offset, 8));
            cycles += self.flush_pipeline();
        }

        self.set_executed_instruction(format!("B C {:#x}", offset));

        cycles
    }

    pub fn thumb_unconditional_branch(&mut self, instruction: u32) -> CYCLES {
        let mut cycles = 1;
        let offset: u32 = sign_extend((instruction & 0x07FF) << 1, 11);
        self.set_pc(self.get_pc() + offset);
        cycles += self.flush_pipeline();
        self.set_executed_instruction(format!("B {:#x}", offset));

        cycles
    }

    pub fn thumb_set_link_register(&mut self, instruction: u32) -> CYCLES {
        
        self.set_register(LINK_REGISTER, self.get_pc() + ((instruction & 0x07FF) << 12));
        
        self.set_executed_instruction(format!("SET LR: {:#x}", self.get_pc()));
        1
    }

    pub fn thumb_long_branch_with_link(&mut self, instruction: u32) -> CYCLES {

        let link_register_val = self.get_register(LINK_REGISTER);
        self.set_register(LINK_REGISTER, (self.get_pc() - 2) | 1);
        let destination = link_register_val + ((instruction & 0x7FF) << 1);
        self.set_pc(destination);

        self.flush_pipeline();

        self.set_executed_instruction(format!("BL: {:#x}", destination));
        1
    }
}

#[cfg(test)]
mod branch_tests {
    use std::sync::{Arc, Mutex};

    use crate::{
        arm7tdmi::cpu::{FlagsRegister, InstructionMode, CPU, LINK_REGISTER},
        memory::Memory,
    };

    #[test]
    fn should_branch_ahead() {
        let memory = Memory::new().unwrap();
        let memory = Arc::new(Mutex::new(memory));
        let mut cpu = CPU::new(memory);
        cpu.inst_mode = InstructionMode::THUMB;

        cpu.fetched_instruction = 0xd006; // beq 12
        cpu.set_pc(0x1a);
        cpu.set_flag(FlagsRegister::Z);
        cpu.execute_cpu_cycle();
        cpu.execute_cpu_cycle();


        assert_eq!(cpu.get_pc(), 0x2c);
    }

    #[test]
    fn should_branch_behind() {
        let memory = Memory::new().unwrap();
        let memory = Arc::new(Mutex::new(memory));
        let mut cpu = CPU::new(memory);
        cpu.inst_mode = InstructionMode::THUMB;

        cpu.fetched_instruction = 0xd0f9; // beq 12
        cpu.set_pc(0x24);
        cpu.set_flag(FlagsRegister::Z);
        cpu.execute_cpu_cycle();
        cpu.execute_cpu_cycle();


        assert_eq!(cpu.get_pc(), 0x1c);
    }

    #[test]
    fn should_set_link_register_and_branch() {
        let memory = Memory::new().unwrap();
        let memory = Arc::new(Mutex::new(memory));
        let mut cpu = CPU::new(memory);
        cpu.inst_mode = InstructionMode::THUMB;

        cpu.fetched_instruction = 0xf000; // set link_register
        cpu.set_pc(0x1a);
        cpu.execute_cpu_cycle();
        cpu.fetched_instruction = 0xf802; // bl 0x20
        cpu.execute_cpu_cycle();
        cpu.execute_cpu_cycle();


        assert_eq!(cpu.get_pc(), 0x24);
        assert_eq!(cpu.get_register(LINK_REGISTER), 0x1d);
    }
}