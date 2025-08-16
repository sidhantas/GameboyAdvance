use std::fmt::Display;

use crate::utils::bits::{self, Bits};

use super::cpu::{CPUMode, FlagsRegister, InstructionMode};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PSR {
    pub mode: CPUMode,
    pub instruction_mode: InstructionMode,
    pub irq_disabled: bool,
    pub fiq_disabled: bool,
    overflow: bool,
    carry: bool,
    zero: bool,
    sign: bool,
}

impl Default for PSR {
    fn default() -> Self {
        Self::new_spsr()
    }
}

impl Display for PSR {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:#x}", u32::from(*self))
    }
}

impl From<PSR> for u32 {
    fn from(psr: PSR) -> u32 {
        let mut psr_bits = 0;
        let mode: u32 = match psr.mode {
            CPUMode::USER => 0b10000,
            CPUMode::FIQ => 0b10001,
            CPUMode::IRQ => 0b10010,
            CPUMode::SVC => 0b10011,
            CPUMode::ABT => 0b10111,
            CPUMode::UND => 0b11011,
            CPUMode::SYS => 0b11111,
            CPUMode::INVALID(bits) => bits,
        };

        psr_bits |= mode;

        match psr.instruction_mode {
            InstructionMode::THUMB => {
                psr_bits.set_bit(5);
            }
            InstructionMode::ARM => {}
        };

        if psr.fiq_disabled {
            psr_bits.set_bit(6);
        }
        if psr.irq_disabled {
            psr_bits.set_bit(7);
        }

        if psr.overflow {
            psr_bits.set_bit(28);
        }
        if psr.carry {
            psr_bits.set_bit(29);
        }
        if psr.zero {
            psr_bits.set_bit(30);
        }

        if psr.sign {
            psr_bits.set_bit(31);
        }

        psr_bits
    }
}

impl From<u32> for PSR {
    fn from(value: u32) -> Self {
        let mode: CPUMode = match value & 0b11111 {
            0b10000 => CPUMode::USER,
            0b10001 => CPUMode::FIQ,
            0b10010 => CPUMode::IRQ,
            0b10011 => CPUMode::SVC,
            0b10111 => CPUMode::ABT,
            0b11011 => CPUMode::UND,
            0b11111 => CPUMode::SYS,
            other => CPUMode::INVALID(other),
        };

        let instruction_mode = if value.bit_is_set(5) {
            InstructionMode::THUMB
        } else {
            InstructionMode::ARM
        };

        let irq_disabled = value.bit_is_set(7);
        let fiq_disabled = value.bit_is_set(6);
        let overflow = value.bit_is_set(28);
        let carry = value.bit_is_set(29);
        let zero = value.bit_is_set(30);
        let sign = value.bit_is_set(31);

        PSR {
            mode,
            instruction_mode,
            irq_disabled,
            fiq_disabled,
            overflow,
            carry,
            zero,
            sign,
        }
    }
}

impl PSR {
    pub fn new_cpsr() -> Self {
        // start in supervisor mode
        // interrupts are disabled
        // start in arm mode
        Self {
            mode: CPUMode::SVC,
            instruction_mode: InstructionMode::ARM,
            irq_disabled: true,
            fiq_disabled: true,
            overflow: false,
            carry: false,
            zero: false,
            sign: false,
        }
    }

    pub fn new_spsr() -> Self {
        // start in supervisor mode
        // interrupts are disabled
        // start in arm mode
        Self {
            mode: CPUMode::USER,
            instruction_mode: InstructionMode::ARM,
            irq_disabled: false,
            fiq_disabled: false,
            overflow: false,
            carry: false,
            zero: false,
            sign: false,
        }
    }

    pub fn set_flag(&mut self, flag: FlagsRegister) {
        match flag {
            FlagsRegister::N => self.sign = true,
            FlagsRegister::Z => self.zero = true,
            FlagsRegister::C => self.carry = true,
            FlagsRegister::V => self.overflow = true,
        }
    }

    pub fn reset_flag(&mut self, flag: FlagsRegister) {
        match flag {
            FlagsRegister::N => self.sign = false,
            FlagsRegister::Z => self.zero = false,
            FlagsRegister::C => self.carry = false,
            FlagsRegister::V => self.overflow = false,
        }
    }

    pub fn get_flag(&self, flag: FlagsRegister) -> bool {
        match flag {
            FlagsRegister::N => self.sign,
            FlagsRegister::Z => self.zero,
            FlagsRegister::C => self.carry,
            FlagsRegister::V => self.overflow,
        }
    }
}
