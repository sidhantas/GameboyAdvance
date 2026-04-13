use crate::utils::bits::{self, Bits};

use super::cpu::{CPUMode, FlagsRegister, InstructionMode};

#[derive(Debug, Clone, Copy)]
pub(crate) struct NewPSR(u32);

impl From<NewPSR> for u32 {
    fn from(value: NewPSR) -> Self {
        value.0
    }
}

impl From<u32> for NewPSR {
    fn from(value: u32) -> Self {
        Self(value)
    }
}

impl Default for NewPSR {
    fn default() -> Self {
        Self(0b0000_0000_0000_0000_0000_0000_1101_0011)
    }
}

const USER_BITS: u32 = 0b10000;
const FIQ_BITS: u32 = 0b10001;
const IRQ_BITS: u32 = 0b10010;
const SVC_BITS: u32 = 0b10011;
const ABT_BITS: u32 = 0b10111;
const UND_BITS: u32 = 0b11011;
const SYS_BITS: u32 = 0b11111;

impl NewPSR {
    pub(crate) fn new_cpsr() -> Self {
        Self(0b0000_0000_0000_0000_0000_0000_1101_0011)
    }

    pub(crate) fn new_spsr() -> Self {
        Self(0)
    }
    pub(crate) fn mode(&self) -> CPUMode {
        match self.0 & 0b11111 {
            USER_BITS => CPUMode::USER,
            FIQ_BITS => CPUMode::FIQ,
            IRQ_BITS => CPUMode::IRQ,
            SVC_BITS => CPUMode::SVC,
            ABT_BITS => CPUMode::ABT,
            UND_BITS => CPUMode::UND,
            SYS_BITS => CPUMode::SYS,
            _ => unreachable!("{:#x}", self.0),
        }
    }

    pub(crate) fn instruction_mode(&self) -> InstructionMode {
        match self.0.bit_is_set(5) {
            true => InstructionMode::THUMB,
            false => InstructionMode::ARM,
        }
    }

    pub(crate) fn set_instruction_mode(&mut self, mode: InstructionMode) {
        match mode {
            InstructionMode::ARM => self.0.reset_bit(5),
            InstructionMode::THUMB => self.0.set_bit(5),
        }
    }

    pub(crate) fn set_flag(&mut self, flag: FlagsRegister) {
        match flag {
            FlagsRegister::N => self.0.set_bit(31),
            FlagsRegister::Z => self.0.set_bit(30),
            FlagsRegister::C => self.0.set_bit(29),
            FlagsRegister::V => self.0.set_bit(28),
        }
    }

    pub(crate) fn reset_flag(&mut self, flag: FlagsRegister) {
        match flag {
            FlagsRegister::N => self.0.reset_bit(31),
            FlagsRegister::Z => self.0.reset_bit(30),
            FlagsRegister::C => self.0.reset_bit(29),
            FlagsRegister::V => self.0.reset_bit(28),
        }
    }

    pub(crate) fn get_flag(&self, flag: FlagsRegister) -> bool {
        match flag {
            FlagsRegister::N => self.0.bit_is_set(31),
            FlagsRegister::Z => self.0.bit_is_set(30),
            FlagsRegister::C => self.0.bit_is_set(29),
            FlagsRegister::V => self.0.bit_is_set(28),
        }
    }

    pub(crate) fn set_irq(&mut self, enabled: bool) {
        if enabled {
            self.0.reset_bit(7);
            return;
        }

        self.0.set_bit(7);
    }

    pub(crate) fn irq_enabled(&mut self) -> bool{
        !self.0.bit_is_set(7)
    }

    pub(crate) fn fiq_enabled(&mut self) -> bool{
        !self.0.bit_is_set(6)
    }


    pub(crate) fn set_fiq(&mut self, enabled: bool) {
        if enabled {
            self.0.reset_bit(6);
            return;
        }

        self.0.set_bit(6);
    }

    pub(crate) fn set_mode(&mut self, mode: CPUMode) {
        self.0 &= !SYS_BITS;
        self.0
            |= match mode {
                CPUMode::USER => USER_BITS,
                CPUMode::FIQ => FIQ_BITS,
                CPUMode::IRQ => IRQ_BITS,
                CPUMode::SVC => SVC_BITS,
                CPUMode::ABT => ABT_BITS,
                CPUMode::UND => UND_BITS,
                CPUMode::SYS => SYS_BITS,
                CPUMode::INVALID(_) => unreachable!(),
            }
    }
}
