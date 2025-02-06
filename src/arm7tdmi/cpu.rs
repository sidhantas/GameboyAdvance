use std::{
    collections::VecDeque,
    fmt::Display,
    fs::{remove_file, File, OpenOptions},
    io::Write,
};

use crate::{
    memory::{
        memory::{GBAMemory},
    },
    types::*,
    utils::bits::Bits,
};


pub const PC_REGISTER: usize = 15;
pub const LINK_REGISTER: u32 = 14;
pub const STACK_POINTER: u32 = 13;

pub enum InstructionMode {
    ARM,
    THUMB,
}

#[derive(PartialEq, Debug)]
pub enum CPUMode {
    USER = 0b10000,
    FIQ = 0b10001, // Fast Interrupt
    IRQ = 0b10010, // IRQ
    SVC = 0b10011, // Supervisor
    ABT = 0b10111, // Abort
    UND = 0b11011, // Undefined
    SYS = 0b11111, // System
}

#[repr(u8)]
pub enum FlagsRegister {
    N = 31,
    Z = 30,
    C = 29,
    V = 28,
}

#[derive(Default, Debug)]
struct Status {
    pub instruction_count: usize,
    pub registers: [WORD; 16],
    pub cpsr: WORD,
    pub cycles: u64,
}

#[derive(Debug)]
pub struct CPU {
    registers: [WORD; 16],
    registers_fiq: [WORD; 8],
    registers_svc: [WORD; 2],
    registers_abt: [WORD; 2],
    registers_irq: [WORD; 2],
    registers_und: [WORD; 2],
    pub(super) is_halted: bool,
    pub prefetch: [Option<WORD>; 2],
    pub executed_instruction_hex: ARMByteCode,
    pub executed_instruction: String,
    pub cpsr: WORD,
    pub spsr: [WORD; 5],
    pub output_file: File,
    pub cycles: u64,
    pub relative_cycles: u64,
    status_history: VecDeque<Status>,
    pub interrupt_triggered: bool
}


const OUTPUT_FILE: &str = "cycle_timings.txt";
const HISTORY_SIZE: usize = 100_000;
pub static mut INSTRUCTION_COUNT: usize = 0;

impl CPU {
    pub fn new() -> Self {
        let _ = remove_file(OUTPUT_FILE);
        let cpu = Self {
            registers: [0; 16],
            executed_instruction_hex: 0,
            executed_instruction: String::with_capacity(50),
            prefetch: [None; 2],
            // start in supervisor mode
            // interrupts are disabled
            // start in arm mode
            cpsr: 0b00000000_00000000_00000000_11010011,
            spsr: [0; 5],
            registers_fiq: [0; 8],
            registers_svc: [0; 2],
            registers_abt: [0; 2],
            registers_irq: [0; 2],
            registers_und: [0; 2],
            output_file: OpenOptions::new()
                .create(true)
                .write(true)
                .open(OUTPUT_FILE)
                .unwrap(),
            cycles: 0,
            relative_cycles: 3,
            status_history: VecDeque::with_capacity(HISTORY_SIZE),
            is_halted: false,
            interrupt_triggered: false
        };
        cpu
    }

    #[no_mangle]
    pub fn execute_cpu_cycle(&mut self, memory: &mut GBAMemory) -> CYCLES {
        self.set_executed_instruction(format_args!(""));
        if self.status_history.len() >= HISTORY_SIZE {
            self.status_history.pop_front();
        }
        unsafe {
            INSTRUCTION_COUNT += 1;
        }
        self.status_history.push_back(self.get_status());
        if self.interrupt_triggered {
            self.raise_irq(memory);
            self.interrupt_triggered = false;
        }
        if self.is_halted {
            self.cycles += 1;
             return 1;
        }
        let mut execution_cycles = 0;
        if let Some(value) = self.prefetch[1] {
            let decoded_instruction = self.decode_instruction(value);
            self.executed_instruction_hex = decoded_instruction.instruction;
            self.prefetch[1] = None;
            execution_cycles +=
                ((decoded_instruction.executable)(self, decoded_instruction.instruction, memory)) as u64;
        }

        if let None = self.prefetch[1] {
            // refill pipeline if decoded instruction doesn't advance the pipeline
            execution_cycles += self.advance_pipeline(memory) as u64;
        }
        self.cycles += execution_cycles;
        execution_cycles as u8
    }

    pub fn flush_pipeline(&mut self, memory: &mut GBAMemory) -> CYCLES {
        let mut cycles = 0;
        self.prefetch[0] = None;
        self.prefetch[1] = None;

        cycles += self.advance_pipeline(memory);
        cycles += self.advance_pipeline(memory);

        cycles
    }

    pub fn advance_pipeline(&mut self, memory: &mut GBAMemory) -> CYCLES {
        self.prefetch[1] = self.prefetch[0];
        self.fetch_instruction(memory)
    }

    pub fn get_pc(&self) -> u32 {
        self.registers[PC_REGISTER] & 0xFFFF_FFFE
    }

    pub fn set_pc(&mut self, address: WORD) {
        self.registers[PC_REGISTER] = address & !1;
    }

    pub fn set_sp(&mut self, address: WORD) {
        self.set_register(13, address);
    }

    pub fn get_sp(&self) -> u32 {
        self.get_register(13)
    }

    pub fn increment_pc(&mut self) {
        match self.get_instruction_mode() {
            InstructionMode::ARM => self.registers[PC_REGISTER] += 4,
            InstructionMode::THUMB => self.registers[PC_REGISTER] += 2,
        }
    }

    fn get_register_ref(&self, register_num: REGISTER) -> &WORD {
        if register_num < 8 || register_num == 15 {
            return &self.registers[register_num as usize];
        }
        match self.get_cpu_mode() {
            CPUMode::FIQ => &self.registers_fiq[(register_num - 8) as usize],
            CPUMode::USER | CPUMode::SYS => &self.registers[register_num as usize],
            _ if register_num < 13 => &self.registers[register_num as usize],
            CPUMode::SVC => &self.registers_svc[(register_num - 13) as usize],
            CPUMode::UND => &self.registers_und[(register_num - 13) as usize],
            CPUMode::IRQ => &self.registers_irq[(register_num - 13) as usize],
            CPUMode::ABT => &self.registers_abt[(register_num - 13) as usize],
        }
    }

    pub fn get_register(&self, register_num: REGISTER) -> WORD {
        assert!(register_num < 16);
        *self.get_register_ref(register_num)
    }

    fn get_register_ref_mut(&mut self, register_num: REGISTER) -> &mut WORD {
        if register_num < 8 || register_num == 15 {
            return &mut self.registers[register_num as usize];
        }
        match self.get_cpu_mode() {
            CPUMode::FIQ => &mut self.registers_fiq[(register_num - 8) as usize],
            CPUMode::USER | CPUMode::SYS => &mut self.registers[register_num as usize],
            _ if register_num < 13 => &mut self.registers[register_num as usize],
            CPUMode::SVC => &mut self.registers_svc[(register_num - 13) as usize],
            CPUMode::UND => &mut self.registers_und[(register_num - 13) as usize],
            CPUMode::IRQ => &mut self.registers_irq[(register_num - 13) as usize],
            CPUMode::ABT => &mut self.registers_abt[(register_num - 13) as usize],
        }
    }

    pub fn set_register(&mut self, register_num: REGISTER, value: WORD) {
        assert!(register_num < 16);
        if register_num == 15 {
            self.set_pc(value);
            return;
        }
        *self.get_register_ref_mut(register_num) = value;
    }

    #[inline(always)]
    pub fn set_flag(&mut self, flag: FlagsRegister) {
        self.cpsr.set_bit(flag as u8);
    }

    #[inline(always)]
    pub fn reset_flag(&mut self, flag: FlagsRegister) {
        self.cpsr.reset_bit(flag as u8);
    }

    #[inline(always)]
    pub fn get_flag(&self, flag: FlagsRegister) -> WORD {
        self.cpsr.get_bit(flag as u8)
    }

    #[inline(always)]
    pub fn set_instruction_mode(&mut self, instruction_mode: InstructionMode) {
        match instruction_mode {
            InstructionMode::ARM => self.cpsr.reset_bit(5),
            InstructionMode::THUMB => self.cpsr.set_bit(5),
        }
    }

    #[inline(always)]
    pub fn get_instruction_mode(&self) -> InstructionMode {
        if self.cpsr.bit_is_set(5) {
            return InstructionMode::THUMB;
        }
        return InstructionMode::ARM;
    }

    pub fn set_mode(&mut self, mode: CPUMode) {
        self.cpsr &= !0x1F; // clear bottom 5 bits
        self.cpsr |= mode as u32;
    }

    pub fn get_cpu_mode(&self) -> CPUMode {
        match (self.cpsr & 0x0000_001F) as u8 {
            x if x == CPUMode::USER as u8 => CPUMode::USER,
            x if x == CPUMode::FIQ as u8 => CPUMode::FIQ,
            x if x == CPUMode::IRQ as u8 => CPUMode::IRQ,
            x if x == CPUMode::SVC as u8 => CPUMode::SVC,
            x if x == CPUMode::ABT as u8 => CPUMode::ABT,
            x if x == CPUMode::UND as u8 => CPUMode::UND,
            x if x == CPUMode::SYS as u8 => CPUMode::SYS,
            _ => panic!("Impossible cpsr value {:#x}", self.cpsr),
        }
    }

    pub fn get_current_spsr(&mut self) -> Option<&mut WORD> {
        match self.get_cpu_mode() {
            CPUMode::FIQ => Some(&mut self.spsr[0]),
            CPUMode::SVC => Some(&mut self.spsr[1]),
            CPUMode::ABT => Some(&mut self.spsr[2]),
            CPUMode::IRQ => Some(&mut self.spsr[3]),
            CPUMode::UND => Some(&mut self.spsr[4]),
            _ => None,
        }
    }

    pub fn set_flag_from_bit(&mut self, flag: FlagsRegister, bit: u8) {
        assert!(bit == 0 || bit == 1);
        if bit == 0 {
            self.reset_flag(flag);
            return;
        }
        self.set_flag(flag);
    }

    pub(super) fn fetch_instruction(&mut self, memory: &mut GBAMemory) -> CYCLES {
        let memory_fetch = {
            match self.get_instruction_mode() {
                InstructionMode::ARM => memory.readu32(self.get_pc() as usize),
                InstructionMode::THUMB => memory.readu16(self.get_pc() as usize).into(),
            }
        };
        self.prefetch[0] = Some(memory_fetch.data);
        self.increment_pc();

        memory_fetch.cycles
    }

    pub fn decode_shifted_register(
        &mut self,
        instruction: ARMByteCode,
        shift_amount: u32,
        operand_register_value: u32,
        set_flags: bool,
    ) -> u32 {
        let shift_type = (instruction & 0x0000_0060) >> 5;

        if !instruction.bit_is_set(4) && shift_amount == 0 {
            // special case for shifting
            return match shift_type {
                // no change
                0x00 => operand_register_value,
                // LSR#32
                0x01 => {
                    if set_flags {
                        self.set_flag_from_bit(
                            FlagsRegister::C,
                            operand_register_value.get_bit(31) as u8,
                        );
                    }
                    0
                }
                // ASR#32
                0x02 => {
                    if operand_register_value.bit_is_set(31) {
                        if set_flags {
                            self.set_flag(FlagsRegister::C);
                        }
                        return u32::MAX;
                    }
                    if set_flags {
                        self.reset_flag(FlagsRegister::C);
                    }
                    0
                }
                // RRX#1
                0x03 => operand_register_value >> 1 | self.get_flag(FlagsRegister::C) << 31,
                _ => panic!("Invalid Shift Type"),
            };
        }

        match shift_type {
            // Logical shift left
            0x00 => {
                if set_flags {
                    if operand_register_value.bit_is_set((32 - shift_amount) as u8) {
                        self.set_flag(FlagsRegister::C);
                    } else {
                        self.reset_flag(FlagsRegister::C);
                    }
                }
                operand_register_value << shift_amount
            }
            // Logical shift right
            0x01 => {
                if set_flags && shift_amount > 0 {
                    self.set_flag_from_bit(
                        FlagsRegister::C,
                        operand_register_value.get_bit(shift_amount as u8 - 1) as u8,
                    );
                }
                operand_register_value >> shift_amount
            }
            // Arithmetic shift right
            0x02 => {
                if set_flags && shift_amount > 0 {
                    self.set_flag_from_bit(
                        FlagsRegister::C,
                        operand_register_value.get_bit(shift_amount as u8 - 1) as u8,
                    );
                }
                (operand_register_value as i32 >> shift_amount) as u32
            }
            // Rotate Right
            0x03 => {
                if set_flags && shift_amount > 0 {
                    if operand_register_value.bit_is_set((shift_amount - 1) as u8) {
                        self.set_flag(FlagsRegister::C);
                    } else {
                        self.reset_flag(FlagsRegister::C);
                    }
                }
                operand_register_value.rotate_right(shift_amount)
            }
            _ => panic!("Invalid Shift Type"),
        }
    }

    fn get_status(&self) -> Status {
        let mut status = Status::default();
        for i in 0..self.registers.len() {
            status.registers[i] = self.get_register(i as REGISTER);
        }
        status.cpsr = self.cpsr;
        status.cycles = 0;
        unsafe {
            status.instruction_count = INSTRUCTION_COUNT;
        }
        status
    }
}

impl Display for Status {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} ", self.instruction_count)?;
        for i in self.registers {
            write!(f, "{:08x} ", i)?;
        }

        write!(f, "{:08x} ", self.cpsr)?;
        write!(f, "{}\n", self.cycles)
    }
}

impl Drop for CPU {
    fn drop(&mut self) {
        for i in self.status_history.iter().skip(1) {
            self.output_file.write(format!("{}", i).as_bytes()).unwrap();
        }
    }
}

#[cfg(test)]
mod cpu_tests {

    use crate::{arm7tdmi::cpu::CPUMode, utils::bits::Bits};

    use super::CPU;

    #[test]
    fn it_sets_and_resets_the_corrects_flags() {

        let mut cpu = CPU::new();

        cpu.set_flag(super::FlagsRegister::C);
        cpu.set_flag(super::FlagsRegister::N);
        cpu.set_flag(super::FlagsRegister::Z);
        cpu.reset_flag(super::FlagsRegister::Z);

        assert!(cpu.cpsr.bit_is_set(super::FlagsRegister::C as u8));
        assert!(cpu.cpsr.bit_is_set(super::FlagsRegister::N as u8));
        assert!(cpu.cpsr.bit_is_set(super::FlagsRegister::Z as u8) == false);
        assert!(cpu.cpsr.bit_is_set(super::FlagsRegister::V as u8) == false);

        cpu.reset_flag(super::FlagsRegister::C);
        assert!(cpu.cpsr.bit_is_set(super::FlagsRegister::C as u8) == false);
        assert!(cpu.cpsr.bit_is_set(super::FlagsRegister::N as u8));
    }

    #[test]
    fn cpu_starts_in_svc_mode() {
        let cpu = CPU::new();

        assert!(matches!(cpu.get_cpu_mode(), CPUMode::SVC));
    }
}
