#![allow(unused)]

use crate::types::ARMByteCode;

use super::cpu::CPU;
pub type ARMExecutable = fn(&mut CPU, ARMByteCode) -> ();

pub struct ARMDecodedInstruction {
    pub executable: ARMExecutable,
    pub instruction: ARMByteCode
}

impl CPU {
    fn set_executed_instruction(&mut self, name: String) {
        self.executed_instruction = name;
    }
    pub fn arm_branch(&mut self, instruction: ARMByteCode)  {
        self.flush_pipeline();
        let offset = instruction & 0x00FF_FFFF;
        let destination = (offset << 2) + self.get_pc();
        self.set_pc(destination);
        self.set_executed_instruction(format!("B {:#010x}", destination));
    }

    pub fn arm_branch_with_link(&mut self, instruction: ARMByteCode) {}

    pub fn arm_nop(&mut self, instruction: ARMByteCode)  {
        self.set_executed_instruction("NOP".into());
    }

    pub fn arm_multiply(&mut self, instruction: ARMByteCode)  {}

    pub fn arm_multiply_accumulate(&mut self, instruction: ARMByteCode)  {}

    pub fn arm_multiply_long(&mut self, instruction: ARMByteCode)  {}

    pub fn arm_single_data_swap(&mut self, instruction: ARMByteCode)  {}

    pub fn arm_branch_and_exchange(&mut self, instruction: ARMByteCode)  {}

    pub fn arm_and(&mut self, instruction: ARMByteCode)  {}

    pub fn arm_eor(&mut self, instruction: ARMByteCode) {}

    pub fn arm_sub(&mut self, instruction: ARMByteCode) {}

    pub fn arm_rsb(&mut self, instruction: ARMByteCode) {}

    pub fn arm_add(&mut self, instruction: ARMByteCode) {}

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

    pub fn arm_not_implemented(&mut self, instruction: ARMByteCode)  {
        self.set_executed_instruction("NOT IMPLEMENTED".into());
    }
}

#[cfg(test)]
mod instruction_tests {
    use std::sync::{Arc, Mutex};

    use crate::{arm7tdmi::cpu::CPU, memory::Memory};

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

}
