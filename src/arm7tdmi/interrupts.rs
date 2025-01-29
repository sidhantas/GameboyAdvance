use crate::utils::bits::Bits;

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
    pub fn raise_exception(&mut self, exception: Exceptions) {
        self.set_mode(exception.into());
        if let Some(_) = self.prefetch[1] {
            self.pipeline_stalled = true;
            return;
        }
        self.pipeline_stalled = false;
        let instruction_size = match self.get_instruction_mode() {
            super::cpu::InstructionMode::ARM => 4,
            super::cpu::InstructionMode::THUMB => 2,
        };
        
        // Store next instruction address for handler to return to
        self.set_register(LINK_REGISTER, self.get_pc() - instruction_size);
        // Store CPSR in SPSR_new_mode
        let cpsr = self.cpsr;
        if let Some(spsr) = self.get_current_spsr() {
            *spsr = cpsr;
        }
        // Switch to ARM mode
        self.set_instruction_mode(InstructionMode::ARM);

        // Update I and F bits and get exception address
        let exception_vector = match exception {
            Exceptions::Reset => {
                self.cpsr.set_bit(7);
                self.cpsr.set_bit(6);
                0x00
            },
            Exceptions::Undefined => {
                self.cpsr.set_bit(7);
                0x04
            },
            Exceptions::Software => {
                self.cpsr.set_bit(7);
                0x08
            }
            Exceptions::IRQ => {
                self.cpsr.set_bit(7);
                0x18
            },
        };

        self.set_pc(exception_vector);
    }
}
