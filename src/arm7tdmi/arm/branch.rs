use crate::{arm7tdmi::{cpu::{InstructionMode, LINK_REGISTER}, instruction_table::{DecodeARMInstructionToString, Execute}}, types::REGISTER, utils::{bits::{sign_extend, Bits}, instruction_to_string::print_register}};

pub struct BranchInstruction(pub u32);

impl BranchInstruction {
    fn set_lr(&self) -> bool {
        self.0.bit_is_set(24)
    }
    
    fn offset(&self) -> u32 {
        sign_extend((self.0 & 0x00FF_FFFF) << 2, 25)
    }
}

impl Execute for BranchInstruction {
    fn execute(self, cpu: &mut crate::arm7tdmi::cpu::CPU, memory: &mut crate::memory::memory::GBAMemory) -> crate::types::CYCLES {
        let mut cycles = 1;
        let destination = cpu.get_pc() + self.offset();
        if self.set_lr() {
            cpu.set_register(LINK_REGISTER, cpu.get_pc() - 4);
        }

        cpu.set_pc(destination);

        cycles += cpu.flush_pipeline(memory);
        cycles
    }
}

impl DecodeARMInstructionToString for BranchInstruction {
    fn instruction_to_string(&self, condition_code: &str) -> String {
        let lr = if self.set_lr() {
            "L"
        } else {
            ""
        };

        format!("B{lr}{condition_code} {:#x}", self.offset())
    }
}

pub struct BranchAndExchangeInstruction(pub u32);

impl BranchAndExchangeInstruction {
    fn rn(&self) -> REGISTER {
        self.0 & 0xF
    }
}

impl Execute for BranchAndExchangeInstruction {
    fn execute(self, cpu: &mut crate::arm7tdmi::cpu::CPU, memory: &mut crate::memory::memory::GBAMemory) -> crate::types::CYCLES {
        let destination = cpu.get_register(self.rn());
        let mut cycles = 1; 

        if destination.bit_is_set(0) {
            cpu.set_instruction_mode(InstructionMode::THUMB);
            cpu.set_pc(destination & !1);
        } else {
            cpu.set_instruction_mode(InstructionMode::ARM);
            cpu.set_pc(destination & !3);
        }

        cycles += cpu.flush_pipeline(memory);

        cycles
    }
}

impl DecodeARMInstructionToString for BranchAndExchangeInstruction {
    fn instruction_to_string(&self, condition_code: &str) -> String {
        format!("bx{condition_code} {}", print_register(&self.rn()))
    }
}
