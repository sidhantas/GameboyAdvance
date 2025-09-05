use std::fmt::{Arguments, Write};

use crate::{
    arm7tdmi::{
        cpu::{FlagsRegister, InstructionMode, CPU, LINK_REGISTER},
        interrupts::Exceptions,
    },
    memory::memory::GBAMemory,
    types::{ARMByteCode, CYCLES, REGISTER},
    utils::bits::{sign_extend, Bits},
};

pub type ARMExecutable = fn(&mut CPU, ARMByteCode, memory: &mut GBAMemory) -> CYCLES;
pub type ALUOperation =
    fn(&mut CPU, rd: REGISTER, operand1: u32, operand2: u32, set_flags: bool) -> ();

#[derive(Clone, Copy)]
pub struct ARMDecodedInstruction {
    pub executable: ARMExecutable,
    pub instruction: u32,
}

impl Default for ARMDecodedInstruction {
    fn default() -> Self {
        ARMDecodedInstruction {
            executable: CPU::arm_nop,
            instruction: 0,
        }
    }
}

impl CPU {
    pub fn set_executed_instruction(&mut self, name: Arguments<'_>) {
       //self.executed_instruction.clear();
       //write!(self.executed_instruction, "{}", name).unwrap();
    }

    pub fn arm_branch(&mut self, instruction: ARMByteCode, memory: &mut GBAMemory) -> CYCLES {
        let mut cycles = 1;
        let offset = instruction & 0x00FF_FFFF;
        let offset = sign_extend(offset << 2, 25);
        let destination = offset + self.get_pc();
        self.set_pc(destination);
        cycles += self.flush_pipeline(memory);
        self.set_executed_instruction(format_args!("B {:#010x}", destination));

        cycles
    }

    pub fn arm_branch_and_link(&mut self, instruction: ARMByteCode, memory: &mut GBAMemory) -> CYCLES {
        let mut cycles = 1;
        self.set_register(LINK_REGISTER, self.get_pc() - 4);
        let offset = instruction & 0x00FF_FFFF;
        let offset = sign_extend(offset << 2, 25);
        let destination = offset + self.get_pc();
        self.set_pc(destination);
        cycles += self.flush_pipeline(memory);
        self.set_executed_instruction(format_args!("B {:#010x}", destination));

        cycles
    }

    pub fn arm_nop(&mut self, _instruction: ARMByteCode, memory: &mut GBAMemory) -> CYCLES {
        self.set_executed_instruction(format_args!("NOP"));
        return 0;
    }

    pub fn arm_multiply(&mut self, instruction: ARMByteCode, memory: &mut GBAMemory) -> CYCLES {
        let rd = (instruction & 0x000F_0000) >> 16;
        let rs = (instruction & 0x0000_0F00) >> 8;
        let rm = instruction & 0x0000_000F;
        let set_flags = instruction.bit_is_set(20);

        let operand1 = self.get_register(rm) as u64;
        let operand2 = self.get_register(rs) as u64;

        let result = (operand1 * operand2) as u32;
        self.set_register(rd, result);

        self.set_flag_from_bit(FlagsRegister::N, result.get_bit(31) as u8);
        if set_flags {
            if result == 0 {
                self.reset_flag(FlagsRegister::Z);
            } else {
                self.set_flag(FlagsRegister::Z);
            }
            self.reset_flag(FlagsRegister::C);
        }

        self.set_executed_instruction(format_args!("MUL {} {} {}", rd, rm, rs));
        if operand2 & 0xFFFF_FF00 == 0 || operand2 & 0xFFFF_FF00 == 0xFFFF_FF00 {
            1
        } else if operand2 & 0xFFFF_0000 == 0 || operand2 & 0xFFFF_0000 == 0xFFFF_0000 {
            2
        } else if operand2 & 0xFF00_0000 == 0 || operand2 & 0xFF00_0000 == 0xFFFF_0000 {
            3
        } else {
            4
        }
    }

    pub fn arm_multiply_accumulate(
        &mut self,
        instruction: ARMByteCode,
        memory: &mut GBAMemory,
    ) -> CYCLES {
        let rd = (instruction & 0x000F_0000) >> 16;
        let rn = (instruction & 0x0000_F000) >> 12;
        let rs = (instruction & 0x0000_0F00) >> 8;
        let rm = instruction & 0x0000_000F;
        let set_flags = instruction.bit_is_set(20);

        let operand1 = self.get_register(rm) as u64;
        let operand2 = self.get_register(rs) as u64;
        let acc = self.get_register(rn) as u64;

        let result = (operand1 * operand2 + acc) as u32;
        if set_flags {
            self.set_flag_from_bit(FlagsRegister::N, result.get_bit(31) as u8);
            if result == 0 {
                self.reset_flag(FlagsRegister::Z);
            } else {
                self.set_flag(FlagsRegister::Z);
            }
            self.reset_flag(FlagsRegister::C);
        }
        self.set_executed_instruction(format_args!("MUL {} {} {} {}", rd, rn, rm, rs));
        if operand2 & 0xFFFF_FF00 == 0 || operand2 & 0xFFFF_FF00 == 0xFFFF_FF00 {
            2
        } else if operand2 & 0xFFFF_0000 == 0 || operand2 & 0xFFFF_0000 == 0xFFFF_0000 {
            3
        } else if operand2 & 0xFF00_0000 == 0 || operand2 & 0xFF00_0000 == 0xFFFF_0000 {
            4
        } else {
            5
        }
    }

    pub fn arm_multiply_long(
        &mut self,
        instruction: ARMByteCode,
        memory: &mut GBAMemory,
    ) -> CYCLES {
        todo!();
    }

    pub fn arm_software_interrupt(
        &mut self,
        _instruction: ARMByteCode,
        memory: &mut GBAMemory,
    ) -> CYCLES {
        let mut cycles = 1;
        cycles += self.raise_exception(Exceptions::Software, memory);
        self.set_executed_instruction(format_args!("SWI"));

        return cycles;
    }

    pub fn arm_branch_and_exchange(
        &mut self,
        instruction: ARMByteCode,
        memory: &mut GBAMemory,
    ) -> CYCLES {
        let mut destination = self.get_register(instruction & 0x0000_000F);
        let mut cycles = 1;
        if destination.bit_is_set(0) {
            self.set_instruction_mode(InstructionMode::THUMB);
        } else {
            destination &= !2; // arm instructions must be word aligned
            self.set_instruction_mode(InstructionMode::ARM);
        }
        self.set_pc(destination & !1); // bit 0 is forced to 0 before storing
        cycles += self.flush_pipeline(memory);
        self.set_executed_instruction(format_args!("BX {:#010x}", destination));

        cycles
    }

    pub fn arm_not_implemented(
        &mut self,
        instruction: ARMByteCode,
        memory: &mut GBAMemory,
    ) -> CYCLES {
        self.set_executed_instruction(format_args!("NOT IMPLEMENTED"));
        panic!("NOT IMPLEMENTED: {:#X}", instruction);
    }
}

#[cfg(test)]
mod instruction_tests {

    use crate::{
        arm7tdmi::cpu::{CPUMode, LINK_REGISTER},
        gba::GBA,
    };

    #[test]
    fn branch_ends_up_at_correct_address() {
        let mut gba = GBA::new_no_bios();

        gba.cpu.prefetch[0] = Some(0xea000002); // b 0x10
        gba.cpu.set_pc(4);

        let expected_destination = 0x10 + 0x8;
        gba.step();
        gba.step();

        assert_eq!(gba.cpu.get_pc(), expected_destination);
    }

    #[test]
    fn branch_can_go_backwards() {
        let mut gba = GBA::new_no_bios();

        gba.cpu.prefetch[0] = Some(0xeafffffa); // b 0x0
        gba.cpu.prefetch[1] = Some(0xe1a00000);

        gba.cpu.set_pc(0x14);

        let expected_destination = 0x8;

        gba.step();
        gba.step();

        assert_eq!(gba.cpu.get_pc(), expected_destination);
    }

    #[test]
    fn branch_with_link_stores_the_instruction_correctly() {
        let mut gba = GBA::new_no_bios();

        gba.cpu.prefetch[0] = Some(0xebfffffa); // b 0
        gba.cpu.set_pc(0x14);

        let expected_destination = 0x8;

        gba.step();
        gba.step();

        assert!(gba.cpu.get_pc() == expected_destination);
        assert!(gba.cpu.get_register(LINK_REGISTER) == 0x14);
    }

    #[test]
    fn software_interrupt_goes_to_the_correct_interrupt_vec() {
        let mut gba = GBA::new_no_bios();
        gba.cpu.set_mode(CPUMode::USER);
        gba.cpu.set_pc(0xF8);

        gba.cpu.prefetch[1] = Some(0xef000000); // SWI
        gba.step();
        assert_eq!(gba.cpu.get_pc(), 0x10);
        assert!(gba.cpu.get_cpu_mode() == CPUMode::SVC);
        assert_eq!(gba.cpu.get_register(LINK_REGISTER), 0xF4);
    }
}
