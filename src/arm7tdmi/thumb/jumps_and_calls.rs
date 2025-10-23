use crate::{
    arm7tdmi::{
        cpu::{FlagsRegister, CPU, LINK_REGISTER},
        instruction_table::{condition_code_as_str, DecodeThumbInstructionToString, Execute},
    },
    memory::memory::GBAMemory,
    types::CYCLES,
    utils::bits::sign_extend,
};

pub struct ThumbConditionalBranch(pub u32);

impl ThumbConditionalBranch {
    fn offset(&self) -> u32 {
        sign_extend((self.0 & 0x00FF) << 1, 8)
    }
    fn condition_code(&self) -> u32 {
        (self.0 & 0x0F00) >> 8
    }
}

impl Execute for ThumbConditionalBranch {
    fn execute(self, cpu: &mut CPU, memory: &mut GBAMemory) -> CYCLES {
        let mut cycles = 0;
        let condition_passed = match self.condition_code() {
            0b0000 => cpu.get_flag(FlagsRegister::Z) == 1, //EQ
            0b0001 => cpu.get_flag(FlagsRegister::Z) == 0, //NE
            0b0010 => cpu.get_flag(FlagsRegister::C) == 1, //CS
            0b0011 => cpu.get_flag(FlagsRegister::C) == 0, //CC
            0b0100 => cpu.get_flag(FlagsRegister::N) == 1, //MI
            0b0101 => cpu.get_flag(FlagsRegister::N) == 0, //PL
            0b0110 => cpu.get_flag(FlagsRegister::V) == 1, //VS
            0b0111 => cpu.get_flag(FlagsRegister::V) == 0, //VC
            0b1000 => cpu.get_flag(FlagsRegister::C) == 1 && cpu.get_flag(FlagsRegister::Z) == 0, //HI
            0b1001 => cpu.get_flag(FlagsRegister::C) == 0 || cpu.get_flag(FlagsRegister::Z) == 1, //LS
            0b1010 => cpu.get_flag(FlagsRegister::N) == cpu.get_flag(FlagsRegister::V), //GE
            0b1011 => cpu.get_flag(FlagsRegister::N) != cpu.get_flag(FlagsRegister::V), //LT
            0b1100 => {
                cpu.get_flag(FlagsRegister::Z) == 0
                    && cpu.get_flag(FlagsRegister::N) == cpu.get_flag(FlagsRegister::V)
            } //GT
            0b1101 => {
                cpu.get_flag(FlagsRegister::Z) == 1
                    || cpu.get_flag(FlagsRegister::N) != cpu.get_flag(FlagsRegister::V)
            } //LE
            condition => panic!("Impossible/Undefined condition code {:#b}", condition),
        };
        let memory_fetch = memory.readu16(cpu.get_pc() as usize);
        cycles += memory_fetch.cycles;
        let destination = cpu.get_pc() + self.offset();
        if !condition_passed {
            return 0;
        }
        cpu.set_pc(destination);
        cycles += cpu.flush_pipeline(memory);

        cycles
    }
}

impl DecodeThumbInstructionToString for ThumbConditionalBranch {
    fn instruction_to_string(&self) -> String {
        format!(
            "b{} {:#x}",
            condition_code_as_str(self.condition_code()),
            self.offset() as i32
        )
    }
}

pub struct ThumbUnconditionalBranch(pub u32);

impl ThumbUnconditionalBranch {
    fn offset(&self) -> u32 {
        sign_extend((self.0 & 0x07FF) << 1, 11)
    }
}

impl Execute for ThumbUnconditionalBranch {
    fn execute(self, cpu: &mut CPU, memory: &mut GBAMemory) -> CYCLES {
        cpu.set_pc(cpu.get_pc() + self.offset());
        cpu.flush_pipeline(memory) + 1
    }
}

impl DecodeThumbInstructionToString for ThumbUnconditionalBranch {
    fn instruction_to_string(&self) -> String {
        format!("b {:#x}", self.offset())
    }
}

pub struct ThumbSetLinkRegister(pub u32);

impl ThumbSetLinkRegister {
    fn offset(&self) -> u32 {
        sign_extend((self.0 & 0x07FF) << 12, 22)
    }
}

impl Execute for ThumbSetLinkRegister {
    fn execute(self, cpu: &mut CPU, memory: &mut GBAMemory) -> CYCLES {
        let value = cpu.get_pc() + self.offset();
        cpu.set_register(LINK_REGISTER, value);
        0
    }
}

impl DecodeThumbInstructionToString for ThumbSetLinkRegister {
    fn instruction_to_string(&self) -> String {
        format!("set lr: {:#x}", self.offset())
    }
}

pub struct ThumbLongBranchWithLink(pub u32);

impl ThumbLongBranchWithLink {
    fn offset(&self) -> u32 {
        (self.0 & 0x7FF) << 1
    }
}

impl Execute for ThumbLongBranchWithLink {
    fn execute(self, cpu: &mut CPU, memory: &mut GBAMemory) -> CYCLES {
        let mut cycles = 0;
        let link_register_val = cpu.get_register(LINK_REGISTER);
        cpu.set_register(LINK_REGISTER, (cpu.get_pc() - 2) | 1);

        let destination = link_register_val + self.offset();
        cpu.set_pc(destination);

        // We don't use the fetched instruction but we need to do it to get the correct cycle count
        let memory_fetch = memory.readu16(cpu.get_pc() as usize);
        cycles += memory_fetch.cycles;
        cycles += cpu.flush_pipeline(memory);

        cycles
    }
}

impl DecodeThumbInstructionToString for ThumbLongBranchWithLink {
    fn instruction_to_string(&self) -> String {
        format!("bl {:#x}", self.offset())
    }
}

#[cfg(test)]
mod branch_tests {
    use crate::{
        arm7tdmi::cpu::{FlagsRegister, InstructionMode, LINK_REGISTER},
        gba::GBA,
    };

    #[test]
    fn should_branch_ahead() {
        let mut gba = GBA::new_no_bios();
        gba.cpu.set_instruction_mode(InstructionMode::THUMB);

        gba.cpu.prefetch[0] = Some(0xd006); // beq 12
        gba.cpu.set_pc(0x1a);
        gba.cpu.set_flag(FlagsRegister::Z);
        gba.step();
        gba.step();

        assert_eq!(gba.cpu.get_pc(), 0x2c);
    }

    #[test]
    fn should_branch_behind() {
        let mut gba = GBA::new_no_bios();
        gba.cpu.set_instruction_mode(InstructionMode::THUMB);

        gba.cpu.prefetch[0] = Some(0xd0f9); // beq 12
        gba.cpu.set_pc(0x24);
        gba.cpu.set_flag(FlagsRegister::Z);
        gba.step();
        gba.step();

        assert_eq!(gba.cpu.get_pc(), 0x1c);
    }

    #[test]
    fn should_set_link_register_and_branch() {
        let mut gba = GBA::new_no_bios();
        gba.cpu.set_instruction_mode(InstructionMode::THUMB);

        gba.cpu.prefetch[0] = Some(0xf000); // set link_register
        gba.cpu.set_pc(0x1a);
        gba.step();
        gba.cpu.prefetch[0] = Some(0xf802); // bl 0x20
        gba.step();
        gba.step();

        assert_eq!(gba.cpu.get_pc(), 0x24);
        assert_eq!(gba.cpu.get_register(LINK_REGISTER), 0x1d);
    }
}
