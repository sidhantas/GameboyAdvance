#![allow(unused)]

use crate::{
    arm7tdmi::cpu::LINK_REGISTER,
    types::{ARMByteCode, REGISTER, WORD},
    utils::bits::{sign_extend, Bits},
};

use super::cpu::CPU;
pub type ARMExecutable = fn(&mut CPU, ARMByteCode) -> ();

#[derive(Clone)]
pub struct ARMDecodedInstruction {
    pub executable: ARMExecutable,
    pub instruction: ARMByteCode,
    pub rd: REGISTER,
    pub rn: REGISTER,
    pub operand2: WORD,
}

impl Default for ARMDecodedInstruction {
    fn default() -> Self {
        ARMDecodedInstruction {
            executable: CPU::arm_nop,
            instruction: 0,
            rd: 0,
            rn: 0,
            operand2: 0,
        }
    }
}

impl CPU {
    fn set_executed_instruction(&mut self, name: String) {
        self.executed_instruction = name;
    }
    pub fn arm_branch(&mut self, instruction: ARMByteCode) {
        self.flush_pipeline();
        if (instruction.bit_is_set(24)) {
            self.set_register(LINK_REGISTER, self.get_pc() - 4);
        }
        let offset = instruction & 0x00FF_FFFF;
        let offset: i32 = sign_extend(offset << 2, 25) as i32;
        let destination = offset as i64 + self.get_pc() as i64;
        self.set_pc(destination as u32);
        self.set_executed_instruction(format!("B {:#010x}", destination));
    }

    pub fn arm_nop(&mut self, instruction: ARMByteCode) {
        self.set_executed_instruction("NOP".into());
    }

    pub fn arm_multiply(&mut self, instruction: ARMByteCode) {}

    pub fn arm_multiply_accumulate(&mut self, instruction: ARMByteCode) {}

    pub fn arm_multiply_long(&mut self, instruction: ARMByteCode) {}

    pub fn arm_single_data_swap(&mut self, instruction: ARMByteCode) {}

    pub fn arm_branch_and_exchange(&mut self, instruction: ARMByteCode) {}

    pub fn arm_and(&mut self, instruction: ARMByteCode) {
        self.set_executed_instruction(format!("AND"))
    }

    pub fn arm_eor(&mut self, instruction: ARMByteCode) {
        self.set_executed_instruction(format!("EOR"))
    }

    pub fn arm_sub(&mut self, instruction: ARMByteCode) {
        self.set_executed_instruction(format!("SUB"))
    }

    pub fn arm_rsb(&mut self, instruction: ARMByteCode) {
        self.set_executed_instruction(format!("RSB"))
    }

    pub fn arm_add(&mut self, instruction: ARMByteCode) {
        let decoded_inst = self.decoded_instruction.clone();
        let result = self.get_register(decoded_inst.rn) + decoded_inst.operand2;
        self.set_register(decoded_inst.rd, result);
        self.set_executed_instruction(format!(
            "ADD {:#x} {:#x} {:#x}",
            decoded_inst.rd, decoded_inst.rn, decoded_inst.operand2
        ));
    }

    pub fn arm_adc(&mut self, instruction: ARMByteCode) {}

    pub fn arm_sbc(&mut self, instruction: ARMByteCode) {}

    pub fn arm_rsc(&mut self, instruction: ARMByteCode) {}

    pub fn arm_tst(&mut self, instruction: ARMByteCode) {}

    pub fn arm_teq(&mut self, instruction: ARMByteCode) {}

    pub fn arm_cmp(&mut self, instruction: ARMByteCode) {}

    pub fn arm_cmn(&mut self, instruction: ARMByteCode) {}

    pub fn arm_orr(&mut self, instruction: ARMByteCode) {}

    pub fn arm_mov(&mut self, instruction: ARMByteCode) {}

    pub fn arm_bic(&mut self, instruction: ARMByteCode) {}

    pub fn arm_mvn(&mut self, instruction: ARMByteCode) {}

    pub fn arm_not_implemented(&mut self, instruction: ARMByteCode) {
        self.set_executed_instruction("NOT IMPLEMENTED".into());
    }
}

#[cfg(test)]
mod instruction_tests {
    use std::sync::{Arc, Mutex};

    use crate::{
        arm7tdmi::cpu::{CPU, LINK_REGISTER},
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
