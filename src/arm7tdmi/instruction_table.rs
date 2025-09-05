use std::fmt::Display;

use crate::{
    memory::memory::GBAMemory,
    types::{CYCLES, REGISTER},
};

use super::{
    arm::{alu::DataProcessingInstruction, instructions::ARMDecodedInstruction},
    cpu::CPU,
};

pub trait Execute {
    fn execute(self, cpu: &mut CPU, memory: &mut GBAMemory) -> CYCLES;
}

pub trait DecodeARMInstructionToString {
    fn instruction_to_string(&self, condition_code: &str) -> String;
}

fn condition_code_as_str(condition_code: u32) -> &'static str {
    match condition_code {
        0b0000 => "eq",
        0b0001 => "ne",
        0b0010 => "cs",
        0b0011 => "cc",
        0b0100 => "mi",
        0b0101 => "pl",
        0b0110 => "vs",
        0b0111 => "vc",
        0b1000 => "hi",
        0b1001 => "ls",
        0b1010 => "ge",
        0b1011 => "lt",
        0b1100 => "gt",
        0b1101 => "le",
        0b1110 => "", //AL
        _ => unreachable!("Impossible Condition Code Encountered"),
    }
}

pub enum Instruction {
    DataProcessing(DataProcessingInstruction),
    Funcpointer(ARMDecodedInstruction),
}

impl Execute for Instruction {
    fn execute(self, cpu: &mut CPU, memory: &mut GBAMemory) -> CYCLES {
        match self {
            Self::Funcpointer(func) => (func.executable)(cpu, func.instruction, memory),
            Self::DataProcessing(instruction) => instruction.execute(cpu, memory),
        }
    }
}

#[derive(Debug)]
pub enum Operand {
    Register(REGISTER),
    Immeidate(u32),
}

impl Display for Operand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Operand::Register(reg) => write!(f, "r{reg}"),
            Operand::Immeidate(imm) => write!(f, "#{imm}"),
        }
    }
}
