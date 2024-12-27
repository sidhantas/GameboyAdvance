use crate::{
    arm7tdmi::cpu::LINK_REGISTER,
    types::{ARMByteCode, CYCLES, REGISTER, WORD},
    utils::bits::{sign_extend, Bits},
};

use super::{
    cpu::{FlagsRegister, InstructionMode, CPU},
    decoder::Instruction, interrupts::Exceptions,
};
pub type ARMExecutable = fn(&mut CPU, ARMByteCode) -> CYCLES;
pub type ExecutingInstruction = fn(&mut CPU) -> ();
pub type InternalOperation = fn(&mut CPU) -> CYCLES;
pub type ALUOperation =
    fn(&mut CPU, rd: REGISTER, operand1: u32, operand2: u32, set_flags: bool) -> ();

#[derive(Clone, Copy)]
pub struct ARMDecodedInstruction {
    pub executable: ARMExecutable,
    pub instruction: ARMByteCode,
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
    pub fn set_executed_instruction(&mut self, name: String) {
        self.executed_instruction = name;
    }

    pub fn arm_branch(&mut self, instruction: ARMByteCode) -> CYCLES {
        let mut cycles = 1;
        if instruction.bit_is_set(24) {
            self.set_register(LINK_REGISTER, self.get_pc() - 4);
        }
        let offset = instruction & 0x00FF_FFFF;
        let offset = sign_extend(offset << 2, 25);
        let destination = offset + self.get_pc();
        self.set_pc(destination);
        cycles += self.flush_pipeline();
        self.set_executed_instruction(format!("B {:#010x}", destination));

        cycles
    }

    pub fn arm_nop(&mut self, _instruction: ARMByteCode) -> CYCLES {
        self.set_executed_instruction("NOP".into());
        return 1;
    }

    pub fn arm_multiply(&mut self, instruction: ARMByteCode) -> CYCLES {
        return 0;
    }

    pub fn arm_multiply_accumulate(&mut self, instruction: ARMByteCode) -> CYCLES {
        return 0;
    }

    pub fn arm_multiply_long(&mut self, instruction: ARMByteCode) -> CYCLES {
        return 0;
    }

    pub fn arm_software_interrupt(&mut self, _instruction: ARMByteCode) -> CYCLES {
        let mut cycles = 1;
        self.raise_exception(Exceptions::Software);
        cycles += self.flush_pipeline();
        self.set_executed_instruction("SWI".into());

        return cycles;
    }


    pub fn arm_branch_and_exchange(&mut self, instruction: ARMByteCode) -> CYCLES {
        let mut destination = self.get_register(instruction & 0x0000_000F);
        let mut cycles = 1;
        if destination.bit_is_set(0) {
            self.set_instruction_mode(InstructionMode::THUMB);
        } else {
            destination &= !2; // arm instructions must be word aligned
            self.set_instruction_mode(InstructionMode::ARM);
        }
        self.set_pc(destination & !1); // bit 0 is forced to 0 before storing
        cycles += self.flush_pipeline();
        self.set_executed_instruction(format!("BX {:#010x}", destination));

        cycles
    }

    pub fn arm_not_implemented(&mut self, instruction: ARMByteCode) -> CYCLES {
        self.set_executed_instruction("NOT IMPLEMENTED".into());
        return 0;
    }
}

#[cfg(test)]
mod instruction_tests {
    use std::sync::{Arc, Mutex};

    use crate::{
        arm7tdmi::cpu::{CPUMode, FlagsRegister, CPU, LINK_REGISTER},
        memory::memory::Memory,
    };

    #[test]
    fn branch_ends_up_at_correct_address() {
        let memory = Memory::new().unwrap();
        let memory = Arc::new(Mutex::new(memory));
        let mut cpu = CPU::new(memory);

        cpu.prefetch[0] = Some(0xea000002); // b 0x10
        cpu.set_pc(4);

        let expected_destination = 0x10 + 0x8;

        cpu.execute_cpu_cycle();
        cpu.execute_cpu_cycle();

        assert_eq!(cpu.get_pc(), expected_destination);
    }

    #[test]
    fn branch_can_go_backwards() {
        let memory = Memory::new().unwrap();
        let memory = Arc::new(Mutex::new(memory));
        let mut cpu = CPU::new(memory);

        cpu.prefetch[0] = Some(0xeafffffa); // b 0x0
        cpu.prefetch[1] = Some(0xe1a00000);

        cpu.set_pc(0x14);

        let expected_destination = 0x8;

        cpu.execute_cpu_cycle();
        cpu.execute_cpu_cycle();

        assert_eq!(cpu.get_pc(), expected_destination);
    }

    #[test]
    fn branch_with_link_stores_the_instruction_correctly() {
        let memory = Memory::new().unwrap();
        let memory = Arc::new(Mutex::new(memory));
        let mut cpu = CPU::new(memory);

        cpu.prefetch[0] = Some(0xebfffffa); // b 0
        cpu.set_pc(0x14);

        let expected_destination = 0x8;

        cpu.execute_cpu_cycle();
        cpu.execute_cpu_cycle();

        assert!(cpu.get_pc() == expected_destination);
        assert!(cpu.get_register(LINK_REGISTER) == 0x14);
    }

    #[test]
    fn software_interrupt_goes_to_the_correct_interrupt_vec() {
        let memory = Memory::new().unwrap();
        let memory = Arc::new(Mutex::new(memory));
        let mut cpu = CPU::new(memory);
        cpu.set_mode(CPUMode::USER);
        cpu.set_pc(0xF8);

        cpu.prefetch[1] = Some(0xef000000); // SWI

        cpu.execute_cpu_cycle();
        assert_eq!(cpu.get_pc(), 0x10);
        assert!(cpu.get_cpu_mode() == CPUMode::SVC);
        assert_eq!(cpu.get_register(LINK_REGISTER), 0xF4);
    }
}
