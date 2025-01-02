use crate::{
    types::{ARMByteCode, CYCLES, REGISTER, WORD},
    utils::bits::Bits,
};

use super::{
    cpu::{CPUMode, FlagsRegister, CPU, PC_REGISTER},
    instructions::ALUOperation,
};

impl CPU {
    pub fn data_processing_instruction(&mut self, instruction: ARMByteCode) -> CYCLES {
        let shift_amount;
        let mut cycles = 1;
        if instruction.bit_is_set(25) {
            shift_amount = ((instruction & 0x0000_0F00) >> 8) * 2;
        } else {
            // The first cycle gets the register we shift by
            // The rest of the operation happens on the next cycle in an I cycle
            if instruction.bit_is_set(4) {
                // shift by register
                self.advance_pipeline();
                cycles += 1;
                let shift_register = (instruction & 0x0000_0F00) >> 8;
                shift_amount = self.get_register(shift_register);
            } else {
                shift_amount = (instruction & 0x0000_0F80) >> 7;
            }
        }
        let opcode = (instruction & 0x01E0_0000) >> 21;
        let operation: ALUOperation = match opcode {
            0x0 => CPU::arm_and,
            0x1 => CPU::arm_eor,
            0x2 => CPU::arm_sub,
            0x3 => CPU::arm_rsb,
            0x4 => CPU::arm_add,
            0x5 => CPU::arm_adc,
            0x6 => CPU::arm_sbc,
            0x7 => CPU::arm_rsc,
            0x8 => {
                if instruction.bit_is_set(20) {
                    CPU::arm_tst
                } else {
                    return self.arm_mrs(instruction);
                }
            }
            0x9 => {
                if instruction.bit_is_set(20) {
                    CPU::arm_teq
                } else {
                    return self.arm_msr(instruction);
                }
            }
            0xa => {
                if instruction.bit_is_set(20) {
                    CPU::arm_cmp
                } else {
                    return self.arm_mrs(instruction);
                }
            }
            0xb => {
                if instruction.bit_is_set(20) {
                    CPU::arm_cmn
                } else {
                    return self.arm_msr(instruction);
                }
            }
            0xc => CPU::arm_orr,
            0xd => CPU::arm_mov,
            0xe => CPU::arm_bic,
            0xf => CPU::arm_mvn,
            _ => panic!("Impossible to decode opcode"),
        };

        let rn = (0x000F_0000 & instruction) >> 16;
        let rd = (0x0000_F000 & instruction) >> 12;

        let set_flags = instruction.bit_is_set(20) && rd != PC_REGISTER as u32;
        if rd == 15 && instruction.bit_is_set(20) {
            if let Some(spsr) = self.get_current_spsr() {
                self.cpsr = *spsr;
            }
        }
        let operand2 = self.decode_operand2(instruction, set_flags, shift_amount);
        operation(self, rd, self.get_register(rn), operand2, set_flags);
        if rd == 15 {
            cycles += self.flush_pipeline();
        }
        return cycles;
    }

    fn decode_operand2(
        &mut self,
        instruction: ARMByteCode,
        set_flags: bool,
        shift_amount: u32,
    ) -> u32 {
        if instruction.bit_is_set(25) {
            // operand 2 is immediate
            let immediate = instruction & 0x0000_00FF;

            return immediate.rotate_right(shift_amount);
        }
        let operand_register = instruction & 0x0000_000F;
        let operand_register_value = self.get_register(operand_register);
        return self.decode_shifted_register(
            instruction,
            shift_amount,
            operand_register_value,
            set_flags,
        );
    }

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
        let operand2 = operand2.twos_complement();
        let result = operand1 + operand2; // use two's complement to make setting flags easier

        self.set_arithmetic_flags(result, operand1, operand2, 1, set_flags);
        self.set_register(rd, result);

        self.set_executed_instruction(format!("SUB {:#x} {:#x} {:#x}", rd, operand1, operand2));
    }

    pub fn arm_rsb(&mut self, rd: REGISTER, operand1: u32, operand2: u32, set_flags: bool) {
        let operand1 = operand1.twos_complement();
        let result = operand2 + operand1; // use two's complement to make setting flags easier

        self.set_arithmetic_flags(result, operand1, operand2, 0, set_flags);
        self.set_register(rd, result);

        self.set_executed_instruction(format!("RSB {:#x} {:#x} {:#x}", rd, operand1, operand2));
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
        let operand2 = operand2.twos_complement();
        let carry = carry.twos_complement();
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
        let operand1 = operand2.twos_complement();
        let carry = carry.twos_complement();
        let result = operand1 + operand2 + carry;

        self.set_arithmetic_flags(result, operand1, operand2, carry, set_flags);
        self.set_register(rd, result);
        self.set_executed_instruction(format!(
            "RSC {:#x} {:#x} {:#x} {:#x}",
            rd, operand1, operand2, carry
        ));
    }

    #[allow(unused)]
    pub fn arm_tst(&mut self, rd: REGISTER, operand1: u32, operand2: u32, set_flags: bool) {
        let result = operand1 & operand2;

        self.set_logical_flags(result, true);
        self.set_executed_instruction(format!("TST {:#x} {:#x} {:#x}", rd, operand1, operand2));
    }

    #[allow(unused)]
    pub fn arm_teq(&mut self, rd: REGISTER, operand1: u32, operand2: u32, set_flags: bool) {
        let result = operand1 ^ operand2;

        self.set_logical_flags(result, true);
        self.set_executed_instruction(format!("TEQ {:#x} {:#x} {:#x}", rd, operand1, operand2));
    }

    #[allow(unused)]
    pub fn arm_cmp(&mut self, rd: REGISTER, operand1: u32, operand2: u32, set_flags: bool) {
        self.set_executed_instruction(format!("CMP {:#x} {:#x}", operand1, operand2));
        let operand2 = !operand2 + 1;
        let result = operand1 + operand2; // use two's complement to make setting flags easier

        self.set_arithmetic_flags(result, operand1, operand2, 1, true);
    }

    #[allow(unused)]
    pub fn arm_cmn(&mut self, rd: REGISTER, operand1: u32, operand2: u32, set_flags: bool) {
        let result = operand1 + operand2;
        self.set_arithmetic_flags(result, operand1, operand2, 0, true);
        self.set_executed_instruction(format!("CMN {:#x} {:#x}", operand1, operand2));
    }

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

    pub fn arm_mrs(&mut self, instruction: ARMByteCode) -> CYCLES {
        let rd = (instruction & 0x0000_F000) >> 12;
        let source_psr = if instruction.bit_is_set(22) {
            match self.get_current_spsr() {
                Some(spsr) => *spsr,
                None => {
                    return 1;
                }
            }
        } else {
            self.cpsr
        };

        self.set_register(rd, source_psr);
        let psr = if instruction.bit_is_set(22) {
            "SPSR"
        } else {
            "CPSR"
        };

        self.set_executed_instruction(format!("MRS {} {}", rd, psr));
        1
    }

    pub fn arm_msr(&mut self, instruction: ARMByteCode) -> CYCLES {
        const FLG_MASK: u32 = 0xFF00_0000;
        const CTL_MASK: u32 = 0x0000_00DF; // can't assign T-bit with this operation
        let current_cpu_mode = self.get_cpu_mode();

        let operand = if instruction.bit_is_set(25) {
            // lower 8 bits rotated right by bits instruction[11:8] * 2
            (instruction & 0x0000_00FF).rotate_right((instruction & 0x0000_0F00) >> 7)
        } else {
            self.get_register(instruction & 0x0000_000F)
        };

        let destination_psr: &mut u32 = if instruction.bit_is_set(22) {
            match self.get_current_spsr() {
                Some(spsr) => spsr,
                None => {
                    return 1;
                }
            }
        } else {
            &mut self.cpsr
        };

        if instruction.bit_is_set(19) {
            (*destination_psr) &= !FLG_MASK;
            (*destination_psr) |= operand & FLG_MASK;
        }

        if instruction.bit_is_set(16) && !matches!(current_cpu_mode, CPUMode::USER) {
            (*destination_psr) &= !CTL_MASK;
            (*destination_psr) |= operand & CTL_MASK;
        }

        let updated_psr = if instruction.bit_is_set(22) {
            "SPSR"
        } else {
            "CPSR"
        };

        self.set_executed_instruction(format!("MSR {} {:#x}", updated_psr, operand));

        1
    }

    pub fn set_logical_flags(&mut self, result: WORD, set_flags: bool) {
        if set_flags == true {
            self.set_flag_from_bit(FlagsRegister::N, result.get_bit(31) as u8);
            if result == 0 {
                self.set_flag(FlagsRegister::Z);
            } else {
                self.reset_flag(FlagsRegister::Z);
            }
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
    

    use rstest::rstest;

    use crate::{
        arm7tdmi::cpu::{CPUMode, FlagsRegister, CPU},
        memory::memory::{ GBAMemory},
        types::REGISTER,
    };

    #[test]
    fn add_instruction_should_set_carry_flag() {
        let memory = GBAMemory::new();
        
        let mut cpu = CPU::new(memory);

        cpu.set_register(2, u32::MAX);
        cpu.set_register(3, 2);

        cpu.prefetch[0] = Some(0xe0931002); // adds r1, r3, r2;

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
        let memory = GBAMemory::new();
        
        let mut cpu = CPU::new(memory);

        cpu.set_register(2, 0x8000_0000);
        cpu.set_register(3, 0x8000_0000);

        cpu.prefetch[0] = Some(0xe0931002); // adds r1, r3, r2;

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
        let memory = GBAMemory::new();
        
        let mut cpu = CPU::new(memory);

        cpu.set_register(2, 0x8000_0000);
        cpu.set_register(3, 0x0000_0001);

        cpu.prefetch[0] = Some(0xe0931002); // adds r1, r3, r2;

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
        let memory = GBAMemory::new();
        
        let mut cpu = CPU::new(memory);

        cpu.set_register(2, 0x0000_FFFF);
        cpu.set_register(3, 0x0000_0001);

        cpu.prefetch[0] = Some(0xe01312a2); // ands r1, r3, r2 LSR 5;

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
        let memory = GBAMemory::new();
        
        let mut cpu = CPU::new(memory);

        cpu.set_register(2, 0x8000_FFFF);
        cpu.set_register(3, 0x8000_0001);

        cpu.prefetch[0] = Some(0xe0131002); // ands r1, r3, r2;

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
        let memory = GBAMemory::new();
        
        let mut cpu = CPU::new(memory);

        cpu.set_register(2, 0x8000_FFFF);
        cpu.set_register(3, 0x0000_0000);

        cpu.prefetch[0] = Some(0xe0131002); // ands r1, r3, r2;

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
        let memory = GBAMemory::new();
        
        let mut cpu = CPU::new(memory);

        cpu.set_register(2, 0x0000_0000);
        cpu.set_register(3, 0x0000_0000);

        cpu.prefetch[0] = Some(0xe1931002); // orrs r1, r3, r2;

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
        let memory = GBAMemory::new();
        
        let mut cpu = CPU::new(memory);

        cpu.set_register(2, 0x0000_0000);
        cpu.set_register(3, 0x0000_0000);

        cpu.prefetch[0] = Some(0xe1831002); // orr r1, r3, r2;

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
        let memory = GBAMemory::new();
        
        let mut cpu = CPU::new(memory);

        cpu.set_register(2, 0x8001_0002);
        cpu.set_register(3, 0x1000_0010);

        cpu.prefetch[0] = Some(0xe0331002); // eors r1, r3, r2;

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
        let memory = GBAMemory::new();
        
        let mut cpu = CPU::new(memory);

        cpu.set_register(2, 0x8001_0002);
        cpu.set_register(3, 0x1000_0010);

        cpu.prefetch[0] = Some(0xe1330002); // teq r3, r2;

        cpu.execute_cpu_cycle();
        cpu.execute_cpu_cycle();
        assert!(cpu.get_flag(FlagsRegister::C) == 0);
        assert!(cpu.get_flag(FlagsRegister::N) == 1);
        assert!(cpu.get_flag(FlagsRegister::Z) == 0);
        assert!(cpu.get_flag(FlagsRegister::V) == 0);
    }

    #[test]
    fn teq_instruction_should_set_z_flag_when_equal() {
        let memory = GBAMemory::new();
        
        let mut cpu = CPU::new(memory);

        cpu.set_register(2, 0x8001_0002);
        cpu.set_register(3, 0x8001_0002);

        cpu.prefetch[0] = Some(0xe1330002); // teq r3, r2;

        cpu.execute_cpu_cycle();
        cpu.execute_cpu_cycle();
        assert!(cpu.get_flag(FlagsRegister::C) == 0);
        assert!(cpu.get_flag(FlagsRegister::N) == 0);
        assert!(cpu.get_flag(FlagsRegister::Z) == 1);
        assert!(cpu.get_flag(FlagsRegister::V) == 0);
    }

    #[test]
    fn tst_instruction_should_set_z_flag_when_no_bits_match() {
        let memory = GBAMemory::new();
        
        let mut cpu = CPU::new(memory);

        cpu.set_register(2, 0x8001_0002);
        cpu.set_register(3, 0x0110_2224);

        cpu.prefetch[0] = Some(0xe1130002); // tst r3, r2;

        cpu.execute_cpu_cycle();
        cpu.execute_cpu_cycle();
        assert!(cpu.get_flag(FlagsRegister::C) == 0);
        assert!(cpu.get_flag(FlagsRegister::N) == 0);
        assert!(cpu.get_flag(FlagsRegister::Z) == 1);
        assert!(cpu.get_flag(FlagsRegister::V) == 0);
    }

    #[test]
    fn bic_instruction_should_reset_all_bits() {
        let memory = GBAMemory::new();
        
        let mut cpu = CPU::new(memory);

        cpu.set_register(3, 0x8001_0002);
        cpu.set_register(2, 0x80F1_0102);

        cpu.prefetch[0] = Some(0xe1d31002); // bics r1, r3, r2;

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
        let memory = GBAMemory::new();
        
        let mut cpu = CPU::new(memory);

        cpu.prefetch[0] = Some(0xe094131f); // adds r1, r3, r15, LSL r3; pc = 0

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
        let memory = GBAMemory::new();
        
        let mut cpu = CPU::new(memory);

        cpu.prefetch[0] = Some(0xe09f1314); //  adds r1, pc, r4, lsl r3; pc = 0

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
        let memory = GBAMemory::new();
        
        
        let mut cpu = CPU::new(memory);

        let _res = cpu.memory
            .writeu32(0x3000000, 0xe25f1008);
        let _res = cpu.memory
            .writeu32(0x3000004, 0xe1a00000);
        let _res = cpu.memory
            .writeu32(0x3000008, 0xe1a00000); // nop
        let _res = cpu.memory
            .writeu32(0x300000C, 0xe1a00000); // nop
        let _res = cpu.memory
            .writeu32(0x3000010, 0xe1a00000); // nop
        let _res = cpu.memory
            .writeu32(0x3000014, 0xe281f000);

        cpu.set_pc(0x3000000);
        cpu.execute_cpu_cycle();
        cpu.execute_cpu_cycle();
        cpu.execute_cpu_cycle();
        cpu.execute_cpu_cycle();
        cpu.execute_cpu_cycle();
        cpu.execute_cpu_cycle();
        cpu.execute_cpu_cycle();
        cpu.execute_cpu_cycle();
        assert_eq!(cpu.decode_instruction(cpu.prefetch[1].unwrap()).instruction, 0xe25f1008);
    }

    #[test]
    fn mov_instruction_should_set_n_flag() {
        let memory = GBAMemory::new();
        
        let mut cpu = CPU::new(memory);

        cpu.set_register(3, 0x8001_0002);

        cpu.prefetch[0] = Some(0xe1b04003); // mov r4, r3;

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
        let memory = GBAMemory::new();
        
        let mut cpu = CPU::new(memory);

        let input = 0xFFFF_FFFF;
        cpu.set_register(4, input);

        cpu.prefetch[0] = Some(0xe1f05004); // mvn r5, r4;

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
        let memory = GBAMemory::new();
        
        let mut cpu = CPU::new(memory);

        cpu.set_register(1, 25);
        cpu.set_register(2, 32);
        cpu.set_flag(FlagsRegister::C);

        cpu.prefetch[0] = Some(0xe0b14002); // adcs r4, r2, r1;

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
        let memory = GBAMemory::new();
        
        let mut cpu = CPU::new(memory);

        cpu.set_register(1, 0xFFFF_FFFF);
        cpu.set_register(2, 0x0);
        cpu.set_flag(FlagsRegister::C);

        cpu.prefetch[0] = Some(0xe0b14002); // adcs r4, r2, r1;

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
        let memory = GBAMemory::new();
        
        let mut cpu = CPU::new(memory);

        cpu.set_register(1, 0x8000_0000);
        cpu.set_register(2, 0x8FFF_FFFF);
        cpu.set_flag(FlagsRegister::C);

        cpu.prefetch[0] = Some(0xe0b14002); // adcs r4, r2, r1;

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
        let memory = GBAMemory::new();
        
        let mut cpu = CPU::new(memory);

        cpu.set_register(1, 0x7FFF_FFFF);
        cpu.set_register(2, 0xFFFF_FFFF); // twos complement of -1

        cpu.prefetch[0] = Some(0xe0514002); // subs r4, r1, r2;

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
        let memory = GBAMemory::new();
        
        let mut cpu = CPU::new(memory);

        cpu.set_register(1, 5);
        cpu.set_register(2, 10);

        cpu.prefetch[0] = Some(0xe0514002); // subs r4, r1, r2;

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
        let memory = GBAMemory::new();
        
        let mut cpu = CPU::new(memory);

        cpu.set_register(1, 10);
        cpu.set_register(2, 5);

        cpu.prefetch[0] = Some(0xe0514002); // subs r4, r1, r2;

        cpu.execute_cpu_cycle();
        cpu.execute_cpu_cycle();
        assert!(cpu.get_register(4) == 0x5);
        assert!(cpu.get_flag(FlagsRegister::C) == 1);
        assert!(cpu.get_flag(FlagsRegister::N) == 0);
        assert!(cpu.get_flag(FlagsRegister::Z) == 0);
        assert!(cpu.get_flag(FlagsRegister::V) == 0);
    }

    #[rstest]
    #[case(0xe10f2000, 0x000000d3, 2, 0x000000d3)]
    #[case(0xe10f2000, 0x330000d3, 2, 0x330000d3)]
    fn mrs_should_move_instruction_from_psr_to_destination_reg(
        #[case] opcode: u32,
        #[case] cpsr: u32,
        #[case] expected_dst: REGISTER,
        #[case] expected_val: u32,
    ) {
        let memory = GBAMemory::new();
        
        let mut cpu = CPU::new(memory);

        cpu.cpsr = cpsr;

        cpu.prefetch[0] = Some(opcode);
        cpu.execute_cpu_cycle();
        cpu.execute_cpu_cycle();

        assert_eq!(cpu.get_register(expected_dst), expected_val);
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
        let memory = GBAMemory::new();
        
        let mut cpu = CPU::new(memory);

        cpu.set_mode(mode);
        cpu.set_register(register, psr_val);

        cpu.prefetch[0] = Some(opcode);
        cpu.execute_cpu_cycle();
        cpu.execute_cpu_cycle();

        assert_eq!(cpu.cpsr, expected_val);
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
        let memory = GBAMemory::new();
        
        let mut cpu = CPU::new(memory);

        cpu.set_mode(mode);
        cpu.set_register(register, psr_val);

        cpu.prefetch[0] = Some(opcode);
        cpu.execute_cpu_cycle();
        cpu.execute_cpu_cycle();

        assert_eq!(*cpu.get_current_spsr().unwrap(), expected_val);
    }

    #[rstest]
    #[case(0xe329f0d0, CPUMode::SVC, 0x000000d0)]  // msr CPSR, 0x24
    #[case(0xe328f20d, CPUMode::SVC, 0xd00000d3)]  // msr CPSR, 0xd0000000
    fn msr_should_move_imm_to_cpsr(
        #[case] opcode: u32,
        #[case] mode: CPUMode,
        #[case] expected_val: u32,
    ) {
        let memory = GBAMemory::new();
        
        let mut cpu = CPU::new(memory);

        cpu.set_mode(mode);

        cpu.prefetch[0] = Some(opcode);
        cpu.execute_cpu_cycle();
        cpu.execute_cpu_cycle();

        assert_eq!(cpu.cpsr, expected_val);
    }
}
