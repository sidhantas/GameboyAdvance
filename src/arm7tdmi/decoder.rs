use crate::types::*;

pub trait InstructionDecoder {
    fn condition_passed(&self, condition_flags: BYTE) -> bool;
    fn get_instruction_bits(&self) -> BYTE;
    fn decode_instruction(&self) -> fn(Self) -> ();
}

#[allow(dead_code)]
pub enum Instruction {
    ADD (WORD),
    BRANCH (WORD),
    NOP,
}

impl InstructionDecoder for ARMInstruction {
    fn get_instruction_bits(&self) -> BYTE {
        ((self & 0x0E00_0000) >> 25) as BYTE
    }

    fn decode_instruction(&self) -> fn(Self) -> () {
        if !(self.condition_passed(0x00)) {
            return super::instructions::nop;
        }

        match self.get_instruction_bits() {
            0b001 => arm_helpers::decode_data_processing_instruction(*self),
            0b000 => super::instructions::multiply,
            0b101 => super::instructions::branch,
                _ => panic!("Unimplemented Decode: {:#X}", self),
        }
    }

    fn condition_passed(&self, condition_flags: BYTE) -> bool {
        let condition = (self & 0xF0000000) >> 28;
        match condition {
            0b1110 => true,
            _ => panic!("Not implemented"),
        }
    }
}

mod arm_helpers {
    use crate::arm7tdmi::instructions;
    use super::{ARMExecutable, ARMInstruction};

    pub fn decode_data_processing_instruction(instruction: ARMInstruction) -> ARMExecutable {
    }
}
