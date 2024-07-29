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
        let offset = instruction & 0x00FF_FFFF;
        self.set_executed_instruction(format!("B {:#010x}", (offset << 2) + self.get_pc()));
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

    pub fn arm_add(&mut self, instruction: ARMByteCode)  {}

    pub fn arm_not_implemented(&mut self, instruction: ARMByteCode)  {
        self.set_executed_instruction("NOT IMPLEMENTED".into());
    }
}
