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
            _ if arm_decoders::is_single_data_swap(instruction) => todo!(),
            _ if arm_decoders::is_halfword_data_transfer_register_offset(instruction) => todo!(),
            _ if arm_decoders::is_halfword_data_transfer_immediate_offset(instruction) => todo!(),
            _ if arm_decoders::is_signed_data_transfer(instruction) => todo!(),
            _ if arm_decoders::is_data_processing_and_psr_transfer(instruction) => todo!(),
            _ if arm_decoders::is_load_or_store_register_unsigned(instruction) => todo!(),
            _ if arm_decoders::is_undefined(instruction) => todo!(),
            _ if arm_decoders::is_block_data_transfer(instruction) => todo!(),
            _ if arm_decoders::is_branch_instruction(instruction) => branch,
            _ if arm_decoders::is_software_interrupt(instruction) => todo!(),
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

    #[inline(always)]
    pub fn is_single_data_swap(instruction: ARMByteCode) -> bool {
        instruction & 0x0FB0_0FF0 == 0x0100_0090
    }

    pub fn is_software_interrupt(instruction: u32) -> bool {
        instruction & 0x0F00_0000 == 0x0F00_0000
    }

    pub fn is_block_data_transfer(instruction: u32) -> bool {
        todo!()
    }

    pub fn is_undefined(instruction: u32) -> bool {
        todo!()
    }

    pub fn is_load_or_store_register_unsigned(instruction: u32) -> bool {
        todo!()
    }

    pub fn is_data_processing_and_psr_transfer(instruction: u32) -> bool {
        todo!()
    }

    pub fn is_signed_data_transfer(instruction: u32) -> bool {
        todo!()
    }

    pub fn is_halfword_data_transfer_immediate_offset(instruction: u32) -> bool { 
        instruction & 0x0E40_0090 == 0x0040_0090
    }

    pub fn is_halfword_data_transfer_register_offset(instruction: u32) -> bool {
        println!("{:#x} {:#x}", instruction, instruction & 0x0E40_0f90);
        instruction & 0x0E40_0f90 == 0x0000_0090
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
    use arm_decoders::*;

    use super::*;

    #[test]
    fn it_recognizes_a_multiplication_instruction() {
        let instruction: ARMByteCode = 0xE0230192;
        assert!(is_multiply_instruction(instruction) == true);
        let instruction: ARMByteCode = 0xE0250391;
        assert!(is_multiply_instruction(instruction) == true);
    }

    #[test]
    fn it_recognizes_a_single_data_swap_instruction() {
        let instruction: ARMByteCode = 0xE1013092;
        assert!(is_single_data_swap(instruction) == true);
        let instruction: ARMByteCode = 0xE1413092;
        assert!(is_single_data_swap(instruction) == true);
    }

    #[test]
    fn it_recognizes_a_software_interrupt_instruction() {
        let instruction: ARMByteCode = 0xef173f18;
        assert!(is_software_interrupt(instruction) == true);
    }

    #[test]
    fn it_recognizes_a_halfword_data_transfer_register_offset() {
        let instruction: ARMByteCode = 0xe19100b3;
        assert!(is_halfword_data_transfer_register_offset(instruction))
    }

    #[test]
    fn it_recognizes_a_halfword_data_transfer_immediate_offset() {
        let instruction: ARMByteCode = 0xe1d207bb;
        assert!(is_halfword_data_transfer_immediate_offset(instruction))
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
