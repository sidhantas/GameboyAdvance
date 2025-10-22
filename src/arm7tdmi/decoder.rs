use data_transfer_instructions::{SdtInstruction, SignedAndHwDtInstruction};
use instructions::ARMDecodedInstruction;
use multiply::MultiplyInstruction;

use crate::{
    arm7tdmi::{
        arm::{
            alu::{ALUInstruction, MRSInstruction, MSRInstruction},
            branch::{BranchAndExchangeInstruction, BranchInstruction, SWI},
            data_transfer_instructions::BlockDTInstruction,
            swap_instruction::SwapInstruction,
        },
        thumb::{
            alu::{
                ThumbALUOperation, ThumbAddToSp, ThumbAdr, ThumbArithmeticImmInstruction, ThumbBx,
                ThumbFullAdder, ThumbHiRegInstruction, ThumbMoveShiftedRegister,
            },
            data_transfer_instructions::{ThumbSdtHwImmOffset, ThumbSdtImmOffset},
        },
    },
    types::*,
};

use super::{
    arm::*,
    cpu::{FlagsRegister, InstructionMode, CPU},
    instruction_table::Instruction,
    thumb::data_transfer_instructions::{LdrPCRelative, ThumbSdtRegisterOffset},
};

impl CPU {
    pub fn decode_instruction(&self, instruction: WORD) -> Instruction {
        return match self.get_instruction_mode() {
            InstructionMode::ARM => self.decode_arm_instruction(instruction),
            InstructionMode::THUMB => self.decode_thumb_instruction(instruction),
        };
    }

    fn condition_passed(&self, instruction: ARMByteCode) -> bool {
        let condition = (instruction & 0xF0000000) >> 28;
        match condition {
            0b0000 => self.get_flag(FlagsRegister::Z) == 1, //EQ
            0b0001 => self.get_flag(FlagsRegister::Z) == 0, //NE
            0b0010 => self.get_flag(FlagsRegister::C) == 1, //CS
            0b0011 => self.get_flag(FlagsRegister::C) == 0, //CC
            0b0100 => self.get_flag(FlagsRegister::N) == 1, //MI
            0b0101 => self.get_flag(FlagsRegister::N) == 0, //PL
            0b0110 => self.get_flag(FlagsRegister::V) == 1, //VS
            0b0111 => self.get_flag(FlagsRegister::V) == 0, //VC
            0b1000 => self.get_flag(FlagsRegister::C) == 1 && self.get_flag(FlagsRegister::Z) == 0, //HI
            0b1001 => self.get_flag(FlagsRegister::C) == 0 || self.get_flag(FlagsRegister::Z) == 1, //LS
            0b1010 => self.get_flag(FlagsRegister::N) == self.get_flag(FlagsRegister::V), //GE
            0b1011 => self.get_flag(FlagsRegister::N) != self.get_flag(FlagsRegister::V), //LT
            0b1100 => {
                self.get_flag(FlagsRegister::Z) == 0
                    && self.get_flag(FlagsRegister::N) == self.get_flag(FlagsRegister::V)
            } //GT
            0b1101 => {
                self.get_flag(FlagsRegister::Z) == 1
                    || self.get_flag(FlagsRegister::N) != self.get_flag(FlagsRegister::V)
            } //LE
            0b1110 => true,                                                               //AL
            _ => unreachable!("Impossible Condition Code Encountered"),
        }
    }

    fn decode_arm_instruction(&self, instruction: ARMByteCode) -> Instruction {
        if !(self.condition_passed(instruction)) {
            return Instruction::Funcpointer(ARMDecodedInstruction {
                executable: CPU::arm_nop,
                instruction,
                ..Default::default()
            });
        }

        match instruction {
            _ if instruction == 0x00 => return Instruction::Nop,
            _ if arm_decoders::is_multiply_instruction(instruction) => {
                return Instruction::Multiply(MultiplyInstruction(instruction))
            }
            _ if arm_decoders::is_block_data_transfer(instruction) => {
                return Instruction::BlockDT(BlockDTInstruction(instruction))
            }
            _ if arm_decoders::is_hw_or_signed_data_transfer(instruction) => {
                return Instruction::SignedAndHwDtInstruction(SignedAndHwDtInstruction(instruction))
            }
            _ if arm_decoders::is_branch_and_exchange_instruction(instruction) => {
                return Instruction::BranchAndExchange(BranchAndExchangeInstruction(instruction))
            }
            _ if arm_decoders::is_mrs(instruction) => {
                return Instruction::MRS(MRSInstruction(instruction))
            }
            _ if arm_decoders::is_msr(instruction) => {
                return Instruction::MSR(MSRInstruction(instruction))
            }
            _ if arm_decoders::is_single_data_swap(instruction) => {
                return Instruction::Swap(SwapInstruction(instruction))
            }
            _ if arm_decoders::is_data_processing(instruction) => {
                return Instruction::ALUInstruction(ALUInstruction(instruction))
            }
            _ if arm_decoders::is_branch_instruction(instruction) => {
                return Instruction::Branch(BranchInstruction(instruction))
            }
            _ if arm_decoders::is_load_or_store_register_unsigned(instruction) => {
                return Instruction::SdtInstruction(SdtInstruction(instruction))
            }
            _ if arm_decoders::is_software_interrupt(instruction) => {
                return Instruction::SWI(SWI(instruction))
            }
            _ => return Instruction::NotImplemented(instruction),
        }
    }

    fn decode_thumb_instruction(&self, instruction: ARMByteCode) -> Instruction {
        Instruction::Funcpointer(match instruction {
            _ if thumb_decoders::is_add_or_subtract_instruction(instruction) => {
                return Instruction::ThumbFullAdder(ThumbFullAdder(instruction))
            }
            _ if thumb_decoders::is_move_shifted_register(instruction) => {
                return Instruction::ThumbMoveShiftedRegister(ThumbMoveShiftedRegister(instruction))
            }
            _ if thumb_decoders::is_move_compare_add_subtract_immediate(instruction) => {
                return Instruction::ThumbArithmeticImmInstruction(ThumbArithmeticImmInstruction(
                    instruction,
                ))
            }
            _ if thumb_decoders::is_alu_operation(instruction) => {
                return Instruction::ThumbAluInstruction(ThumbALUOperation(instruction))
            }
            _ if thumb_decoders::is_thumb_bx(instruction) => {
                return Instruction::ThumbBx(ThumbBx(instruction))
            }
            _ if thumb_decoders::is_thumb_hi_reg_operation(instruction) => {
                return Instruction::ThumbHiRegisterInstruction(ThumbHiRegInstruction(instruction))
            }
            _ if thumb_decoders::is_load_pc_relative(instruction) => {
                return Instruction::LdrPcRelative(LdrPCRelative(instruction))
            }
            _ if thumb_decoders::is_sdt_register_offset(instruction) => {
                return Instruction::ThumbSdtOffset(ThumbSdtRegisterOffset(instruction))
            }
            _ if thumb_decoders::is_thumb_swi(instruction) => {
                return Instruction::SWI(SWI(instruction))
            }
            _ if thumb_decoders::is_sdt_imm_offset(instruction) => {
                return Instruction::ThumbSdtImmOffset(ThumbSdtImmOffset(instruction))
            }
            _ if thumb_decoders::is_sdt_halfword(instruction) => return Instruction::ThumbSdtHwImmOffset(ThumbSdtHwImmOffset(instruction)),
            _ if thumb_decoders::is_sdt_sp_imm(instruction) => ARMDecodedInstruction {
                instruction,
                executable: CPU::thumb_sdt_sp_imm,
            },
            _ if thumb_decoders::is_get_relative_address(instruction) => {
                return Instruction::ThumbAdr(ThumbAdr(instruction))
            }
            _ if thumb_decoders::is_add_offset_to_sp(instruction) => {
                return Instruction::ThumbAddToSp(ThumbAddToSp(instruction))
            }
            _ if thumb_decoders::is_push_pop(instruction) => ARMDecodedInstruction {
                instruction,
                executable: CPU::thumb_push_pop,
            },
            _ if thumb_decoders::is_thumb_block_dt(instruction) => ARMDecodedInstruction {
                instruction,
                executable: CPU::thumb_multiple_load_or_store,
            },
            _ if thumb_decoders::is_conditional_branch(instruction) => ARMDecodedInstruction {
                instruction,
                executable: CPU::thumb_conditional_branch,
            },
            _ if thumb_decoders::is_unconditional_branch(instruction) => ARMDecodedInstruction {
                instruction,
                executable: CPU::thumb_unconditional_branch,
            },
            _ if thumb_decoders::is_set_link_register(instruction) => ARMDecodedInstruction {
                instruction,
                executable: CPU::thumb_set_link_register,
            },
            _ if thumb_decoders::is_long_branch_with_link(instruction) => ARMDecodedInstruction {
                instruction,
                executable: CPU::thumb_long_branch_with_link,
            },
            _ => return Instruction::NotImplemented(instruction),
        })
    }
}

mod arm_decoders {
    use super::ARMByteCode;

    pub fn is_multiply_instruction(instruction: ARMByteCode) -> bool {
        instruction & 0b0000_1111_1100_0000_0000_0000_1111_0000
            == 0b0000_0000_0000_0000_0000_0000_0000_1001_0000
    }

    pub fn is_multiply_long_instruction(instruction: ARMByteCode) -> bool {
        instruction & 0b0000_1111_1000_0000_0000_0000_1111_0000
            == 0b0000_0000_1000_0000_0000_0000_0000_1001_0000
    }

    pub fn is_branch_instruction(instruction: ARMByteCode) -> bool {
        instruction & 0x0E00_0000 == 0x0A00_0000
    }

    pub fn is_branch_and_link_instruction(instruction: ARMByteCode) -> bool {
        instruction & 0x0F00_0000 == 0x0B00_0000
    }

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

    pub fn is_msr(instruction: u32) -> bool {
        instruction & 0x0DB0_0000 == 0x0120_0000
    }

    pub fn is_mrs(instruction: u32) -> bool {
        instruction & 0x0DB0_0000 == 0x0100_0000
    }

    pub fn is_data_processing(instruction: u32) -> bool {
        instruction & 0x0C00_0000 == 0x0000_0000
    }

    pub fn is_hw_or_signed_data_transfer(instruction: u32) -> bool {
        instruction & 0x0E00_0090 == 0x0000_0090
    }

    pub fn is_branch_and_exchange_instruction(instruction: u32) -> bool {
        instruction & 0x0FFF_FF00 == 0x012F_FF00
    }
}

mod thumb_decoders {
    pub fn is_add_or_subtract_instruction(instruction: u32) -> bool {
        instruction & 0xF800 == 0x1800
    }

    pub fn is_move_shifted_register(instruction: u32) -> bool {
        instruction & 0xE000 == 0x0000
    }

    pub fn is_move_compare_add_subtract_immediate(instruction: u32) -> bool {
        instruction & 0xE000 == 0x2000
    }

    pub fn is_alu_operation(instruction: u32) -> bool {
        instruction & 0xFC00 == 0x4000
    }

    pub fn is_thumb_hi_reg_operation(instruction: u32) -> bool {
        instruction & 0xFC00 == 0x4400
    }

    pub fn is_thumb_bx(instruction: u32) -> bool {
        instruction & 0xFF00 == 0x4700
    }

    pub fn is_load_pc_relative(instruction: u32) -> bool {
        instruction & 0xF800 == 0x4800
    }

    pub fn is_sdt_register_offset(instruction: u32) -> bool {
        instruction & 0xF000 == 0x5000
    }

    pub fn is_sdt_imm_offset(instruction: u32) -> bool {
        instruction & 0xE000 == 0x6000
    }

    pub fn is_sdt_halfword(instruction: u32) -> bool {
        instruction & 0xF000 == 0x8000
    }

    pub fn is_sdt_sp_imm(instruction: u32) -> bool {
        instruction & 0xF000 == 0x9000
    }

    pub fn is_get_relative_address(instruction: u32) -> bool {
        instruction & 0xF000 == 0xA000
    }

    pub fn is_add_offset_to_sp(instruction: u32) -> bool {
        instruction & 0xFF00 == 0xB000
    }

    pub fn is_push_pop(instruction: u32) -> bool {
        instruction & 0xF600 == 0xB400
    }

    pub fn is_thumb_block_dt(instruction: u32) -> bool {
        instruction & 0xF000 == 0xC000
    }

    pub fn is_conditional_branch(instruction: u32) -> bool {
        instruction & 0xF000 == 0xD000
    }
    pub fn is_unconditional_branch(instruction: u32) -> bool {
        instruction & 0xF800 == 0xE000
    }
    pub fn is_set_link_register(instruction: u32) -> bool {
        instruction & 0xF800 == 0xF000
    }
    pub fn is_long_branch_with_link(instruction: u32) -> bool {
        instruction & 0xF800 == 0xF800
    }
    pub fn is_thumb_swi(instruction: u32) -> bool {
        instruction & 0xFF00 == 0xDF00
    }
}

mod sub_decoders {
    use crate::{arm7tdmi::cpu::CPU, utils::bits::Bits};

    use super::{instructions::ARMDecodedInstruction, ARMByteCode};

    impl CPU {
        pub fn decode_multiply(&self, instruction: ARMByteCode) -> ARMDecodedInstruction {
            if instruction.bit_is_set(21) {
                return ARMDecodedInstruction {
                    executable: CPU::arm_multiply_accumulate,
                    instruction,
                    ..Default::default()
                };
            }
            return ARMDecodedInstruction {
                executable: CPU::arm_multiply,
                instruction,
                ..Default::default()
            };
        }
    }
}

#[cfg(test)]
mod arm_decoders_tests {

    use arm_decoders::*;

    use crate::{gba::GBA, memory::memory::GBAMemory};

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
    fn it_recognizes_a_jump() {
        let branch_instruction = vec![0xE35E0000];
        test_decoder(is_branch_instruction, branch_instruction);
    }

    #[test]
    fn it_recognizes_a_halfword_data_transfer_register_offset() {
        let instructions = vec![0xe1c130b0];
        test_decoder(is_hw_or_signed_data_transfer, instructions);
    }

    #[test]
    fn it_recognizes_a_halfword_data_transfer_immediate_offset() {
        let instructions = vec![0xe1d207bb];
        test_decoder(is_hw_or_signed_data_transfer, instructions);
    }

    #[test]
    fn it_recognizes_a_single_data_transfer_instruction() {
        let instructions = vec![0xe7910003];
        test_decoder(is_load_or_store_register_unsigned, instructions);
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
        test_decoder(is_data_processing, instructions);
    }

    #[test]
    fn it_recognizes_a_load_store_instruction() {
        let instructions = vec![0xe59f101c, 0xe58f101c];
        test_decoder(is_load_or_store_register_unsigned, instructions);
    }

    //    #[test]
    //    fn it_finds_single_data_swap() {
    //        let mut gba = GBA::new_no_bios();
    //        let instruction = 0xe1014093;
    //        assert!(gba.cpu.decode_arm_instruction(instruction).executable == CPU::single_data_swap)
    //    }
    //
    //    #[test]
    //    fn it_finds_block_data_transfer() {
    //        let mut gba = GBA::new_no_bios();
    //        let instruction = 0xe895001f;
    //        assert!(gba.cpu.decode_arm_instruction(instruction).executable == CPU::block_dt_execution)
    //    }
    //    #[test]
    //    fn it_finds_a_branch_and_exchange_instruction() {
    //        let mut gba = GBA::new_no_bios();
    //        let instruction = 0xe12fff10;
    //        assert!(
    //            gba.cpu.decode_arm_instruction(instruction).executable == CPU::arm_branch_and_exchange
    //        )
    //    }
    //
    //    #[test]
    //    fn it_finds_swi_instruction() {
    //        let mut gba = GBA::new_no_bios();
    //        let instruction = 0xef001234;
    //
    //        assert!(
    //            gba.cpu.decode_arm_instruction(instruction).executable == CPU::arm_software_interrupt
    //        );
    //    }
}
//
//#[cfg(test)]
//mod sub_decoder_tests {
//
//    use crate::{arm7tdmi::decoder::*, gba::GBA, memory::memory::GBAMemory};
//
//    #[test]
//    fn it_decodes_an_instruction_if_eq_satisfied() {
//        let mut gba = GBA::new_no_bios();
//        let instruction: ARMByteCode = 0x028210c8; // addeq r1, r2, 200
//        gba.cpu.set_flag(FlagsRegister::Z);
//        let decoded_instruction = gba.cpu.decode_instruction(instruction);
//        assert!(decoded_instruction.executable != CPU::arm_nop);
//        assert!(decoded_instruction.executable == CPU::data_processing_instruction);
//    }
//
//    #[test]
//    fn it_does_not_decode_an_instruction_if_eq_not_satisfied() {
//        let mut gba = GBA::new_no_bios();
//        let instruction: ARMByteCode = 0x028210c8; // addeq r1, r2, 200
//        gba.cpu.reset_flag(FlagsRegister::Z);
//        let decoded_instruction = gba.cpu.decode_instruction(instruction);
//        assert!(decoded_instruction.executable == CPU::arm_nop);
//        assert!(decoded_instruction.executable != CPU::data_processing_instruction);
//    }
//
//    #[test]
//    fn it_does_decode_an_instruction_if_ne_satisfied() {
//        let mut gba = GBA::new_no_bios();
//        let instruction: ARMByteCode = 0x128210c8; // addne r1, r2, 200
//        gba.cpu.reset_flag(FlagsRegister::Z);
//        let decoded_instruction = gba.cpu.decode_instruction(instruction);
//        assert!(decoded_instruction.executable != CPU::arm_nop);
//        assert!(decoded_instruction.executable == CPU::data_processing_instruction);
//    }
//
//    #[test]
//    fn it_does_not_decode_an_instruction_if_ne_not_satisfied() {
//        let mut gba = GBA::new_no_bios();
//        let instruction: ARMByteCode = 0x128210c8; // addne r1, r2, 200
//        gba.cpu.set_flag(FlagsRegister::Z);
//        let decoded_instruction = gba.cpu.decode_instruction(instruction);
//        assert!(decoded_instruction.executable == CPU::arm_nop);
//        assert!(decoded_instruction.executable != CPU::data_processing_instruction);
//    }
//
//    #[test]
//    fn it_does_decode_an_instruction_if_cs_satisfied() {
//        let mut gba = GBA::new_no_bios();
//        let instruction: ARMByteCode = 0x228210c8; // addcs r1, r2, 200
//        gba.cpu.set_flag(FlagsRegister::C);
//        let decoded_instruction = gba.cpu.decode_instruction(instruction);
//        assert!(decoded_instruction.executable != CPU::arm_nop);
//        assert!(decoded_instruction.executable == CPU::data_processing_instruction);
//    }
//    #[test]
//    fn it_does_decode_an_instruction_if_cc_satisfied() {
//        let mut gba = GBA::new_no_bios();
//        let instruction: ARMByteCode = 0x328210c8; // addcc r1, r2, 200
//        gba.cpu.reset_flag(FlagsRegister::C);
//        let decoded_instruction = gba.cpu.decode_instruction(instruction);
//        assert!(decoded_instruction.executable != CPU::arm_nop);
//        assert!(decoded_instruction.executable == CPU::data_processing_instruction);
//    }
//    #[test]
//    fn it_does_decode_an_instruction_if_mi_satisfied() {
//        let mut gba = GBA::new_no_bios();
//        let instruction: ARMByteCode = 0x428210c8; // addmi r1, r2, 200
//        gba.cpu.set_flag(FlagsRegister::N);
//        let decoded_instruction = gba.cpu.decode_instruction(instruction);
//        assert!(decoded_instruction.executable != CPU::arm_nop);
//        assert!(decoded_instruction.executable == CPU::data_processing_instruction);
//    }
//    #[test]
//    fn it_does_decode_an_instruction_if_pl_satisfied() {
//        let mut gba = GBA::new_no_bios();
//        let instruction: ARMByteCode = 0x528210c8; // addpl r1, r2, 200
//        gba.cpu.reset_flag(FlagsRegister::C);
//        let decoded_instruction = gba.cpu.decode_instruction(instruction);
//        assert!(decoded_instruction.executable != CPU::arm_nop);
//        assert!(decoded_instruction.executable == CPU::data_processing_instruction);
//    }
//    #[test]
//    fn it_does_decode_an_instruction_if_vs_satisfied() {
//        let mut gba = GBA::new_no_bios();
//        let instruction: ARMByteCode = 0x628210c8; // addvs r1, r2, 200
//        gba.cpu.set_flag(FlagsRegister::V);
//        let decoded_instruction = gba.cpu.decode_instruction(instruction);
//        assert!(decoded_instruction.executable != CPU::arm_nop);
//        assert!(decoded_instruction.executable == CPU::data_processing_instruction);
//    }
//    #[test]
//    fn it_does_decode_an_instruction_if_vc_satisfied() {
//        let mut gba = GBA::new_no_bios();
//        let instruction: ARMByteCode = 0x728210c8; // addvc r1, r2, 200
//        gba.cpu.reset_flag(FlagsRegister::V);
//        let decoded_instruction = gba.cpu.decode_instruction(instruction);
//        assert!(decoded_instruction.executable != CPU::arm_nop);
//        assert!(decoded_instruction.executable == CPU::data_processing_instruction);
//    }
//    #[test]
//    fn it_does_decode_an_instruction_if_hi_satisfied() {
//        let mut gba = GBA::new_no_bios();
//        let instruction: ARMByteCode = 0x828210c8; // addhi r1, r2, 200
//        gba.cpu.set_flag(FlagsRegister::C);
//        gba.cpu.reset_flag(FlagsRegister::Z);
//        let decoded_instruction = gba.cpu.decode_instruction(instruction);
//        assert!(decoded_instruction.executable != CPU::arm_nop);
//        assert!(decoded_instruction.executable == CPU::data_processing_instruction);
//    }
//    #[test]
//    fn it_does_decode_an_instruction_if_ls_satisfied() {
//        let mut gba = GBA::new_no_bios();
//        let instruction: ARMByteCode = 0x928210c8; // addls r1, r2, 200
//        gba.cpu.reset_flag(FlagsRegister::C);
//        gba.cpu.reset_flag(FlagsRegister::Z);
//        let decoded_instruction = gba.cpu.decode_instruction(instruction);
//        assert!(decoded_instruction.executable != CPU::arm_nop);
//        assert!(decoded_instruction.executable == CPU::data_processing_instruction);
//
//        gba.cpu.set_flag(FlagsRegister::C);
//        gba.cpu.set_flag(FlagsRegister::Z);
//        let decoded_instruction = gba.cpu.decode_instruction(instruction);
//        assert!(decoded_instruction.executable != CPU::arm_nop);
//        assert!(decoded_instruction.executable == CPU::data_processing_instruction);
//
//        gba.cpu.reset_flag(FlagsRegister::C);
//        gba.cpu.set_flag(FlagsRegister::Z);
//        let decoded_instruction = gba.cpu.decode_instruction(instruction);
//        assert!(decoded_instruction.executable != CPU::arm_nop);
//        assert!(decoded_instruction.executable == CPU::data_processing_instruction);
//    }
//
//    #[test]
//    fn it_does_decode_an_instruction_if_ge_satisfied() {
//        let mut gba = GBA::new_no_bios();
//        let instruction: ARMByteCode = 0xa28210c8; // addge r1, r2, 200
//        gba.cpu.set_flag(FlagsRegister::N);
//        gba.cpu.set_flag(FlagsRegister::V);
//        let decoded_instruction = gba.cpu.decode_instruction(instruction);
//        assert!(decoded_instruction.executable != CPU::arm_nop);
//        assert!(decoded_instruction.executable == CPU::data_processing_instruction);
//
//        gba.cpu.reset_flag(FlagsRegister::N);
//        gba.cpu.reset_flag(FlagsRegister::V);
//        let decoded_instruction = gba.cpu.decode_instruction(instruction);
//        assert!(decoded_instruction.executable != CPU::arm_nop);
//        assert!(decoded_instruction.executable == CPU::data_processing_instruction);
//    }
//    #[test]
//    fn it_does_decode_an_instruction_if_lt_satisfied() {
//        let mut gba = GBA::new_no_bios();
//        let instruction: ARMByteCode = 0xb28210c8; // addlt r1, r2, 200
//        gba.cpu.reset_flag(FlagsRegister::N);
//        gba.cpu.set_flag(FlagsRegister::V);
//        let decoded_instruction = gba.cpu.decode_instruction(instruction);
//        assert!(decoded_instruction.executable != CPU::arm_nop);
//        assert!(decoded_instruction.executable == CPU::data_processing_instruction);
//
//        gba.cpu.set_flag(FlagsRegister::N);
//        gba.cpu.reset_flag(FlagsRegister::V);
//        let decoded_instruction = gba.cpu.decode_instruction(instruction);
//        assert!(decoded_instruction.executable != CPU::arm_nop);
//        assert!(decoded_instruction.executable == CPU::data_processing_instruction);
//    }
//
//    #[test]
//    fn it_does_decode_an_instruction_if_gt_satisfied() {
//        let mut gba = GBA::new_no_bios();
//        let instruction: ARMByteCode = 0xc28210c8; // addgt r1, r2, 200
//        gba.cpu.reset_flag(FlagsRegister::Z);
//        gba.cpu.set_flag(FlagsRegister::N);
//        gba.cpu.set_flag(FlagsRegister::V);
//        let decoded_instruction = gba.cpu.decode_instruction(instruction);
//        assert!(decoded_instruction.executable != CPU::arm_nop);
//        assert!(decoded_instruction.executable == CPU::data_processing_instruction);
//
//        gba.cpu.reset_flag(FlagsRegister::Z);
//        gba.cpu.reset_flag(FlagsRegister::N);
//        gba.cpu.reset_flag(FlagsRegister::V);
//        let decoded_instruction = gba.cpu.decode_instruction(instruction);
//        assert!(decoded_instruction.executable != CPU::arm_nop);
//        assert!(decoded_instruction.executable == CPU::data_processing_instruction);
//    }
//
//    #[test]
//    fn it_does_decode_an_instruction_if_le_satisfied() {
//        let mut gba = GBA::new_no_bios();
//        let instruction: ARMByteCode = 0xd28210c8; // addle r1, r2, 200
//
//        gba.cpu.set_flag(FlagsRegister::Z);
//        gba.cpu.reset_flag(FlagsRegister::N);
//        gba.cpu.reset_flag(FlagsRegister::V);
//        let decoded_instruction = gba.cpu.decode_instruction(instruction);
//        assert!(decoded_instruction.executable != CPU::arm_nop);
//        assert!(decoded_instruction.executable == CPU::data_processing_instruction);
//
//        gba.cpu.reset_flag(FlagsRegister::Z);
//        gba.cpu.set_flag(FlagsRegister::N);
//        gba.cpu.reset_flag(FlagsRegister::V);
//        let decoded_instruction = gba.cpu.decode_instruction(instruction);
//        assert!(decoded_instruction.executable != CPU::arm_nop);
//        assert!(decoded_instruction.executable == CPU::data_processing_instruction);
//
//        gba.cpu.reset_flag(FlagsRegister::Z);
//        gba.cpu.reset_flag(FlagsRegister::N);
//        gba.cpu.set_flag(FlagsRegister::V);
//        let decoded_instruction = gba.cpu.decode_instruction(instruction);
//        assert!(decoded_instruction.executable != CPU::arm_nop);
//        assert!(decoded_instruction.executable == CPU::data_processing_instruction);
//    }
//    #[test]
//    fn it_does_decode_an_instruction_if_al_satisfied() {
//        let mut gba = GBA::new_no_bios();
//        let instruction: ARMByteCode = 0xe28210c8; // addal r1, r2, 200
//        let decoded_instruction = gba.cpu.decode_instruction(instruction);
//        assert!(decoded_instruction.executable != CPU::arm_nop);
//        assert!(decoded_instruction.executable == CPU::data_processing_instruction);
//    }
//
//    //    #[test]
//    //    fn it_returns_a_multiply_instruction() {
//    //        let memory = Memory::new().unwrap();
//    //
//    //        let mut gba = GBA::new_no_bios();
//    //
//    //        let instruction: ARMByteCode = 0xE0230192;
//    //        let decoded_instruction = cpu.decode_instruction(instruction);
//    //        assert!(cpu.decoded_instruction.executable == CPU::arm_multiply_accumulate);
//    //        let instruction: ARMByteCode = 0xE0050091;
//    //        let decoded_instruction = cpu.decode_instruction(instruction);
//    //        assert!(cpu.decoded_instruction.executable == CPU::arm_multiply);
//    //    }
//    //
//    //    #[test]
//    //    fn it_returns_a_branch_instruction() {
//    //        let memory = Memory::new().unwrap();
//    //
//    //        let mut gba = GBA::new_no_bios();
//    //
//    //        let instruction: ARMByteCode = 0xea000005;
//    //        let decoded_instruction = cpu.decode_instruction(instruction);
//    //        assert!(cpu.decoded_instruction.executable == CPU::arm_branch);
//    //    }
//
//    //    #[test]
//    //    fn it_returns_a_cmp_instruction() {
//    //        let memory = Memory::new().unwrap();
//    //
//    //        let mut gba = GBA::new_no_bios();
//    //
//    //        let instruction: ARMByteCode = 0xe1530312; //  cmp r3, r2, lsl r3
//    //                                                   //  shift by register in
//    //                                                   // order to stall the alu by
//    //                                                   // one clock cycle
//    //        cpu.prefetch[0] = instruction;
//    //        cpu.execute_cpu_cycle();
//    //        cpu.execute_cpu_cycle();
//    //        assert!(cpu.alu_executable.operation == CPU::arm_cmp);
//    //    }
//
//    //    #[test]
//    //    fn it_returns_an_add_instruction_with_an_imm_op2() {
//    //        let memory = Memory::new().unwrap();
//    //
//    //        let mut gba = GBA::new_no_bios();
//    //
//    //        let instruction: ARMByteCode = 0xe2812020; // add r
//    //        let decoded_instruction = cpu.decode_instruction(instruction);
//    //        assert!(cpu.decoded_instruction.executable == CPU::arm_add);
//    //        assert!(cpu.decoded_instruction.operand2 == 32);
//    //        assert!(cpu.decoded_instruction.rd == 0x2);
//    //        assert!(cpu.decoded_instruction.rn == 0x1);
//    //    }
//    //
//    //    #[test]
//    //    fn it_returns_an_add_instruction_an_lsl_operand2() {
//    //        let memory = Memory::new().unwrap();
//    //
//    //        let mut gba = GBA::new_no_bios();
//    //
//    //        let instruction: ARMByteCode = 0xe0831102; // add r1, r3, r2 LSL 2
//    //        cpu.set_register(2, 1);
//    //        let decoded_instruction = cpu.decode_instruction(instruction);
//    //        assert!(cpu.decoded_instruction.executable == CPU::arm_add);
//    //        assert!(cpu.decoded_instruction.rd == 0x1);
//    //        assert!(cpu.decoded_instruction.rn == 0x3);
//    //        assert!(cpu.decoded_instruction.operand2 == (1 << 2));
//    //    }
//    //
//    //    #[test]
//    //    fn it_returns_an_add_instruction_with_ror_10() {
//    //        let memory = Memory::new().unwrap();
//    //
//    //        let mut gba = GBA::new_no_bios();
//    //
//    //        let instruction: ARMByteCode = 0xe0831562; // add r1, r3, r2 ROR#10
//    //        cpu.set_register(2, 5);
//    //        let decoded_instruction = cpu.decode_instruction(instruction);
//    //        assert!(cpu.decoded_instruction.executable == CPU::arm_add);
//    //        assert!(cpu.decoded_instruction.rd == 0x1);
//    //        assert!(cpu.decoded_instruction.rn == 0x3);
//    //        assert!(cpu.decoded_instruction.operand2 == (5 as u32).rotate_right(10));
//    //    }
//    //
//    //    #[test]
//    //    fn it_returns_an_add_instruction_with_asr_10() {
//    //        let memory = Memory::new().unwrap();
//    //
//    //        let mut gba = GBA::new_no_bios();
//    //
//    //        let instruction: ARMByteCode = 0xe0831542; // add r1, r3, r2 ASR#10
//    //        cpu.set_register(2, 0xB000_0000);
//    //        let decoded_instruction = cpu.decode_instruction(instruction);
//    //        assert!(cpu.decoded_instruction.executable == CPU::arm_add);
//    //        assert!(cpu.decoded_instruction.rd == 0x1);
//    //        assert!(cpu.decoded_instruction.rn == 0x3);
//    //        assert!(cpu.decoded_instruction.operand2 == 0xFFEC0000);
//    //    }
//    //
//    //    #[test]
//    //    fn it_returns_an_add_instruction_with_lsr_10() {
//    //        let memory = Memory::new().unwrap();
//    //
//    //        let mut gba = GBA::new_no_bios();
//    //
//    //        let instruction: ARMByteCode = 0xe0831522; // add r1, r3, r2 LSR#10
//    //        cpu.set_register(2, 0xB000_0000);
//    //        let decoded_instruction = cpu.decode_instruction(instruction);
//    //        assert!(cpu.decoded_instruction.executable == CPU::arm_add);
//    //        assert!(cpu.decoded_instruction.rd == 0x1);
//    //        assert!(cpu.decoded_instruction.rn == 0x3);
//    //        assert!(cpu.decoded_instruction.operand2 == 0xB000_0000 >> 10);
//    //    }
//    //
//    //    #[test]
//    //    fn it_returns_an_add_instruction_with_lsr_32() {
//    //        let memory = Memory::new().unwrap();
//    //
//    //        let mut gba = GBA::new_no_bios();
//    //
//    //        let instruction: ARMByteCode = 0xe0931022; // adds r1, r3, r2 LSR#32
//    //        cpu.set_register(2, u32::MAX);
//    //        let decoded_instruction = cpu.decode_instruction(instruction);
//    //        assert!(cpu.decoded_instruction.executable == CPU::arm_add);
//    //        assert!(cpu.decoded_instruction.rd == 0x1);
//    //        assert!(cpu.decoded_instruction.rn == 0x3);
//    //        assert!(cpu.decoded_instruction.operand2 == 0);
//    //        assert!(cpu.get_flag(FlagsRegister::C) == 1);
//    //    }
//    //
//    //    #[test]
//    //    fn it_returns_an_add_instruction_with_an_asr_32_negative() {
//    //        let memory = Memory::new().unwrap();
//    //
//    //        let mut gba = GBA::new_no_bios();
//    //
//    //        let instruction: ARMByteCode = 0xe0931042; // adds r1, r3, r2 ASR#32
//    //        cpu.set_register(2, 0xF000_1000);
//    //        let decoded_instruction = cpu.decode_instruction(instruction);
//    //        assert!(cpu.decoded_instruction.executable == CPU::arm_add);
//    //        assert!(cpu.decoded_instruction.rd == 0x1);
//    //        assert!(cpu.decoded_instruction.rn == 0x3);
//    //        assert!(cpu.decoded_instruction.operand2 == u32::MAX);
//    //        assert!(cpu.get_flag(FlagsRegister::C) == 1);
//    //    }
//    //
//    //    #[test]
//    //    fn it_returns_an_add_instruction_with_an_asr_32_positive() {
//    //        let memory = Memory::new().unwrap();
//    //
//    //        let mut gba = GBA::new_no_bios();
//    //
//    //        let instruction: ARMByteCode = 0xe0831042; // add r1, r3, r2 ASR#32
//    //        cpu.set_register(2, 0x0000_1000);
//    //        let decoded_instruction = cpu.decode_instruction(instruction);
//    //        assert!(cpu.decoded_instruction.executable == CPU::arm_add);
//    //        assert!(cpu.decoded_instruction.rd == 0x1);
//    //        assert!(cpu.decoded_instruction.rn == 0x3);
//    //        assert!(cpu.decoded_instruction.operand2 == 0);
//    //        assert!(cpu.get_flag(FlagsRegister::C) == 0);
//    //    }
//    //
//    //    #[test]
//    //    fn it_returns_an_add_instruction_with_op2_shifted_by_register() {
//    //        let memory = Memory::new().unwrap();
//    //
//    //        let mut gba = GBA::new_no_bios();
//    //
//    //        let instruction: ARMByteCode = 0xe0831412; // add r1, r3, r2 LSL r4
//    //        cpu.set_register(2, 0x0000_1000);
//    //        cpu.set_register(4, 5);
//    //        let decoded_instruction = cpu.decode_instruction(instruction);
//    //        assert!(cpu.decoded_instruction.executable == CPU::arm_add);
//    //        assert!(cpu.decoded_instruction.rd == 0x1);
//    //        assert!(cpu.decoded_instruction.rn == 0x3);
//    //        assert!(cpu.decoded_instruction.operand2 == 0x0002_0000);
//    //    }
//}
//
//#[cfg(test)]
//mod thumb_decoder_tests {
//
//    use crate::{
//        arm7tdmi::cpu::{InstructionMode, CPU},
//        gba::GBA,
//        memory::memory::GBAMemory,
//    };
//
//    #[test]
//    fn it_recognizes_sdt_imm_offset() {
//        let instruction = 0x68cd; // ldr r5, [r1, 12]
//
//        let mut gba = GBA::new_no_bios();
//        gba.cpu.set_instruction_mode(InstructionMode::THUMB);
//
//        let decoded_instruction = gba.cpu.decode_instruction(instruction);
//        assert!(decoded_instruction.executable != CPU::arm_nop);
//        assert!(decoded_instruction.executable == CPU::sdt_imm_offset);
//    }
//
//    #[test]
//    fn it_recognizes_sdt_sp_imm_offset() {
//        let instruction = 0x9d03; // ldr r5, [sp, 12]
//
//        let mut gba = GBA::new_no_bios();
//        gba.cpu.set_instruction_mode(InstructionMode::THUMB);
//
//        let decoded_instruction = gba.cpu.decode_instruction(instruction);
//        assert!(decoded_instruction.executable != CPU::arm_nop);
//        assert!(decoded_instruction.executable == CPU::thumb_sdt_sp_imm);
//    }
//
//    #[test]
//    fn it_recognizes_add_offset_to_sp() {
//        let instruction = 0xb07d; // add sp, 500
//
//        let mut gba = GBA::new_no_bios();
//        gba.cpu.set_instruction_mode(InstructionMode::THUMB);
//
//        let decoded_instruction = gba.cpu.decode_instruction(instruction);
//        assert!(decoded_instruction.executable != CPU::arm_nop);
//        assert!(decoded_instruction.executable == CPU::thumb_add_offset_to_sp);
//    }
//
//    #[test]
//    fn it_recognizes_thumb_push() {
//        let instruction = 0xb503; // push {r0-r1, lr}
//
//        let mut gba = GBA::new_no_bios();
//        gba.cpu.set_instruction_mode(InstructionMode::THUMB);
//
//        let decoded_instruction = gba.cpu.decode_instruction(instruction);
//        assert!(decoded_instruction.executable != CPU::arm_nop);
//        assert!(decoded_instruction.executable == CPU::thumb_push_pop);
//    }
//
//    #[test]
//    fn it_recognizes_thumb_bdt() {
//        let instruction = 0xc107; // stmia r1 {r0-r2}
//
//        let mut gba = GBA::new_no_bios();
//        gba.cpu.set_instruction_mode(InstructionMode::THUMB);
//
//        let decoded_instruction = gba.cpu.decode_instruction(instruction);
//        assert!(decoded_instruction.executable != CPU::arm_nop);
//        assert!(decoded_instruction.executable == CPU::thumb_multiple_load_or_store);
//    }
//}
