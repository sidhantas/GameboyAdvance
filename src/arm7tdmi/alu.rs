use crate::{
    types::{REGISTER, WORD},
    utils::bits::Bits,
};

use super::{
    cpu::{FlagsRegister, CPU},
    instructions::ALUExecutable,
};

#[derive(Clone)]
pub struct ALUInstruction {
    pub executable: ALUExecutable,
    pub rd: REGISTER,
    pub operand1: u32,
    pub operand2: u32,
    pub set_flags: bool,
}

impl Default for ALUInstruction {
    fn default() -> Self {
        Self {
            executable: CPU::arm_add,
            rd: 0,
            operand1: 0,
            operand2: 0,
            set_flags: false,
        }
    }
}

impl CPU {
    pub fn arm_add(&mut self) {
        let alu_executable = self.alu_executable.clone();
        let operand1 = self.alu_executable.operand1;
        let operand2 = self.alu_executable.operand2;
        let result = operand1 + operand2;

        if self.alu_executable.set_flags == true {
            self.set_flag_from_bit(FlagsRegister::N, result.get_bit(31) as u8);
            if result == 0 {
                self.set_flag(FlagsRegister::Z);
            } else {
                self.reset_flag(FlagsRegister::Z);
            }
            if (operand1.get_bit(31) == operand2.get_bit(31))
                && result.get_bit(31) != operand2.get_bit(31)
            {
                self.set_flag(FlagsRegister::V);
            } else {
                self.reset_flag(FlagsRegister::V);
            }
            if result < operand1 || result < operand2 {
                self.set_flag(FlagsRegister::C);
            } else {
                self.reset_flag(FlagsRegister::C);
            }
        }
        self.set_register(alu_executable.rd, result as u32);
        self.set_executed_instruction(format!(
            "ADD {:#x} {:#x} {:#x}",
            alu_executable.rd, alu_executable.operand1, alu_executable.operand2
        ));
    }

    pub fn arm_and(&mut self) {
        let decoded_inst = self.alu_executable.clone();
        let operand1 = self.alu_executable.operand1;
        let operand2 = self.alu_executable.operand2;
        let result = operand1 & operand2;

        self.set_logical_flags(result, self.alu_executable.set_flags);
        self.set_register(decoded_inst.rd, result as u32);
        self.set_executed_instruction(format!(
            "AND {:#x} {:#x} {:#x}",
            decoded_inst.rd, decoded_inst.operand1, decoded_inst.operand2
        ));
    }

    pub fn arm_eor(&mut self) {
        let decoded_inst = self.alu_executable.clone();
        let operand1 = self.alu_executable.operand1;
        let operand2 = self.alu_executable.operand2;
        let result = operand1 ^ operand2;

        self.set_logical_flags(result, self.alu_executable.set_flags);
        self.set_register(decoded_inst.rd, result as u32);
        self.set_executed_instruction(format!(
            "EOR {:#x} {:#x} {:#x}",
            decoded_inst.rd, decoded_inst.operand1, decoded_inst.operand2
        ));
    }

    pub fn arm_sub(&mut self) {
        let decoded_inst = self.alu_executable.clone();
        let result = self.get_register(decoded_inst.operand1) - decoded_inst.operand2;
        self.set_executed_instruction(format!("SUB"))
    }

    pub fn arm_rsb(&mut self) {
        self.set_executed_instruction(format!("RSB"))
    }

    pub fn arm_adc(&mut self) {}

    pub fn arm_sbc(&mut self) {}

    pub fn arm_rsc(&mut self) {}

    pub fn arm_tst(&mut self) {
        let decoded_inst = self.alu_executable.clone();
        let operand1 = self.alu_executable.operand1;
        let operand2 = self.alu_executable.operand2;
        let result = operand1 & operand2;

        self.set_logical_flags(result, true);
        self.set_executed_instruction(format!(
            "TST {:#x} {:#x} {:#x}",
            decoded_inst.rd, decoded_inst.operand1, decoded_inst.operand2
        ));
    }

    pub fn arm_teq(&mut self) {
        let decoded_inst = self.alu_executable.clone();
        let operand1 = self.alu_executable.operand1;
        let operand2 = self.alu_executable.operand2;
        let result = operand1 ^ operand2;

        self.set_logical_flags(result, true);
        self.set_executed_instruction(format!(
            "TEQ {:#x} {:#x} {:#x}",
            decoded_inst.rd, decoded_inst.operand1, decoded_inst.operand2
        ));
    }

    pub fn arm_cmp(&mut self) {}

    pub fn arm_cmn(&mut self) {}

    pub fn arm_orr(&mut self) {
        let decoded_inst = self.alu_executable.clone();
        let operand1 = self.alu_executable.operand1;
        let operand2 = self.alu_executable.operand2;
        let result = operand1 | operand2;

        self.set_logical_flags(result, self.alu_executable.set_flags);
        self.set_register(decoded_inst.rd, result as u32);
        self.set_executed_instruction(format!(
            "ORR {:#x} {:#x} {:#x}",
            decoded_inst.rd, decoded_inst.operand1, decoded_inst.operand2
        ));
    }

    pub fn arm_mov(&mut self) {}

    pub fn arm_bic(&mut self) {
        let decoded_inst = self.alu_executable.clone();
        let operand1 = self.alu_executable.operand1;
        let operand2 = self.alu_executable.operand2;
        let result = operand1 & !operand2;

        self.set_logical_flags(result, self.alu_executable.set_flags);
        self.set_register(decoded_inst.rd, result as u32);
        self.set_executed_instruction(format!(
            "BIC {:#x} {:#x} {:#x}",
            decoded_inst.rd, decoded_inst.operand1, decoded_inst.operand2
        ));
    }

    pub fn arm_mvn(&mut self) {}

    fn set_logical_flags(&mut self, result: WORD, set_flags: bool) {
        if set_flags == true {
            self.set_flag_from_bit(FlagsRegister::N, result.get_bit(31) as u8);
            if result == 0 {
                self.set_flag(FlagsRegister::Z);
            } else {
                self.reset_flag(FlagsRegister::Z);
            }
        }
    }
}
#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use crate::{
        arm7tdmi::cpu::{FlagsRegister, CPU},
        memory::Memory,
    };

    #[test]
    fn add_instruction_should_set_carry_flag() {
        let memory = Memory::new().unwrap();
        let memory = Arc::new(Mutex::new(memory));
        let mut cpu = CPU::new(memory);

        cpu.set_register(2, u32::MAX);
        cpu.set_register(3, 2);

        cpu.fetched_instruction = 0xe0931002; // adds r1, r3, r2;

        cpu.execute_cpu_cycle();
        cpu.execute_cpu_cycle();
        assert!(cpu.get_register(1) == 1);
        assert!(cpu.get_flag(FlagsRegister::C) == 1);
        assert!(cpu.get_flag(FlagsRegister::N) == 0);
        assert!(cpu.get_flag(FlagsRegister::Z) == 0);
        assert!(cpu.get_flag(FlagsRegister::V) == 0);
    }

    #[test]
    fn add_instruction_should_set_overflow_and_carry_flags() {
        let memory = Memory::new().unwrap();
        let memory = Arc::new(Mutex::new(memory));
        let mut cpu = CPU::new(memory);

        cpu.set_register(2, 0x8000_0000);
        cpu.set_register(3, 0x8000_0000);

        cpu.fetched_instruction = 0xe0931002; // adds r1, r3, r2;

        cpu.execute_cpu_cycle();
        cpu.execute_cpu_cycle();
        assert!(cpu.get_register(1) == 0);
        assert!(cpu.get_flag(FlagsRegister::C) == 1);
        assert!(cpu.get_flag(FlagsRegister::N) == 0);
        assert!(cpu.get_flag(FlagsRegister::Z) == 1);
        assert!(cpu.get_flag(FlagsRegister::V) == 1);
    }

    #[test]
    fn add_instruction_should_set_n_flag() {
        let memory = Memory::new().unwrap();
        let memory = Arc::new(Mutex::new(memory));
        let mut cpu = CPU::new(memory);

        cpu.set_register(2, 0x8000_0000);
        cpu.set_register(3, 0x0000_0001);

        cpu.fetched_instruction = 0xe0931002; // adds r1, r3, r2;

        cpu.execute_cpu_cycle();
        cpu.execute_cpu_cycle();
        assert!(cpu.get_register(1) == 0x8000_0001);
        assert!(cpu.get_flag(FlagsRegister::C) == 0);
        assert!(cpu.get_flag(FlagsRegister::N) == 1);
        assert!(cpu.get_flag(FlagsRegister::Z) == 0);
        assert!(cpu.get_flag(FlagsRegister::V) == 0);
    }

    #[test]
    fn and_instruction_should_set_c_flag() {
        let memory = Memory::new().unwrap();
        let memory = Arc::new(Mutex::new(memory));
        let mut cpu = CPU::new(memory);

        cpu.set_register(2, 0x0000_FFFF);
        cpu.set_register(3, 0x0000_0001);

        cpu.fetched_instruction = 0xe01312a2; // ands r1, r3, r2 LSR 5;

        cpu.execute_cpu_cycle();
        cpu.execute_cpu_cycle();
        assert!(cpu.get_flag(FlagsRegister::C) == 1);
        assert!(cpu.get_flag(FlagsRegister::N) == 0);
        assert!(cpu.get_flag(FlagsRegister::Z) == 0);
        assert!(cpu.get_flag(FlagsRegister::V) == 0);
        assert!(cpu.get_register(1) == 0x0000_0001);
    }

    #[test]
    fn and_instruction_should_set_n_flag() {
        let memory = Memory::new().unwrap();
        let memory = Arc::new(Mutex::new(memory));
        let mut cpu = CPU::new(memory);

        cpu.set_register(2, 0x8000_FFFF);
        cpu.set_register(3, 0x8000_0001);

        cpu.fetched_instruction = 0xe0131002; // ands r1, r3, r2;

        cpu.execute_cpu_cycle();
        cpu.execute_cpu_cycle();
        assert!(cpu.get_flag(FlagsRegister::C) == 0);
        assert!(cpu.get_flag(FlagsRegister::N) == 1);
        assert!(cpu.get_flag(FlagsRegister::Z) == 0);
        assert!(cpu.get_flag(FlagsRegister::V) == 0);
        assert!(cpu.get_register(1) == 0x8000_0001);
    }

    #[test]
    fn and_instruction_should_set_z_flag() {
        let memory = Memory::new().unwrap();
        let memory = Arc::new(Mutex::new(memory));
        let mut cpu = CPU::new(memory);

        cpu.set_register(2, 0x8000_FFFF);
        cpu.set_register(3, 0x0000_0000);

        cpu.fetched_instruction = 0xe0131002; // ands r1, r3, r2;

        cpu.execute_cpu_cycle();
        cpu.execute_cpu_cycle();
        assert!(cpu.get_flag(FlagsRegister::C) == 0);
        assert!(cpu.get_flag(FlagsRegister::N) == 0);
        assert!(cpu.get_flag(FlagsRegister::Z) == 1);
        assert!(cpu.get_flag(FlagsRegister::V) == 0);
        assert!(cpu.get_register(1) == 0x0000_0000);
    }

    #[test]
    fn orr_instruction_should_set_z_flag() {
        let memory = Memory::new().unwrap();
        let memory = Arc::new(Mutex::new(memory));
        let mut cpu = CPU::new(memory);

        cpu.set_register(2, 0x0000_0000);
        cpu.set_register(3, 0x0000_0000);

        cpu.fetched_instruction = 0xe1931002; // orrs r1, r3, r2;

        cpu.execute_cpu_cycle();
        cpu.execute_cpu_cycle();
        assert!(cpu.get_flag(FlagsRegister::C) == 0);
        assert!(cpu.get_flag(FlagsRegister::N) == 0);
        assert!(cpu.get_flag(FlagsRegister::Z) == 1);
        assert!(cpu.get_flag(FlagsRegister::V) == 0);
        assert!(cpu.get_register(1) == 0x0000_0000);
    }

    #[test]
    fn orr_instruction_should_not_set_any_flags() {
        let memory = Memory::new().unwrap();
        let memory = Arc::new(Mutex::new(memory));
        let mut cpu = CPU::new(memory);

        cpu.set_register(2, 0x0000_0000);
        cpu.set_register(3, 0x0000_0000);

        cpu.fetched_instruction = 0xe1831002; // orr r1, r3, r2;

        cpu.execute_cpu_cycle();
        cpu.execute_cpu_cycle();
        assert!(cpu.get_flag(FlagsRegister::C) == 0);
        assert!(cpu.get_flag(FlagsRegister::N) == 0);
        assert!(cpu.get_flag(FlagsRegister::Z) == 0);
        assert!(cpu.get_flag(FlagsRegister::V) == 0);
        assert!(cpu.get_register(1) == 0x0000_0000);
    }

    #[test]
    fn eor_instruction_should_set_n_flag() {
        let memory = Memory::new().unwrap();
        let memory = Arc::new(Mutex::new(memory));
        let mut cpu = CPU::new(memory);

        cpu.set_register(2, 0x8001_0002);
        cpu.set_register(3, 0x1000_0010);

        cpu.fetched_instruction = 0xe0331002; // eors r1, r3, r2;

        cpu.execute_cpu_cycle();
        cpu.execute_cpu_cycle();
        assert!(cpu.get_flag(FlagsRegister::C) == 0);
        assert!(cpu.get_flag(FlagsRegister::N) == 1);
        assert!(cpu.get_flag(FlagsRegister::Z) == 0);
        assert!(cpu.get_flag(FlagsRegister::V) == 0);
        assert!(cpu.get_register(1) == 0x9001_0012);
    }

    #[test]
    fn teq_instruction_should_set_n_flag() {
        let memory = Memory::new().unwrap();
        let memory = Arc::new(Mutex::new(memory));
        let mut cpu = CPU::new(memory);

        cpu.set_register(2, 0x8001_0002);
        cpu.set_register(3, 0x1000_0010);

        cpu.fetched_instruction = 0xe1330002; // teq r3, r2;

        cpu.execute_cpu_cycle();
        cpu.execute_cpu_cycle();
        assert!(cpu.get_flag(FlagsRegister::C) == 0);
        assert!(cpu.get_flag(FlagsRegister::N) == 1);
        assert!(cpu.get_flag(FlagsRegister::Z) == 0);
        assert!(cpu.get_flag(FlagsRegister::V) == 0);
    }

    #[test]
    fn teq_instruction_should_set_z_flag_when_equal() {
        let memory = Memory::new().unwrap();
        let memory = Arc::new(Mutex::new(memory));
        let mut cpu = CPU::new(memory);

        cpu.set_register(2, 0x8001_0002);
        cpu.set_register(3, 0x8001_0002);

        cpu.fetched_instruction = 0xe1330002; // teq r3, r2;

        cpu.execute_cpu_cycle();
        cpu.execute_cpu_cycle();
        assert!(cpu.get_flag(FlagsRegister::C) == 0);
        assert!(cpu.get_flag(FlagsRegister::N) == 0);
        assert!(cpu.get_flag(FlagsRegister::Z) == 1);
        assert!(cpu.get_flag(FlagsRegister::V) == 0);
    }

    #[test]
    fn tst_instruction_should_set_z_flag_when_no_bits_match() {
        let memory = Memory::new().unwrap();
        let memory = Arc::new(Mutex::new(memory));
        let mut cpu = CPU::new(memory);

        cpu.set_register(2, 0x8001_0002);
        cpu.set_register(3, 0x0110_2224);

        cpu.fetched_instruction = 0xe1130002; // tst r3, r2;

        cpu.execute_cpu_cycle();
        cpu.execute_cpu_cycle();
        assert!(cpu.get_flag(FlagsRegister::C) == 0);
        assert!(cpu.get_flag(FlagsRegister::N) == 0);
        assert!(cpu.get_flag(FlagsRegister::Z) == 1);
        assert!(cpu.get_flag(FlagsRegister::V) == 0);
    }

    #[test]
    fn bic_instruction_should_reset_all_bits() {
        let memory = Memory::new().unwrap();
        let memory = Arc::new(Mutex::new(memory));
        let mut cpu = CPU::new(memory);

        cpu.set_register(3, 0x8001_0002);
        cpu.set_register(2, 0x80F1_0102);

        cpu.fetched_instruction = 0xe1d31002; // bics r1, r3, r2;

        cpu.execute_cpu_cycle();
        cpu.execute_cpu_cycle();
        assert!(cpu.get_flag(FlagsRegister::C) == 0);
        assert!(cpu.get_flag(FlagsRegister::N) == 0);
        assert!(cpu.get_flag(FlagsRegister::Z) == 1);
        assert!(cpu.get_flag(FlagsRegister::V) == 0);
        assert!(cpu.get_register(1) == 0x00);
    }

    #[test]
    fn data_processing_with_pc_as_operand2_and_register_shift_delays_pc() {
        let memory = Memory::new().unwrap();
        let memory = Arc::new(Mutex::new(memory));
        let mut cpu = CPU::new(memory);

        cpu.fetched_instruction = 0xe094131f; // adds r1, r3, r15, LSL r3; pc = 0

        cpu.set_register(3, 0x01);
        let test_pc = 4; // points at next instruction
        cpu.set_pc(test_pc);

        cpu.execute_cpu_cycle();
        cpu.execute_cpu_cycle();
        cpu.execute_cpu_cycle();
        assert!(cpu.get_register(1) == (test_pc + 8) << 1);
        dbg!(cpu.get_register(1));
        assert!(cpu.get_flag(FlagsRegister::C) == 0);
        assert!(cpu.get_flag(FlagsRegister::N) == 0);
        assert!(cpu.get_flag(FlagsRegister::Z) == 0);
        assert!(cpu.get_flag(FlagsRegister::V) == 0);
    }
}
