use crate::types::REGISTER;

use super::arm::alu::DataProcessingInstruction;

pub enum Instruction {
    NOP,
    DataProcessing(DataProcessingInstruction)
}

pub enum Operand {
    Register(REGISTER),
    Immeidate(u32)
}
