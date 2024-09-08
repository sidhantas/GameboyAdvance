#![allow(unused)]
use crate::{
    arm7tdmi::cpu::LINK_REGISTER,
    types::{ARMByteCode, CYCLES, REGISTER, WORD},
    utils::bits::{sign_extend, Bits},
};

use super::cpu::{FlagsRegister, CPU};
pub type ARMExecutable = fn(&mut CPU, ARMByteCode) -> CYCLES;
pub type ExecutingInstruction = fn(&mut CPU) -> ();
pub type InternalOperation = fn(&mut CPU) -> CYCLES;
pub type ALUOperation =
    fn(&mut CPU, rd: REGISTER, operand1: u32, operand2: u32, set_flags: bool) -> ();

#[derive(Clone, Copy)]
pub struct ARMDecodedInstruction {
    pub executable: ARMExecutable,
    pub instruction: ARMByteCode,
}

impl Default for ARMDecodedInstruction {
    fn default() -> Self {
        ARMDecodedInstruction {
            executable: CPU::arm_nop,
            instruction: 0,
        }
    }
}

impl CPU {
    pub fn set_executed_instruction(&mut self, name: String) {
        self.executed_instruction = name;
    }
    pub fn arm_branch(&mut self, instruction: ARMByteCode) -> CYCLES {
        self.flush_pipeline();
        if (instruction.bit_is_set(24)) {
            self.set_register(LINK_REGISTER, self.get_pc() - 4);
        }
        let offset = instruction & 0x00FF_FFFF;
        let offset: i32 = sign_extend(offset << 2, 25) as i32;
        let destination = offset as i64 + self.get_pc() as i64;
        self.set_pc(destination as u32);
        self.set_executed_instruction(format!("B {:#010x}", destination));
        return 1;
    }

    pub fn arm_nop(&mut self, instruction: ARMByteCode) -> CYCLES {
        self.set_executed_instruction("NOP".into());
        return 1;
    }

    pub fn arm_multiply(&mut self, instruction: ARMByteCode) -> CYCLES {
        return 0;
    }

    pub fn arm_multiply_accumulate(&mut self, instruction: ARMByteCode) -> CYCLES {
        return 0;
    }

    pub fn arm_multiply_long(&mut self, instruction: ARMByteCode) -> CYCLES {
        return 0;
    }

    pub fn arm_single_data_swap(&mut self, instruction: ARMByteCode) -> CYCLES {
        return 0;
    }

    pub fn arm_branch_and_exchange(&mut self, instruction: ARMByteCode) -> CYCLES {
        return 0;
    }

    pub fn arm_load_store_instruction(&mut self, instruction: ARMByteCode) -> CYCLES {
        return 0;
    }

    pub fn arm_not_implemented(&mut self, instruction: ARMByteCode) -> CYCLES {
        return 0;
        self.set_executed_instruction("NOT IMPLEMENTED".into());
    }
}

#[cfg(test)]
mod instruction_tests {
    use std::sync::{Arc, Mutex};

    use crate::{
        arm7tdmi::cpu::{FlagsRegister, CPU, LINK_REGISTER},
        memory::Memory,
    };

    use super::ARMDecodedInstruction;

    #[test]
    fn branch_ends_up_at_correct_address() {
        let memory = Memory::new().unwrap();
        let memory = Arc::new(Mutex::new(memory));
        let mut cpu = CPU::new(memory);

        cpu.fetched_instruction = 0xea000005;
        cpu.set_pc(4);

        let expected_destination = 0x14 + cpu.get_pc() + 8;

        cpu.execute_cpu_cycle();
        cpu.execute_cpu_cycle();

        assert!(cpu.get_pc() == expected_destination);
    }

    #[test]
    fn branch_can_go_backwards() {
        let memory = Memory::new().unwrap();
        let memory = Arc::new(Mutex::new(memory));
        let mut cpu = CPU::new(memory);

        cpu.fetched_instruction = 0xeafffff4;
        cpu.set_pc(0x34);

        let expected_destination = 12;

        cpu.execute_cpu_cycle();
        cpu.execute_cpu_cycle();

        println!("PC: {:#x}", cpu.get_pc());
        assert!(cpu.get_pc() == expected_destination);
    }

    #[test]
    fn branch_with_link_stores_the_instruction_correctly() {
        let memory = Memory::new().unwrap();
        let memory = Arc::new(Mutex::new(memory));
        let mut cpu = CPU::new(memory);

        cpu.fetched_instruction = 0xeb000005;
        cpu.set_pc(4);

        let expected_destination = 0x14 + cpu.get_pc() + 8;

        cpu.execute_cpu_cycle();
        cpu.execute_cpu_cycle();

        assert!(cpu.get_pc() == expected_destination);
        println!("LR: {:#x}", cpu.get_register(LINK_REGISTER));
        assert!(cpu.get_register(LINK_REGISTER) == 4);
    }
}
