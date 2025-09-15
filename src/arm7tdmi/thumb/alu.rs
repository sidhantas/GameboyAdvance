use crate::{
    arm7tdmi::{
        arm::alu::ArithmeticInstruction,
        cpu::{FlagsRegister, InstructionMode, CPU, PC_REGISTER},
        instruction_table::{Execute, Operand},
        thumb,
    },
    memory::memory::GBAMemory,
    types::{CYCLES, REGISTER},
    utils::bits::{sign_extend, Bits},
};

pub struct ThumbFullAdder(
    pub ThumbFullAdderOperations,
    pub REGISTER,
    pub REGISTER,
    pub Operand,
);

pub enum ThumbFullAdderOperations {
    Add,
    Sub,
}

impl CPU {
    pub fn decode_full_adder(instruction: u32) -> ThumbFullAdder {
        let opcode = (instruction & 0x0600) >> 9;
        let operand2 = (instruction & 0x01C0) >> 6;

        let rd = instruction & 0x0007;
        let rs = (instruction & 0x0038) >> 3;

        let (full_adder_operation, operand2) = match opcode {
            0 => (ThumbFullAdderOperations::Add, Operand::Register(operand2)),
            1 => (ThumbFullAdderOperations::Sub, Operand::Register(operand2)),
            2 => (ThumbFullAdderOperations::Add, Operand::Immediate(operand2)),
            3 => (ThumbFullAdderOperations::Sub, Operand::Immediate(operand2)),
            _ => unreachable!(),
        };

        ThumbFullAdder(full_adder_operation, rd, rs, operand2)
    }
}

impl Execute for ThumbFullAdder {
    fn execute(self, cpu: &mut CPU, _memory: &mut GBAMemory) -> CYCLES {
        let ThumbFullAdder(operation, rd, rs, op2) = self;

        let rs_val = cpu.get_register(rs);
        let op2 = match op2 {
            Operand::Register(reg) => cpu.get_register(reg),
            Operand::Immediate(imm) => imm,
        };

        let result = match operation {
            ThumbFullAdderOperations::Add => {
                let result = rs_val + op2;
                cpu.set_arithmetic_flags(result, rs_val, op2, 0, true);
                result
            }
            ThumbFullAdderOperations::Sub => {
                let result = rs_val - op2;
                cpu.set_arithmetic_flags(result, rs_val, !op2, 1, true);
                result
            }
        };

        cpu.set_register(rd, result);

        0
    }
}

pub enum ThumbALUInstruction {
    Arithmetic(ThumbArithmeticInstruction, REGISTER, REGISTER),
    Logical(ThumbLogicalInstruction, REGISTER, REGISTER),
    Shift(ThumbShiftInstruction, REGISTER, REGISTER),
}

pub enum ThumbArithmeticInstruction {
    Adc,
    Sbc,
    Neg,
    Cmp,
    Cmn,
    Mul,
}

pub enum ThumbLogicalInstruction {
    And,
    Eor,
    Tst,
    Orr,
    Bic,
    Mvn,
}

pub enum ThumbShiftInstruction {
    Lsl,
    Lsr,
    Asr,
    Ror,
}

impl CPU {
    pub fn decode_thumb_alu_instruction(instruction: u32) -> ThumbALUInstruction {
        let opcode = (instruction & 0x03C0) >> 6;

        let rd = instruction & 0x0007;
        let rs = (instruction & 0x0038) >> 3;

        match opcode {
            0x0 => ThumbALUInstruction::Logical(ThumbLogicalInstruction::And, rd, rs),
            0x1 => ThumbALUInstruction::Logical(ThumbLogicalInstruction::Eor, rd, rs),
            0x2 => ThumbALUInstruction::Shift(ThumbShiftInstruction::Lsl, rd, rs),
            0x3 => ThumbALUInstruction::Shift(ThumbShiftInstruction::Lsr, rd, rs),
            0x4 => ThumbALUInstruction::Shift(ThumbShiftInstruction::Asr, rd, rs),
            0x5 => ThumbALUInstruction::Arithmetic(ThumbArithmeticInstruction::Adc, rd, rs),
            0x6 => ThumbALUInstruction::Arithmetic(ThumbArithmeticInstruction::Sbc, rd, rs),
            0x7 => ThumbALUInstruction::Shift(ThumbShiftInstruction::Ror, rd, rs),
            0x8 => ThumbALUInstruction::Logical(ThumbLogicalInstruction::Tst, rd, rs),
            0x9 => ThumbALUInstruction::Arithmetic(ThumbArithmeticInstruction::Neg, rd, rs),
            0xa => ThumbALUInstruction::Arithmetic(ThumbArithmeticInstruction::Cmp, rd, rs),
            0xb => ThumbALUInstruction::Arithmetic(ThumbArithmeticInstruction::Cmn, rd, rs),
            0xc => ThumbALUInstruction::Logical(ThumbLogicalInstruction::Orr, rd, rs),
            0xd => ThumbALUInstruction::Arithmetic(ThumbArithmeticInstruction::Mul, rd, rs),
            0xe => ThumbALUInstruction::Logical(ThumbLogicalInstruction::Bic, rd, rs),
            0xf => ThumbALUInstruction::Logical(ThumbLogicalInstruction::Mvn, rd, rs),
            _ => unreachable!(),
        }
    }
}

impl Execute for ThumbALUInstruction {
    fn execute(self, cpu: &mut CPU, memory: &mut GBAMemory) -> CYCLES {
        match self {
            ThumbALUInstruction::Arithmetic(thumb_arithmetic_instruction, rd, rs) => {
                cpu.execute_thumb_arithmetic_instruction(thumb_arithmetic_instruction, rd, rs)
            }
            ThumbALUInstruction::Logical(thumb_logical_instruction, rd, rs) => {
                cpu.execute_thumb_logical_instruction(thumb_logical_instruction, rd, rs)
            }
            ThumbALUInstruction::Shift(thumb_shift_instruction, rd, rs) => {
                cpu.execute_thumb_shift_instruction(thumb_shift_instruction, rd, rs)
            }
        }
    }
}

impl CPU {
    fn execute_thumb_arithmetic_instruction(
        &mut self,
        instruction: ThumbArithmeticInstruction,
        rd: REGISTER,
        rs: REGISTER,
    ) -> CYCLES {
        let rs_val = self.get_register(rs);
        let rd_val = self.get_register(rd);
        let mut cycles = 0;
        match instruction {
            ThumbArithmeticInstruction::Adc => {
                let carry = self.get_flag(FlagsRegister::C);
                let result = rd_val + rs_val + carry;
                self.set_arithmetic_flags(result, rd_val, rs_val, carry, true);
                self.set_register(rd, result);
            }
            ThumbArithmeticInstruction::Sbc => {
                let carry = self.get_flag(FlagsRegister::C);
                let result = rd_val - rs_val + carry - 1;
                self.set_arithmetic_flags(result, rd_val, !rs_val, carry, true);
                self.set_register(rd, result);
            }
            ThumbArithmeticInstruction::Neg => {
                let result = 0 - rs_val;
                self.set_arithmetic_flags(result, 0, !rs_val, 1, true);
                self.set_register(rd, result);
            }
            ThumbArithmeticInstruction::Cmp => {
                let result = rd_val - rs_val;
                self.set_arithmetic_flags(result, rd_val, !rs_val, 1, true);
            }
            ThumbArithmeticInstruction::Cmn => {
                let result = rd_val + rs_val;
                self.set_arithmetic_flags(result, rd_val, rs_val, 0, true);
            }
            ThumbArithmeticInstruction::Mul => {
                let multiplier = self.get_register(rd);
                cycles += if multiplier & 0xFFFF_FF00 == 0
                    || multiplier & 0xFFFF_FF00 == 0xFFFF_FF00
                {
                    1
                } else if multiplier & 0xFFFF_0000 == 0 || multiplier & 0xFFFF_0000 == 0xFFFF_0000 {
                    2
                } else if multiplier & 0xFF00_0000 == 0 || multiplier & 0xFF00_0000 == 0xFFFF_0000 {
                    3
                } else {
                    4
                };
                self.thumb_mul(rd, rd_val, rs_val, true);
            }
        };

        cycles
    }

    fn execute_thumb_logical_instruction(
        &mut self,
        instruction: ThumbLogicalInstruction,
        rd: REGISTER,
        rs: REGISTER,
    ) -> CYCLES {
        let rs_val = self.get_register(rs);
        let rd_val = self.get_register(rd);

        let result = match instruction {
            ThumbLogicalInstruction::And => {
                let result = rd_val & rs_val;
                self.set_register(rd, result);
                result
            }
            ThumbLogicalInstruction::Eor => {
                let result = rd_val ^ rs_val;
                self.set_register(rd, result);
                result
            }
            ThumbLogicalInstruction::Tst => rd_val & rs_val,
            ThumbLogicalInstruction::Orr => {
                let result = rd_val | rs_val;
                self.set_register(rd, result);
                result
            }
            ThumbLogicalInstruction::Bic => {
                let result = rd_val & !rs_val;
                self.set_register(rd, result);
                result
            }
            ThumbLogicalInstruction::Mvn => {
                let result = !rs_val;
                self.set_register(rd, result);
                result
            }
        };

        self.set_logical_flags(result, true);

        0
    }

    fn execute_thumb_shift_instruction(
        &mut self,
        instruction: ThumbShiftInstruction,
        rd: REGISTER,
        rs: REGISTER,
    ) -> CYCLES {
        let mut shift_amount = self.get_register(rs) & 0xFF;
        let mut rd_val = self.get_register(rd);

        if rs == PC_REGISTER as u32 {
            shift_amount += 2;
        }

        if rd == PC_REGISTER as u32 {
            rd_val += 2;
        }

        let result = match instruction {
            ThumbShiftInstruction::Lsl => {
                if shift_amount == 0 {
                    self.shifter_output = self.get_flag(FlagsRegister::C);
                    rd_val
                } else if shift_amount < 32 {
                    self.shifter_output = rd_val.get_bit(32 - shift_amount as u8);
                    rd_val << shift_amount
                } else if shift_amount == 32 {
                    self.shifter_output = rd_val & 0x1;
                    0
                } else {
                    self.shifter_output = 0;
                    0
                }
            }
            ThumbShiftInstruction::Lsr => {
                if shift_amount == 0 {
                    self.shifter_output = self.get_flag(FlagsRegister::C);
                    rd_val
                } else if shift_amount < 32 {
                    self.shifter_output = rd_val.get_bit((shift_amount as u8) - 1);
                    rd_val >> shift_amount
                } else if shift_amount == 32 {
                    self.shifter_output = rd_val >> 31;
                    0
                } else {
                    self.shifter_output = 0;
                    0
                }
            }
            ThumbShiftInstruction::Asr => {
                if shift_amount == 0 {
                    self.shifter_output = self.get_flag(FlagsRegister::C);
                    rd_val
                } else if shift_amount < 32 {
                    self.shifter_output = rd_val.get_bit((shift_amount as u8) - 1);
                    ((rd_val as i32) >> shift_amount) as u32
                } else if (rd_val >> 31) > 0 {
                    self.shifter_output = 1;
                    u32::MAX
                } else {
                    self.shifter_output = 0;
                    0
                }
            }
            ThumbShiftInstruction::Ror => {
                let rotate_amount = shift_amount & 0x1F;
                if shift_amount == 0 {
                    self.shifter_output = self.get_flag(FlagsRegister::C);
                    rd_val
                } else if rotate_amount > 0 {
                    self.shifter_output = rd_val.get_bit((rotate_amount as u8) - 1);
                    rd_val.rotate_right(rotate_amount)
                } else {
                    self.shifter_output = rd_val >> 31;
                    rd_val
                }
            }
        };

        self.set_register(rd, result);
        self.set_arm_logical_flags(result, true);

        1
    }
}

impl CPU {
    pub fn thumb_move_shifted_register_instruction(
        &mut self,
        instruction: u32,
        memory: &mut GBAMemory,
    ) -> CYCLES {
        let opcode = (instruction & 0x1800) >> 11;
        let rs_val = self.get_register((instruction & 0x0038) >> 3);
        let rd = instruction & 0x0007;
        let offset: u8 = ((instruction & 0x07C0) >> 6) as u8;
        let operation = match opcode {
            0b00 => CPU::thumb_lsl,
            0b01 => CPU::thumb_lsr,
            0b10 => CPU::thumb_asr,
            _ => {
                panic!()
            }
        };

        operation(self, rd, rs_val, offset.into(), true);
        0
    }

    fn thumb_lsl(&mut self, rd: REGISTER, rs_val: u32, offset: u32, set_flags: bool) {
        let offset = offset & 0xFF;
        let result = rs_val << offset;
        self.set_register(rd, result);

        if set_flags {
            if offset > 0 {
                if rs_val.bit_is_set(32 - offset as u8) {
                    self.set_flag(FlagsRegister::C)
                } else {
                    self.reset_flag(FlagsRegister::C)
                }
            }

            if result == 0 {
                self.set_flag(FlagsRegister::Z);
            } else {
                self.reset_flag(FlagsRegister::Z);
            }

            if result.bit_is_set(31) {
                self.set_flag(FlagsRegister::N);
            } else {
                self.reset_flag(FlagsRegister::N);
            }
        }

        self.set_executed_instruction(format_args!("LSL {rd} {:#X} {:#X}", rs_val, offset));
    }

    fn thumb_lsr(&mut self, rd: REGISTER, rs_val: u32, offset: u32, set_flags: bool) {
        let offset = offset & 0xFF;
        if offset == 0 {
            // LSR#32
            if set_flags {
                self.set_flag(FlagsRegister::Z);
                self.reset_flag(FlagsRegister::N);
                self.set_flag_from_bit(FlagsRegister::C, rs_val.get_bit(31) as u8);
            }
            self.set_register(rd, 0);
            return;
        }
        let result = rs_val >> offset;

        if set_flags {
            if rs_val.bit_is_set((offset - 1) as u8) {
                self.set_flag(FlagsRegister::C);
            } else {
                self.reset_flag(FlagsRegister::C);
            }

            if result == 0 {
                self.set_flag(FlagsRegister::Z);
            } else {
                self.reset_flag(FlagsRegister::Z);
            }

            if result.bit_is_set(31) {
                self.set_flag(FlagsRegister::N);
            } else {
                self.reset_flag(FlagsRegister::N);
            }
        }

        self.set_register(rd, result);
        self.set_executed_instruction(format_args!("LSR {rd} {:#X} {:#X}", rs_val, offset));
    }

    fn thumb_asr(&mut self, rd: REGISTER, rs_val: u32, offset: u32, set_flags: bool) {
        let offset = offset & 0xFF;
        if offset == 0 {
            if rs_val.bit_is_set(31) {
                if set_flags {
                    self.set_flag(FlagsRegister::C);
                    self.set_flag(FlagsRegister::N);
                    self.reset_flag(FlagsRegister::Z);
                }
                self.set_register(rd, u32::MAX);
            } else {
                if set_flags {
                    self.reset_flag(FlagsRegister::C);
                    self.set_flag(FlagsRegister::Z);
                    self.reset_flag(FlagsRegister::N);
                }
                self.set_register(rd, 0);
            }

            return;
        }

        let result = (rs_val as i32 >> offset) as u32;
        if set_flags {
            if rs_val.bit_is_set((offset - 1) as u8) {
                self.set_flag(FlagsRegister::C);
            } else {
                self.reset_flag(FlagsRegister::C);
            }

            if result == 0 {
                self.set_flag(FlagsRegister::Z);
            } else {
                self.reset_flag(FlagsRegister::Z);
            }

            if result.bit_is_set(31) {
                self.set_flag(FlagsRegister::N);
            } else {
                self.reset_flag(FlagsRegister::N);
            }
        }
        self.set_register(rd, result);
        self.set_executed_instruction(format_args!("ASR {rd} {:#X} {:#X}", rs_val, offset));
    }

    pub fn thumb_add_or_subtract_instruction(
        &mut self,
        instruction: u32,
        memory: &mut GBAMemory,
    ) -> CYCLES {
        let opcode = (instruction & 0x0600) >> 9;
        let operand2 = (instruction & 0x01C0) >> 6;
        let operand2_value;
        let rd = instruction & 0x0007;
        let rs_value = self.get_register((instruction & 0x0038) >> 3);

        let operation = match opcode {
            0b00 => {
                operand2_value = self.get_register(operand2);
                CPU::arm_add
            }
            0b01 => {
                operand2_value = self.get_register(operand2);
                CPU::arm_sub
            }
            0b10 => {
                operand2_value = operand2;
                CPU::arm_add
            }
            0b11 => {
                operand2_value = operand2;
                CPU::arm_sub
            }
            _ => {
                panic!()
            }
        };

        operation(self, rd, rs_value, operand2_value, true);
        0
    }

    pub fn thumb_move_add_compare_add_subtract_immediate(
        &mut self,
        instruction: u32,
        memory: &mut GBAMemory,
    ) -> CYCLES {
        let opcode = (instruction & 0x1800) >> 11;
        let rd = (instruction & 0x0700) >> 8;
        let imm: u8 = (instruction & 0x00FF) as u8;

        let operation = match opcode {
            0b00 => CPU::thumb_move_imm,
            0b01 => CPU::thumb_cmp_imm,
            0b10 => CPU::thumb_add_imm,
            0b11 => CPU::thumb_sub_imm,
            _ => panic!(),
        };

        operation(self, rd, imm);

        0
    }

    fn thumb_move_imm(&mut self, rd: REGISTER, imm: u8) {
        self.reset_flag(FlagsRegister::N);
        if imm == 0 {
            self.set_flag(FlagsRegister::Z);
        } else {
            self.reset_flag(FlagsRegister::Z);
        }
        self.set_register(rd, imm.into());
        self.set_executed_instruction(format_args!("MOV {} {:#X}", rd, imm));
    }

    fn thumb_cmp_imm(&mut self, rd: REGISTER, imm: u8) {
        let minuend = self.get_register(rd);
        let result = minuend - imm as u32;
        self.set_arithmetic_flags(result, minuend, !(imm as u32), 1, true);
        self.set_executed_instruction(format_args!("CMP r{} {:#X}", rd, imm));
    }

    fn thumb_add_imm(&mut self, rd: REGISTER, imm: u8) {
        let addend1 = self.get_register(rd);
        let result = addend1 + imm as u32;
        self.set_arithmetic_flags(result, addend1, imm as u32, 0, true);
        self.set_register(rd, result);
        self.set_executed_instruction(format_args!("ADD {} {:#X}", rd, imm));
    }

    fn thumb_sub_imm(&mut self, rd: REGISTER, imm: u8) {
        let minuend = self.get_register(rd);
        let imm = !(imm as u32);
        let result = minuend + imm + 1;
        self.set_arithmetic_flags(result, minuend, imm, 1, true);
        self.set_register(rd, result);
        self.set_executed_instruction(format_args!("SUB {} {:#X}", rd, imm));
    }

    pub fn thumb_alu_instructions(&mut self, instruction: u32, memory: &mut GBAMemory) -> CYCLES {
        let opcode = (instruction & 0x03C0) >> 6;
        let mut cycles = 0;

        let rd = instruction & 0x0007;
        let rs = (instruction & 0x0038) >> 3;

        let operation = match opcode {
            0x0 => CPU::arm_and,
            0x1 => CPU::arm_eor,
            0x2 => {
                cycles += self.advance_pipeline(memory) + 1;
                CPU::thumb_lsl
            }
            0x3 => {
                cycles += self.advance_pipeline(memory) + 1;
                CPU::thumb_lsr_register
            }
            0x4 => {
                cycles += self.advance_pipeline(memory) + 1;
                CPU::thumb_asr_register
            }
            0x5 => CPU::arm_adc,
            0x6 => CPU::arm_sbc,
            0x7 => {
                cycles += self.advance_pipeline(memory) + 1;
                CPU::thumb_ror
            }
            0x8 => CPU::arm_tst,
            0x9 => CPU::thumb_neg,
            0xA => CPU::arm_cmp,
            0xB => CPU::arm_cmn,
            0xC => CPU::arm_orr,
            0xD => {
                let multiplier = self.get_register(rd);
                cycles += if multiplier & 0xFFFF_FF00 == 0
                    || multiplier & 0xFFFF_FF00 == 0xFFFF_FF00
                {
                    1
                } else if multiplier & 0xFFFF_0000 == 0 || multiplier & 0xFFFF_0000 == 0xFFFF_0000 {
                    2
                } else if multiplier & 0xFF00_0000 == 0 || multiplier & 0xFF00_0000 == 0xFFFF_0000 {
                    3
                } else {
                    4
                };
                CPU::thumb_mul
            }
            0xE => CPU::arm_bic,
            0xF => CPU::arm_mvn,
            _ => panic!("Unimplemented operation"),
        };

        operation(self, rd, self.get_register(rd), self.get_register(rs), true);

        cycles
    }

    fn thumb_lsr_register(&mut self, rd: REGISTER, rs_val: u32, offset: u32, set_flags: bool) {
        let offset = offset & 0xFF;
        let result = rs_val >> offset;

        if set_flags {
            if offset > 0 && rs_val.bit_is_set((offset - 1) as u8) {
                self.set_flag(FlagsRegister::C);
            } else {
                self.reset_flag(FlagsRegister::C);
            }

            if result == 0 {
                self.set_flag(FlagsRegister::Z);
            } else {
                self.reset_flag(FlagsRegister::Z);
            }

            if result.bit_is_set(31) {
                self.set_flag(FlagsRegister::N);
            } else {
                self.reset_flag(FlagsRegister::N);
            }
        }

        self.set_register(rd, result);
    }

    fn thumb_asr_register(&mut self, rd: REGISTER, rs_val: u32, offset: u32, set_flags: bool) {
        let offset = offset & 0xFF;
        let result = (rs_val as i32 >> offset) as u32;
        if set_flags {
            if offset > 0 && rs_val.bit_is_set((offset - 1) as u8) {
                self.set_flag(FlagsRegister::C);
            } else {
                self.reset_flag(FlagsRegister::C);
            }

            if result == 0 {
                self.set_flag(FlagsRegister::Z);
            } else {
                self.reset_flag(FlagsRegister::Z);
            }

            if result.bit_is_set(31) {
                self.set_flag(FlagsRegister::N);
            } else {
                self.reset_flag(FlagsRegister::N);
            }
        }
        self.set_register(rd, result);
    }

    fn thumb_ror(&mut self, rd: REGISTER, operand1: u32, operand2: u32, set_flags: bool) {
        let result = operand1.rotate_right(operand2 & 0xFF);
        if set_flags {
            if operand2 > 0 && operand1.bit_is_set((operand2 - 1) as u8) {
                self.set_flag(FlagsRegister::C);
            } else {
                self.reset_flag(FlagsRegister::C);
            }
        }
        self.set_logical_flags(result, set_flags);
        self.set_register(rd, result);
    }

    #[allow(unused)]
    fn thumb_neg(&mut self, rd: REGISTER, operand1: u32, operand2: u32, set_flags: bool) {
        self.arm_rsb(rd, operand2, 0, set_flags);
    }

    fn thumb_mul(&mut self, rd: REGISTER, operand1: u32, operand2: u32, set_flags: bool) {
        let result = (operand1 as u64 * operand2 as u64) as u32;
        if set_flags {
            self.set_flag_from_bit(FlagsRegister::N, result.get_bit(31) as u8);
            if result == 0 {
                self.set_flag(FlagsRegister::Z);
            } else {
                self.reset_flag(FlagsRegister::Z);
            }
        }
        self.set_register(rd, result);
    }

    pub fn thumb_hi_reg_operations(&mut self, instruction: u32, memory: &mut GBAMemory) -> CYCLES {
        let mut cycles = 0;
        let opcode = (instruction & 0x0300) >> 8;

        let rd = (instruction.get_bit(7) << 3) | (instruction & 0x0007);
        let rs = (instruction.get_bit(6) << 3) | ((instruction & 0x0038) >> 3);

        let operation = match opcode {
            0b00 => CPU::arm_add,
            0b01 => CPU::arm_cmp,
            0b10 => CPU::arm_mov,
            _ => panic!(),
        };

        operation(
            self,
            rd,
            self.get_register(rd),
            self.get_register(rs),
            false,
        );

        if rd == PC_REGISTER as u32 {
            cycles += self.flush_pipeline(memory);
        }

        cycles
    }

    pub fn thumb_bx(&mut self, instruction: u32, memory: &mut GBAMemory) -> CYCLES {
        let mut cycles = 1;
        let rs = (instruction.get_bit(6) << 3) | ((instruction & 0x0038) >> 3);
        let mut destination = self.get_register(rs);
        if destination.bit_is_set(0) {
            self.set_instruction_mode(InstructionMode::THUMB);
        } else {
            destination &= !2; // arm instructions must be word aligned
            self.set_instruction_mode(InstructionMode::ARM);
        };

        self.set_pc(destination & !1); // bit 0 is forced to 0 before storing
        cycles += self.flush_pipeline(memory);
        self.set_executed_instruction(format_args!("BX {:#010x}", destination));

        cycles
    }

    pub fn thumb_get_relative_address(
        &mut self,
        instruction: u32,
        memory: &mut GBAMemory,
    ) -> CYCLES {
        let opcode = instruction.get_bit(11);
        let rd = (instruction & 0x0700) >> 8;
        let imm = (instruction & 0x00FF) * 4;

        let result = match opcode {
            0b0 => (self.get_pc() & !2) + imm,
            0b1 => self.get_sp() + imm,
            _ => panic!(),
        };

        self.set_register(rd, result);

        self.set_executed_instruction(format_args!("ADR r{}, {:#X}", rd, imm));
        0
    }

    pub fn thumb_add_offset_to_sp(&mut self, instruction: u32, memory: &mut GBAMemory) -> CYCLES {
        let opcode = instruction.get_bit(7);
        let imm = (instruction & 0x007F) * 4;

        let result = match opcode {
            0b0 => self.get_sp() + imm,
            0b1 => self.get_sp() - imm,
            _ => panic!(),
        };

        self.set_sp(result);
        self.set_executed_instruction(format_args!(
            "ADD SP, {}{:#X}",
            if opcode == 0b0 { "+" } else { "-" },
            imm
        ));

        0
    }
}

#[cfg(test)]
mod thumb_add_and_subtract_tests {

    use rstest::rstest;

    use crate::{
        arm7tdmi::{
            cpu::{FlagsRegister, InstructionMode},
            instruction_table::{Execute, Operand},
            thumb::alu::{ThumbFullAdder, ThumbFullAdderOperations},
        },
        gba::GBA,
    };

    #[rstest]
    #[case(ThumbFullAdderOperations::Add, 20, 43, 63)]
    #[case(ThumbFullAdderOperations::Add, 0xFFFFFFFF, 1, 0)]
    #[case(ThumbFullAdderOperations::Sub, 0xFFFFFFFF, 1, 0xFFFFFFFE)]
    fn able_to_execute_full_adder_instructions_with_register_op2(
        #[case] operation: ThumbFullAdderOperations,
        #[case] register2_val: u32,
        #[case] register1_val: u32,
        #[case] expected_result: u32,
    ) {
        let mut gba = GBA::new_no_bios();
        gba.cpu.set_register(1, register1_val);
        gba.cpu.set_register(2, register2_val);
        let instruction = ThumbFullAdder(operation, 3, 2, Operand::Register(1));

        instruction.execute(&mut gba.cpu, &mut gba.memory);

        assert_eq!(gba.cpu.get_register(3), expected_result);
    }

    #[test]
    fn should_add_two_registers_together() {
        let mut gba = GBA::new_no_bios();
        gba.cpu.set_instruction_mode(InstructionMode::THUMB);

        gba.cpu.set_register(1, 20);
        gba.cpu.set_register(2, 43);
        gba.cpu.prefetch[0] = Some(0x1888); // adds r0, r1, r2
        gba.step();
        gba.step();

        assert_eq!(gba.cpu.get_register(0), 63);
        assert!(gba.cpu.get_flag(FlagsRegister::C) == 0);
        assert!(gba.cpu.get_flag(FlagsRegister::N) == 0);
        assert!(gba.cpu.get_flag(FlagsRegister::Z) == 0);
        assert!(gba.cpu.get_flag(FlagsRegister::V) == 0);
    }

    #[test]
    fn should_add_two_registers_together_and_set_n_flag() {
        let mut gba = GBA::new_no_bios();
        gba.cpu.set_instruction_mode(InstructionMode::THUMB);

        gba.cpu.set_register(1, 20);
        gba.cpu.set_register(2, (-43 as i32) as u32);
        gba.cpu.prefetch[0] = Some(0x1888); // adds r0, r1, r2
        gba.step();
        gba.step();

        assert_eq!(gba.cpu.get_register(0), (-23 as i32) as u32);
        assert!(gba.cpu.get_flag(FlagsRegister::C) == 0);
        assert!(gba.cpu.get_flag(FlagsRegister::N) == 1);
        assert!(gba.cpu.get_flag(FlagsRegister::Z) == 0);
        assert!(gba.cpu.get_flag(FlagsRegister::V) == 0);
    }

    #[test]
    fn should_add_two_registers_together_and_set_z_and_c_flag() {
        let mut gba = GBA::new_no_bios();
        gba.cpu.set_instruction_mode(InstructionMode::THUMB);

        gba.cpu.set_register(1, 20);
        gba.cpu.set_register(2, (-20 as i32) as u32);
        gba.cpu.prefetch[0] = Some(0x1888); // adds r0, r1, r2
        gba.step();
        gba.step();

        assert_eq!(gba.cpu.get_register(0), 0);
        assert!(gba.cpu.get_flag(FlagsRegister::C) == 1);
        assert!(gba.cpu.get_flag(FlagsRegister::N) == 0);
        assert!(gba.cpu.get_flag(FlagsRegister::Z) == 1);
        assert!(gba.cpu.get_flag(FlagsRegister::V) == 0);
    }

    #[test]
    fn should_add_two_registers_together_and_set_z_flag() {
        let mut gba = GBA::new_no_bios();
        gba.cpu.set_instruction_mode(InstructionMode::THUMB);

        gba.cpu.set_register(1, 0);
        gba.cpu.set_register(2, 0);
        gba.cpu.prefetch[0] = Some(0x1888); // adds r0, r1, r2
        gba.step();
        gba.step();

        assert_eq!(gba.cpu.get_register(0), 0);
        assert!(gba.cpu.get_flag(FlagsRegister::C) == 0);
        assert!(gba.cpu.get_flag(FlagsRegister::N) == 0);
        assert!(gba.cpu.get_flag(FlagsRegister::Z) == 1);
        assert!(gba.cpu.get_flag(FlagsRegister::V) == 0);
    }

    #[test]
    fn should_add_two_registers_together_and_set_v_flag() {
        let mut gba = GBA::new_no_bios();
        gba.cpu.set_instruction_mode(InstructionMode::THUMB);

        gba.cpu.set_register(1, 0x1);
        gba.cpu.set_register(2, 0x7FFF_FFFF);
        gba.cpu.prefetch[0] = Some(0x1888); // adds r0, r1, r2
        gba.step();
        gba.step();

        assert_eq!(gba.cpu.get_register(0), 0x8000_0000);
        assert!(gba.cpu.get_flag(FlagsRegister::C) == 0);
        assert!(gba.cpu.get_flag(FlagsRegister::N) == 1);
        assert!(gba.cpu.get_flag(FlagsRegister::Z) == 0);
        assert!(gba.cpu.get_flag(FlagsRegister::V) == 1);
    }

    #[test]
    fn should_add_register_and_immediate_and_set_n_flag() {
        let mut gba = GBA::new_no_bios();
        gba.cpu.set_instruction_mode(InstructionMode::THUMB);

        gba.cpu.set_register(1, -10 as i32 as u32);
        gba.cpu.prefetch[0] = Some(0x1d48); // adds r0, r1, 5
        gba.step();
        gba.step();

        assert_eq!(gba.cpu.get_register(0), -5 as i32 as u32);
        assert!(gba.cpu.get_flag(FlagsRegister::C) == 0);
        assert!(gba.cpu.get_flag(FlagsRegister::N) == 1);
        assert!(gba.cpu.get_flag(FlagsRegister::Z) == 0);
        assert!(gba.cpu.get_flag(FlagsRegister::V) == 0);
    }

    #[test]
    fn should_add_register_and_immediate_and_set_z_flag() {
        let mut gba = GBA::new_no_bios();
        gba.cpu.set_instruction_mode(InstructionMode::THUMB);

        gba.cpu.set_register(1, -5 as i32 as u32);
        gba.cpu.prefetch[0] = Some(0x1d48); // adds r0, r1, 5
        gba.step();
        gba.step();

        assert_eq!(gba.cpu.get_register(0), 0 as i32 as u32);
        assert!(gba.cpu.get_flag(FlagsRegister::C) == 1);
        assert!(gba.cpu.get_flag(FlagsRegister::N) == 0);
        assert!(gba.cpu.get_flag(FlagsRegister::Z) == 1);
        assert!(gba.cpu.get_flag(FlagsRegister::V) == 0);
    }

    #[test]
    fn should_add_register_and_immediate_and_set_v_flag() {
        let mut gba = GBA::new_no_bios();
        gba.cpu.set_instruction_mode(InstructionMode::THUMB);

        gba.cpu.set_register(1, 0x7FFF_FFFF);
        gba.cpu.prefetch[0] = Some(0x1d48); // adds r0, r1, 5
        gba.step();
        gba.step();

        assert_eq!(gba.cpu.get_register(0), 0x8000_0004);
        assert!(gba.cpu.get_flag(FlagsRegister::C) == 0);
        assert!(gba.cpu.get_flag(FlagsRegister::N) == 1);
        assert!(gba.cpu.get_flag(FlagsRegister::Z) == 0);
        assert!(gba.cpu.get_flag(FlagsRegister::V) == 1);
    }

    #[test]
    fn should_add_register_and_immediate_and_set_c_flag() {
        let mut gba = GBA::new_no_bios();
        gba.cpu.set_instruction_mode(InstructionMode::THUMB);

        gba.cpu.set_register(1, 0xFFFF_FFFF);
        gba.cpu.prefetch[0] = Some(0x1d48); // adds r0, r1, 5
        gba.step();
        gba.step();

        assert_eq!(gba.cpu.get_register(0), 0x0000_0004);
        assert!(gba.cpu.get_flag(FlagsRegister::C) == 1);
        assert!(gba.cpu.get_flag(FlagsRegister::N) == 0);
        assert!(gba.cpu.get_flag(FlagsRegister::Z) == 0);
        assert!(gba.cpu.get_flag(FlagsRegister::V) == 0);
    }

    #[test]
    fn should_sub_two_registers() {
        let mut gba = GBA::new_no_bios();
        gba.cpu.set_instruction_mode(InstructionMode::THUMB);

        gba.cpu.set_register(1, 50);
        gba.cpu.set_register(2, 20);
        gba.cpu.prefetch[0] = Some(0x1a88); // subs r0, r1, r2
        gba.step();
        gba.step();

        assert_eq!(gba.cpu.get_register(0), 30);
        assert!(gba.cpu.get_flag(FlagsRegister::C) == 1);
        assert!(gba.cpu.get_flag(FlagsRegister::N) == 0);
        assert!(gba.cpu.get_flag(FlagsRegister::Z) == 0);
        assert!(gba.cpu.get_flag(FlagsRegister::V) == 0);
    }

    #[test]
    fn should_sub_two_registers_and_reset_c_flag() {
        let mut gba = GBA::new_no_bios();
        gba.cpu.set_instruction_mode(InstructionMode::THUMB);

        gba.cpu.set_register(1, 25);
        gba.cpu.set_register(2, 50);
        gba.cpu.prefetch[0] = Some(0x1a88); // subs r0, r1, r2
        gba.step();
        gba.step();

        assert_eq!(gba.cpu.get_register(0), -25 as i32 as u32);
        assert_eq!(gba.cpu.get_flag(FlagsRegister::C), 0);
        assert_eq!(gba.cpu.get_flag(FlagsRegister::N), 1);
        assert_eq!(gba.cpu.get_flag(FlagsRegister::Z), 0);
        assert_eq!(gba.cpu.get_flag(FlagsRegister::V), 0);
    }
}

#[cfg(test)]
mod thumb_move_shifted_register_tests {

    use crate::{
        arm7tdmi::cpu::{FlagsRegister, InstructionMode, CPU},
        gba::GBA,
        memory::memory::GBAMemory,
    };

    #[test]
    fn should_left_shift_a_register_and_set_c_flag() {
        let mut gba = GBA::new_no_bios();
        gba.cpu.set_instruction_mode(InstructionMode::THUMB);

        gba.cpu.set_register(1, 0x0F00_0000);
        gba.cpu.prefetch[0] = Some(0x0148); // lsls r0, r1, 5
        gba.step();
        gba.step();

        assert_eq!(gba.cpu.get_register(0), 0x0F00_0000 << 5);
        assert_eq!(gba.cpu.get_flag(FlagsRegister::C), 1);
        assert_eq!(gba.cpu.get_flag(FlagsRegister::N), 1);
        assert_eq!(gba.cpu.get_flag(FlagsRegister::Z), 0);
    }

    #[test]
    fn should_not_left_shift_register_and_not_change_c_flag() {
        let mut gba = GBA::new_no_bios();
        gba.cpu.set_instruction_mode(InstructionMode::THUMB);

        gba.cpu.set_register(1, 0x0F00_0000);
        gba.cpu.set_flag(FlagsRegister::C);
        gba.cpu.prefetch[0] = Some(0x0008); // lsls r0, r1, 0
        gba.step();
        gba.step();

        assert_eq!(gba.cpu.get_register(0), 0x0F00_0000);
        assert_eq!(gba.cpu.get_flag(FlagsRegister::C), 1);
        assert_eq!(gba.cpu.get_flag(FlagsRegister::N), 0);
        assert_eq!(gba.cpu.get_flag(FlagsRegister::Z), 0);
    }

    #[test]
    fn should_lsl_register_and_not_affect_v_flag() {
        let mut gba = GBA::new_no_bios();
        gba.cpu.set_instruction_mode(InstructionMode::THUMB);

        gba.cpu.set_register(1, 0xF000_0000);
        gba.cpu.set_flag(FlagsRegister::V);
        gba.cpu.prefetch[0] = Some(0x0148); // lsls r0, r1, 5
        gba.step();
        gba.step();

        assert_eq!(gba.cpu.get_register(0), 0x0);
        assert_eq!(gba.cpu.get_flag(FlagsRegister::C), 0);
        assert_eq!(gba.cpu.get_flag(FlagsRegister::N), 0);
        assert_eq!(gba.cpu.get_flag(FlagsRegister::Z), 1);
        assert_eq!(gba.cpu.get_flag(FlagsRegister::V), 1);
    }

    #[test]
    fn should_asr_register_and_set_c_flag() {
        let mut gba = GBA::new_no_bios();
        gba.cpu.set_instruction_mode(InstructionMode::THUMB);

        gba.cpu.set_register(1, 0x0000_008F);
        gba.cpu.prefetch[0] = Some(0x1108); // asrs r0, r1, 4
        gba.step();
        gba.step();

        assert_eq!(gba.cpu.get_register(0), 0x0000_0008);
        assert_eq!(gba.cpu.get_flag(FlagsRegister::C), 1);
        assert_eq!(gba.cpu.get_flag(FlagsRegister::N), 0);
        assert_eq!(gba.cpu.get_flag(FlagsRegister::Z), 0);
    }

    #[test]
    fn should_asr_register_and_maintain_sign() {
        let mut gba = GBA::new_no_bios();
        gba.cpu.set_instruction_mode(InstructionMode::THUMB);

        gba.cpu.set_register(1, 0x8000_008F);
        gba.cpu.prefetch[0] = Some(0x1108); // asrs r0, r1, 4
        gba.step();
        gba.step();

        assert_eq!(gba.cpu.get_register(0), 0xF800_0008);
        assert_eq!(gba.cpu.get_flag(FlagsRegister::C), 1);
        assert_eq!(gba.cpu.get_flag(FlagsRegister::N), 1);
        assert_eq!(gba.cpu.get_flag(FlagsRegister::Z), 0);
    }

    #[test]
    fn should_asr_register_and_set_all_ones() {
        let mut gba = GBA::new_no_bios();
        gba.cpu.set_instruction_mode(InstructionMode::THUMB);

        gba.cpu.set_register(1, 0x8000_0000);
        gba.cpu.prefetch[0] = Some(0x1008); // asrs r0, r1, 32
        gba.step();
        gba.step();

        assert_eq!(gba.cpu.get_register(0), 0xFFFF_FFFF);
        assert_eq!(gba.cpu.get_flag(FlagsRegister::C), 1);
        assert_eq!(gba.cpu.get_flag(FlagsRegister::N), 1);
        assert_eq!(gba.cpu.get_flag(FlagsRegister::Z), 0);
    }

    #[test]
    fn should_lsr_register() {
        let mut gba = GBA::new_no_bios();
        gba.cpu.set_instruction_mode(InstructionMode::THUMB);

        gba.cpu.set_register(1, 0x8000_008F);
        gba.cpu.prefetch[0] = Some(0x0a88); // lsrs r0, r1, 10
        gba.step();
        gba.step();

        assert_eq!(gba.cpu.get_register(0), 0x8000_008F >> 10);
        assert_eq!(gba.cpu.get_flag(FlagsRegister::C), 0);
        assert_eq!(gba.cpu.get_flag(FlagsRegister::N), 0);
        assert_eq!(gba.cpu.get_flag(FlagsRegister::Z), 0);
    }

    #[test]
    fn should_lsr_register_and_clear_register() {
        let mut gba = GBA::new_no_bios();
        gba.cpu.set_instruction_mode(InstructionMode::THUMB);

        gba.cpu.set_register(1, 0x8000_008F);
        gba.cpu.prefetch[0] = Some(0x0808); // lsrs r0, r1, 32
        gba.step();
        gba.step();

        assert_eq!(gba.cpu.get_register(0), 0);
        assert_eq!(gba.cpu.get_flag(FlagsRegister::C), 1);
        assert_eq!(gba.cpu.get_flag(FlagsRegister::N), 0);
        assert_eq!(gba.cpu.get_flag(FlagsRegister::Z), 1);
    }
}

#[cfg(test)]
mod thumb_move_compare_add_subtract_tests {

    use crate::{
        arm7tdmi::cpu::{FlagsRegister, InstructionMode, CPU},
        gba::GBA,
        memory::memory::GBAMemory,
    };

    #[test]
    fn should_move_immediate_into_r0() {
        let mut gba = GBA::new_no_bios();
        gba.cpu.set_instruction_mode(InstructionMode::THUMB);

        gba.cpu.prefetch[0] = Some(0x200f); // movs r0, 15
        gba.step();
        gba.step();

        assert_eq!(gba.cpu.get_register(0), 15);
        assert_eq!(gba.cpu.get_flag(FlagsRegister::C), 0);
        assert_eq!(gba.cpu.get_flag(FlagsRegister::N), 0);
        assert_eq!(gba.cpu.get_flag(FlagsRegister::Z), 0);
    }

    #[test]
    fn should_move_immediate_into_r0_and_not_set_n_flag() {
        let mut gba = GBA::new_no_bios();
        gba.cpu.set_instruction_mode(InstructionMode::THUMB);

        gba.cpu.prefetch[0] = Some(0x2096); // movs r0, 150
        gba.step();
        gba.step();

        assert_eq!(gba.cpu.get_register(0), 150);
        assert_eq!(gba.cpu.get_flag(FlagsRegister::C), 0);
        assert_eq!(gba.cpu.get_flag(FlagsRegister::N), 0);
        assert_eq!(gba.cpu.get_flag(FlagsRegister::Z), 0);
    }

    #[test]
    fn should_move_immediate_into_r0_and_set_z_flag() {
        let mut gba = GBA::new_no_bios();
        gba.cpu.set_instruction_mode(InstructionMode::THUMB);

        gba.cpu.prefetch[0] = Some(0x2000); // movs r0, 0
        gba.step();
        gba.step();

        assert_eq!(gba.cpu.get_register(0), 0);
        assert_eq!(gba.cpu.get_flag(FlagsRegister::C), 0);
        assert_eq!(gba.cpu.get_flag(FlagsRegister::N), 0);
        assert_eq!(gba.cpu.get_flag(FlagsRegister::Z), 1);
    }

    #[test]
    fn should_sub_imm_from_r0_and_set_z_flag() {
        let mut gba = GBA::new_no_bios();
        gba.cpu.set_instruction_mode(InstructionMode::THUMB);

        gba.cpu.set_register(0, 15);
        gba.cpu.prefetch[0] = Some(0x380f); // subs r0, 15
        gba.step();
        gba.step();

        assert_eq!(gba.cpu.get_register(0), 0);
        assert_eq!(gba.cpu.get_flag(FlagsRegister::C), 1);
        assert_eq!(gba.cpu.get_flag(FlagsRegister::N), 0);
        assert_eq!(gba.cpu.get_flag(FlagsRegister::Z), 1);
    }

    #[test]
    fn should_add_imm_to_r0_and_set_n_flag() {
        let mut gba = GBA::new_no_bios();
        gba.cpu.set_instruction_mode(InstructionMode::THUMB);

        gba.cpu.set_register(0, 0x7FFF_FFFF);
        gba.cpu.prefetch[0] = Some(0x300f); // adds r0, 15
        gba.step();
        gba.step();

        assert_eq!(gba.cpu.get_register(0), 0x8000_000E);
        assert_eq!(gba.cpu.get_flag(FlagsRegister::C), 0);
        assert_eq!(gba.cpu.get_flag(FlagsRegister::N), 1);
        assert_eq!(gba.cpu.get_flag(FlagsRegister::Z), 0);
    }
}

#[cfg(test)]
mod thumb_alu_operations_tests {

    use crate::{
        arm7tdmi::cpu::{FlagsRegister, InstructionMode, CPU},
        gba::GBA,
        memory::memory::GBAMemory,
    };

    #[test]
    fn should_and_two_numbers_together() {
        let mut gba = GBA::new_no_bios();
        gba.cpu.set_instruction_mode(InstructionMode::THUMB);

        gba.cpu.set_register(0, 0x8123_2344);
        gba.cpu.set_register(1, 0x8000_2344);
        gba.cpu.prefetch[0] = Some(0x4008); // ands r0, r1
        gba.step();
        gba.step();

        assert_eq!(gba.cpu.get_register(0), 0x8000_2344);
        assert_eq!(gba.cpu.get_flag(FlagsRegister::N), 1);
        assert_eq!(gba.cpu.get_flag(FlagsRegister::Z), 0);
    }

    #[test]
    fn should_eor_two_numbers_together() {
        let mut gba = GBA::new_no_bios();
        gba.cpu.set_instruction_mode(InstructionMode::THUMB);

        gba.cpu.set_register(0, 0x1010_1010);
        gba.cpu.set_register(1, 0x0101_0101);
        gba.cpu.prefetch[0] = Some(0x4048); // eors r0, r1
        gba.step();
        gba.step();

        assert_eq!(gba.cpu.get_register(0), 0x1111_1111);
        assert_eq!(gba.cpu.get_flag(FlagsRegister::N), 0);
        assert_eq!(gba.cpu.get_flag(FlagsRegister::Z), 0);
    }

    #[test]
    fn should_lsl_rd_by_5() {
        let mut gba = GBA::new_no_bios();
        gba.cpu.set_instruction_mode(InstructionMode::THUMB);

        gba.cpu.set_register(0, 0x0F11_1230);
        gba.cpu.set_register(1, 5);
        gba.cpu.prefetch[0] = Some(0x4088); // lsl r0, r1
        gba.step();
        gba.step();

        assert_eq!(gba.cpu.get_register(0), 0xE222_4600);
        assert_eq!(gba.cpu.get_flag(FlagsRegister::N), 1);
        assert_eq!(gba.cpu.get_flag(FlagsRegister::C), 1);
        assert_eq!(gba.cpu.get_flag(FlagsRegister::Z), 0);
    }

    #[test]
    fn should_lsr_rd_by_0() {
        let mut gba = GBA::new_no_bios();
        gba.cpu.set_instruction_mode(InstructionMode::THUMB);

        gba.cpu.set_register(0, 0x0F11_1230);
        gba.cpu.set_register(1, 0);
        gba.cpu.prefetch[0] = Some(0x40c8); // lsr r0, r1
        gba.step();
        gba.step();

        assert_eq!(gba.cpu.get_register(0), 0x0F11_1230);
        assert_eq!(gba.cpu.get_flag(FlagsRegister::N), 0);
        assert_eq!(gba.cpu.get_flag(FlagsRegister::C), 0);
        assert_eq!(gba.cpu.get_flag(FlagsRegister::Z), 0);
    }
}

#[cfg(test)]
mod thumb_hi_reg_operations {

    use crate::{
        arm7tdmi::cpu::{FlagsRegister, InstructionMode, CPU},
        gba::GBA,
        memory::memory::GBAMemory,
    };

    #[test]
    fn should_add_two_regs_together_and_not_affect_flags() {
        let mut gba = GBA::new_no_bios();
        gba.cpu.set_instruction_mode(InstructionMode::THUMB);

        gba.cpu.set_register(0, 20);
        gba.cpu.set_register(11, 15);
        gba.cpu.set_flag(FlagsRegister::N);
        gba.cpu.prefetch[1] = Some(0x4458); // add r0, r11
        gba.step();

        assert_eq!(gba.cpu.get_register(0), 35);
        assert_eq!(gba.cpu.get_flag(FlagsRegister::N), 1);
        assert_eq!(gba.cpu.get_flag(FlagsRegister::C), 0);
        assert_eq!(gba.cpu.get_flag(FlagsRegister::Z), 0);
    }

    #[test]
    fn should_cmp_registers_and_set_flags() {
        let mut gba = GBA::new_no_bios();
        gba.cpu.set_instruction_mode(InstructionMode::THUMB);

        gba.cpu.set_register(0, 20);
        gba.cpu.set_register(11, 20);
        gba.cpu.set_flag(FlagsRegister::N);
        gba.cpu.prefetch[0] = Some(0x4558); // cmp r0, r11
        gba.step();
        gba.step();

        assert_eq!(gba.cpu.get_register(0), 20);
        assert_eq!(gba.cpu.get_flag(FlagsRegister::N), 0);
        assert_eq!(gba.cpu.get_flag(FlagsRegister::C), 1);
        assert_eq!(gba.cpu.get_flag(FlagsRegister::Z), 1);
    }

    #[test]
    fn should_mov_register() {
        let mut gba = GBA::new_no_bios();
        gba.cpu.set_instruction_mode(InstructionMode::THUMB);

        gba.cpu.set_register(0, 20);
        gba.cpu.set_register(11, 55);
        gba.cpu.set_flag(FlagsRegister::N);
        gba.cpu.prefetch[0] = Some(0x4658); // cmp r0, r11
        gba.step();
        gba.step();

        assert_eq!(gba.cpu.get_register(0), 55);
    }
}

#[cfg(test)]
mod thumb_bx_tests {

    use crate::{
        arm7tdmi::cpu::{InstructionMode, CPU},
        gba::GBA,
        memory::memory::GBAMemory,
    };

    #[test]
    fn should_switch_to_arm_mode_and_align_address() {
        let mut gba = GBA::new_no_bios();
        gba.cpu.set_instruction_mode(InstructionMode::THUMB);

        gba.cpu.set_register(5, 0x16);
        gba.cpu.prefetch[0] = Some(0x4728); // bx r5
        gba.step();
        gba.step();

        assert_eq!(gba.cpu.get_pc(), 0x1C);
        assert!(matches!(
            gba.cpu.get_instruction_mode(),
            InstructionMode::ARM
        ));
    }

    #[test]
    fn should_switch_to_arm_mode_when_pc_operand() {
        let mut gba = GBA::new_no_bios();
        gba.cpu.set_instruction_mode(InstructionMode::THUMB);

        gba.cpu.set_pc(0x16);
        gba.cpu.prefetch[0] = Some(0x4778); // bx r15
        gba.step();
        gba.step();

        assert_eq!(gba.cpu.get_pc(), 0x20);
        assert!(matches!(
            gba.cpu.get_instruction_mode(),
            InstructionMode::ARM
        ));
    }
}

#[cfg(test)]
mod get_relative_address_tests {

    use crate::{
        arm7tdmi::cpu::{InstructionMode, CPU},
        gba::GBA,
        memory::memory::GBAMemory,
    };

    #[test]
    fn should_add_12_to_pc() {
        let mut gba = GBA::new_no_bios();
        gba.cpu.set_instruction_mode(InstructionMode::THUMB);

        gba.cpu.prefetch[0] = Some(0xa503); // add r5, pc, 12
        gba.cpu.set_pc(2);
        gba.step();
        gba.step();

        assert_eq!(gba.cpu.get_pc(), 6);
        assert_eq!(gba.cpu.get_register(5), 16);
    }

    #[test]
    fn should_add_16_to_sp_and_store_in_r5() {
        let mut gba = GBA::new_no_bios();
        gba.cpu.set_instruction_mode(InstructionMode::THUMB);

        gba.cpu.prefetch[0] = Some(0xad04); // add r5, sp, 16
        gba.cpu.set_sp(2);
        gba.step();
        gba.step();

        assert_eq!(gba.cpu.get_register(5), 18);
    }

    #[test]
    fn should_add_500_to_sp() {
        let mut gba = GBA::new_no_bios();
        gba.cpu.set_instruction_mode(InstructionMode::THUMB);

        gba.cpu.prefetch[0] = Some(0xb07d); // add sp, 500
        gba.cpu.set_sp(2);
        gba.step();
        gba.step();

        assert_eq!(gba.cpu.get_sp(), 502);
    }

    #[test]
    fn should_sub_500_to_sp() {
        let mut gba = GBA::new_no_bios();
        gba.cpu.set_instruction_mode(InstructionMode::THUMB);

        gba.cpu.prefetch[0] = Some(0xb0fd); // add sp, 500
        gba.cpu.set_sp(2);
        gba.step();
        gba.step();

        assert_eq!(gba.cpu.get_sp(), (2 - 500) as i32 as u32);
    }
}
