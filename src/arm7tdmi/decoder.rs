use super::{
    cpu::{InstructionMode, CPU},
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
            InstructionMode::ARM => self.decode_arm_instruction(instruction),
            InstructionMode::THUMB => self.decode_thumb_instruction(instruction),
        };
    }

    fn condition_passed(instruction: ARMByteCode, _condition_flags: BYTE) -> bool {
        let condition = (instruction & 0xF0000000) >> 28;
        match condition {
            0b1110 => true,
            _ => true,
        }
    }

    fn decode_arm_instruction(&mut self, instruction: ARMByteCode) -> ARMDecodedInstruction {
        if !(Self::condition_passed(instruction, 0x00)) {
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
            _ if arm_decoders::is_multiply_long_instruction(instruction) => ARMDecodedInstruction {
                executable: CPU::arm_multiply_long,
                instruction,
                ..Default::default()
            },
            _ if arm_decoders::is_data_processing_and_psr_transfer(instruction) => {
                ARMDecodedInstruction {
                    executable: CPU::decode_data_processing_instruction,
                    instruction,
                }
            }
            _ if arm_decoders::is_branch_instruction(instruction) => ARMDecodedInstruction {
                executable: CPU::arm_branch,
                instruction,
                ..Default::default()
            },
            _ if arm_decoders::is_load_or_store_register_unsigned(instruction) => {
                self.decode_load_or_store_register_unsigned(instruction)
            }
            _ => ARMDecodedInstruction {
                executable: CPU::arm_not_implemented,
                instruction,
                ..Default::default()
            },
        }
    }

    fn decode_thumb_instruction(&self, instruction: ARMByteCode) -> ARMDecodedInstruction {
        ARMDecodedInstruction {
            instruction,
            executable: CPU::arm_not_implemented,
            ..Default::default()
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
    use crate::{
        arm7tdmi::{
            alu::{ALUInstruction},
            cpu::{FlagsRegister, CPU, PC_REGISTER},
            instructions::{ALUExecutable, ARMDecodedInstruction},
        },
        utils::bits::Bits,
    };

    use super::ARMByteCode;

    impl CPU {
        pub fn decode_data_processing_instruction(&mut self, instruction: ARMByteCode) {
            let opcode = (instruction & 0x01E0_0000) >> 21;
            let executable: ALUExecutable = match opcode {
                0x0 => CPU::arm_and,
                0x1 => CPU::arm_eor,
                0x2 => CPU::arm_sub,
                0x3 => CPU::arm_rsb,
                0x4 => CPU::arm_add,
                0x5 => CPU::arm_adc,
                0x6 => CPU::arm_sbc,
                0x7 => CPU::arm_rsc,
                0x8 => CPU::arm_tst,
                0x9 => CPU::arm_teq,
                0xa => CPU::arm_cmp,
                0xb => CPU::arm_cmn,
                0xc => CPU::arm_orr,
                0xd => CPU::arm_mov,
                0xe => CPU::arm_bic,
                0xf => CPU::arm_mvn,
                _ => panic!("Impossible to decode opcode"),
            };

            let rn = (0x000F_0000 & instruction) >> 16;
            let rd = (0x0000_F000 & instruction) >> 12;
            let set_flags = instruction.bit_is_set(20) && rd != PC_REGISTER as u32;

            let operand1 = self.get_register(rn);
            let operand2 = self.decode_operand2(instruction, set_flags);

            let alu_instruction = ALUInstruction {
                executable,
                rd,
                operand1,
                operand2,
                set_flags,
            };

            self.alu_executable = alu_instruction;

            if !instruction.bit_is_set(4) {
                (self.alu_executable.executable)(self);
            } else {
                self.alu_stalled = true;
            }
        }

        fn decode_operand2(&mut self, instruction: ARMByteCode, set_flags: bool) -> u32 {
            let shift_amount;
            if instruction.bit_is_set(25) {
                // operand 2 is immediate
                shift_amount = ((instruction & 0x0000_0F00) >> 8) * 2;
                let immediate = instruction & 0x0000_00FF;

                return immediate.rotate_right(shift_amount);
            }
            let shift_type = (instruction & 0x0000_0060) >> 5;
            let operand_register = instruction & 0x0000_000F;
            let mut operand_register_value = self.get_register(operand_register);

            if instruction.bit_is_set(4) {
                // shift by register
                let shift_register = (instruction & 0x0000_0F00) >> 8;
                shift_amount = self.get_register(shift_register);
                if operand_register == 15 {
                    operand_register_value += 4;
                }
            } else {
                shift_amount = (instruction & 0x0000_0F80) >> 7;
                if shift_amount == 0 {
                    // special case for shifting
                    return match shift_type {
                        // no change
                        0x00 => operand_register_value,
                        // LSR#32
                        0x01 => {
                            if set_flags {
                                self.set_flag_from_bit(
                                    FlagsRegister::C,
                                    operand_register_value.get_bit(31) as u8,
                                );
                            }
                            0
                        }
                        // ASR#32
                        0x02 => {
                            if operand_register_value.bit_is_set(31) {
                                if set_flags {
                                    self.set_flag(FlagsRegister::C);
                                }
                                return u32::MAX;
                            }
                            if set_flags {
                                self.reset_flag(FlagsRegister::C);
                            }
                            0
                        }
                        // RRX#1
                        0x03 => operand_register_value >> 1 | self.get_flag(FlagsRegister::C) << 31,
                        _ => panic!("Invalid Shift Type"),
                    };
                }
            }

            match shift_type {
                // Logical shift left
                0x00 => {
                    if operand_register_value.bit_is_set((32 - shift_amount) as u8) {
                        self.set_flag(FlagsRegister::C);
                    } else {
                        self.reset_flag(FlagsRegister::C);
                    }
                    operand_register_value << shift_amount
                }
                // Logical shift right
                0x01 => {
                    if operand_register_value.bit_is_set((shift_amount - 1) as u8) {
                        self.set_flag(FlagsRegister::C);
                    } else {
                        self.reset_flag(FlagsRegister::C);
                    }
                    operand_register_value >> shift_amount
                }
                // Arithmetic shift right
                0x02 => {
                    if operand_register_value.bit_is_set((shift_amount - 1) as u8) {
                        self.set_flag(FlagsRegister::C);
                    } else {
                        self.reset_flag(FlagsRegister::C);
                    }
                    (operand_register_value as i32 >> shift_amount) as u32
                }
                // Rotate Right
                0x03 => {
                    if set_flags {
                        if operand_register_value.bit_is_set((shift_amount - 1) as u8) {
                            self.set_flag(FlagsRegister::C);
                        } else {
                            self.reset_flag(FlagsRegister::C);
                        }
                    }
                    operand_register_value.rotate_right(shift_amount)
                }
                _ => panic!("Invalid Shift Type"),
            }
        }

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

        pub fn decode_branch_instruction(&self, instruction: ARMByteCode) -> ARMDecodedInstruction {
            ARMDecodedInstruction {
                executable: CPU::arm_branch,
                instruction,
                ..Default::default()
            }
        }

        pub fn decode_load_or_store_register_unsigned(
            &self,
            instruction: ARMByteCode,
        ) -> ARMDecodedInstruction {
            return ARMDecodedInstruction {
                executable: CPU::arm_not_implemented,
                instruction,
                ..Default::default()
            };
        }
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
    use std::sync::{Arc, Mutex};

    use crate::{
        arm7tdmi::{decoder::*},
        memory::Memory,
    };

    #[test]
    fn it_returns_a_multiply_instruction() {
        let memory = Memory::new().unwrap();
        let memory = Arc::new(Mutex::new(memory));
        let mut cpu = CPU::new(memory);

        let instruction: ARMByteCode = 0xE0230192;
        cpu.decode_instruction(instruction);
        assert!(cpu.decoded_instruction.executable == CPU::arm_multiply_accumulate);
        let instruction: ARMByteCode = 0xE0050091;
        cpu.decode_instruction(instruction);
        assert!(cpu.decoded_instruction.executable == CPU::arm_multiply);
    }

    #[test]
    fn it_returns_a_branch_instruction() {
        let memory = Memory::new().unwrap();
        let memory = Arc::new(Mutex::new(memory));
        let mut cpu = CPU::new(memory);

        let instruction: ARMByteCode = 0xea000005;
        cpu.decode_instruction(instruction);
        assert!(cpu.decoded_instruction.executable == CPU::arm_branch);
    }

    #[test]
    fn it_returns_a_cmp_instruction() {
        let memory = Memory::new().unwrap();
        let memory = Arc::new(Mutex::new(memory));
        let mut cpu = CPU::new(memory);

        let instruction: ARMByteCode = 0xe35e0000;
        cpu.decode_instruction(instruction);
        cpu.execute_cpu_cycle();
        assert!(cpu.alu_executable.executable == CPU::arm_cmp);
    }

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
