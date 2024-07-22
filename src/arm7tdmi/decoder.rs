use crate::types::*;
use crate::utils::bits::Bits;

pub type ARMINSTRUCTION = WORD;
#[allow(dead_code)]
pub type THUMBINSTRUCTION = HWORD;

pub trait InstructionDecoder {
    fn condition_passed(&self, condition_flags: BYTE) -> bool;
    fn get_instruction_bits(&self) -> BYTE;
    fn decode_instruction(&self) -> Instruction;
}

pub enum Instruction {
    ADD {set_flags: bool, Rn: u8, Rd: u8, shifter: u16},
    BRANCH {link: bool, immediate: u32},
    NOP
}

impl InstructionDecoder for ARMINSTRUCTION {
    fn get_instruction_bits(&self) -> BYTE {
        ((self & 0x0E00_0000) >> 25) as BYTE
    }

    fn decode_instruction(&self) -> Instruction {
        if !(self.condition_passed(0x00)) {
            return Instruction::NOP;
        }

        match self.get_instruction_bits() {
            0b101 =>  Instruction::BRANCH {link: self.bit_is_set(24), immediate: self & 0x00FF_FFFF},
            _ => Instruction::NOP
        }
    }

    fn condition_passed(&self, condition_flags: BYTE) -> bool {
         let condition = (self & 0xF0000000) >> 28;
         match condition {
             0b1110 => true,
             _ => panic!("Not implemented")
         }
    }
}
