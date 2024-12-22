use crate::{
    arm7tdmi::cpu::{FlagsRegister, InstructionMode, CPU, PC_REGISTER},
    types::{CYCLES, REGISTER},
    utils::bits::{sign_extend, Bits},
};

impl CPU {
    pub fn thumb_move_shifted_register_instruction(&mut self, instruction: u32) -> CYCLES {
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
        1
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

        self.set_executed_instruction(format!("LSL {rd} {:#x} {:#x}", rs_val, offset));
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
        self.set_executed_instruction(format!("LSR {rd} {:#x} {:#x}", rs_val, offset));
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
        self.set_executed_instruction(format!("ASR {rd} {:#x} {:#x}", rs_val, offset));
    }

    pub fn thumb_add_or_subtract_instruction(&mut self, instruction: u32) -> CYCLES {
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
        1
    }

    pub fn thumb_move_add_compare_add_subtract_immediate(&mut self, instruction: u32) -> CYCLES {
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

        1
    }

    fn thumb_move_imm(&mut self, rd: REGISTER, imm: u8) {
        self.set_flag_from_bit(FlagsRegister::N, imm.get_bit(7));
        if imm == 0 {
            self.set_flag(FlagsRegister::Z);
        } else {
            self.reset_flag(FlagsRegister::Z);
        }
        self.set_register(rd, imm.into());
        self.set_executed_instruction(format!("MOV {} {:#x}", rd, imm));
    }

    fn thumb_cmp_imm(&mut self, rd: REGISTER, imm: u8) {
        let minuend = self.get_register(rd);
        let imm: u32 = sign_extend(imm as u32, 7).twos_complement();
        let result = minuend + imm;
        self.set_arithmetic_flags(result, minuend, imm, 0, true);
        self.set_executed_instruction(format!("CMP {} {:#x}", rd, imm));
    }

    fn thumb_add_imm(&mut self, rd: REGISTER, imm: u8) {
        let addend1 = self.get_register(rd);
        let result = addend1 + imm as u32;
        self.set_arithmetic_flags(result, addend1, imm as u32, 0, true);
        self.set_register(rd, result);
        self.set_executed_instruction(format!("ADD {} {:#x}", rd, imm));
    }

    fn thumb_sub_imm(&mut self, rd: REGISTER, imm: u8) {
        let minuend = self.get_register(rd);
        let imm: u32 = sign_extend(imm as u32, 7).twos_complement();
        let result = minuend + imm;
        self.set_arithmetic_flags(result, minuend, imm, 0, true);
        self.set_register(rd, result);
        self.set_executed_instruction(format!("SUB {} {:#x}", rd, imm));
    }

    pub fn thumb_alu_instructions(&mut self, instruction: u32) -> CYCLES {
        let opcode = (instruction & 0x03C0) >> 6;

        let rd = instruction & 0x0007;
        let rs = (instruction & 0x0038) >> 3;

        let operation = match opcode {
            0x0 => CPU::arm_and,
            0x1 => CPU::arm_eor,
            0x2 => CPU::thumb_lsl,
            0x3 => CPU::thumb_lsr_register,
            0x4 => CPU::thumb_asr_register,
            0x5 => CPU::arm_adc,
            0x6 => CPU::arm_sbc,
            0x7 => CPU::thumb_ror,
            0x8 => CPU::arm_tst,
            0x9 => CPU::thumb_neg,
            0xA => CPU::arm_cmp,
            0xB => CPU::arm_cmn,
            0xC => CPU::arm_orr,
            0xD => CPU::thumb_mul,
            0xE => CPU::arm_bic,
            0xF => CPU::arm_mvn,
            _ => panic!("Unimplemented operation"),
        };

        operation(self, rd, self.get_register(rd), self.get_register(rs), true);

        1
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
        self.arm_rsb(rd, 0, operand2, set_flags);
    }

    fn thumb_mul(&mut self, rd: REGISTER, operand1: u32, operand2: u32, set_flags: bool) {
        let result = operand1 * operand2;
        if set_flags {
            self.set_flag_from_bit(FlagsRegister::N, result.get_bit(31) as u8);
            if result == 0 {
                self.set_flag(FlagsRegister::Z);
            } else {
                self.reset_flag(FlagsRegister::Z);
            }
            self.reset_flag(FlagsRegister::N);
        }
        self.set_register(rd, result);
    }

    pub fn thumb_hi_reg_operations(&mut self, instruction: u32) -> CYCLES {
        let mut cycles = 1;
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
            cycles += self.flush_pipeline();
        }

        cycles
    }

    pub fn thumb_bx(&mut self, instruction: u32) -> CYCLES {
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
        cycles += self.flush_pipeline();
        self.set_executed_instruction(format!("BX {:#010x}", destination));

        cycles
    }

    pub fn thumb_get_relative_address(&mut self, instruction: u32) -> CYCLES {
        let cycles = 1;
        let opcode = instruction.get_bit(11);
        let rd = (instruction & 0x0700) >> 8;
        let imm = (instruction & 0x00FF) * 4;

        let result = match opcode {
            0b0 => (self.get_pc() & !2) + imm,
            0b1 => self.get_sp() + imm,
            _ => panic!(),
        };

        self.set_register(rd, result);

        cycles
    }

    pub fn thumb_add_offset_to_sp(&mut self, instruction: u32) -> CYCLES {
        let cycles = 1;
        let opcode = instruction.get_bit(7);
        let imm = (instruction & 0x007F) * 4;

        let result = match opcode {
            0b0 => self.get_sp() + imm,
            0b1 => self.get_sp() - imm,
            _ => panic!()
        };

        self.set_sp(result);

        cycles
    }
}

#[cfg(test)]
mod thumb_add_and_subtract_tests {
    use std::sync::{Arc, Mutex};

    use crate::{
        arm7tdmi::cpu::{FlagsRegister, InstructionMode, CPU},
        memory::Memory,
    };

    #[test]
    fn should_add_two_registers_together() {
        let memory = Memory::new().unwrap();
        let memory = Arc::new(Mutex::new(memory));
        let mut cpu = CPU::new(memory);
        cpu.set_instruction_mode(InstructionMode::THUMB);

        cpu.set_register(1, 20);
        cpu.set_register(2, 43);
        cpu.fetched_instruction = 0x1888; // adds r0, r1, r2
        cpu.execute_cpu_cycle();
        cpu.execute_cpu_cycle();

        assert_eq!(cpu.get_register(0), 63);
        assert!(cpu.get_flag(FlagsRegister::C) == 0);
        assert!(cpu.get_flag(FlagsRegister::N) == 0);
        assert!(cpu.get_flag(FlagsRegister::Z) == 0);
        assert!(cpu.get_flag(FlagsRegister::V) == 0);
    }

    #[test]
    fn should_add_two_registers_together_and_set_n_flag() {
        let memory = Memory::new().unwrap();
        let memory = Arc::new(Mutex::new(memory));
        let mut cpu = CPU::new(memory);
        cpu.set_instruction_mode(InstructionMode::THUMB);

        cpu.set_register(1, 20);
        cpu.set_register(2, (-43 as i32) as u32);
        cpu.fetched_instruction = 0x1888; // adds r0, r1, r2
        cpu.execute_cpu_cycle();
        cpu.execute_cpu_cycle();

        assert_eq!(cpu.get_register(0), (-23 as i32) as u32);
        assert!(cpu.get_flag(FlagsRegister::C) == 0);
        assert!(cpu.get_flag(FlagsRegister::N) == 1);
        assert!(cpu.get_flag(FlagsRegister::Z) == 0);
        assert!(cpu.get_flag(FlagsRegister::V) == 0);
    }

    #[test]
    fn should_add_two_registers_together_and_set_z_and_c_flag() {
        let memory = Memory::new().unwrap();
        let memory = Arc::new(Mutex::new(memory));
        let mut cpu = CPU::new(memory);
        cpu.set_instruction_mode(InstructionMode::THUMB);

        cpu.set_register(1, 20);
        cpu.set_register(2, (-20 as i32) as u32);
        cpu.fetched_instruction = 0x1888; // adds r0, r1, r2
        cpu.execute_cpu_cycle();
        cpu.execute_cpu_cycle();

        assert_eq!(cpu.get_register(0), 0);
        assert!(cpu.get_flag(FlagsRegister::C) == 1);
        assert!(cpu.get_flag(FlagsRegister::N) == 0);
        assert!(cpu.get_flag(FlagsRegister::Z) == 1);
        assert!(cpu.get_flag(FlagsRegister::V) == 0);
    }

    #[test]
    fn should_add_two_registers_together_and_set_z_flag() {
        let memory = Memory::new().unwrap();
        let memory = Arc::new(Mutex::new(memory));
        let mut cpu = CPU::new(memory);
        cpu.set_instruction_mode(InstructionMode::THUMB);

        cpu.set_register(1, 0);
        cpu.set_register(2, 0);
        cpu.fetched_instruction = 0x1888; // adds r0, r1, r2
        cpu.execute_cpu_cycle();
        cpu.execute_cpu_cycle();

        assert_eq!(cpu.get_register(0), 0);
        assert!(cpu.get_flag(FlagsRegister::C) == 0);
        assert!(cpu.get_flag(FlagsRegister::N) == 0);
        assert!(cpu.get_flag(FlagsRegister::Z) == 1);
        assert!(cpu.get_flag(FlagsRegister::V) == 0);
    }

    #[test]
    fn should_add_two_registers_together_and_set_v_flag() {
        let memory = Memory::new().unwrap();
        let memory = Arc::new(Mutex::new(memory));
        let mut cpu = CPU::new(memory);
        cpu.set_instruction_mode(InstructionMode::THUMB);

        cpu.set_register(1, 0x1);
        cpu.set_register(2, 0x7FFF_FFFF);
        cpu.fetched_instruction = 0x1888; // adds r0, r1, r2
        cpu.execute_cpu_cycle();
        cpu.execute_cpu_cycle();

        assert_eq!(cpu.get_register(0), 0x8000_0000);
        assert!(cpu.get_flag(FlagsRegister::C) == 0);
        assert!(cpu.get_flag(FlagsRegister::N) == 1);
        assert!(cpu.get_flag(FlagsRegister::Z) == 0);
        assert!(cpu.get_flag(FlagsRegister::V) == 1);
    }

    #[test]
    fn should_add_register_and_immediate_and_set_n_flag() {
        let memory = Memory::new().unwrap();
        let memory = Arc::new(Mutex::new(memory));
        let mut cpu = CPU::new(memory);
        cpu.set_instruction_mode(InstructionMode::THUMB);

        cpu.set_register(1, -10 as i32 as u32);
        cpu.fetched_instruction = 0x1d48; // adds r0, r1, 5
        cpu.execute_cpu_cycle();
        cpu.execute_cpu_cycle();

        assert_eq!(cpu.get_register(0), -5 as i32 as u32);
        assert!(cpu.get_flag(FlagsRegister::C) == 0);
        assert!(cpu.get_flag(FlagsRegister::N) == 1);
        assert!(cpu.get_flag(FlagsRegister::Z) == 0);
        assert!(cpu.get_flag(FlagsRegister::V) == 0);
    }

    #[test]
    fn should_add_register_and_immediate_and_set_z_flag() {
        let memory = Memory::new().unwrap();
        let memory = Arc::new(Mutex::new(memory));
        let mut cpu = CPU::new(memory);
        cpu.set_instruction_mode(InstructionMode::THUMB);

        cpu.set_register(1, -5 as i32 as u32);
        cpu.fetched_instruction = 0x1d48; // adds r0, r1, 5
        cpu.execute_cpu_cycle();
        cpu.execute_cpu_cycle();

        assert_eq!(cpu.get_register(0), 0 as i32 as u32);
        assert!(cpu.get_flag(FlagsRegister::C) == 1);
        assert!(cpu.get_flag(FlagsRegister::N) == 0);
        assert!(cpu.get_flag(FlagsRegister::Z) == 1);
        assert!(cpu.get_flag(FlagsRegister::V) == 0);
    }

    #[test]
    fn should_add_register_and_immediate_and_set_v_flag() {
        let memory = Memory::new().unwrap();
        let memory = Arc::new(Mutex::new(memory));
        let mut cpu = CPU::new(memory);
        cpu.set_instruction_mode(InstructionMode::THUMB);

        cpu.set_register(1, 0x7FFF_FFFF);
        cpu.fetched_instruction = 0x1d48; // adds r0, r1, 5
        cpu.execute_cpu_cycle();
        cpu.execute_cpu_cycle();

        assert_eq!(cpu.get_register(0), 0x8000_0004);
        assert!(cpu.get_flag(FlagsRegister::C) == 0);
        assert!(cpu.get_flag(FlagsRegister::N) == 1);
        assert!(cpu.get_flag(FlagsRegister::Z) == 0);
        assert!(cpu.get_flag(FlagsRegister::V) == 1);
    }

    #[test]
    fn should_add_register_and_immediate_and_set_c_flag() {
        let memory = Memory::new().unwrap();
        let memory = Arc::new(Mutex::new(memory));
        let mut cpu = CPU::new(memory);
        cpu.set_instruction_mode(InstructionMode::THUMB);

        cpu.set_register(1, 0xFFFF_FFFF);
        cpu.fetched_instruction = 0x1d48; // adds r0, r1, 5
        cpu.execute_cpu_cycle();
        cpu.execute_cpu_cycle();

        assert_eq!(cpu.get_register(0), 0x0000_0004);
        assert!(cpu.get_flag(FlagsRegister::C) == 1);
        assert!(cpu.get_flag(FlagsRegister::N) == 0);
        assert!(cpu.get_flag(FlagsRegister::Z) == 0);
        assert!(cpu.get_flag(FlagsRegister::V) == 0);
    }

    #[test]
    fn should_sub_two_registers() {
        let memory = Memory::new().unwrap();
        let memory = Arc::new(Mutex::new(memory));
        let mut cpu = CPU::new(memory);
        cpu.set_instruction_mode(InstructionMode::THUMB);

        cpu.set_register(1, 50);
        cpu.set_register(2, 20);
        cpu.fetched_instruction = 0x1a88; // subs r0, r1, r2
        cpu.execute_cpu_cycle();
        cpu.execute_cpu_cycle();

        assert_eq!(cpu.get_register(0), 30);
        assert!(cpu.get_flag(FlagsRegister::C) == 1);
        assert!(cpu.get_flag(FlagsRegister::N) == 0);
        assert!(cpu.get_flag(FlagsRegister::Z) == 0);
        assert!(cpu.get_flag(FlagsRegister::V) == 0);
    }

    #[test]
    fn should_sub_two_registers_and_reset_c_flag() {
        let memory = Memory::new().unwrap();
        let memory = Arc::new(Mutex::new(memory));
        let mut cpu = CPU::new(memory);
        cpu.set_instruction_mode(InstructionMode::THUMB);

        cpu.set_register(1, 25);
        cpu.set_register(2, 50);
        cpu.fetched_instruction = 0x1a88; // subs r0, r1, r2
        cpu.execute_cpu_cycle();
        cpu.execute_cpu_cycle();

        assert_eq!(cpu.get_register(0), -25 as i32 as u32);
        assert_eq!(cpu.get_flag(FlagsRegister::C), 0);
        assert_eq!(cpu.get_flag(FlagsRegister::N), 1);
        assert_eq!(cpu.get_flag(FlagsRegister::Z), 0);
        assert_eq!(cpu.get_flag(FlagsRegister::V), 0);
    }
}

#[cfg(test)]
mod thumb_move_shifted_register_tests {
    use std::sync::{Arc, Mutex};

    use crate::{
        arm7tdmi::cpu::{FlagsRegister, InstructionMode, CPU},
        memory::Memory,
    };

    #[test]
    fn should_left_shift_a_register_and_set_c_flag() {
        let memory = Memory::new().unwrap();
        let memory = Arc::new(Mutex::new(memory));
        let mut cpu = CPU::new(memory);
        cpu.set_instruction_mode(InstructionMode::THUMB);

        cpu.set_register(1, 0x0F00_0000);
        cpu.fetched_instruction = 0x0148; // lsls r0, r1, 5
        cpu.execute_cpu_cycle();
        cpu.execute_cpu_cycle();

        assert_eq!(cpu.get_register(0), 0x0F00_0000 << 5);
        assert_eq!(cpu.get_flag(FlagsRegister::C), 1);
        assert_eq!(cpu.get_flag(FlagsRegister::N), 1);
        assert_eq!(cpu.get_flag(FlagsRegister::Z), 0);
    }

    #[test]
    fn should_not_left_shift_register_and_not_change_c_flag() {
        let memory = Memory::new().unwrap();
        let memory = Arc::new(Mutex::new(memory));
        let mut cpu = CPU::new(memory);
        cpu.set_instruction_mode(InstructionMode::THUMB);

        cpu.set_register(1, 0x0F00_0000);
        cpu.set_flag(FlagsRegister::C);
        cpu.fetched_instruction = 0x0008; // lsls r0, r1, 0
        cpu.execute_cpu_cycle();
        cpu.execute_cpu_cycle();

        assert_eq!(cpu.get_register(0), 0x0F00_0000);
        assert_eq!(cpu.get_flag(FlagsRegister::C), 1);
        assert_eq!(cpu.get_flag(FlagsRegister::N), 0);
        assert_eq!(cpu.get_flag(FlagsRegister::Z), 0);
    }

    #[test]
    fn should_lsl_register_and_not_affect_v_flag() {
        let memory = Memory::new().unwrap();
        let memory = Arc::new(Mutex::new(memory));
        let mut cpu = CPU::new(memory);
        cpu.set_instruction_mode(InstructionMode::THUMB);

        cpu.set_register(1, 0xF000_0000);
        cpu.set_flag(FlagsRegister::V);
        cpu.fetched_instruction = 0x0148; // lsls r0, r1, 5
        cpu.execute_cpu_cycle();
        cpu.execute_cpu_cycle();

        assert_eq!(cpu.get_register(0), 0x0);
        assert_eq!(cpu.get_flag(FlagsRegister::C), 0);
        assert_eq!(cpu.get_flag(FlagsRegister::N), 0);
        assert_eq!(cpu.get_flag(FlagsRegister::Z), 1);
        assert_eq!(cpu.get_flag(FlagsRegister::V), 1);
    }

    #[test]
    fn should_asr_register_and_set_c_flag() {
        let memory = Memory::new().unwrap();
        let memory = Arc::new(Mutex::new(memory));
        let mut cpu = CPU::new(memory);
        cpu.set_instruction_mode(InstructionMode::THUMB);

        cpu.set_register(1, 0x0000_008F);
        cpu.fetched_instruction = 0x1108; // asrs r0, r1, 4
        cpu.execute_cpu_cycle();
        cpu.execute_cpu_cycle();

        assert_eq!(cpu.get_register(0), 0x0000_0008);
        assert_eq!(cpu.get_flag(FlagsRegister::C), 1);
        assert_eq!(cpu.get_flag(FlagsRegister::N), 0);
        assert_eq!(cpu.get_flag(FlagsRegister::Z), 0);
    }

    #[test]
    fn should_asr_register_and_maintain_sign() {
        let memory = Memory::new().unwrap();
        let memory = Arc::new(Mutex::new(memory));
        let mut cpu = CPU::new(memory);
        cpu.set_instruction_mode(InstructionMode::THUMB);

        cpu.set_register(1, 0x8000_008F);
        cpu.fetched_instruction = 0x1108; // asrs r0, r1, 4
        cpu.execute_cpu_cycle();
        cpu.execute_cpu_cycle();

        assert_eq!(cpu.get_register(0), 0xF800_0008);
        assert_eq!(cpu.get_flag(FlagsRegister::C), 1);
        assert_eq!(cpu.get_flag(FlagsRegister::N), 1);
        assert_eq!(cpu.get_flag(FlagsRegister::Z), 0);
    }

    #[test]
    fn should_asr_register_and_set_all_ones() {
        let memory = Memory::new().unwrap();
        let memory = Arc::new(Mutex::new(memory));
        let mut cpu = CPU::new(memory);
        cpu.set_instruction_mode(InstructionMode::THUMB);

        cpu.set_register(1, 0x8000_0000);
        cpu.fetched_instruction = 0x1008; // asrs r0, r1, 32
        cpu.execute_cpu_cycle();
        cpu.execute_cpu_cycle();

        assert_eq!(cpu.get_register(0), 0xFFFF_FFFF);
        assert_eq!(cpu.get_flag(FlagsRegister::C), 1);
        assert_eq!(cpu.get_flag(FlagsRegister::N), 1);
        assert_eq!(cpu.get_flag(FlagsRegister::Z), 0);
    }

    #[test]
    fn should_lsr_register() {
        let memory = Memory::new().unwrap();
        let memory = Arc::new(Mutex::new(memory));
        let mut cpu = CPU::new(memory);
        cpu.set_instruction_mode(InstructionMode::THUMB);

        cpu.set_register(1, 0x8000_008F);
        cpu.fetched_instruction = 0x0a88; // lsrs r0, r1, 10
        cpu.execute_cpu_cycle();
        cpu.execute_cpu_cycle();

        assert_eq!(cpu.get_register(0), 0x8000_008F >> 10);
        assert_eq!(cpu.get_flag(FlagsRegister::C), 0);
        assert_eq!(cpu.get_flag(FlagsRegister::N), 0);
        assert_eq!(cpu.get_flag(FlagsRegister::Z), 0);
    }

    #[test]
    fn should_lsr_register_and_clear_register() {
        let memory = Memory::new().unwrap();
        let memory = Arc::new(Mutex::new(memory));
        let mut cpu = CPU::new(memory);
        cpu.set_instruction_mode(InstructionMode::THUMB);

        cpu.set_register(1, 0x8000_008F);
        cpu.fetched_instruction = 0x0808; // lsrs r0, r1, 32
        cpu.execute_cpu_cycle();
        cpu.execute_cpu_cycle();

        assert_eq!(cpu.get_register(0), 0);
        assert_eq!(cpu.get_flag(FlagsRegister::C), 1);
        assert_eq!(cpu.get_flag(FlagsRegister::N), 0);
        assert_eq!(cpu.get_flag(FlagsRegister::Z), 1);
    }
}

#[cfg(test)]
mod thumb_move_compare_add_subtract_tests {
    use std::sync::{Arc, Mutex};

    use crate::{
        arm7tdmi::cpu::{FlagsRegister, InstructionMode, CPU},
        memory::Memory,
    };

    #[test]
    fn should_move_immediate_into_r0() {
        let memory = Memory::new().unwrap();
        let memory = Arc::new(Mutex::new(memory));
        let mut cpu = CPU::new(memory);
        cpu.set_instruction_mode(InstructionMode::THUMB);

        cpu.fetched_instruction = 0x200f; // movs r0, 15
        cpu.execute_cpu_cycle();
        cpu.execute_cpu_cycle();

        assert_eq!(cpu.get_register(0), 15);
        assert_eq!(cpu.get_flag(FlagsRegister::C), 0);
        assert_eq!(cpu.get_flag(FlagsRegister::N), 0);
        assert_eq!(cpu.get_flag(FlagsRegister::Z), 0);
    }

    #[test]
    fn should_move_immediate_into_r0_and_set_n_flag() {
        let memory = Memory::new().unwrap();
        let memory = Arc::new(Mutex::new(memory));
        let mut cpu = CPU::new(memory);
        cpu.set_instruction_mode(InstructionMode::THUMB);

        cpu.fetched_instruction = 0x2096; // movs r0, 150
        cpu.execute_cpu_cycle();
        cpu.execute_cpu_cycle();

        assert_eq!(cpu.get_register(0), 150);
        assert_eq!(cpu.get_flag(FlagsRegister::C), 0);
        assert_eq!(cpu.get_flag(FlagsRegister::N), 1);
        assert_eq!(cpu.get_flag(FlagsRegister::Z), 0);
    }

    #[test]
    fn should_move_immediate_into_r0_and_set_z_flag() {
        let memory = Memory::new().unwrap();
        let memory = Arc::new(Mutex::new(memory));
        let mut cpu = CPU::new(memory);
        cpu.set_instruction_mode(InstructionMode::THUMB);

        cpu.fetched_instruction = 0x2000; // movs r0, 0
        cpu.execute_cpu_cycle();
        cpu.execute_cpu_cycle();

        assert_eq!(cpu.get_register(0), 0);
        assert_eq!(cpu.get_flag(FlagsRegister::C), 0);
        assert_eq!(cpu.get_flag(FlagsRegister::N), 0);
        assert_eq!(cpu.get_flag(FlagsRegister::Z), 1);
    }

    #[test]
    fn should_sub_imm_from_r0_and_set_z_flag() {
        let memory = Memory::new().unwrap();
        let memory = Arc::new(Mutex::new(memory));
        let mut cpu = CPU::new(memory);
        cpu.set_instruction_mode(InstructionMode::THUMB);

        cpu.set_register(0, 15);
        cpu.fetched_instruction = 0x380f; // subs r0, 15
        cpu.execute_cpu_cycle();
        cpu.execute_cpu_cycle();

        assert_eq!(cpu.get_register(0), 0);
        assert_eq!(cpu.get_flag(FlagsRegister::C), 1);
        assert_eq!(cpu.get_flag(FlagsRegister::N), 0);
        assert_eq!(cpu.get_flag(FlagsRegister::Z), 1);
    }

    #[test]
    fn should_add_imm_to_r0_and_set_n_flag() {
        let memory = Memory::new().unwrap();
        let memory = Arc::new(Mutex::new(memory));
        let mut cpu = CPU::new(memory);
        cpu.set_instruction_mode(InstructionMode::THUMB);

        cpu.set_register(0, 0x7FFF_FFFF);
        cpu.fetched_instruction = 0x300f; // adds r0, 15
        cpu.execute_cpu_cycle();
        cpu.execute_cpu_cycle();

        assert_eq!(cpu.get_register(0), 0x8000_000E);
        assert_eq!(cpu.get_flag(FlagsRegister::C), 0);
        assert_eq!(cpu.get_flag(FlagsRegister::N), 1);
        assert_eq!(cpu.get_flag(FlagsRegister::Z), 0);
    }
}

#[cfg(test)]
mod thumb_alu_operations_tests {
    use std::sync::{Arc, Mutex};

    use crate::{
        arm7tdmi::cpu::{FlagsRegister, InstructionMode, CPU},
        memory::Memory,
    };

    #[test]
    fn should_and_two_numbers_together() {
        let memory = Memory::new().unwrap();
        let memory = Arc::new(Mutex::new(memory));
        let mut cpu = CPU::new(memory);
        cpu.set_instruction_mode(InstructionMode::THUMB);

        cpu.set_register(0, 0x8123_2344);
        cpu.set_register(1, 0x8000_2344);
        cpu.fetched_instruction = 0x4008; // ands r0, r1
        cpu.execute_cpu_cycle();
        cpu.execute_cpu_cycle();

        assert_eq!(cpu.get_register(0), 0x8000_2344);
        assert_eq!(cpu.get_flag(FlagsRegister::N), 1);
        assert_eq!(cpu.get_flag(FlagsRegister::Z), 0);
    }

    #[test]
    fn should_eor_two_numbers_together() {
        let memory = Memory::new().unwrap();
        let memory = Arc::new(Mutex::new(memory));
        let mut cpu = CPU::new(memory);
        cpu.set_instruction_mode(InstructionMode::THUMB);

        cpu.set_register(0, 0x1010_1010);
        cpu.set_register(1, 0x0101_0101);
        cpu.fetched_instruction = 0x4048; // eors r0, r1
        cpu.execute_cpu_cycle();
        cpu.execute_cpu_cycle();

        assert_eq!(cpu.get_register(0), 0x1111_1111);
        assert_eq!(cpu.get_flag(FlagsRegister::N), 0);
        assert_eq!(cpu.get_flag(FlagsRegister::Z), 0);
    }

    #[test]
    fn should_lsl_rd_by_5() {
        let memory = Memory::new().unwrap();
        let memory = Arc::new(Mutex::new(memory));
        let mut cpu = CPU::new(memory);
        cpu.set_instruction_mode(InstructionMode::THUMB);

        cpu.set_register(0, 0x0F11_1230);
        cpu.set_register(1, 5);
        cpu.fetched_instruction = 0x4088; // lsl r0, r1
        cpu.execute_cpu_cycle();
        cpu.execute_cpu_cycle();

        assert_eq!(cpu.get_register(0), 0xE222_4600);
        assert_eq!(cpu.get_flag(FlagsRegister::N), 1);
        assert_eq!(cpu.get_flag(FlagsRegister::C), 1);
        assert_eq!(cpu.get_flag(FlagsRegister::Z), 0);
    }

    #[test]
    fn should_lsr_rd_by_0() {
        let memory = Memory::new().unwrap();
        let memory = Arc::new(Mutex::new(memory));
        let mut cpu = CPU::new(memory);
        cpu.set_instruction_mode(InstructionMode::THUMB);

        cpu.set_register(0, 0x0F11_1230);
        cpu.set_register(1, 0);
        cpu.fetched_instruction = 0x40c8; // lsr r0, r1
        cpu.execute_cpu_cycle();
        cpu.execute_cpu_cycle();

        assert_eq!(cpu.get_register(0), 0x0F11_1230);
        assert_eq!(cpu.get_flag(FlagsRegister::N), 0);
        assert_eq!(cpu.get_flag(FlagsRegister::C), 0);
        assert_eq!(cpu.get_flag(FlagsRegister::Z), 0);
    }
}

#[cfg(test)]
mod thumb_hi_reg_operations {
    use std::sync::{Arc, Mutex};

    use crate::{
        arm7tdmi::cpu::{FlagsRegister, InstructionMode, CPU},
        memory::Memory,
    };

    #[test]
    fn should_add_two_regs_together_and_not_affect_flags() {
        let memory = Memory::new().unwrap();
        let memory = Arc::new(Mutex::new(memory));
        let mut cpu = CPU::new(memory);
        cpu.set_instruction_mode(InstructionMode::THUMB);

        cpu.set_register(0, 20);
        cpu.set_register(11, 15);
        cpu.set_flag(FlagsRegister::N);
        cpu.fetched_instruction = 0x4458; // add r0, r11
        cpu.execute_cpu_cycle();
        cpu.execute_cpu_cycle();

        assert_eq!(cpu.get_register(0), 35);
        assert_eq!(cpu.get_flag(FlagsRegister::N), 1);
        assert_eq!(cpu.get_flag(FlagsRegister::C), 0);
        assert_eq!(cpu.get_flag(FlagsRegister::Z), 0);
    }

    #[test]
    fn should_cmp_registers_and_set_flags() {
        let memory = Memory::new().unwrap();
        let memory = Arc::new(Mutex::new(memory));
        let mut cpu = CPU::new(memory);
        cpu.set_instruction_mode(InstructionMode::THUMB);

        cpu.set_register(0, 20);
        cpu.set_register(11, 20);
        cpu.set_flag(FlagsRegister::N);
        cpu.fetched_instruction = 0x4558; // cmp r0, r11
        cpu.execute_cpu_cycle();
        cpu.execute_cpu_cycle();

        assert_eq!(cpu.get_register(0), 20);
        assert_eq!(cpu.get_flag(FlagsRegister::N), 0);
        assert_eq!(cpu.get_flag(FlagsRegister::C), 1);
        assert_eq!(cpu.get_flag(FlagsRegister::Z), 1);
    }

    #[test]
    fn should_mov_register() {
        let memory = Memory::new().unwrap();
        let memory = Arc::new(Mutex::new(memory));
        let mut cpu = CPU::new(memory);
        cpu.set_instruction_mode(InstructionMode::THUMB);

        cpu.set_register(0, 20);
        cpu.set_register(11, 55);
        cpu.set_flag(FlagsRegister::N);
        cpu.fetched_instruction = 0x4658; // cmp r0, r11
        cpu.execute_cpu_cycle();
        cpu.execute_cpu_cycle();

        assert_eq!(cpu.get_register(0), 55);
    }
}

#[cfg(test)]
mod thumb_bx_tests {
    use std::sync::{Arc, Mutex};

    use crate::{
        arm7tdmi::cpu::{InstructionMode, CPU},
        memory::Memory,
    };

    #[test]
    fn should_switch_to_arm_mode_and_align_address() {
        let memory = Memory::new().unwrap();
        let memory = Arc::new(Mutex::new(memory));
        let mut cpu = CPU::new(memory);
        cpu.set_instruction_mode(InstructionMode::THUMB);

        cpu.set_register(5, 0x16);
        cpu.fetched_instruction = 0x4728; // bx r5
        cpu.execute_cpu_cycle();
        cpu.execute_cpu_cycle();

        assert_eq!(cpu.get_pc(), 0x1C);
        assert!(matches!(cpu.get_instruction_mode(), InstructionMode::ARM));
    }

    #[test]
    fn should_switch_to_arm_mode_when_pc_operand() {
        let memory = Memory::new().unwrap();
        let memory = Arc::new(Mutex::new(memory));
        let mut cpu = CPU::new(memory);
        cpu.set_instruction_mode(InstructionMode::THUMB);

        cpu.set_pc(0x16);
        cpu.fetched_instruction = 0x4778; // bx r15
        cpu.execute_cpu_cycle();
        cpu.execute_cpu_cycle();

        assert_eq!(cpu.get_pc(), 0x20);
        assert!(matches!(cpu.get_instruction_mode(), InstructionMode::ARM));
    }
}

#[cfg(test)]
mod get_relative_address_tests {
    use std::sync::{Arc, Mutex};

    use crate::{
        arm7tdmi::cpu::{InstructionMode, CPU},
        memory::Memory,
    };

    #[test]
    fn should_add_12_to_pc() {
        let memory = Memory::new().unwrap();
        let memory = Arc::new(Mutex::new(memory));
        let mut cpu = CPU::new(memory);
        cpu.set_instruction_mode(InstructionMode::THUMB);

        cpu.fetched_instruction = 0xa503; // add r5, pc, 12
        cpu.set_pc(2);
        cpu.execute_cpu_cycle();
        cpu.execute_cpu_cycle();

        assert_eq!(cpu.get_pc(), 6);
        assert_eq!(cpu.get_register(5), 16);
    }

    #[test]
    fn should_add_16_to_sp_and_store_in_r5() {
        let memory = Memory::new().unwrap();
        let memory = Arc::new(Mutex::new(memory));
        let mut cpu = CPU::new(memory);
        cpu.set_instruction_mode(InstructionMode::THUMB);

        cpu.fetched_instruction = 0xad04; // add r5, sp, 16
        cpu.set_sp(2);
        cpu.execute_cpu_cycle();
        cpu.execute_cpu_cycle();

        assert_eq!(cpu.get_register(5), 18);
    }

    #[test]
    fn should_add_500_to_sp() {
        let memory = Memory::new().unwrap();
        let memory = Arc::new(Mutex::new(memory));
        let mut cpu = CPU::new(memory);
        cpu.set_instruction_mode(InstructionMode::THUMB);

        cpu.fetched_instruction = 0xb07d; // add sp, 500
        cpu.set_sp(2);
        cpu.execute_cpu_cycle();
        cpu.execute_cpu_cycle();

        assert_eq!(cpu.get_sp(), 502);
    }

    #[test]
    fn should_sub_500_to_sp() {
        let memory = Memory::new().unwrap();
        let memory = Arc::new(Mutex::new(memory));
        let mut cpu = CPU::new(memory);
        cpu.set_instruction_mode(InstructionMode::THUMB);

        cpu.fetched_instruction = 0xb0fd; // add sp, 500
        cpu.set_sp(2);
        cpu.execute_cpu_cycle();
        cpu.execute_cpu_cycle();

        assert_eq!(cpu.get_sp(), (2 - 500) as i32 as u32);
    }
}
