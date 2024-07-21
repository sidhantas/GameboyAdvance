use crate::types::*;

pub type ARMINSTRUCTION = WORD;
type THUMBINSTRUCTION = HWORD;

pub trait InstructionDecoder {
    fn condition_passed(&self, condition_flags: BYTE) -> bool;
    fn decode_instruction(&self);
}



impl InstructionDecoder for ARMINSTRUCTION {
    fn decode_instruction(&self) {
        println!("{}", self.condition_passed(0x11));
    }

    fn condition_passed(&self, condition_flags: BYTE) -> bool {
         let condition = (self & 0xF0000000) >> 28;
         match condition {
             0b1110 => true,
             _ => panic!("Not implemented")
         }
    }
}
