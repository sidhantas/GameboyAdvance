use super::{
    cpu::{FlagsRegister, InstructionMode, CPU},
    instructions::ARMDecodedInstruction,
};
use crate::types::*;

#[allow(dead_code)]
pub enum Instruction {
    ADD(WORD),
    BRANCH(WORD),
    NOP,
}

impl CPU {
    pub fn decode_instruction(&mut self, instruction: ARMByteCode) {
        self.decoded_instruction = match self.inst_mode {
            InstructionMode::ARM => Some(self.decode_arm_instruction(instruction)),
            InstructionMode::THUMB => Some(self.decode_thumb_instruction(instruction)),
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
            _ => panic!("Impossible Condition Code Encountered"),
        }
    }

    fn decode_arm_instruction(&mut self, instruction: ARMByteCode) -> ARMDecodedInstruction {
        if !(self.condition_passed(instruction)) {
            return ARMDecodedInstruction {
                executable: CPU::arm_nop,
                instruction,
                ..Default::default()
            };
        }

        match instruction {
            _ if instruction == 0x00 => ARMDecodedInstruction {
                executable: CPU::arm_nop,
                instruction,
                ..Default::default()
            },
            _ if arm_decoders::is_multiply_instruction(instruction) => {
                self.decode_multiply(instruction)
            }
            _ if arm_decoders::is_block_data_transfer(instruction) => ARMDecodedInstruction {
                executable: CPU::block_dt_execution,
                instruction,
            },
            _ if arm_decoders::is_single_data_swap(instruction) => ARMDecodedInstruction {
                executable: CPU::single_data_swap,
                instruction,
            },
            _ if arm_decoders::is_hw_or_signed_data_transfer(instruction) => {
                ARMDecodedInstruction {
                    executable: CPU::hw_or_signed_data_transfer,
                    instruction,
                }
            }
            _ if arm_decoders::is_multiply_long_instruction(instruction) => ARMDecodedInstruction {
                executable: CPU::arm_multiply_long,
                instruction,
                ..Default::default()
            },
            _ if arm_decoders::is_branch_and_exchange_instruction(instruction) => {
                ARMDecodedInstruction {
                    executable: CPU::arm_branch_and_exchange,
                    instruction,
                }
            }
            _ if arm_decoders::is_data_processing_and_psr_transfer(instruction) => {
                ARMDecodedInstruction {
                    executable: CPU::data_processing_instruction,
                    instruction,
                }
            }
            _ if arm_decoders::is_branch_instruction(instruction) => ARMDecodedInstruction {
                executable: CPU::arm_branch,
                instruction,
                ..Default::default()
            },
            _ if arm_decoders::is_load_or_store_register_unsigned(instruction) => {
                ARMDecodedInstruction {
                    instruction,
                    executable: CPU::sdt_instruction_execution,
                }
            }
            _ => ARMDecodedInstruction {
                executable: CPU::arm_not_implemented,
                instruction,
                ..Default::default()
            },
        }
    }

    fn decode_thumb_instruction(&self, instruction: ARMByteCode) -> ARMDecodedInstruction {
        match instruction {
            _ if thumb_decoders::is_add_or_subtract_instruction(instruction) => {
                ARMDecodedInstruction {
                    instruction,
                    executable: CPU::thumb_add_or_subtract_instruction
                }
            }

            _ if thumb_decoders::is_move_shifted_register(instruction) => {
                ARMDecodedInstruction {
                    instruction,
                    executable: CPU::thumb_move_shifted_register_instruction
                }
            }
            _ if thumb_decoders::is_move_compare_add_subtract_immediate(instruction) => {
                ARMDecodedInstruction {
                    instruction,
                    executable: CPU::thumb_move_add_compare_add_subtract_immediate
                }
            }
            _ => ARMDecodedInstruction {
                instruction,
                executable: CPU::arm_not_implemented,
                ..Default::default()
            },
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
}

mod sub_decoders {
    use crate::{
        arm7tdmi::{cpu::CPU, instructions::ARMDecodedInstruction},
        utils::bits::Bits,
    };

    use super::ARMByteCode;

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
    use std::sync::{Arc, Mutex};

    use arm_decoders::*;

    use crate::memory::Memory;

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
        test_decoder(is_data_processing_and_psr_transfer, instructions);
    }

    #[test]
    fn it_recognizes_a_load_store_instruction() {
        let instructions = vec![0xe59f101c, 0xe58f101c];
        test_decoder(is_load_or_store_register_unsigned, instructions);
    }

    #[test]
    fn it_finds_single_data_swap() {
        let memory = Memory::new().unwrap();
        let cpu_memory = Arc::new(Mutex::new(memory));
        let mut cpu = CPU::new(cpu_memory);
        let instruction = 0xe1014093;
        assert!(cpu.decode_arm_instruction(instruction).executable == CPU::single_data_swap)
    }

    #[test]
    fn it_finds_block_data_transfer() {
        let memory = Memory::new().unwrap();
        let cpu_memory = Arc::new(Mutex::new(memory));
        let mut cpu = CPU::new(cpu_memory);
        let instruction = 0xe895001f;
        assert!(cpu.decode_arm_instruction(instruction).executable == CPU::block_dt_execution)
    }
    #[test]
    fn it_finds_a_branch_and_exchange_instruction() {
        let memory = Memory::new().unwrap();
        let cpu_memory = Arc::new(Mutex::new(memory));
        let mut cpu = CPU::new(cpu_memory);
        let instruction = 0xe12fff10;
        assert!(cpu.decode_arm_instruction(instruction).executable == CPU::arm_branch_and_exchange)
    }
}

#[cfg(test)]
mod sub_decoder_tests {
    use std::sync::{Arc, Mutex};

    use crate::{arm7tdmi::decoder::*, memory::Memory};

    #[test]
    fn it_decodes_an_instruction_if_eq_satisfied() {
        let memory = Memory::new().unwrap();
        let memory = Arc::new(Mutex::new(memory));
        let mut cpu = CPU::new(memory);
        let instruction: ARMByteCode = 0x028210c8; // addeq r1, r2, 200
        cpu.set_flag(FlagsRegister::Z);
        cpu.decode_instruction(instruction);
        assert!(cpu.decoded_instruction.unwrap().executable != CPU::arm_nop);
        assert!(cpu.decoded_instruction.unwrap().executable == CPU::data_processing_instruction);
    }

    #[test]
    fn it_does_not_decode_an_instruction_if_eq_not_satisfied() {
        let memory = Memory::new().unwrap();
        let memory = Arc::new(Mutex::new(memory));
        let mut cpu = CPU::new(memory);
        let instruction: ARMByteCode = 0x028210c8; // addeq r1, r2, 200
        cpu.reset_flag(FlagsRegister::Z);
        cpu.decode_instruction(instruction);
        assert!(cpu.decoded_instruction.unwrap().executable == CPU::arm_nop);
        assert!(cpu.decoded_instruction.unwrap().executable != CPU::data_processing_instruction);
    }

    #[test]
    fn it_does_decode_an_instruction_if_ne_satisfied() {
        let memory = Memory::new().unwrap();
        let memory = Arc::new(Mutex::new(memory));
        let mut cpu = CPU::new(memory);
        let instruction: ARMByteCode = 0x128210c8; // addne r1, r2, 200
        cpu.reset_flag(FlagsRegister::Z);
        cpu.decode_instruction(instruction);
        assert!(cpu.decoded_instruction.unwrap().executable != CPU::arm_nop);
        assert!(cpu.decoded_instruction.unwrap().executable == CPU::data_processing_instruction);
    }

    #[test]
    fn it_does_not_decode_an_instruction_if_ne_not_satisfied() {
        let memory = Memory::new().unwrap();
        let memory = Arc::new(Mutex::new(memory));
        let mut cpu = CPU::new(memory);
        let instruction: ARMByteCode = 0x128210c8; // addne r1, r2, 200
        cpu.set_flag(FlagsRegister::Z);
        cpu.decode_instruction(instruction);
        assert!(cpu.decoded_instruction.unwrap().executable == CPU::arm_nop);
        assert!(cpu.decoded_instruction.unwrap().executable != CPU::data_processing_instruction);
    }

    #[test]
    fn it_does_decode_an_instruction_if_cs_satisfied() {
        let memory = Memory::new().unwrap();
        let memory = Arc::new(Mutex::new(memory));
        let mut cpu = CPU::new(memory);
        let instruction: ARMByteCode = 0x228210c8; // addcs r1, r2, 200
        cpu.set_flag(FlagsRegister::C);
        cpu.decode_instruction(instruction);
        assert!(cpu.decoded_instruction.unwrap().executable != CPU::arm_nop);
        assert!(cpu.decoded_instruction.unwrap().executable == CPU::data_processing_instruction);
    }
    #[test]
    fn it_does_decode_an_instruction_if_cc_satisfied() {
        let memory = Memory::new().unwrap();
        let memory = Arc::new(Mutex::new(memory));
        let mut cpu = CPU::new(memory);
        let instruction: ARMByteCode = 0x328210c8; // addcc r1, r2, 200
        cpu.reset_flag(FlagsRegister::C);
        cpu.decode_instruction(instruction);
        assert!(cpu.decoded_instruction.unwrap().executable != CPU::arm_nop);
        assert!(cpu.decoded_instruction.unwrap().executable == CPU::data_processing_instruction);
    }
    #[test]
    fn it_does_decode_an_instruction_if_mi_satisfied() {
        let memory = Memory::new().unwrap();
        let memory = Arc::new(Mutex::new(memory));
        let mut cpu = CPU::new(memory);
        let instruction: ARMByteCode = 0x428210c8; // addmi r1, r2, 200
        cpu.set_flag(FlagsRegister::N);
        cpu.decode_instruction(instruction);
        assert!(cpu.decoded_instruction.unwrap().executable != CPU::arm_nop);
        assert!(cpu.decoded_instruction.unwrap().executable == CPU::data_processing_instruction);
    }
    #[test]
    fn it_does_decode_an_instruction_if_pl_satisfied() {
        let memory = Memory::new().unwrap();
        let memory = Arc::new(Mutex::new(memory));
        let mut cpu = CPU::new(memory);
        let instruction: ARMByteCode = 0x528210c8; // addpl r1, r2, 200
        cpu.reset_flag(FlagsRegister::C);
        cpu.decode_instruction(instruction);
        assert!(cpu.decoded_instruction.unwrap().executable != CPU::arm_nop);
        assert!(cpu.decoded_instruction.unwrap().executable == CPU::data_processing_instruction);
    }
    #[test]
    fn it_does_decode_an_instruction_if_vs_satisfied() {
        let memory = Memory::new().unwrap();
        let memory = Arc::new(Mutex::new(memory));
        let mut cpu = CPU::new(memory);
        let instruction: ARMByteCode = 0x628210c8; // addvs r1, r2, 200
        cpu.set_flag(FlagsRegister::V);
        cpu.decode_instruction(instruction);
        assert!(cpu.decoded_instruction.unwrap().executable != CPU::arm_nop);
        assert!(cpu.decoded_instruction.unwrap().executable == CPU::data_processing_instruction);
    }
    #[test]
    fn it_does_decode_an_instruction_if_vc_satisfied() {
        let memory = Memory::new().unwrap();
        let memory = Arc::new(Mutex::new(memory));
        let mut cpu = CPU::new(memory);
        let instruction: ARMByteCode = 0x728210c8; // addvc r1, r2, 200
        cpu.reset_flag(FlagsRegister::V);
        cpu.decode_instruction(instruction);
        assert!(cpu.decoded_instruction.unwrap().executable != CPU::arm_nop);
        assert!(cpu.decoded_instruction.unwrap().executable == CPU::data_processing_instruction);
    }
    #[test]
    fn it_does_decode_an_instruction_if_hi_satisfied() {
        let memory = Memory::new().unwrap();
        let memory = Arc::new(Mutex::new(memory));
        let mut cpu = CPU::new(memory);
        let instruction: ARMByteCode = 0x828210c8; // addhi r1, r2, 200
        cpu.set_flag(FlagsRegister::C);
        cpu.reset_flag(FlagsRegister::Z);
        cpu.decode_instruction(instruction);
        assert!(cpu.decoded_instruction.unwrap().executable != CPU::arm_nop);
        assert!(cpu.decoded_instruction.unwrap().executable == CPU::data_processing_instruction);
    }
    #[test]
    fn it_does_decode_an_instruction_if_ls_satisfied() {
        let memory = Memory::new().unwrap();
        let memory = Arc::new(Mutex::new(memory));
        let mut cpu = CPU::new(memory);
        let instruction: ARMByteCode = 0x928210c8; // addls r1, r2, 200
        cpu.reset_flag(FlagsRegister::C);
        cpu.reset_flag(FlagsRegister::Z);
        cpu.decode_instruction(instruction);
        assert!(cpu.decoded_instruction.unwrap().executable != CPU::arm_nop);
        assert!(cpu.decoded_instruction.unwrap().executable == CPU::data_processing_instruction);

        cpu.set_flag(FlagsRegister::C);
        cpu.set_flag(FlagsRegister::Z);
        cpu.decode_instruction(instruction);
        assert!(cpu.decoded_instruction.unwrap().executable != CPU::arm_nop);
        assert!(cpu.decoded_instruction.unwrap().executable == CPU::data_processing_instruction);

        cpu.reset_flag(FlagsRegister::C);
        cpu.set_flag(FlagsRegister::Z);
        cpu.decode_instruction(instruction);
        assert!(cpu.decoded_instruction.unwrap().executable != CPU::arm_nop);
        assert!(cpu.decoded_instruction.unwrap().executable == CPU::data_processing_instruction);
    }

    #[test]
    fn it_does_decode_an_instruction_if_ge_satisfied() {
        let memory = Memory::new().unwrap();
        let memory = Arc::new(Mutex::new(memory));
        let mut cpu = CPU::new(memory);
        let instruction: ARMByteCode = 0xa28210c8; // addge r1, r2, 200
        cpu.set_flag(FlagsRegister::N);
        cpu.set_flag(FlagsRegister::V);
        cpu.decode_instruction(instruction);
        assert!(cpu.decoded_instruction.unwrap().executable != CPU::arm_nop);
        assert!(cpu.decoded_instruction.unwrap().executable == CPU::data_processing_instruction);

        cpu.reset_flag(FlagsRegister::N);
        cpu.reset_flag(FlagsRegister::V);
        cpu.decode_instruction(instruction);
        assert!(cpu.decoded_instruction.unwrap().executable != CPU::arm_nop);
        assert!(cpu.decoded_instruction.unwrap().executable == CPU::data_processing_instruction);
    }
    #[test]
    fn it_does_decode_an_instruction_if_lt_satisfied() {
        let memory = Memory::new().unwrap();
        let memory = Arc::new(Mutex::new(memory));
        let mut cpu = CPU::new(memory);
        let instruction: ARMByteCode = 0xb28210c8; // addlt r1, r2, 200
        cpu.reset_flag(FlagsRegister::N);
        cpu.set_flag(FlagsRegister::V);
        cpu.decode_instruction(instruction);
        assert!(cpu.decoded_instruction.unwrap().executable != CPU::arm_nop);
        assert!(cpu.decoded_instruction.unwrap().executable == CPU::data_processing_instruction);

        cpu.set_flag(FlagsRegister::N);
        cpu.reset_flag(FlagsRegister::V);
        cpu.decode_instruction(instruction);
        assert!(cpu.decoded_instruction.unwrap().executable != CPU::arm_nop);
        assert!(cpu.decoded_instruction.unwrap().executable == CPU::data_processing_instruction);
    }

    #[test]
    fn it_does_decode_an_instruction_if_gt_satisfied() {
        let memory = Memory::new().unwrap();
        let memory = Arc::new(Mutex::new(memory));
        let mut cpu = CPU::new(memory);
        let instruction: ARMByteCode = 0xc28210c8; // addgt r1, r2, 200
        cpu.reset_flag(FlagsRegister::Z);
        cpu.set_flag(FlagsRegister::N);
        cpu.set_flag(FlagsRegister::V);
        cpu.decode_instruction(instruction);
        assert!(cpu.decoded_instruction.unwrap().executable != CPU::arm_nop);
        assert!(cpu.decoded_instruction.unwrap().executable == CPU::data_processing_instruction);

        cpu.reset_flag(FlagsRegister::Z);
        cpu.reset_flag(FlagsRegister::N);
        cpu.reset_flag(FlagsRegister::V);
        cpu.decode_instruction(instruction);
        assert!(cpu.decoded_instruction.unwrap().executable != CPU::arm_nop);
        assert!(cpu.decoded_instruction.unwrap().executable == CPU::data_processing_instruction);
    }

    #[test]
    fn it_does_decode_an_instruction_if_le_satisfied() {
        let memory = Memory::new().unwrap();
        let memory = Arc::new(Mutex::new(memory));
        let mut cpu = CPU::new(memory);
        let instruction: ARMByteCode = 0xd28210c8; // addle r1, r2, 200

        cpu.set_flag(FlagsRegister::Z);
        cpu.reset_flag(FlagsRegister::N);
        cpu.reset_flag(FlagsRegister::V);
        cpu.decode_instruction(instruction);
        assert!(cpu.decoded_instruction.unwrap().executable != CPU::arm_nop);
        assert!(cpu.decoded_instruction.unwrap().executable == CPU::data_processing_instruction);

        cpu.reset_flag(FlagsRegister::Z);
        cpu.set_flag(FlagsRegister::N);
        cpu.reset_flag(FlagsRegister::V);
        cpu.decode_instruction(instruction);
        assert!(cpu.decoded_instruction.unwrap().executable != CPU::arm_nop);
        assert!(cpu.decoded_instruction.unwrap().executable == CPU::data_processing_instruction);

        cpu.reset_flag(FlagsRegister::Z);
        cpu.reset_flag(FlagsRegister::N);
        cpu.set_flag(FlagsRegister::V);
        cpu.decode_instruction(instruction);
        assert!(cpu.decoded_instruction.unwrap().executable != CPU::arm_nop);
        assert!(cpu.decoded_instruction.unwrap().executable == CPU::data_processing_instruction);
    }
    #[test]
    fn it_does_decode_an_instruction_if_al_satisfied() {
        let memory = Memory::new().unwrap();
        let memory = Arc::new(Mutex::new(memory));
        let mut cpu = CPU::new(memory);
        let instruction: ARMByteCode = 0xe28210c8; // addal r1, r2, 200
        cpu.decode_instruction(instruction);
        assert!(cpu.decoded_instruction.unwrap().executable != CPU::arm_nop);
        assert!(cpu.decoded_instruction.unwrap().executable == CPU::data_processing_instruction);
    }

    //    #[test]
    //    fn it_returns_a_multiply_instruction() {
    //        let memory = Memory::new().unwrap();
    //        let memory = Arc::new(Mutex::new(memory));
    //        let mut cpu = CPU::new(memory);
    //
    //        let instruction: ARMByteCode = 0xE0230192;
    //        cpu.decode_instruction(instruction);
    //        assert!(cpu.decoded_instruction.executable == CPU::arm_multiply_accumulate);
    //        let instruction: ARMByteCode = 0xE0050091;
    //        cpu.decode_instruction(instruction);
    //        assert!(cpu.decoded_instruction.executable == CPU::arm_multiply);
    //    }
    //
    //    #[test]
    //    fn it_returns_a_branch_instruction() {
    //        let memory = Memory::new().unwrap();
    //        let memory = Arc::new(Mutex::new(memory));
    //        let mut cpu = CPU::new(memory);
    //
    //        let instruction: ARMByteCode = 0xea000005;
    //        cpu.decode_instruction(instruction);
    //        assert!(cpu.decoded_instruction.executable == CPU::arm_branch);
    //    }

    //    #[test]
    //    fn it_returns_a_cmp_instruction() {
    //        let memory = Memory::new().unwrap();
    //        let memory = Arc::new(Mutex::new(memory));
    //        let mut cpu = CPU::new(memory);
    //
    //        let instruction: ARMByteCode = 0xe1530312; //  cmp r3, r2, lsl r3
    //                                                   //  shift by register in
    //                                                   // order to stall the alu by
    //                                                   // one clock cycle
    //        cpu.fetched_instruction = instruction;
    //        cpu.execute_cpu_cycle();
    //        cpu.execute_cpu_cycle();
    //        assert!(cpu.alu_executable.operation == CPU::arm_cmp);
    //    }

    //    #[test]
    //    fn it_returns_an_add_instruction_with_an_imm_op2() {
    //        let memory = Memory::new().unwrap();
    //        let memory = Arc::new(Mutex::new(memory));
    //        let mut cpu = CPU::new(memory);
    //
    //        let instruction: ARMByteCode = 0xe2812020; // add r
    //        cpu.decode_instruction(instruction);
    //        assert!(cpu.decoded_instruction.executable == CPU::arm_add);
    //        assert!(cpu.decoded_instruction.operand2 == 32);
    //        assert!(cpu.decoded_instruction.rd == 0x2);
    //        assert!(cpu.decoded_instruction.rn == 0x1);
    //    }
    //
    //    #[test]
    //    fn it_returns_an_add_instruction_an_lsl_operand2() {
    //        let memory = Memory::new().unwrap();
    //        let memory = Arc::new(Mutex::new(memory));
    //        let mut cpu = CPU::new(memory);
    //
    //        let instruction: ARMByteCode = 0xe0831102; // add r1, r3, r2 LSL 2
    //        cpu.set_register(2, 1);
    //        cpu.decode_instruction(instruction);
    //        assert!(cpu.decoded_instruction.executable == CPU::arm_add);
    //        assert!(cpu.decoded_instruction.rd == 0x1);
    //        assert!(cpu.decoded_instruction.rn == 0x3);
    //        assert!(cpu.decoded_instruction.operand2 == (1 << 2));
    //    }
    //
    //    #[test]
    //    fn it_returns_an_add_instruction_with_ror_10() {
    //        let memory = Memory::new().unwrap();
    //        let memory = Arc::new(Mutex::new(memory));
    //        let mut cpu = CPU::new(memory);
    //
    //        let instruction: ARMByteCode = 0xe0831562; // add r1, r3, r2 ROR#10
    //        cpu.set_register(2, 5);
    //        cpu.decode_instruction(instruction);
    //        assert!(cpu.decoded_instruction.executable == CPU::arm_add);
    //        assert!(cpu.decoded_instruction.rd == 0x1);
    //        assert!(cpu.decoded_instruction.rn == 0x3);
    //        assert!(cpu.decoded_instruction.operand2 == (5 as u32).rotate_right(10));
    //    }
    //
    //    #[test]
    //    fn it_returns_an_add_instruction_with_asr_10() {
    //        let memory = Memory::new().unwrap();
    //        let memory = Arc::new(Mutex::new(memory));
    //        let mut cpu = CPU::new(memory);
    //
    //        let instruction: ARMByteCode = 0xe0831542; // add r1, r3, r2 ASR#10
    //        cpu.set_register(2, 0xB000_0000);
    //        cpu.decode_instruction(instruction);
    //        assert!(cpu.decoded_instruction.executable == CPU::arm_add);
    //        assert!(cpu.decoded_instruction.rd == 0x1);
    //        assert!(cpu.decoded_instruction.rn == 0x3);
    //        assert!(cpu.decoded_instruction.operand2 == 0xFFEC0000);
    //    }
    //
    //    #[test]
    //    fn it_returns_an_add_instruction_with_lsr_10() {
    //        let memory = Memory::new().unwrap();
    //        let memory = Arc::new(Mutex::new(memory));
    //        let mut cpu = CPU::new(memory);
    //
    //        let instruction: ARMByteCode = 0xe0831522; // add r1, r3, r2 LSR#10
    //        cpu.set_register(2, 0xB000_0000);
    //        cpu.decode_instruction(instruction);
    //        assert!(cpu.decoded_instruction.executable == CPU::arm_add);
    //        assert!(cpu.decoded_instruction.rd == 0x1);
    //        assert!(cpu.decoded_instruction.rn == 0x3);
    //        assert!(cpu.decoded_instruction.operand2 == 0xB000_0000 >> 10);
    //    }
    //
    //    #[test]
    //    fn it_returns_an_add_instruction_with_lsr_32() {
    //        let memory = Memory::new().unwrap();
    //        let memory = Arc::new(Mutex::new(memory));
    //        let mut cpu = CPU::new(memory);
    //
    //        let instruction: ARMByteCode = 0xe0931022; // adds r1, r3, r2 LSR#32
    //        cpu.set_register(2, u32::MAX);
    //        cpu.decode_instruction(instruction);
    //        assert!(cpu.decoded_instruction.executable == CPU::arm_add);
    //        assert!(cpu.decoded_instruction.rd == 0x1);
    //        assert!(cpu.decoded_instruction.rn == 0x3);
    //        assert!(cpu.decoded_instruction.operand2 == 0);
    //        assert!(cpu.get_flag(FlagsRegister::C) == 1);
    //    }
    //
    //    #[test]
    //    fn it_returns_an_add_instruction_with_an_asr_32_negative() {
    //        let memory = Memory::new().unwrap();
    //        let memory = Arc::new(Mutex::new(memory));
    //        let mut cpu = CPU::new(memory);
    //
    //        let instruction: ARMByteCode = 0xe0931042; // adds r1, r3, r2 ASR#32
    //        cpu.set_register(2, 0xF000_1000);
    //        cpu.decode_instruction(instruction);
    //        assert!(cpu.decoded_instruction.executable == CPU::arm_add);
    //        assert!(cpu.decoded_instruction.rd == 0x1);
    //        assert!(cpu.decoded_instruction.rn == 0x3);
    //        assert!(cpu.decoded_instruction.operand2 == u32::MAX);
    //        assert!(cpu.get_flag(FlagsRegister::C) == 1);
    //    }
    //
    //    #[test]
    //    fn it_returns_an_add_instruction_with_an_asr_32_positive() {
    //        let memory = Memory::new().unwrap();
    //        let memory = Arc::new(Mutex::new(memory));
    //        let mut cpu = CPU::new(memory);
    //
    //        let instruction: ARMByteCode = 0xe0831042; // add r1, r3, r2 ASR#32
    //        cpu.set_register(2, 0x0000_1000);
    //        cpu.decode_instruction(instruction);
    //        assert!(cpu.decoded_instruction.executable == CPU::arm_add);
    //        assert!(cpu.decoded_instruction.rd == 0x1);
    //        assert!(cpu.decoded_instruction.rn == 0x3);
    //        assert!(cpu.decoded_instruction.operand2 == 0);
    //        assert!(cpu.get_flag(FlagsRegister::C) == 0);
    //    }
    //
    //    #[test]
    //    fn it_returns_an_add_instruction_with_op2_shifted_by_register() {
    //        let memory = Memory::new().unwrap();
    //        let memory = Arc::new(Mutex::new(memory));
    //        let mut cpu = CPU::new(memory);
    //
    //        let instruction: ARMByteCode = 0xe0831412; // add r1, r3, r2 LSL r4
    //        cpu.set_register(2, 0x0000_1000);
    //        cpu.set_register(4, 5);
    //        cpu.decode_instruction(instruction);
    //        assert!(cpu.decoded_instruction.executable == CPU::arm_add);
    //        assert!(cpu.decoded_instruction.rd == 0x1);
    //        assert!(cpu.decoded_instruction.rn == 0x3);
    //        assert!(cpu.decoded_instruction.operand2 == 0x0002_0000);
    //    }
}
