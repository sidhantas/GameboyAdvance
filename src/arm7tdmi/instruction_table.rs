use std::fmt::Display;

use tui::widgets::Block;

use crate::{
    arm7tdmi::{
        arm::{alu::{ALUInstruction, MRSInstruction, MSRInstruction}, branch::{BranchAndExchangeInstruction, BranchInstruction, SWI}, data_transfer_instructions::BlockDTInstruction},
        thumb::{
            self,
            alu::{
                ThumbALUInstruction, ThumbALUOperation, ThumbAddToSp, ThumbAdr, ThumbArithmeticImmInstruction, ThumbBx, ThumbFullAdder, ThumbHiRegInstruction, ThumbMoveShiftedRegister
            },
        },
    },
    memory::memory::GBAMemory,
    types::{CYCLES, REGISTER},
};

use super::{arm::{data_transfer_instructions::{SdtInstruction, SignedAndHwDtInstruction}, instructions::ARMDecodedInstruction}, cpu::CPU};

pub trait Execute {
    fn execute(self, cpu: &mut CPU, memory: &mut GBAMemory) -> CYCLES;
}

pub trait DecodeARMInstructionToString {
    fn instruction_to_string(&self, condition_code: &str) -> String;
}

pub trait DecodeThumbInstructionToString {
    fn instruction_to_string(&self) -> String;
}

pub fn condition_code_as_str(condition_code: u32) -> &'static str {
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
    Funcpointer(ARMDecodedInstruction),
    ALUInstruction(ALUInstruction),
    MSR(MSRInstruction),
    MRS(MRSInstruction),
    ThumbFullAdder(ThumbFullAdder),
    ThumbAluInstruction(ThumbALUOperation),
    ThumbMoveShiftedRegister(ThumbMoveShiftedRegister),
    ThumbArithmeticImmInstruction(ThumbArithmeticImmInstruction),
    ThumbHiRegisterInstruction(ThumbHiRegInstruction),
    ThumbBx(ThumbBx),
    ThumbAdr(ThumbAdr),
    ThumbAddToSp(ThumbAddToSp),
    SdtInstruction(SdtInstruction),
    SignedAndHwDtInstruction(SignedAndHwDtInstruction),
    BlockDT(BlockDTInstruction),
    Branch(BranchInstruction),
    BranchAndExchange(BranchAndExchangeInstruction),
    SWI(SWI)
}

impl Execute for Instruction {
    fn execute(self, cpu: &mut CPU, memory: &mut GBAMemory) -> CYCLES {
        match self {
            Instruction::ALUInstruction(alu_instruction) => alu_instruction.execute(cpu, memory),
            Instruction::MSR(psr_transfer) => psr_transfer.execute(cpu, memory),
            Instruction::MRS(instruction) => instruction.execute(cpu, memory),
            Instruction::Funcpointer(func) => (func.executable)(cpu, func.instruction, memory),
            Instruction::ThumbFullAdder(thumb_full_adder) => thumb_full_adder.execute(cpu, memory),
            Instruction::ThumbAluInstruction(thumb_alu_instruction) => {
                thumb_alu_instruction.execute(cpu, memory)
            }
            Instruction::ThumbMoveShiftedRegister(instruction) => instruction.execute(cpu, memory),
            Instruction::ThumbArithmeticImmInstruction(instruction) => instruction.execute(cpu, memory),
            Instruction::ThumbHiRegisterInstruction(instruction) => instruction.execute(cpu, memory),
            Instruction::ThumbBx(instruction) => instruction.execute(cpu, memory),
            Instruction::ThumbAdr(instruction) => instruction.execute(cpu, memory),
            Instruction::ThumbAddToSp(instruction) => instruction.execute(cpu, memory),
            Instruction::SdtInstruction(instruction) => instruction.execute(cpu, memory),
            Instruction::SignedAndHwDtInstruction(instruction) => instruction.execute(cpu, memory),
            Instruction::BlockDT(instruction) => instruction.execute(cpu, memory),
            Instruction::Branch(instruction) => instruction.execute(cpu, memory),
            Instruction::BranchAndExchange(instruction) => instruction.execute(cpu, memory),
            Instruction::SWI(instruction) => instruction.execute(cpu, memory),
        }
    }
}

#[derive(Debug)]
pub enum Operand {
    Register(REGISTER),
    Immediate(u32),
}
