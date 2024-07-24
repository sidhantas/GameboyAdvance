use crate::types::*;
use super::instructions::*;

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

impl InstructionDecoder for ARMByteCode {
    fn get_instruction_bits(&self) -> BYTE {
        ((self & 0x0E00_0000) >> 25) as BYTE
    }

    fn decode_instruction(&self) -> fn(Self) -> () {
        if !(self.condition_passed(0x00)) {
            return super::instructions::nop;
        }

        let instruction = *self;
        match instruction {
            _ if arm_decoders::is_multiply_instruction(instruction) => sub_decoders::decode_multiply(instruction),
            _ if arm_decoders::is_multiply_long_instruction(instruction) => multiply_long,
            _ if arm_decoders::is_branch_instruction(instruction) => branch,
            0b001 => sub_decoders::decode_data_processing_with_immediate_instruction(instruction),
            0b101 => super::instructions::branch,
                _ => panic!("Unimplemented Decode: {:#X}", instruction),
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

mod arm_decoders {
    use super::ARMByteCode;

    #[inline(always)]
    pub fn is_multiply_instruction(instruction: ARMByteCode) -> bool {
        instruction & 0b0000_1111_1100_0000_0000_0000_1111_0000 == 0b0000_0000_0000_0000_0000_0000_0000_1001_0000
    }

    #[inline(always)]
    pub fn is_multiply_long_instruction(instruction: ARMByteCode) -> bool {
        instruction & 0b0000_1111_1000_0000_0000_0000_1111_0000 == 0b0000_0000_1000_0000_0000_0000_0000_1001_0000
    }

    #[inline(always)]
    pub fn is_branch_instruction(instruction: ARMByteCode) -> bool {
        instruction & 0x0E00_0000 == 0x0A00_0000
    }

}

mod sub_decoders {
    use crate::utils::bits::Bits;
    use super::*;

    pub fn decode_data_processing_with_immediate_instruction(instruction: ARMByteCode) -> ARMExecutable {
        todo!()
    }

    pub fn decode_multiply(instruction: ARMByteCode) -> ARMExecutable {
        if instruction.bit_is_set(21) {
            return multiply_accumulate;
        }
        multiply
    }
}

#[cfg(test)]
mod arm_decoders_tests {
    use arm_decoders::is_multiply_instruction;

    use super::*;

    #[test]
    fn it_recognizes_a_multiplication_instruction() {
        let instruction: ARMByteCode = 0xE0230192;
        assert!(is_multiply_instruction(instruction) == true);
        let instruction: ARMByteCode = 0xE0250391;
        assert!(is_multiply_instruction(instruction) == true);
    }
}

#[cfg(test)]
mod sub_decoder_tests {
    use crate::arm7tdmi::decoder::*;

    #[test]
    fn it_returns_a_multiply_instruction() {
        let instruction: ARMByteCode = 0xE0230192;
        assert!(instruction.decode_instruction() == multiply_accumulate);
        let instruction: ARMByteCode = 0xE0050091;
        assert!(instruction.decode_instruction() == multiply);
    }

    #[test]
    fn it_returns_a_branch_instruction() {
        let instruction: ARMByteCode = 0xea000005;
        assert!(instruction.decode_instruction() == branch);
    }
}
