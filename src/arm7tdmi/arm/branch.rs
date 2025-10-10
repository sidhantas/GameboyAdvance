use crate::{arm7tdmi::{cpu::LINK_REGISTER, instruction_table::{DecodeARMInstructionToString, Execute}}, utils::bits::{sign_extend, Bits}};

pub(crate) struct BranchInstruction(pub u32);

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
