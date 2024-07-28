use sub_decoders::decode_data_processing_with_immediate_instruction;

use super::instructions::*;
use crate::types::*;

pub trait InstructionDecoder {
    fn condition_passed(&self, condition_flags: BYTE) -> bool;
    fn get_instruction_bits(&self) -> BYTE;
    fn decode_instruction(&self) -> fn(Self) -> ();
}

#[allow(dead_code)]
pub enum Instruction {
    ADD(WORD),
    BRANCH(WORD),
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
            _ if arm_decoders::is_multiply_instruction(instruction) => {
                sub_decoders::decode_multiply(instruction)
            }
            _ if arm_decoders::is_multiply_long_instruction(instruction) => multiply_long,
            _ if arm_decoders::is_branch_and_exchange_instruction(instruction) => todo!(),
            _ if arm_decoders::is_single_data_swap(instruction) => todo!(),
            _ if arm_decoders::is_halfword_data_transfer_register_offset(instruction) => todo!(),
            _ if arm_decoders::is_halfword_data_transfer_immediate_offset(instruction) => todo!(),
            _ if arm_decoders::is_single_data_transfer(instruction) => todo!(),
            _ if arm_decoders::is_data_processing_and_psr_transfer(instruction) => decode_data_processing_with_immediate_instruction(instruction),
            _ if arm_decoders::is_load_or_store_register_unsigned(instruction) => todo!(),
            _ if arm_decoders::is_undefined(instruction) => todo!(),
            _ if arm_decoders::is_block_data_transfer(instruction) => todo!(),
            _ if arm_decoders::is_branch_instruction(instruction) => branch,
            _ if arm_decoders::is_software_interrupt(instruction) => todo!(),
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
        instruction & 0b0000_1111_1100_0000_0000_0000_1111_0000
            == 0b0000_0000_0000_0000_0000_0000_0000_1001_0000
    }

    #[inline(always)]
    pub fn is_multiply_long_instruction(instruction: ARMByteCode) -> bool {
        instruction & 0b0000_1111_1000_0000_0000_0000_1111_0000
            == 0b0000_0000_1000_0000_0000_0000_0000_1001_0000
    }

    #[inline(always)]
    pub fn is_branch_instruction(instruction: ARMByteCode) -> bool {
        instruction & 0x0E00_0000 == 0x0A00_0000
    }

    #[inline(always)]
    pub fn is_single_data_swap(instruction: ARMByteCode) -> bool {
        instruction & 0x0FB0_0FF0 == 0x0100_0090
    }

    pub fn is_software_interrupt(instruction: u32) -> bool {
        instruction & 0x0F00_0000 == 0x0F00_0000
    }

    pub fn is_block_data_transfer(instruction: u32) -> bool {
        instruction & 0x0E00_0000 == 0x0800_0000
    }

    pub fn is_undefined(instruction: u32) -> bool {
        instruction & 0x0E00_0010 == 0x0600_0010
    }

    pub fn is_load_or_store_register_unsigned(instruction: u32) -> bool {
        instruction & 0x0C00_0000 == 0x0400_0000
    }

    pub fn is_data_processing_and_psr_transfer(instruction: u32) -> bool {
        instruction & 0x0E00_0000 == 0x0200_0000
    }

    pub fn is_single_data_transfer(instruction: u32) -> bool {
        instruction & 0x0E00_0000 == 0x0600_0000
    }

    pub fn is_halfword_data_transfer_immediate_offset(instruction: u32) -> bool {
        instruction & 0x0E40_0090 == 0x0040_0090
    }

    pub fn is_halfword_data_transfer_register_offset(instruction: u32) -> bool {
        instruction & 0x0E40_0f90 == 0x0000_0090
    }

    pub fn is_branch_and_exchange_instruction(instruction: u32) -> bool {
        instruction & 0x0FFF_FFF0 == 0x012FFF10
    }
}

mod sub_decoders {
    use super::*;
    use crate::utils::bits::Bits;

    pub fn decode_data_processing_with_immediate_instruction(
        instruction: ARMByteCode,
    ) -> ARMExecutable {
        let opcode = instruction & 0x01E0_0000;
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
    use arm_decoders::*;

    use super::*;

    fn test_decoder(decoder: fn(ARMByteCode) -> bool, instructions: Vec<u32>) {
        for instruction in instructions {
            assert!(decoder(instruction) == true);
        }
    }

    #[test]
    fn it_recognizes_a_multiplication_instruction() {
        let multiplication_instructions = vec![0xE0230192, 0xE0250391];
        test_decoder(is_multiply_instruction, multiplication_instructions);
    }

    #[test]
    fn it_recognizes_a_single_data_swap_instruction() {
        let single_data_swap_instructions = vec![0xE1013092, 0xE1413092];
        test_decoder(is_single_data_swap, single_data_swap_instructions);
    }

    #[test]
    fn it_recognizes_a_software_interrupt_instruction() {
        let software_interrupt_ininstructions = vec![0xef173f18];
        test_decoder(is_software_interrupt, software_interrupt_ininstructions);
    }

    #[test]
    fn it_recognizes_a_halfword_data_transfer_register_offset() {
        let instructions = vec![0xe19100b3];
        test_decoder(is_halfword_data_transfer_register_offset, instructions);
    }

    #[test]
    fn it_recognizes_a_halfword_data_transfer_immediate_offset() {
        let instructions = vec![0xe1d207bb];
        test_decoder(is_halfword_data_transfer_immediate_offset, instructions);
    }

    #[test]
    fn it_recognizes_a_single_data_transfer_instruction() {
        let instructions = vec![0xe7910003];
        test_decoder(is_single_data_transfer, instructions);
    }

    #[test]
    fn it_recognizes_an_undefined_instruction() {
        let instructions = vec![0xe7000010];
        test_decoder(is_undefined, instructions);
    }

    #[test]
    fn it_recognizes_a_block_data_transfer() {
        let instructions = vec![0xe891003c];
        test_decoder(is_block_data_transfer, instructions);
    }

    #[test]
    fn it_recognizes_a_branch_instruction() {
        let instructions = vec![0xea000005];
        test_decoder(is_branch_instruction, instructions);
    }

    #[test]
    fn it_recognizes_a_branch_and_exchange_instruction() {
        let instructions = vec![0xe12fff11];
        test_decoder(is_branch_and_exchange_instruction, instructions);
    }

    #[test]
    fn it_recognizes_a_data_processing_instruction() {
        let instructions = vec![0xe2811001, 0xe2411001];
        test_decoder(is_data_processing_and_psr_transfer, instructions);
    }

    #[test]
    fn it_recognizes_a_load_store_instruction() {
        let instructions = vec![0xe59f101c, 0xe58f101c];
        test_decoder(is_load_or_store_register_unsigned, instructions);
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

    //    #[test]
    //    fn it_returns_a_branch_instruction() {
    //        let instruction: ARMByteCode = 0xea000005;
    //        assert!(instruction.decode_instruction() == branch);
    //    }
}
