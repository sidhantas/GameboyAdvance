use crate::{memory::memory::GBAMemory, types::CYCLES, utils::bits::Bits};

use super::cpu::{CPUMode, InstructionMode, CPU, LINK_REGISTER};

#[derive(Clone, Copy)]
pub enum Exceptions {
    Reset,
    Undefined,
    Software,
    IRQ,
}

impl From<Exceptions> for CPUMode {
    fn from(exception: Exceptions) -> Self {
        match exception {
            Exceptions::Reset => CPUMode::SVC,
            Exceptions::Undefined => CPUMode::UND,
            Exceptions::Software => CPUMode::SVC,
            Exceptions::IRQ => CPUMode::IRQ,
        }
    }
}

impl CPU {
    pub fn raise_exception(&mut self, exception: Exceptions, memory: &mut GBAMemory) -> CYCLES {
        self.is_halted = false;
        let instruction_size = match self.get_instruction_mode() {
            super::cpu::InstructionMode::ARM => 4,
            super::cpu::InstructionMode::THUMB => 0,
        };

        // Store CPSR in SPSR_new_mode
        let cpsr = self.get_cpsr();
        self.set_mode(exception.into());
        // Store next instruction address for handler to return to
        self.set_register(LINK_REGISTER, self.get_pc() - instruction_size);
        if let Some(spsr) = self.get_current_spsr() {
            *spsr = cpsr;
        }
        // Switch to ARM mode
        self.set_instruction_mode(InstructionMode::ARM);
        // Switch to privileged mode

        // Update I and F bits and get exception address
        let exception_vector = match exception {
            Exceptions::Reset => {
                self.disable_irq();
                self.disable_fiq();
                0x00
            }
            Exceptions::Undefined => {
                self.disable_irq();
                0x04
            }
            Exceptions::Software => {
                self.disable_irq();
                0x08
            }
            Exceptions::IRQ => {
                self.disable_irq();
                0x18
            }
        };

        self.set_pc(exception_vector);
        self.flush_pipeline(memory)
    }

    pub fn halt(&mut self) {
        self.is_halted = true;
    }

    pub fn raise_irq(&mut self, memory: &mut GBAMemory) {
        if !self.get_cpsr().irq_disabled {
            self.raise_exception(Exceptions::IRQ, memory);
        }
    }
}
