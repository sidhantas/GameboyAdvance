use crate::{
    arm7tdmi::cpu::{FlagsRegister, CPU},
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

        operation(self, rd, rs_val, offset);
        1
    }

    fn thumb_lsl(&mut self, rd: REGISTER, rs_val: u32, offset: u8) {
        let result = rs_val << offset;
        self.set_register(rd, result);

        if offset > 0 {
            if rs_val.bit_is_set(32 - offset) {
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

    fn thumb_lsr(&mut self, rd: REGISTER, rs_val: u32, offset: u8) {
        if offset == 0 {
            // LSR#32
            self.set_flag(FlagsRegister::Z);
            self.reset_flag(FlagsRegister::N);
            self.set_flag_from_bit(FlagsRegister::C, rs_val.get_bit(31) as u8);
            self.set_register(rd, 0);
            return;
        }
        let result = rs_val >> offset;

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

        self.set_register(rd, result);
    }

    fn thumb_asr(&mut self, rd: REGISTER, rs_val: u32, offset: u8) {
        if offset == 0 {
            if rs_val.bit_is_set(31) {
                self.set_flag(FlagsRegister::C);
                self.set_flag(FlagsRegister::N);
                self.reset_flag(FlagsRegister::Z);
                self.set_register(rd, u32::MAX);
            } else {
                self.reset_flag(FlagsRegister::C);
                self.set_flag(FlagsRegister::Z);
                self.reset_flag(FlagsRegister::N);
                self.set_register(rd, 0);
            }

            return;
        }

        let result = (rs_val as i32 >> offset) as u32;
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
        self.set_register(rd, result);
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
            _ => panic!()
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
    }

    fn thumb_cmp_imm(&mut self, rd: REGISTER, imm: u8) {
        let minuend = self.get_register(rd);
        let imm: u32 = sign_extend(imm as u32, 7).twos_complement();
        let result =  minuend + imm;
        self.set_arithmetic_flags(result, minuend, imm, 0, true);
    }
    
    fn thumb_add_imm(&mut self, rd: REGISTER, imm: u8) {
        let addend1 = self.get_register(rd);
        let result = addend1 + imm as u32;
        self.set_arithmetic_flags(result, addend1, imm as u32, 0, true);
        self.set_register(rd, result);
    }

    fn thumb_sub_imm(&mut self, rd: REGISTER, imm: u8) {
        let minuend = self.get_register(rd);
        let imm: u32 = sign_extend(imm as u32, 7).twos_complement();
        let result =  minuend + imm;
        self.set_arithmetic_flags(result, minuend, imm, 0, true);
        self.set_register(rd, result);
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
        cpu.inst_mode = InstructionMode::THUMB;

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
        cpu.inst_mode = InstructionMode::THUMB;

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
        cpu.inst_mode = InstructionMode::THUMB;

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
        cpu.inst_mode = InstructionMode::THUMB;

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
        cpu.inst_mode = InstructionMode::THUMB;

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
        cpu.inst_mode = InstructionMode::THUMB;

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
        cpu.inst_mode = InstructionMode::THUMB;

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
        cpu.inst_mode = InstructionMode::THUMB;

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
        cpu.inst_mode = InstructionMode::THUMB;

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
        cpu.inst_mode = InstructionMode::THUMB;

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
        cpu.inst_mode = InstructionMode::THUMB;

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
        cpu.inst_mode = InstructionMode::THUMB;

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
        cpu.inst_mode = InstructionMode::THUMB;

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
        cpu.inst_mode = InstructionMode::THUMB;

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
        cpu.inst_mode = InstructionMode::THUMB;

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
        cpu.inst_mode = InstructionMode::THUMB;

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
        cpu.inst_mode = InstructionMode::THUMB;

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
        cpu.inst_mode = InstructionMode::THUMB;

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
        cpu.inst_mode = InstructionMode::THUMB;

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
        cpu.inst_mode = InstructionMode::THUMB;

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
        cpu.inst_mode = InstructionMode::THUMB;

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
        cpu.inst_mode = InstructionMode::THUMB;

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
        cpu.inst_mode = InstructionMode::THUMB;

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
        cpu.inst_mode = InstructionMode::THUMB;

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
