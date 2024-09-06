use crate::{
    types::{REGISTER, WORD},
    utils::bits::Bits,
};

use super::{
    cpu::{FlagsRegister, CPU},
    instructions::ALUOperation,
};

#[derive(Clone)]
pub struct ALUInstruction {
    pub instruction: u32,
    pub operation: ALUOperation,
    pub rd: REGISTER,
    pub rn: REGISTER,
    pub set_flags: bool,
}

impl Default for ALUInstruction {
    fn default() -> Self {
        Self {
            instruction: 0,
            operation: CPU::arm_add,
            rd: 0,
            rn: 0,
            set_flags: false,
        }
    }
}

impl CPU {
    pub fn arm_add(&mut self, rd: REGISTER, operand1: u32, operand2: u32, set_flags: bool) {
        let result = operand1 + operand2;
        self.set_arithmetic_flags(result, operand1, operand2, 0, set_flags);
        self.set_register(rd, result);
        self.set_executed_instruction(format!("ADD {:#x} {:#x} {:#x}", rd, operand1, operand2));
    }

    pub fn arm_and(&mut self, rd: REGISTER, operand1: u32, operand2: u32, set_flags: bool) {
        let result = operand1 & operand2;

        self.set_logical_flags(result, set_flags);
        self.set_register(rd, result);
        self.set_executed_instruction(format!("AND {:#x} {:#x} {:#x}", rd, operand1, operand2));
    }

    pub fn arm_eor(&mut self, rd: REGISTER, operand1: u32, operand2: u32, set_flags: bool) {
        let result = operand1 ^ operand2;

        self.set_logical_flags(result, set_flags);
        self.set_register(rd, result);
        self.set_executed_instruction(format!("EOR {:#x} {:#x} {:#x}", rd, operand1, operand2));
    }

    pub fn arm_sub(&mut self, rd: REGISTER, operand1: u32, operand2: u32, set_flags: bool) {
        let operand2 = !operand2 + 1;
        let result = operand1 + operand2; // use two's complement to make setting flags easier
        
        self.set_arithmetic_flags(result, operand1, operand2, 0, set_flags);
        self.set_register(rd, result);

        self.set_executed_instruction(format!("SUB"))
    }

    pub fn arm_rsb(&mut self, rd: REGISTER, operand1: u32, operand2: u32, set_flags: bool) {
        let operand1 = !operand1 + 1;
        let result = operand1 + operand2; // use two's complement to make setting flags easier
        
        self.set_arithmetic_flags(result, operand1, operand2, 0, set_flags);
        self.set_register(rd, result);

        self.set_executed_instruction(format!("RSB"))
    }

    pub fn arm_adc(&mut self, rd: REGISTER, operand1: u32, operand2: u32, set_flags: bool) {
        let carry = self.get_flag(FlagsRegister::C);
        let result = operand1 + operand2 + carry;

        self.set_arithmetic_flags(result, operand1, operand2, carry, set_flags);
        self.set_register(rd, result);
        self.set_executed_instruction(format!(
            "ADC {:#x} {:#x} {:#x} {:#x}",
            rd, operand1, operand2, carry
        ));
    }

    pub fn arm_sbc(&mut self, rd: REGISTER, operand1: u32, operand2: u32, set_flags: bool) {
        let carry = self.get_flag(FlagsRegister::C);
        let operand2 = !operand2 + 1;
        let carry = !carry + 1;
        let result = operand1 + operand2 + carry;

        self.set_arithmetic_flags(result, operand1, operand2, carry, set_flags);
        self.set_register(rd, result);
        self.set_executed_instruction(format!(
            "SBC {:#x} {:#x} {:#x} {:#x}",
            rd, operand1, operand2, carry
        ));

    }

    pub fn arm_rsc(&mut self, rd: REGISTER, operand1: u32, operand2: u32, set_flags: bool) {
        let carry = self.get_flag(FlagsRegister::C);
        let operand1 = !operand1 + 1;
        let carry = !carry + 1;
        let result = operand1 + operand2 + carry;

        self.set_arithmetic_flags(result, operand1, operand2, carry, set_flags);
        self.set_register(rd, result);
        self.set_executed_instruction(format!(
            "RSC {:#x} {:#x} {:#x} {:#x}",
            rd, operand1, operand2, carry
        ));
    }

    pub fn arm_tst(&mut self, rd: REGISTER, operand1: u32, operand2: u32, set_flags: bool) {
        let result = operand1 & operand2;

        self.set_logical_flags(result, true);
        self.set_executed_instruction(format!("TST {:#x} {:#x} {:#x}", rd, operand1, operand2));
    }

    pub fn arm_teq(&mut self, rd: REGISTER, operand1: u32, operand2: u32, set_flags: bool) {
        let result = operand1 ^ operand2;

        self.set_logical_flags(result, true);
        self.set_executed_instruction(format!("TEQ {:#x} {:#x} {:#x}", rd, operand1, operand2));
    }

    pub fn arm_cmp(&mut self, rd: REGISTER, operand1: u32, operand2: u32, set_flags: bool) {}

    pub fn arm_cmn(&mut self, rd: REGISTER, operand1: u32, operand2: u32, set_flags: bool) {}

    pub fn arm_orr(&mut self, rd: REGISTER, operand1: u32, operand2: u32, set_flags: bool) {
        let result = operand1 | operand2;

        self.set_logical_flags(result, set_flags);
        self.set_register(rd, result);
        self.set_executed_instruction(format!("ORR {:#x} {:#x} {:#x}", rd, operand1, operand2));
    }

    #[allow(unused)]
    pub fn arm_mov(&mut self, rd: REGISTER, operand1: u32, operand2: u32, set_flags: bool) {
        self.set_register(rd, operand2);
        self.set_logical_flags(operand2, set_flags);
        self.set_executed_instruction(format!("MOV {:#x} {:#x}", rd, operand2));
    }

    pub fn arm_bic(&mut self, rd: REGISTER, operand1: u32, operand2: u32, set_flags: bool) {
        let result = operand1 & !operand2;

        self.set_logical_flags(result, set_flags);
        self.set_register(rd, result);
        self.set_executed_instruction(format!("BIC {:#x} {:#x} {:#x}", rd, operand1, operand2));
    }

    #[allow(unused)]
    pub fn arm_mvn(&mut self, rd: REGISTER, operand1: u32, operand2: u32, set_flags: bool) {
        let result = !operand2;
        self.set_register(rd, result);
        self.set_logical_flags(result, set_flags);
        self.set_executed_instruction(format!("MVN {:#x} {:#x}", rd, operand2));
    }

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

    fn set_arithmetic_flags(&mut self, result: WORD, operand1: u32, operand2: u32, carry: u32, set_flags: bool) {
        if set_flags == true {
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
            if result < operand1 || result < operand2 || result < carry {
                self.set_flag(FlagsRegister::C);
            } else {
                self.reset_flag(FlagsRegister::C);
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

        cpu.execute_cpu_cycle(); // pc == 8
        cpu.execute_cpu_cycle(); // pc == 12
        cpu.execute_cpu_cycle(); // pc == 16
        assert!(cpu.get_register(1) == (test_pc + 8) << 1);
        assert!(cpu.get_flag(FlagsRegister::C) == 0);
        assert!(cpu.get_flag(FlagsRegister::N) == 0);
        assert!(cpu.get_flag(FlagsRegister::Z) == 0);
        assert!(cpu.get_flag(FlagsRegister::V) == 0);
    }

    #[test]
    fn data_processing_with_pc_as_operand1_and_register_shift_delays_pc() {
        let memory = Memory::new().unwrap();
        let memory = Arc::new(Mutex::new(memory));
        let mut cpu = CPU::new(memory);

        cpu.fetched_instruction = 0xe09f1314; //  adds r1, pc, r4, lsl r3; pc = 0

        cpu.set_register(3, 0x01);
        cpu.set_register(4, 0);
        let test_pc = 4; // points at next instruction
        cpu.set_pc(test_pc);

        cpu.execute_cpu_cycle();
        cpu.execute_cpu_cycle();
        cpu.execute_cpu_cycle();
        assert!(cpu.get_register(1) == test_pc + 8);
        assert!(cpu.get_flag(FlagsRegister::C) == 0);
        assert!(cpu.get_flag(FlagsRegister::N) == 0);
        assert!(cpu.get_flag(FlagsRegister::Z) == 0);
        assert!(cpu.get_flag(FlagsRegister::V) == 0);
    }

    #[test]
    fn data_processing_with_pc_as_destination_should_start_from_result() {
        let memory = Memory::new().unwrap();
        let cpu_memory = Arc::new(Mutex::new(memory));
        let mem = Arc::clone(&cpu_memory);
        let mut cpu = CPU::new(cpu_memory);

        let _res = mem.lock().unwrap().writeu32(0x0, 0xe2931002);
        let _res = mem.lock().unwrap().writeu32(0x4, 0xe2931002);
        let _res = mem.lock().unwrap().writeu32(0x8, 0xe1a00000); // nop
        let _res = mem.lock().unwrap().writeu32(0xC, 0xe1a00000); // nop
        let _res = mem.lock().unwrap().writeu32(0x10, 0xe1a00000); // nop
        let _res = mem.lock().unwrap().writeu32(0x14, 0xe091f001);

        cpu.execute_cpu_cycle();
        cpu.execute_cpu_cycle();
        cpu.execute_cpu_cycle();
        cpu.execute_cpu_cycle();
        cpu.execute_cpu_cycle();
        cpu.execute_cpu_cycle();
        cpu.execute_cpu_cycle();
        cpu.execute_cpu_cycle();
        assert!(cpu.fetched_instruction == 0xe2931002);
    }

    #[test]
    fn mov_instruction_should_set_n_flag() {
        let memory = Memory::new().unwrap();
        let cpu_memory = Arc::new(Mutex::new(memory));
        let mut cpu = CPU::new(cpu_memory);

        cpu.set_register(3, 0x8001_0002);

        cpu.fetched_instruction = 0xe1b04003; // mov r4, r3;

        cpu.execute_cpu_cycle();
        cpu.execute_cpu_cycle();
        assert!(cpu.get_flag(FlagsRegister::C) == 0);
        assert!(cpu.get_flag(FlagsRegister::N) == 1);
        assert!(cpu.get_flag(FlagsRegister::Z) == 0);
        assert!(cpu.get_flag(FlagsRegister::V) == 0);
        assert!(cpu.get_register(4) == cpu.get_register(3));
        assert!(cpu.get_register(4) == 0x8001_0002);
    }

    #[test]
    fn mvn_instruction_should_set_z_flag() {
        let memory = Memory::new().unwrap();
        let cpu_memory = Arc::new(Mutex::new(memory));
        let mut cpu = CPU::new(cpu_memory);

        let input = 0xFFFF_FFFF;
        cpu.set_register(4, input);

        cpu.fetched_instruction = 0xe1f05004; // mvn r5, r4;

        cpu.execute_cpu_cycle();
        cpu.execute_cpu_cycle();
        assert!(cpu.get_flag(FlagsRegister::C) == 0);
        assert!(cpu.get_flag(FlagsRegister::N) == 0);
        assert!(cpu.get_flag(FlagsRegister::Z) == 1);
        assert!(cpu.get_flag(FlagsRegister::V) == 0);
        assert!(cpu.get_register(5) == !cpu.get_register(4));
        assert!(cpu.get_register(5) == !input);
    }

    #[test]
    fn adc_instruction_should_add_2_registers_and_carry() {
        let memory = Memory::new().unwrap();
        let cpu_memory = Arc::new(Mutex::new(memory));
        let mut cpu = CPU::new(cpu_memory);

        cpu.set_register(1, 25);
        cpu.set_register(2, 32);
        cpu.set_flag(FlagsRegister::C);

        cpu.fetched_instruction = 0xe0b14002; // adcs r4, r2, r1;

        cpu.execute_cpu_cycle();
        cpu.execute_cpu_cycle();
        assert!(cpu.get_flag(FlagsRegister::C) == 0);
        assert!(cpu.get_flag(FlagsRegister::N) == 0);
        assert!(cpu.get_flag(FlagsRegister::Z) == 0);
        assert!(cpu.get_flag(FlagsRegister::V) == 0);
        assert!(cpu.get_register(4) == 58);
    }

    #[test]
    fn adc_instruction_should_set_carry_register() {
        let memory = Memory::new().unwrap();
        let cpu_memory = Arc::new(Mutex::new(memory));
        let mut cpu = CPU::new(cpu_memory);

        cpu.set_register(1, 0xFFFF_FFFF);
        cpu.set_register(2, 0x0);
        cpu.set_flag(FlagsRegister::C);

        cpu.fetched_instruction = 0xe0b14002; // adcs r4, r2, r1;

        cpu.execute_cpu_cycle();
        cpu.execute_cpu_cycle();
        assert!(cpu.get_flag(FlagsRegister::C) == 1);
        assert!(cpu.get_flag(FlagsRegister::N) == 0);
        assert!(cpu.get_flag(FlagsRegister::Z) == 1);
        assert!(cpu.get_flag(FlagsRegister::V) == 0);
        assert!(cpu.get_register(4) == 0x0000_0000);
    }

    #[test]
    fn adc_instruction_should_set_v_register() {
        let memory = Memory::new().unwrap();
        let cpu_memory = Arc::new(Mutex::new(memory));
        let mut cpu = CPU::new(cpu_memory);

        cpu.set_register(1, 0x8000_0000);
        cpu.set_register(2, 0x8FFF_FFFF);
        cpu.set_flag(FlagsRegister::C);

        cpu.fetched_instruction = 0xe0b14002; // adcs r4, r2, r1;

        cpu.execute_cpu_cycle();
        cpu.execute_cpu_cycle();
        assert!(cpu.get_flag(FlagsRegister::C) == 1);
        assert!(cpu.get_flag(FlagsRegister::N) == 0);
        assert!(cpu.get_flag(FlagsRegister::Z) == 0);
        assert!(cpu.get_flag(FlagsRegister::V) == 1);
        assert!(cpu.get_register(4) == 0x1000_0000);
    }

    #[test]
    fn sub_instruction_should_set_v_flag() {
        let memory = Memory::new().unwrap();
        let cpu_memory = Arc::new(Mutex::new(memory));
        let mut cpu = CPU::new(cpu_memory);

        cpu.set_register(1, 0x7FFF_FFFF);
        cpu.set_register(2, 0xFFFF_FFFF); // twos complement of -1

        cpu.fetched_instruction = 0xe0514002; // subs r4, r1, r2;

        cpu.execute_cpu_cycle();
        cpu.execute_cpu_cycle();
        assert!(cpu.get_flag(FlagsRegister::C) == 0);
        assert!(cpu.get_flag(FlagsRegister::N) == 1);
        assert!(cpu.get_flag(FlagsRegister::Z) == 0);
        assert!(cpu.get_flag(FlagsRegister::V) == 1);
        assert!(cpu.get_register(4) == 0x8000_0000);
    }

    #[test]
    fn sub_instruction_should_reset_c_flag() {
        let memory = Memory::new().unwrap();
        let cpu_memory = Arc::new(Mutex::new(memory));
        let mut cpu = CPU::new(cpu_memory);

        cpu.set_register(1, 5);
        cpu.set_register(2, 10);

        cpu.fetched_instruction = 0xe0514002; // subs r4, r1, r2;

        cpu.execute_cpu_cycle();
        cpu.execute_cpu_cycle();
        assert!(cpu.get_register(4) == 0xFFFF_FFFB);
        assert!(cpu.get_flag(FlagsRegister::C) == 0);
        assert!(cpu.get_flag(FlagsRegister::N) == 1);
        assert!(cpu.get_flag(FlagsRegister::Z) == 0);
        assert!(cpu.get_flag(FlagsRegister::V) == 0);
    }

    #[test]
    fn sub_instruction_should_set_c_flag() {
        let memory = Memory::new().unwrap();
        let cpu_memory = Arc::new(Mutex::new(memory));
        let mut cpu = CPU::new(cpu_memory);

        cpu.set_register(1, 10);
        cpu.set_register(2, 5);

        cpu.fetched_instruction = 0xe0514002; // subs r4, r1, r2;

        cpu.execute_cpu_cycle();
        cpu.execute_cpu_cycle();
        assert!(cpu.get_register(4) == 0x5);
        assert!(cpu.get_flag(FlagsRegister::C) == 1);
        assert!(cpu.get_flag(FlagsRegister::N) == 0);
        assert!(cpu.get_flag(FlagsRegister::Z) == 0);
        assert!(cpu.get_flag(FlagsRegister::V) == 0);
    }
}
