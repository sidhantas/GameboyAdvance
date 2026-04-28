use std::fmt::Display;

use tui::widgets::Block;

use crate::{
    arm7tdmi::{
        arm::{
            alu::{ALUInstruction, MRSInstruction, MSRInstruction},
            branch::{BranchAndExchangeInstruction, BranchInstruction, SWI},
            data_transfer_instructions::BlockDTInstruction,
            swap_instruction::SwapInstruction,
        },
        thumb::{
            self,
            alu::{
                ThumbALUInstruction, ThumbALUOperation, ThumbAddToSp, ThumbAdr,
                ThumbArithmeticImmInstruction, ThumbBx, ThumbFullAdder, ThumbHiRegInstruction,
                ThumbMoveShiftedRegister,
            },
            data_transfer_instructions::{
                ThumbBlockDT, ThumbPushPop, ThumbSdtHwImmOffset, ThumbSdtImmOffset, ThumbSdtSpImm,
            },
            jumps_and_calls::{
                ThumbConditionalBranch, ThumbLongBranchWithLink, ThumbSWI, ThumbSetLinkRegister, ThumbUnconditionalBranch
            },
        },
    },
    memory::memory::GBAMemory,
    types::{CYCLES, REGISTER},
};

use super::{
    arm::{
        data_transfer_instructions::{SdtInstruction, SignedAndHwDtInstruction},
        instructions::ARMDecodedInstruction,
        multiply::MultiplyInstruction,
    },
    cpu::CPU,
    thumb::data_transfer_instructions::{LdrPCRelative, ThumbSdtRegisterOffset},
};

pub(crate) trait Execute {
    fn execute(self, cpu: &mut CPU, memory: &mut GBAMemory) -> CYCLES;
}

pub(crate) trait DecodeARMInstructionToString {
    fn instruction_to_string(&self, condition_code: &str) -> String;
}

pub(crate) trait DecodeThumbInstructionToString {
    fn instruction_to_string(&self) -> String;
}

pub(crate) fn condition_code_as_str(condition_code: u32) -> &'static str {
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

pub(crate) enum Instruction {
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
    Swap(SwapInstruction),
    SWI(SWI),
    ThumbSWI(ThumbSWI),
    Multiply(MultiplyInstruction),
    LdrPcRelative(LdrPCRelative),
    ThumbSdtOffset(ThumbSdtRegisterOffset),
    ThumbSdtImmOffset(ThumbSdtImmOffset),
    ThumbSdtHwImmOffset(ThumbSdtHwImmOffset),
    ThumbSdtSpImm(ThumbSdtSpImm),
    ThumbPushPop(ThumbPushPop),
    ThumbBlockDT(ThumbBlockDT),
    ThumbConditionalBranch(ThumbConditionalBranch),
    ThumbUnconditionalBranch(ThumbUnconditionalBranch),
    ThumbSetLinkRegister(ThumbSetLinkRegister),
    ThumbLongBranchWithLink(ThumbLongBranchWithLink),
    NotImplemented(u32),
    Nop,
}

impl Execute for Instruction {
    fn execute(self, cpu: &mut CPU, memory: &mut GBAMemory) -> CYCLES {
        match self {
            Instruction::ALUInstruction(alu_instruction) => alu_instruction.execute(cpu, memory),
            Instruction::MSR(psr_transfer) => psr_transfer.execute(cpu, memory),
            Instruction::MRS(instruction) => instruction.execute(cpu, memory),
            Instruction::ThumbFullAdder(thumb_full_adder) => thumb_full_adder.execute(cpu, memory),
            Instruction::ThumbAluInstruction(thumb_alu_instruction) => {
                thumb_alu_instruction.execute(cpu, memory)
            }
            Instruction::ThumbMoveShiftedRegister(instruction) => instruction.execute(cpu, memory),
            Instruction::ThumbArithmeticImmInstruction(instruction) => {
                instruction.execute(cpu, memory)
            }
            Instruction::ThumbHiRegisterInstruction(instruction) => {
                instruction.execute(cpu, memory)
            }
            Instruction::ThumbBx(instruction) => instruction.execute(cpu, memory),
            Instruction::ThumbAdr(instruction) => instruction.execute(cpu, memory),
            Instruction::ThumbAddToSp(instruction) => instruction.execute(cpu, memory),
            Instruction::SdtInstruction(instruction) => instruction.execute(cpu, memory),
            Instruction::SignedAndHwDtInstruction(instruction) => instruction.execute(cpu, memory),
            Instruction::BlockDT(instruction) => instruction.execute(cpu, memory),
            Instruction::Branch(instruction) => instruction.execute(cpu, memory),
            Instruction::BranchAndExchange(instruction) => instruction.execute(cpu, memory),
            Instruction::SWI(instruction) => instruction.execute(cpu, memory),
            Instruction::Swap(instruction) => instruction.execute(cpu, memory),
            Instruction::Multiply(instruction) => instruction.execute(cpu, memory),
            Instruction::LdrPcRelative(instruction) => instruction.execute(cpu, memory),
            Instruction::ThumbSdtOffset(instruction) => instruction.execute(cpu, memory),
            Instruction::ThumbSdtImmOffset(instruction) => instruction.execute(cpu, memory),
            Instruction::ThumbSdtHwImmOffset(instruction) => instruction.execute(cpu, memory),
            Instruction::ThumbSdtSpImm(instruction) => instruction.execute(cpu, memory),
            Instruction::ThumbPushPop(instruction) => instruction.execute(cpu, memory),
            Instruction::ThumbBlockDT(instruction) => instruction.execute(cpu, memory),
            Instruction::ThumbConditionalBranch(instruction) => instruction.execute(cpu, memory),
            Instruction::ThumbUnconditionalBranch(instruction) => instruction.execute(cpu, memory),
            Instruction::ThumbSetLinkRegister(instruction) => instruction.execute(cpu, memory),
            Instruction::ThumbLongBranchWithLink(instruction) => instruction.execute(cpu, memory),
            Instruction::ThumbSWI(instruction) => instruction.execute(cpu, memory),
            Instruction::NotImplemented(instruction) => {
                panic!("Not implemented: {:#x}", instruction)
            }
            Instruction::Nop => return 0,
        }
    }
}

pub(crate) fn instruction_to_string(condition_code: u32, instruction: Instruction) -> String {
    let condition_code = condition_code_as_str(condition_code);

    let executed_instruction_print = match instruction {
        Instruction::ALUInstruction(data_processing_instruction) => {
            data_processing_instruction.instruction_to_string(condition_code)
        }
        Instruction::MRS(data_processing_instruction) => {
            data_processing_instruction.instruction_to_string(condition_code)
        }
        Instruction::MSR(data_processing_instruction) => {
            data_processing_instruction.instruction_to_string(condition_code)
        }
        Instruction::ThumbFullAdder(full_adder) => full_adder.instruction_to_string(),
        Instruction::ThumbMoveShiftedRegister(instruction) => instruction.instruction_to_string(),
        Instruction::ThumbAluInstruction(instruction) => instruction.instruction_to_string(),
        Instruction::ThumbArithmeticImmInstruction(instruction) => {
            instruction.instruction_to_string()
        }
        Instruction::ThumbHiRegisterInstruction(instruction) => instruction.instruction_to_string(),
        Instruction::ThumbBx(instruction) => instruction.instruction_to_string(),
        Instruction::ThumbAdr(instruction) => instruction.instruction_to_string(),
        Instruction::ThumbAddToSp(instruction) => instruction.instruction_to_string(),
        Instruction::ThumbSdtImmOffset(instruction) => instruction.instruction_to_string(),
        Instruction::ThumbSdtHwImmOffset(instruction) => instruction.instruction_to_string(),
        Instruction::SdtInstruction(instruction) => {
            instruction.instruction_to_string(condition_code)
        }
        Instruction::SignedAndHwDtInstruction(instruction) => {
            instruction.instruction_to_string(condition_code)
        }
        Instruction::BlockDT(instruction) => instruction.instruction_to_string(condition_code),
        Instruction::Branch(instruction) => instruction.instruction_to_string(condition_code),
        Instruction::BranchAndExchange(instruction) => {
            instruction.instruction_to_string(condition_code)
        }
        Instruction::SWI(instruction) => instruction.instruction_to_string(condition_code),
        Instruction::Swap(instruction) => instruction.instruction_to_string(condition_code),
        Instruction::Multiply(instruction) => instruction.instruction_to_string(condition_code),
        Instruction::LdrPcRelative(instruction) => instruction.instruction_to_string(),
        Instruction::ThumbSdtOffset(instruction) => instruction.instruction_to_string(),
        Instruction::ThumbSdtSpImm(instruction) => instruction.instruction_to_string(),
        Instruction::ThumbPushPop(instruction) => instruction.instruction_to_string(),
        Instruction::ThumbBlockDT(instruction) => instruction.instruction_to_string(),
        Instruction::ThumbConditionalBranch(instruction) => instruction.instruction_to_string(),
        Instruction::ThumbUnconditionalBranch(instruction) => instruction.instruction_to_string(),
        Instruction::ThumbSetLinkRegister(instruction) => instruction.instruction_to_string(),
        Instruction::ThumbLongBranchWithLink(instruction) => instruction.instruction_to_string(),
        Instruction::ThumbSWI(instruction) => instruction.instruction_to_string(),
        Instruction::Nop => "nop".into(),
        Instruction::NotImplemented(_) => "not implemented".into(),
    };

    executed_instruction_print
}

#[derive(Debug)]
pub(crate) enum Operand {
    Register(REGISTER),
    Immediate(u32),
}
