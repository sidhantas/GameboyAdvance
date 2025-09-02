#![allow(unused)]
use num_traits::PrimInt;

use crate::{
    arm7tdmi::{
        cpu::{CPUMode, FlagsRegister, CPU, PC_REGISTER},
        instruction_table::Operand,
    },
    memory::memory::GBAMemory,
    types::{ARMByteCode, CYCLES, REGISTER, WORD},
    utils::bits::Bits,
};

#[derive(Debug)]
pub enum DataProcessingInstruction {
    Arithmetic(
        ArithmeticInstruction,
        Option<REGISTER>,
        REGISTER,
        Operand,
        Shift,
        bool,
    ),
    Logical(
        LogicalInstruction,
        Option<REGISTER>,
        REGISTER,
        Operand,
        Shift,
        bool,
    ),
}

#[derive(Debug)]
pub enum ArithmeticInstruction {
    Sub,
    Rsb,
    Add,
    Adc,
    Sbc,
    Rsc,
    Cmp,
    Cmn,
}

#[derive(Debug)]
pub enum LogicalInstruction {
    And,
    Eor,
    Tst,
    Teq,
    Orr,
    Mov,
    Bic,
    Mvn,
}

#[derive(Debug)]
pub struct Shift(ShiftType, Operand);

#[derive(Debug, Clone, Copy)]
pub enum ShiftType {
    LSL,
    LSR,
    ASR,
    ROR,
    RRX,
}

impl CPU {
    pub fn decode_data_processing_instruction(
        instruction: ARMByteCode,
    ) -> DataProcessingInstruction {
        let opcode = (instruction & 0x01E0_0000) >> 21;
        let rn = (0x000F_0000 & instruction) >> 16;
        let rd = (0x0000_F000 & instruction) >> 12;
        let (operand2, shift) = get_operand2_and_shift(instruction);
        let set_flags = instruction.bit_is_set(20) && rd != PC_REGISTER as u32;
        match opcode {
            0x0 => DataProcessingInstruction::Logical(
                LogicalInstruction::And,
                Some(rd),
                rn,
                operand2,
                shift,
                set_flags,
            ),
            0x1 => DataProcessingInstruction::Logical(
                LogicalInstruction::Eor,
                Some(rd),
                rn,
                operand2,
                shift,
                set_flags,
            ),
            0x2 => DataProcessingInstruction::Arithmetic(
                ArithmeticInstruction::Sub,
                Some(rd),
                rn,
                operand2,
                shift,
                set_flags,
            ),
            0x3 => DataProcessingInstruction::Arithmetic(
                ArithmeticInstruction::Rsb,
                Some(rd),
                rn,
                operand2,
                shift,
                set_flags,
            ),
            0x4 => DataProcessingInstruction::Arithmetic(
                ArithmeticInstruction::Add,
                Some(rd),
                rn,
                operand2,
                shift,
                set_flags,
            ),
            0x5 => DataProcessingInstruction::Arithmetic(
                ArithmeticInstruction::Adc,
                Some(rd),
                rn,
                operand2,
                shift,
                set_flags,
            ),
            0x6 => DataProcessingInstruction::Arithmetic(
                ArithmeticInstruction::Sbc,
                Some(rd),
                rn,
                operand2,
                shift,
                set_flags,
            ),
            0x7 => DataProcessingInstruction::Arithmetic(
                ArithmeticInstruction::Rsc,
                Some(rd),
                rn,
                operand2,
                shift,
                set_flags,
            ),
            0x8 => DataProcessingInstruction::Logical(
                LogicalInstruction::Tst,
                None,
                rn,
                operand2,
                shift,
                true,
            ),
            0x9 => DataProcessingInstruction::Logical(
                LogicalInstruction::Teq,
                None,
                rn,
                operand2,
                shift,
                true,
            ),
            0xA => DataProcessingInstruction::Arithmetic(
                ArithmeticInstruction::Cmp,
                None,
                rn,
                operand2,
                shift,
                true,
            ),
            0xB => DataProcessingInstruction::Arithmetic(
                ArithmeticInstruction::Cmn,
                None,
                rn,
                operand2,
                shift,
                true,
            ),
            0xC => DataProcessingInstruction::Logical(
                LogicalInstruction::Orr,
                Some(rd),
                rn,
                operand2,
                shift,
                set_flags,
            ),
            0xD => DataProcessingInstruction::Logical(
                LogicalInstruction::Mov,
                Some(rd),
                rn,
                operand2,
                shift,
                set_flags,
            ),
            0xE => DataProcessingInstruction::Logical(
                LogicalInstruction::Bic,
                Some(rd),
                rn,
                operand2,
                shift,
                set_flags,
            ),
            0xF => DataProcessingInstruction::Logical(
                LogicalInstruction::Mvn,
                Some(rd),
                rn,
                operand2,
                shift,
                set_flags,
            ),
            _ => unreachable!(),
        }
    }

    pub fn execute_data_processing_instruction(
        &mut self,
        memory: &mut GBAMemory,
        data_processing_instruction: DataProcessingInstruction,
    ) {
        let destination;
        let set_cpsr_flags;
        let result = match data_processing_instruction {
            DataProcessingInstruction::Arithmetic(
                instruction,
                rd,
                rn,
                operand2,
                shift,
                set_flags,
            ) => {
                destination = rd;
                set_cpsr_flags = set_flags;
                self.execute_arithmetic_instruction(
                    memory,
                    instruction,
                    rn,
                    operand2,
                    shift,
                    set_flags,
                )
            }
            DataProcessingInstruction::Logical(instruction, rd, rn, operand2, shift, set_flags) => {
                destination = rd;
                set_cpsr_flags = set_flags;
                self.execute_logical_instruction(
                    memory,
                    instruction,
                    rn,
                    operand2,
                    shift,
                    set_flags,
                )
            }
        };

        if let Some(destination) = destination {
            self.set_register(destination, result);
            if destination == PC_REGISTER as u32 {
                if set_cpsr_flags {
                    self.pop_spsr();
                }
                self.flush_pipeline(memory);
            }
        };
    }

    fn execute_arithmetic_instruction(
        &mut self,
        memory: &mut GBAMemory,
        instruction: ArithmeticInstruction,
        rn: REGISTER,
        operand2: Operand,
        shift: Shift,
        set_flags: bool,
    ) -> u32 {
        let rn_val = self.get_register(rn);
        let shifted_operand2 = match operand2 {
            Operand::Immeidate(imm) => self.execute_immediate_shift(imm, shift),
            Operand::Register(reg) => self.execute_register_shift(memory, reg, shift, set_flags),
        };
        match instruction {
            ArithmeticInstruction::Sub => {
                let result = rn_val - shifted_operand2;
                self.set_arithmetic_flags(result, rn_val, !shifted_operand2, 1, set_flags);
                result
            }
            ArithmeticInstruction::Rsb => {
                let result = shifted_operand2 - rn_val;
                self.set_arithmetic_flags(result, !rn_val, shifted_operand2, 1, set_flags);
                result
            }
            ArithmeticInstruction::Add => {
                let result = rn_val + shifted_operand2;
                self.set_arithmetic_flags(result, rn_val, shifted_operand2, 0, set_flags);
                result
            }
            ArithmeticInstruction::Adc => {
                let carry = self.get_flag(FlagsRegister::C);
                let result = rn_val + shifted_operand2 + carry;
                self.set_arithmetic_flags(result, rn_val, shifted_operand2, carry, set_flags);
                result
            }

            ArithmeticInstruction::Sbc => {
                let carry = self.get_flag(FlagsRegister::C);
                let result = rn_val - shifted_operand2 + carry - 1;
                self.set_arithmetic_flags(
                    result,
                    rn_val,
                    shifted_operand2.twos_complement(),
                    carry.twos_complement(),
                    set_flags,
                );
                result
            }
            ArithmeticInstruction::Rsc => {
                let carry = self.get_flag(FlagsRegister::C);
                let result = shifted_operand2 - rn_val + carry - 1;
                self.set_arithmetic_flags(
                    result,
                    rn_val.twos_complement(),
                    shifted_operand2,
                    carry.twos_complement(),
                    set_flags,
                );
                result
            }
            ArithmeticInstruction::Cmp => {
                let result = rn_val - shifted_operand2;
                self.set_arithmetic_flags(result, rn_val, !shifted_operand2, 1, true);
                result
            }
            ArithmeticInstruction::Cmn => {
                let result = rn_val + shifted_operand2;
                self.set_arithmetic_flags(result, rn_val, shifted_operand2, 0, true);
                result
            }
        }
    }

    fn execute_logical_instruction(
        &mut self,
        memory: &mut GBAMemory,
        instruction: LogicalInstruction,
        rn: REGISTER,
        operand2: Operand,
        shift: Shift,
        set_flags: bool,
    ) -> u32 {
        let rn_val = self.get_register(rn);
        let shifted_operand2 = match operand2 {
            Operand::Immeidate(imm) => self.execute_immediate_shift(imm, shift),
            Operand::Register(reg) => self.execute_register_shift(memory, reg, shift, set_flags),
        };

        let result = match instruction {
            LogicalInstruction::And => rn_val & shifted_operand2,
            LogicalInstruction::Eor => rn_val ^ shifted_operand2,
            LogicalInstruction::Tst => rn_val & shifted_operand2,
            LogicalInstruction::Teq => rn_val ^ shifted_operand2,
            LogicalInstruction::Orr => rn_val | shifted_operand2,
            LogicalInstruction::Mov => shifted_operand2,
            LogicalInstruction::Bic => rn_val & !shifted_operand2,
            LogicalInstruction::Mvn => !shifted_operand2,
        };
        self.set_logical_flags(result, set_flags);
        result
    }

    fn execute_immediate_shift(&mut self, imm: u32, shift: Shift) -> u32 {
        let Shift(ShiftType::ROR, Operand::Immeidate(rotate_amount)) = shift else {
            panic!("Invalid immediate shift");
        };

        if rotate_amount == 0 {
            self.shifter_output = self.get_flag(FlagsRegister::C);
            imm
        } else {
            self.shifter_output = imm.get_bit((rotate_amount as u8) - 1);
            imm.rotate_right(rotate_amount)
        }
    }

    fn execute_register_shift(
        &mut self,
        memory: &mut GBAMemory,
        operand2_register: REGISTER,
        shift: Shift,
        set_flags: bool,
    ) -> u32 {
        let Shift(shift_type, shift_amount_operand) = shift;
        let shift_amount = match shift_amount_operand {
            Operand::Register(register) => {
                self.advance_pipeline(memory);
                self.get_register(register)
            }
            Operand::Immeidate(imm) => imm,
        } & 0xFF;

        let operand2 = self.get_register(operand2_register);
        // Special cases
        match (shift_type, shift_amount_operand) {
            (ShiftType::LSL, Operand::Immeidate(0)) => {
                self.shifter_output = self.get_flag(FlagsRegister::C);
                return operand2;
            }
            (ShiftType::LSR, Operand::Immeidate(32)) => {
                self.shifter_output = operand2.get_bit(31);
                return 0;
            }
            (ShiftType::ASR, Operand::Immeidate(shift_amount @ 32)) => {
                self.shifter_output = operand2.get_bit(31);
                return ((operand2 as i32) >> shift_amount) as u32;
            }
            _ => {}
        };

        match (shift_type, shift_amount) {
            (ShiftType::LSL, shift_amount) => {
                if shift_amount == 0 {
                    self.shifter_output = self.get_flag(FlagsRegister::C);
                    return operand2;
                } else if shift_amount < 31 {
                    self.shifter_output = operand2.get_bit(32 - shift_amount as u8);
                    return operand2 << shift_amount;
                } else if shift_amount == 32 {
                    self.shifter_output = operand2 & 0x1;
                    return 0;
                } else {
                    self.shifter_output = 0;
                    return 0;
                }
            }
            (ShiftType::LSR, shift_amount) => {
                if shift_amount == 0 {
                    self.shifter_output = self.get_flag(FlagsRegister::C);
                    return operand2;
                } else if shift_amount < 31 {
                    self.shifter_output = operand2.get_bit((shift_amount as u8) - 1);
                    return operand2 >> shift_amount;
                } else if shift_amount == 32 {
                    self.shifter_output = operand2 >> 31;
                    return 0;
                } else {
                    self.shifter_output = 0;
                    return 0;
                }
            }
            (ShiftType::ASR, shift_amount) => {
                if shift_amount == 0 {
                    self.shifter_output = self.get_flag(FlagsRegister::C);
                    return operand2;
                } else if shift_amount < 31 {
                    self.shifter_output = operand2.get_bit((shift_amount as u8) - 1);
                    return ((operand2 as i32) >> shift_amount) as u32;
                } else if (operand2 >> 31) > 0 {
                    self.shifter_output = 1;
                    return u32::MAX;
                } else {
                    self.shifter_output = 0;
                    return 0;
                }
            }
            (ShiftType::ROR, shift_amount) => {
                let rotate_amount = shift_amount & 0x1F;
                if shift_amount == 0 {
                    self.shifter_output = self.get_flag(FlagsRegister::C);
                    return operand2;
                } else if rotate_amount > 0 {
                    self.shifter_output = operand2.get_bit((rotate_amount as u8) - 1);
                    return operand2.rotate_right(rotate_amount);
                } else {
                    self.shifter_output = operand2 >> 31;
                    return operand2;
                }
            }
            (ShiftType::RRX, _) => {
                self.shifter_output = operand2.get_bit(0);
                return operand2 >> 1 | self.get_flag(FlagsRegister::C) << 31;
            }
        }
    }

    pub fn data_processing_instruction(
        &mut self,
        instruction: ARMByteCode,
        memory: &mut GBAMemory,
    ) -> CYCLES {
        let opcode = (instruction & 0x01E0_0000) >> 21;
        let shift_amount;
        let mut cycles = 0;
        if instruction.bit_is_set(25) {
            shift_amount = (instruction & 0x0000_0F00) >> 7;
        } else {
            // The first cycle gets the register we shift by
            // The rest of the operation happens on the next cycle in an I cycle
            if instruction.bit_is_set(4) {
                // shift by register
                cycles += self.advance_pipeline(memory) + 1;
                let shift_register = (instruction & 0x0000_0F00) >> 8;
                shift_amount = self.get_register(shift_register);
            } else {
                shift_amount = (instruction & 0x0000_0F80) >> 7;
            }
        }
        let rn = (0x000F_0000 & instruction) >> 16;
        let rd = (0x0000_F000 & instruction) >> 12;

        let set_flags = instruction.bit_is_set(20) && rd != PC_REGISTER as u32;
        let operand2 = if instruction.bit_is_set(25) {
            // operand 2 is immediate
            let immediate = instruction & 0x0000_00FF;

            let operand2 = immediate.rotate_right(shift_amount);
            if set_flags && operand2 > 255 {
                match opcode {
                    0x0..=0x1 | 0x8..=0x9 | 0xc..=0xf => {
                        self.set_flag_from_bit(FlagsRegister::C, operand2.get_bit(31) as u8)
                    }
                    _ => {}
                }
            }
            operand2
        } else {
            let operand_register = instruction & 0x0000_000F;
            let operand_register_value = self.get_register(operand_register);
            self.decode_shifted_register(
                instruction,
                shift_amount,
                operand_register_value,
                set_flags,
            )
        };

        // Calling within the match branch is faster than getting the function and
        // then dynamically dispatching
        match opcode {
            0x0 => self.arm_and(rd, self.get_register(rn), operand2, set_flags),
            0x1 => self.arm_eor(rd, self.get_register(rn), operand2, set_flags),
            0x2 => self.arm_sub(rd, self.get_register(rn), operand2, set_flags),
            0x3 => self.arm_rsb(rd, self.get_register(rn), operand2, set_flags),
            0x4 => self.arm_add(rd, self.get_register(rn), operand2, set_flags),
            0x5 => self.arm_adc(rd, self.get_register(rn), operand2, set_flags),
            0x6 => self.arm_sbc(rd, self.get_register(rn), operand2, set_flags),
            0x7 => self.arm_rsc(rd, self.get_register(rn), operand2, set_flags),
            0x8 => {
                if instruction.bit_is_set(20) {
                    self.arm_tst(rd, self.get_register(rn), operand2, set_flags)
                } else {
                    return self.arm_mrs(instruction, memory);
                }
            }
            0x9 => {
                if instruction.bit_is_set(20) {
                    self.arm_teq(rd, self.get_register(rn), operand2, set_flags)
                } else {
                    return self.arm_msr(instruction, memory);
                }
            }
            0xa => {
                if instruction.bit_is_set(20) {
                    self.arm_cmp(rd, self.get_register(rn), operand2, set_flags)
                } else {
                    return self.arm_mrs(instruction, memory);
                }
            }
            0xb => {
                if instruction.bit_is_set(20) {
                    self.arm_cmn(rd, self.get_register(rn), operand2, set_flags)
                } else {
                    return self.arm_msr(instruction, memory);
                }
            }
            0xc => self.arm_orr(rd, self.get_register(rn), operand2, set_flags),
            0xd => self.arm_mov(rd, self.get_register(rn), operand2, set_flags),
            0xe => self.arm_bic(rd, self.get_register(rn), operand2, set_flags),
            0xf => self.arm_mvn(rd, self.get_register(rn), operand2, set_flags),
            _ => unreachable!("Impossible to decode opcode"),
        };
        if rd == 15 {
            if instruction.bit_is_set(20) {
                self.pop_spsr();
            }
            cycles += self.flush_pipeline(memory);
        }
        return cycles;
    }

    pub fn arm_add(&mut self, rd: REGISTER, operand1: u32, operand2: u32, set_flags: bool) {
        let result = operand1 + operand2;
        self.set_arithmetic_flags(result, operand1, operand2, 0, set_flags);
        self.set_register(rd, result);
    }

    pub fn arm_and(&mut self, rd: REGISTER, operand1: u32, operand2: u32, set_flags: bool) {
        let result = operand1 & operand2;

        self.set_logical_flags(result, set_flags);
        self.set_register(rd, result);
    }

    pub fn arm_eor(&mut self, rd: REGISTER, operand1: u32, operand2: u32, set_flags: bool) {
        let result = operand1 ^ operand2;

        self.set_logical_flags(result, set_flags);
        self.set_register(rd, result);
    }

    pub fn arm_sub(&mut self, rd: REGISTER, operand1: u32, operand2: u32, set_flags: bool) {
        let result = operand1 - operand2; // use two's complement to make setting flags easier

        self.set_register(rd, result);
        self.set_arithmetic_flags(result, operand1, !operand2, 1, set_flags);
    }

    pub fn arm_rsb(&mut self, rd: REGISTER, operand1: u32, operand2: u32, set_flags: bool) {
        let operand1 = !operand1;
        let result = operand2 + operand1 + 1; // use two's complement to make setting flags easier

        self.set_arithmetic_flags(result, operand1, operand2, 1, set_flags);
        self.set_register(rd, result);
    }

    pub fn arm_adc(&mut self, rd: REGISTER, operand1: u32, operand2: u32, set_flags: bool) {
        let carry = self.get_flag(FlagsRegister::C);
        let result = operand1 + operand2 + carry;

        self.set_arithmetic_flags(result, operand1, operand2, carry, set_flags);
        self.set_register(rd, result);
    }

    pub fn arm_sbc(&mut self, rd: REGISTER, operand1: u32, operand2: u32, set_flags: bool) {
        let carry = self.get_flag(FlagsRegister::C);
        let operand2 = operand2.twos_complement();
        let carry = carry.twos_complement();
        let result = operand1 + operand2 + carry;

        self.set_arithmetic_flags(result, operand1, operand2, carry, set_flags);
        self.set_register(rd, result);
    }

    pub fn arm_rsc(&mut self, rd: REGISTER, _operand1: u32, operand2: u32, set_flags: bool) {
        let carry = self.get_flag(FlagsRegister::C);
        let operand1 = operand2.twos_complement();
        let carry = carry.twos_complement();
        let result = operand1 + operand2 + carry;

        self.set_arithmetic_flags(result, operand1, operand2, carry, set_flags);
        self.set_register(rd, result);
    }

    pub fn arm_tst(&mut self, _rd: REGISTER, operand1: u32, operand2: u32, _set_flags: bool) {
        let result = operand1 & operand2;

        self.set_logical_flags(result, true);
    }

    pub fn arm_teq(&mut self, _rd: REGISTER, operand1: u32, operand2: u32, _set_flags: bool) {
        let result = operand1 ^ operand2;

        self.set_logical_flags(result, true);
    }

    pub fn arm_cmp(&mut self, _rd: REGISTER, operand1: u32, operand2: u32, _set_flags: bool) {
        let operand2 = !operand2;
        let result = operand1 + operand2 + 1; // use two's complement to make setting flags easier

        self.set_arithmetic_flags(result, operand1, operand2, 1, true);
        self.set_executed_instruction(format_args!("CMP {:#X} {:#X}", operand1, operand2));
    }

    pub fn arm_cmn(&mut self, _rd: REGISTER, operand1: u32, operand2: u32, _set_flags: bool) {
        let result = operand1 + operand2;
        self.set_arithmetic_flags(result, operand1, operand2, 0, true);
    }

    pub fn arm_orr(&mut self, rd: REGISTER, operand1: u32, operand2: u32, set_flags: bool) {
        let result = operand1 | operand2;

        self.set_logical_flags(result, set_flags);
        self.set_register(rd, result);
    }

    pub fn arm_mov(&mut self, rd: REGISTER, _operand1: u32, operand2: u32, set_flags: bool) {
        self.set_register(rd, operand2);
        self.set_logical_flags(operand2, set_flags);
    }

    pub fn arm_bic(&mut self, rd: REGISTER, operand1: u32, operand2: u32, set_flags: bool) {
        let result = operand1 & !operand2;

        self.set_logical_flags(result, set_flags);
        self.set_register(rd, result);
    }

    pub fn arm_mvn(&mut self, rd: REGISTER, _operand1: u32, operand2: u32, set_flags: bool) {
        let result = !operand2;
        self.set_register(rd, result);
        self.set_logical_flags(result, set_flags);
    }

    pub fn arm_mrs(&mut self, instruction: ARMByteCode, _memory: &mut GBAMemory) -> CYCLES {
        let rd = (instruction & 0x0000_F000) >> 12;
        let source_psr = if instruction.bit_is_set(22) {
            match self.get_current_spsr() {
                Some(spsr) => *spsr,
                None => {
                    return 1;
                }
            }
        } else {
            self.get_cpsr()
        };

        self.set_register(rd, source_psr.into());
        let psr = if instruction.bit_is_set(22) {
            "SPSR"
        } else {
            "CPSR"
        };

        self.set_executed_instruction(format_args!("MRS {} {}", rd, psr));
        1
    }

    pub fn arm_msr(&mut self, instruction: ARMByteCode, memory: &mut GBAMemory) -> CYCLES {
        const FLG_MASK: u32 = 0xFF00_0000;
        const CTL_MASK: u32 = 0x0000_00DF; // can't assign T-bit with this operation
        let current_cpu_mode = self.get_cpu_mode();

        let operand = if instruction.bit_is_set(25) {
            // lower 8 bits rotated right by bits instruction[11:8] * 2
            (instruction & 0x0000_00FF).rotate_right((instruction & 0x0000_0F00) >> 7)
        } else {
            self.get_register(instruction & 0x0000_000F)
        };

        let mut destination_psr: u32 = if instruction.bit_is_set(22) {
            match self.get_current_spsr() {
                Some(spsr) => (*spsr).into(),
                None => {
                    return 0;
                }
            }
        } else {
            self.get_cpsr().into()
        };

        if instruction.bit_is_set(19) {
            destination_psr &= !FLG_MASK;
            destination_psr |= operand & FLG_MASK;
        }

        if instruction.bit_is_set(16) && !matches!(current_cpu_mode, CPUMode::USER) {
            destination_psr &= !CTL_MASK;
            destination_psr |= operand & CTL_MASK;
        }

        if instruction.bit_is_set(22) {
            let Some(spsr) = self.get_current_spsr() else {
                return 0;
            };
            *spsr = destination_psr.into();
        } else {
            self.set_cpsr(destination_psr.into());
        };

        let updated_psr = if instruction.bit_is_set(22) {
            "SPSR"
        } else {
            "CPSR"
        };

        self.set_executed_instruction(format_args!("MSR {} {:#X}", updated_psr, operand));

        0
    }

    pub fn set_logical_flags(&mut self, result: WORD, set_flags: bool) {
        if set_flags == false {
            return;
        }
        if result.get_bit(31) > 0 {
            self.set_flag(FlagsRegister::N);
        } else {
            self.reset_flag(FlagsRegister::N);
        }
        if result == 0 {
            self.set_flag(FlagsRegister::Z);
        } else {
            self.reset_flag(FlagsRegister::Z);
        }
        if self.shifter_output > 0 {
            self.set_flag(FlagsRegister::C);
        } else {
            self.reset_flag(FlagsRegister::C);
        }
    }

    pub fn set_arithmetic_flags(
        &mut self,
        result: WORD,
        operand1: u32,
        operand2: u32,
        carry: u32,
        set_flags: bool,
    ) {
        if set_flags == false {
            return;
        }
        let result_sign = result.get_bit(31);
        let operand2_sign = operand2.get_bit(31);
        if result_sign > 0 {
            self.set_flag(FlagsRegister::N);
        } else {
            self.reset_flag(FlagsRegister::N);
        }
        if result == 0 {
            self.set_flag(FlagsRegister::Z);
        } else {
            self.reset_flag(FlagsRegister::Z);
        }
        if operand1.get_bit(31) == operand2_sign && result_sign != operand2_sign {
            self.set_flag(FlagsRegister::V);
        } else {
            self.reset_flag(FlagsRegister::V);
        }
        let complete_add: u64 = operand1 as u64 + operand2 as u64 + carry as u64;

        if result as u64 == complete_add {
            self.reset_flag(FlagsRegister::C);
        } else {
            self.set_flag(FlagsRegister::C);
        }
    }
}

fn get_operand2_and_shift(instruction: u32) -> (Operand, Shift) {
    if instruction.bit_is_set(25) {
        let shift_amount = (instruction & 0x0000_0F00) >> 7;
        return (
            Operand::Immeidate(instruction & 0x0000_00FF),
            Shift(ShiftType::ROR, Operand::Immeidate(shift_amount)),
        );
    } else {
        let operand2 = Operand::Register(instruction & 0x0000_000F);
        let shift_type = (instruction & 0x0000_0060) >> 5;
        let shift_amount = if instruction.bit_is_set(4) {
            Operand::Register((instruction & 0x0000_0F00) >> 8)
        } else {
            let immediate = (instruction & 0x0000_0F80) >> 7;
            if immediate == 0 {
                // Special Cases
                return (
                    operand2,
                    match shift_type {
                        0x0 => Shift(ShiftType::LSL, Operand::Immeidate(0)),
                        0x1 => Shift(ShiftType::LSR, Operand::Immeidate(32)),
                        0x2 => Shift(ShiftType::ASR, Operand::Immeidate(32)),
                        0x3 => Shift(ShiftType::RRX, Operand::Immeidate(1)),
                        _ => unreachable!(),
                    },
                );
            }
            Operand::Immeidate(immediate)
        };
        let shift_type = match shift_type {
            0x0 => ShiftType::LSL,
            0x1 => ShiftType::LSR,
            0x2 => ShiftType::ASR,
            0x3 => ShiftType::ROR,
            _ => unreachable!(),
        };
        return (operand2, Shift(shift_type, shift_amount));
    };
}

#[cfg(test)]
mod enum_data_processing_instruction_tests {
    use rstest::rstest;

    use crate::arm7tdmi::arm::alu::{
        ArithmeticInstruction, DataProcessingInstruction, LogicalInstruction, Shift, ShiftType,
    };
    use crate::arm7tdmi::cpsr::PSR;
    use crate::arm7tdmi::cpu::CPU;
    use crate::arm7tdmi::instruction_table::Operand;
    use crate::gba::GBA;
    use crate::{
        arm7tdmi::cpu::{CPUMode, FlagsRegister},
        types::REGISTER,
    };

    #[rstest]
    #[case(
        0xe0931002, // adds r1, r3, r2
        DataProcessingInstruction::Arithmetic(
            ArithmeticInstruction::Add,
            Some(1),
            3,
            Operand::Register(2),
            Shift(ShiftType::LSL, Operand::Immeidate(0)),
            true
        )
    )]
    #[case(
        0xe01312a2, // ands r1, r3, r2 LSR #5
        DataProcessingInstruction::Logical(
            LogicalInstruction::And,
            Some(1),
            3,
            Operand::Register(2),
            Shift(ShiftType::LSR, Operand::Immeidate(5)),
            true
        )
    )]
    #[case(
        0xe1931002, // orrs r1, r3, r2
        DataProcessingInstruction::Logical(
            LogicalInstruction::Orr,
            Some(1),
            3,
            Operand::Register(2),
            Shift(ShiftType::LSL, Operand::Immeidate(0)),
            true
        )
    )]
    #[case(
        0xe1831002, // orr r1, r3, r2
        DataProcessingInstruction::Logical(
            LogicalInstruction::Orr,
            Some(1),
            3,
            Operand::Register(2),
            Shift(ShiftType::LSL, Operand::Immeidate(0)),
            false
        )
    )]
    #[case(
        0xe0331002, // eors r1, r3, r2
        DataProcessingInstruction::Logical(
            LogicalInstruction::Eor,
            Some(1),
            3,
            Operand::Register(2),
            Shift(ShiftType::LSL, Operand::Immeidate(0)),
            true
        )
    )]
    fn able_to_decode_data_processing_instructions(
        #[case] opcode: u32,
        #[case] expected_instruction: DataProcessingInstruction,
    ) {
        let decoded_instruction = CPU::decode_data_processing_instruction(opcode);

        assert!(matches!(decoded_instruction, expected_instruction))
    }

    #[test]
    fn able_to_execute_data_processing_instruction() {
        let mut gba = GBA::new_no_bios();

        gba.cpu.set_register(3, 5);
        gba.cpu.set_register(2, 5);
        gba.cpu.execute_data_processing_instruction(
            &mut gba.memory,
            DataProcessingInstruction::Arithmetic(
                ArithmeticInstruction::Add,
                Some(1),
                3,
                Operand::Register(2),
                Shift(ShiftType::LSL, Operand::Immeidate(0)),
                true,
            ),
        );

        assert_eq!(gba.cpu.get_register(1), 10);
    }
}

#[cfg(test)]
mod tests {

    use rstest::rstest;

    use crate::arm7tdmi::cpsr::PSR;
    use crate::gba::GBA;
    use crate::{
        arm7tdmi::cpu::{CPUMode, FlagsRegister},
        types::REGISTER,
    };

    #[test]
    fn add_instruction_should_set_carry_flag() {
        let mut gba = GBA::new_no_bios();

        gba.cpu.set_register(2, u32::MAX);
        gba.cpu.set_register(3, 2);

        gba.cpu.prefetch[0] = Some(0xe0931002); // adds r1, r3, r2;

        gba.step();
        gba.step();
        assert!(gba.cpu.get_register(1) == 1);
        assert!(gba.cpu.get_flag(FlagsRegister::C) == 1);
        assert!(gba.cpu.get_flag(FlagsRegister::N) == 0);
        assert!(gba.cpu.get_flag(FlagsRegister::Z) == 0);
        assert!(gba.cpu.get_flag(FlagsRegister::V) == 0);
    }

    #[test]
    fn add_instruction_should_set_overflow_and_carry_flags() {
        let mut gba = GBA::new_no_bios();

        gba.cpu.set_register(2, 0x8000_0000);
        gba.cpu.set_register(3, 0x8000_0000);

        gba.cpu.prefetch[0] = Some(0xe0931002); // adds r1, r3, r2;

        gba.step();
        gba.step();
        assert!(gba.cpu.get_register(1) == 0);
        assert!(gba.cpu.get_flag(FlagsRegister::C) == 1);
        assert!(gba.cpu.get_flag(FlagsRegister::N) == 0);
        assert!(gba.cpu.get_flag(FlagsRegister::Z) == 1);
        assert!(gba.cpu.get_flag(FlagsRegister::V) == 1);
    }

    #[test]
    fn add_instruction_should_set_n_flag() {
        let mut gba = GBA::new_no_bios();

        gba.cpu.set_register(2, 0x8000_0000);
        gba.cpu.set_register(3, 0x0000_0001);

        gba.cpu.prefetch[0] = Some(0xe0931002); // adds r1, r3, r2;

        gba.step();
        gba.step();
        assert!(gba.cpu.get_register(1) == 0x8000_0001);
        assert!(gba.cpu.get_flag(FlagsRegister::C) == 0);
        assert!(gba.cpu.get_flag(FlagsRegister::N) == 1);
        assert!(gba.cpu.get_flag(FlagsRegister::Z) == 0);
        assert!(gba.cpu.get_flag(FlagsRegister::V) == 0);
    }

    #[test]
    fn and_instruction_should_set_c_flag() {
        let mut gba = GBA::new_no_bios();

        gba.cpu.set_register(2, 0x0000_FFFF);
        gba.cpu.set_register(3, 0x0000_0001);

        gba.cpu.prefetch[0] = Some(0xe01312a2); // ands r1, r3, r2 LSR 5;

        gba.step();
        gba.step();
        assert!(gba.cpu.get_flag(FlagsRegister::C) == 1);
        assert!(gba.cpu.get_flag(FlagsRegister::N) == 0);
        assert!(gba.cpu.get_flag(FlagsRegister::Z) == 0);
        assert!(gba.cpu.get_flag(FlagsRegister::V) == 0);
        assert!(gba.cpu.get_register(1) == 0x0000_0001);
    }

    #[test]
    fn and_instruction_should_set_n_flag() {
        let mut gba = GBA::new_no_bios();

        gba.cpu.set_register(2, 0x8000_FFFF);
        gba.cpu.set_register(3, 0x8000_0001);

        gba.cpu.prefetch[0] = Some(0xe0131002); // ands r1, r3, r2;

        gba.step();
        gba.step();
        assert!(gba.cpu.get_flag(FlagsRegister::C) == 0);
        assert!(gba.cpu.get_flag(FlagsRegister::N) == 1);
        assert!(gba.cpu.get_flag(FlagsRegister::Z) == 0);
        assert!(gba.cpu.get_flag(FlagsRegister::V) == 0);
        assert!(gba.cpu.get_register(1) == 0x8000_0001);
    }

    #[test]
    fn and_instruction_should_set_z_flag() {
        let mut gba = GBA::new_no_bios();

        gba.cpu.set_register(2, 0x8000_FFFF);
        gba.cpu.set_register(3, 0x0000_0000);

        gba.cpu.prefetch[0] = Some(0xe0131002); // ands r1, r3, r2;

        gba.step();
        gba.step();
        assert!(gba.cpu.get_flag(FlagsRegister::C) == 0);
        assert!(gba.cpu.get_flag(FlagsRegister::N) == 0);
        assert!(gba.cpu.get_flag(FlagsRegister::Z) == 1);
        assert!(gba.cpu.get_flag(FlagsRegister::V) == 0);
        assert!(gba.cpu.get_register(1) == 0x0000_0000);
    }

    #[test]
    fn orr_instruction_should_set_z_flag() {
        let mut gba = GBA::new_no_bios();

        gba.cpu.set_register(2, 0x0000_0000);
        gba.cpu.set_register(3, 0x0000_0000);

        gba.cpu.prefetch[0] = Some(0xe1931002); // orrs r1, r3, r2;

        gba.step();
        gba.step();
        assert!(gba.cpu.get_flag(FlagsRegister::C) == 0);
        assert!(gba.cpu.get_flag(FlagsRegister::N) == 0);
        assert!(gba.cpu.get_flag(FlagsRegister::Z) == 1);
        assert!(gba.cpu.get_flag(FlagsRegister::V) == 0);
        assert!(gba.cpu.get_register(1) == 0x0000_0000);
    }

    #[test]
    fn orr_instruction_should_not_set_any_flags() {
        let mut gba = GBA::new_no_bios();

        gba.cpu.set_register(2, 0x0000_0000);
        gba.cpu.set_register(3, 0x0000_0000);

        gba.cpu.prefetch[0] = Some(0xe1831002); // orr r1, r3, r2;

        gba.step();
        gba.step();
        assert!(gba.cpu.get_flag(FlagsRegister::C) == 0);
        assert!(gba.cpu.get_flag(FlagsRegister::N) == 0);
        assert!(gba.cpu.get_flag(FlagsRegister::Z) == 0);
        assert!(gba.cpu.get_flag(FlagsRegister::V) == 0);
        assert!(gba.cpu.get_register(1) == 0x0000_0000);
    }

    #[test]
    fn eor_instruction_should_set_n_flag() {
        let mut gba = GBA::new_no_bios();

        gba.cpu.set_register(2, 0x8001_0002);
        gba.cpu.set_register(3, 0x1000_0010);

        gba.cpu.prefetch[0] = Some(0xe0331002); // eors r1, r3, r2;

        gba.step();
        gba.step();
        assert!(gba.cpu.get_flag(FlagsRegister::C) == 0);
        assert!(gba.cpu.get_flag(FlagsRegister::N) == 1);
        assert!(gba.cpu.get_flag(FlagsRegister::Z) == 0);
        assert!(gba.cpu.get_flag(FlagsRegister::V) == 0);
        assert!(gba.cpu.get_register(1) == 0x9001_0012);
    }

    #[test]
    fn teq_instruction_should_set_n_flag() {
        let mut gba = GBA::new_no_bios();

        gba.cpu.set_register(2, 0x8001_0002);
        gba.cpu.set_register(3, 0x1000_0010);

        gba.cpu.prefetch[0] = Some(0xe1330002); // teq r3, r2;

        gba.step();
        gba.step();
        assert!(gba.cpu.get_flag(FlagsRegister::C) == 0);
        assert!(gba.cpu.get_flag(FlagsRegister::N) == 1);
        assert!(gba.cpu.get_flag(FlagsRegister::Z) == 0);
        assert!(gba.cpu.get_flag(FlagsRegister::V) == 0);
    }

    #[test]
    fn teq_instruction_should_set_z_flag_when_equal() {
        let mut gba = GBA::new_no_bios();

        gba.cpu.set_register(2, 0x8001_0002);
        gba.cpu.set_register(3, 0x8001_0002);

        gba.cpu.prefetch[0] = Some(0xe1330002); // teq r3, r2;

        gba.step();
        gba.step();
        assert!(gba.cpu.get_flag(FlagsRegister::C) == 0);
        assert!(gba.cpu.get_flag(FlagsRegister::N) == 0);
        assert!(gba.cpu.get_flag(FlagsRegister::Z) == 1);
        assert!(gba.cpu.get_flag(FlagsRegister::V) == 0);
    }

    #[test]
    fn tst_instruction_should_set_z_flag_when_no_bits_match() {
        let mut gba = GBA::new_no_bios();

        gba.cpu.set_register(2, 0x8001_0002);
        gba.cpu.set_register(3, 0x0110_2224);

        gba.cpu.prefetch[0] = Some(0xe1130002); // tst r3, r2;

        gba.step();
        gba.step();
        assert!(gba.cpu.get_flag(FlagsRegister::C) == 0);
        assert!(gba.cpu.get_flag(FlagsRegister::N) == 0);
        assert!(gba.cpu.get_flag(FlagsRegister::Z) == 1);
        assert!(gba.cpu.get_flag(FlagsRegister::V) == 0);
    }

    #[test]
    fn bic_instruction_should_reset_all_bits() {
        let mut gba = GBA::new_no_bios();

        gba.cpu.set_register(3, 0x8001_0002);
        gba.cpu.set_register(2, 0x80F1_0102);

        gba.cpu.prefetch[0] = Some(0xe1d31002); // bics r1, r3, r2;

        gba.step();
        gba.step();
        assert!(gba.cpu.get_flag(FlagsRegister::C) == 0);
        assert!(gba.cpu.get_flag(FlagsRegister::N) == 0);
        assert!(gba.cpu.get_flag(FlagsRegister::Z) == 1);
        assert!(gba.cpu.get_flag(FlagsRegister::V) == 0);
        assert!(gba.cpu.get_register(1) == 0x00);
    }

    #[test]
    fn data_processing_with_pc_as_operand2_and_register_shift_delays_pc() {
        let mut gba = GBA::new_no_bios();

        gba.cpu.prefetch[0] = Some(0xe094131f); // adds r1, r3, r15, LSL r3; pc = 0

        gba.cpu.set_register(3, 0x01);
        let test_pc = 4; // points at next instruction
        gba.cpu.set_pc(test_pc);

        gba.step(); // pc == 8
        gba.step(); // pc == 12
        gba.step(); // pc == 16
        assert!(gba.cpu.get_register(1) == (test_pc + 8) << 1);
        assert!(gba.cpu.get_flag(FlagsRegister::C) == 0);
        assert!(gba.cpu.get_flag(FlagsRegister::N) == 0);
        assert!(gba.cpu.get_flag(FlagsRegister::Z) == 0);
        assert!(gba.cpu.get_flag(FlagsRegister::V) == 0);
    }

    #[test]
    fn data_processing_with_pc_as_operand1_and_register_shift_delays_pc() {
        let mut gba = GBA::new_no_bios();

        gba.cpu.prefetch[0] = Some(0xe09f1314); //  adds r1, pc, r4, lsl r3; pc = 0

        gba.cpu.set_register(3, 0x01);
        gba.cpu.set_register(4, 0);
        let test_pc = 4; // points at next instruction
        gba.cpu.set_pc(test_pc);

        gba.step();
        gba.step();
        gba.step();
        assert!(gba.cpu.get_register(1) == test_pc + 8);
        assert!(gba.cpu.get_flag(FlagsRegister::C) == 0);
        assert!(gba.cpu.get_flag(FlagsRegister::N) == 0);
        assert!(gba.cpu.get_flag(FlagsRegister::Z) == 0);
        assert!(gba.cpu.get_flag(FlagsRegister::V) == 0);
    }

    #[test]
    fn data_processing_with_pc_as_destination_should_start_from_result() {
        let mut gba = GBA::new_no_bios();

        let _res = gba.memory.writeu32(0x3000000, 0xe25f1008);
        let _res = gba.memory.writeu32(0x3000004, 0xe1a00000);
        let _res = gba.memory.writeu32(0x3000008, 0xe1a00000); // nop
        let _res = gba.memory.writeu32(0x300000C, 0xe1a00000); // nop
        let _res = gba.memory.writeu32(0x3000010, 0xe1a00000); // nop
        let _res = gba.memory.writeu32(0x3000014, 0xe281f000);

        gba.cpu.set_pc(0x3000000);
        gba.step();
        gba.step();
        gba.step();
        gba.step();
        gba.step();
        gba.step();
        gba.step();
        gba.step();
        assert_eq!(
            gba.cpu
                .decode_instruction(gba.cpu.prefetch[1].unwrap())
                .instruction,
            0xe25f1008
        );
    }

    #[test]
    fn mov_instruction_should_set_n_flag() {
        let mut gba = GBA::new_no_bios();

        gba.cpu.set_register(3, 0x8001_0002);

        gba.cpu.prefetch[0] = Some(0xe1b04003); // mov r4, r3;

        gba.step();
        gba.step();
        assert!(gba.cpu.get_flag(FlagsRegister::C) == 0);
        assert!(gba.cpu.get_flag(FlagsRegister::N) == 1);
        assert!(gba.cpu.get_flag(FlagsRegister::Z) == 0);
        assert!(gba.cpu.get_flag(FlagsRegister::V) == 0);
        assert!(gba.cpu.get_register(4) == gba.cpu.get_register(3));
        assert!(gba.cpu.get_register(4) == 0x8001_0002);
    }

    #[test]
    fn mvn_instruction_should_set_z_flag() {
        let mut gba = GBA::new_no_bios();

        let input = 0xFFFF_FFFF;
        gba.cpu.set_register(4, input);

        gba.cpu.prefetch[0] = Some(0xe1f05004); // mvn r5, r4;

        gba.step();
        gba.step();
        assert!(gba.cpu.get_flag(FlagsRegister::C) == 0);
        assert!(gba.cpu.get_flag(FlagsRegister::N) == 0);
        assert!(gba.cpu.get_flag(FlagsRegister::Z) == 1);
        assert!(gba.cpu.get_flag(FlagsRegister::V) == 0);
        assert!(gba.cpu.get_register(5) == !gba.cpu.get_register(4));
        assert!(gba.cpu.get_register(5) == !input);
    }

    #[test]
    fn adc_instruction_should_add_2_registers_and_carry() {
        let mut gba = GBA::new_no_bios();

        gba.cpu.set_register(1, 25);
        gba.cpu.set_register(2, 32);
        gba.cpu.set_flag(FlagsRegister::C);

        gba.cpu.prefetch[0] = Some(0xe0b14002); // adcs r4, r2, r1;

        gba.step();
        gba.step();
        assert!(gba.cpu.get_flag(FlagsRegister::C) == 0);
        assert!(gba.cpu.get_flag(FlagsRegister::N) == 0);
        assert!(gba.cpu.get_flag(FlagsRegister::Z) == 0);
        assert!(gba.cpu.get_flag(FlagsRegister::V) == 0);
        assert!(gba.cpu.get_register(4) == 58);
    }

    #[test]
    fn adc_instruction_should_set_carry_register() {
        let mut gba = GBA::new_no_bios();

        gba.cpu.set_register(1, 0xFFFF_FFFF);
        gba.cpu.set_register(2, 0x0);
        gba.cpu.set_flag(FlagsRegister::C);

        gba.cpu.prefetch[0] = Some(0xe0b14002); // adcs r4, r2, r1;

        gba.step();
        gba.step();
        assert!(gba.cpu.get_flag(FlagsRegister::C) == 1);
        assert!(gba.cpu.get_flag(FlagsRegister::N) == 0);
        assert!(gba.cpu.get_flag(FlagsRegister::Z) == 1);
        assert!(gba.cpu.get_flag(FlagsRegister::V) == 0);
        assert!(gba.cpu.get_register(4) == 0x0000_0000);
    }

    #[test]
    fn adc_instruction_should_set_v_register() {
        let mut gba = GBA::new_no_bios();

        gba.cpu.set_register(1, 0x8000_0000);
        gba.cpu.set_register(2, 0x8FFF_FFFF);
        gba.cpu.set_flag(FlagsRegister::C);

        gba.cpu.prefetch[0] = Some(0xe0b14002); // adcs r4, r2, r1;

        gba.step();
        gba.step();
        assert!(gba.cpu.get_flag(FlagsRegister::C) == 1);
        assert!(gba.cpu.get_flag(FlagsRegister::N) == 0);
        assert!(gba.cpu.get_flag(FlagsRegister::Z) == 0);
        assert!(gba.cpu.get_flag(FlagsRegister::V) == 1);
        assert!(gba.cpu.get_register(4) == 0x1000_0000);
    }

    #[test]
    fn sub_instruction_should_set_v_flag() {
        let mut gba = GBA::new_no_bios();

        gba.cpu.set_register(1, 0x7FFF_FFFF);
        gba.cpu.set_register(2, 0xFFFF_FFFF); // twos complement of -1

        gba.cpu.prefetch[0] = Some(0xe0514002); // subs r4, r1, r2;

        gba.step();
        gba.step();
        assert_eq!(gba.cpu.get_flag(FlagsRegister::C), 0);
        assert_eq!(gba.cpu.get_flag(FlagsRegister::N), 1);
        assert_eq!(gba.cpu.get_flag(FlagsRegister::Z), 0);
        assert_eq!(gba.cpu.get_flag(FlagsRegister::V), 1);
        assert_eq!(gba.cpu.get_register(4), 0x8000_0000);
    }

    #[test]
    fn sub_instruction_should_reset_c_flag() {
        let mut gba = GBA::new_no_bios();

        gba.cpu.set_register(1, 5);
        gba.cpu.set_register(2, 10);

        gba.cpu.prefetch[1] = Some(0xe0514002); // subs r4, r1, r2;

        gba.step();
        assert!(gba.cpu.get_register(4) == 0xFFFF_FFFB);
        assert!(gba.cpu.get_flag(FlagsRegister::C) == 0);
        assert!(gba.cpu.get_flag(FlagsRegister::N) == 1);
        assert!(gba.cpu.get_flag(FlagsRegister::Z) == 0);
        assert!(gba.cpu.get_flag(FlagsRegister::V) == 0);
    }

    #[test]
    fn sub_instruction_should_set_c_flag() {
        let mut gba = GBA::new_no_bios();

        gba.cpu.set_register(1, 10);
        gba.cpu.set_register(2, 5);

        gba.cpu.prefetch[0] = Some(0xe0514002); // subs r4, r1, r2;

        gba.step();
        gba.step();
        assert!(gba.cpu.get_register(4) == 0x5);
        assert!(gba.cpu.get_flag(FlagsRegister::C) == 1);
        assert!(gba.cpu.get_flag(FlagsRegister::N) == 0);
        assert!(gba.cpu.get_flag(FlagsRegister::Z) == 0);
        assert!(gba.cpu.get_flag(FlagsRegister::V) == 0);
    }

    #[rstest]
    #[case(0xe10f2000, 0x000000d3, 2, 0x000000d3)]
    #[case(0xe10f2000, 0x300000d3, 2, 0x300000d3)]
    fn mrs_should_move_instruction_from_psr_to_destination_reg(
        #[case] opcode: u32,
        #[case] cpsr: u32,
        #[case] expected_dst: REGISTER,
        #[case] expected_val: u32,
    ) {
        let mut gba = GBA::new_no_bios();

        gba.cpu.set_cpsr(cpsr.into());

        gba.cpu.prefetch[0] = Some(opcode);
        gba.step();
        gba.step();

        assert_eq!(gba.cpu.get_register(expected_dst), expected_val);
    }

    #[rstest]
    #[case(0xe129f002, CPUMode::SVC, 0x000000d3, 2, 0x000000d3)] //msr CPSR_fc, r2
    #[case(0xe129f002, CPUMode::SVC, 0x00FFFFd3, 2, 0x000000d3)] //msr CPSR_fc, r2
    #[case(0xe129f002, CPUMode::SVC, 0xf0FFFFf3, 2, 0xf00000d3)] //msr CPSR_fc, r2
    //thumb bit should not get used
    #[case(0xe121f002, CPUMode::SVC, 0xF0FFFFd3, 2, 0x000000d3)] //msr CPSR_c, r2
    #[case(0xe128f002, CPUMode::SVC, 0xF0FFFFFF, 2, 0xF00000d3)] //msr CPSR_f, r2
    #[case(0xe129f002, CPUMode::USER, 0xF0FFFFd3, 2, 0xF00000d0)] //msr CPSR_fc, r2
                                                                  // shouldn't set C bits
    fn msr_should_move_psr_from_register_to_cpsr(
        #[case] opcode: u32,
        #[case] mode: CPUMode,
        #[case] psr_val: u32,
        #[case] register: u32,
        #[case] expected_val: u32,
    ) {
        let mut gba = GBA::new_no_bios();

        gba.cpu.set_mode(mode);
        gba.cpu.set_register(register, psr_val);

        gba.cpu.prefetch[0] = Some(opcode);
        gba.step();
        gba.step();

        assert_eq!(gba.cpu.get_cpsr(), expected_val.into());
    }

    #[rstest]
    #[case(0xe169f002, CPUMode::SVC, 0x000000df, 2, 0x000000df)] // msr SPSR r2
    #[case(0xe169f002, CPUMode::SVC, 0x000000df, 2, 0x000000df)]
    #[case(0xe169f002, CPUMode::ABT, 0xF0FFFFdf, 2, 0xf00000df)]
    fn msr_should_move_psr_from_register_to_spsr(
        #[case] opcode: u32,
        #[case] mode: CPUMode,
        #[case] psr_val: u32,
        #[case] register: u32,
        #[case] expected_val: u32,
    ) {
        let mut gba = GBA::new_no_bios();

        gba.cpu.set_mode(mode);
        gba.cpu.set_register(register, psr_val);

        gba.cpu.prefetch[0] = Some(opcode);
        gba.step();
        gba.step();

        assert_eq!(*gba.cpu.get_current_spsr().unwrap(), expected_val.into());
    }

    #[rstest]
    #[case(0xe329f0d0, CPUMode::SVC, 0x000000d0)] // msr CPSR, 0x24
    #[case(0xe328f20d, CPUMode::SVC, 0xd00000d3)] // msr CPSR, 0xd0000000
    fn msr_should_move_imm_to_cpsr(
        #[case] opcode: u32,
        #[case] mode: CPUMode,
        #[case] expected_val: u32,
    ) {
        use crate::gba::GBA;

        let mut gba = GBA::new_no_bios();

        gba.cpu.set_mode(mode);

        gba.cpu.prefetch[0] = Some(opcode);
        gba.step();
        gba.step();

        assert_eq!(gba.cpu.get_cpsr(), expected_val.into());
    }

    #[rstest]
    #[case(0xe14f0000, CPUMode::IRQ, 0x000000d0)] // mrs r0, SPSR
    fn mrs_should_move_spsr_to_reg(
        #[case] opcode: u32,
        #[case] mode: CPUMode,
        #[case] expected_val: u32,
    ) {
        use crate::gba::GBA;

        let mut gba = GBA::new_no_bios();

        gba.cpu.spsr[3] = PSR::from(expected_val);
        gba.cpu.set_mode(mode);

        gba.cpu.prefetch[0] = Some(opcode);
        gba.step();
        gba.step();

        assert_eq!(gba.cpu.get_register(0), expected_val.into());
    }
}
