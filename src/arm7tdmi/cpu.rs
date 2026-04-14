use core::panic;
use std::{
    collections::VecDeque,
    fmt::Display,
    fs::{remove_file, File, OpenOptions},
    io::Write,
};

use crate::{
    arm7tdmi::{cpsr::PSR, instruction_table::Instruction},
    memory::memory::GBAMemory,
    types::*,
    utils::bits::Bits,
};

use super::{instruction_table::Execute, registers::Registers};

pub(crate) const PC_REGISTER: usize = 15;
pub(crate) const LINK_REGISTER: u32 = 14;
pub(crate) const STACK_POINTER: u32 = 13;

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum InstructionMode {
    ARM,
    THUMB,
}

#[derive(PartialEq, Debug, Clone, Copy)]
pub(crate) enum CPUMode {
    USER,
    FIQ, // Fast Interrupt
    IRQ, // IRQ
    SVC, // Supervisor
    ABT, // Abort
    UND, // Undefined
    SYS, // System
    INVALID(u32),
}

#[repr(u8)]
pub(crate) enum FlagsRegister {
    N = 31,
    Z = 30,
    C = 29,
    V = 28,
}

#[derive(Default, Debug)]
struct Status {
    pub(crate) instruction_count: usize,
    pub(crate) registers: [WORD; 16],
    pub(crate) cpsr: PSR,
    pub(crate) cycles: u64,
}

#[derive(Debug)]
pub(crate) struct CPU {
    registers: Registers,
    pub(crate) is_halted: bool,
    pub(crate) prefetch: [WORD; 2],
    pub(crate) executed_instruction_hex: ARMByteCode,
    cpsr: PSR,
    pub(super) shifter_output: u32,
    pub(crate) spsr: [PSR; 5],
    pub(crate) output_file: File,
    pub(crate) cycles: u64,
    status_history: VecDeque<Status>,
    pub(crate) interrupt_triggered: bool,
    instruction_count: usize,
    pub(crate) show_executed_instructions: bool,
}

const OUTPUT_FILE: &str = "cycle_timings.txt";
const HISTORY_SIZE: usize = 100_000;
pub(crate) static mut INSTRUCTION_COUNT: usize = 0;

impl CPU {
    pub(crate) fn new() -> Self {
        let _ = remove_file(OUTPUT_FILE);
        let cpu = Self {
            registers: Registers::new(),
            executed_instruction_hex: 0,
            prefetch: [0; 2],
            cpsr: PSR::new_cpsr(),
            spsr: [PSR::new_spsr(); 5],
            shifter_output: 0,
            output_file: OpenOptions::new()
                .create(true)
                .write(true)
                .open(OUTPUT_FILE)
                .unwrap(),
            cycles: 0,
            status_history: VecDeque::with_capacity(HISTORY_SIZE),
            is_halted: false,
            interrupt_triggered: false,
            instruction_count: 0,
            show_executed_instructions: false,
        };
        cpu
    }

    pub(crate) fn reset(&mut self) {
        let new_cpu = CPU::new();
        *self = new_cpu;
    }

    pub(crate) fn execute_cpu_cycle(&mut self, memory: &mut GBAMemory) -> CYCLES {
        self.set_executed_instruction(format_args!(""));

        //self.status_history.push_back(Status {
        //    cycles: self.cycles,
        //    registers: self.registers.active_registers.clone(),
        //    cpsr: self.cpsr.clone(),
        //    instruction_count: self.instruction_count,
        //});

        //if self.status_history.len() > HISTORY_SIZE {
        //    self.status_history.pop_front();
        //}
        //self.instruction_count += 1;

        if self.interrupt_triggered {
            self.raise_irq(memory);
            self.interrupt_triggered = false;
        }
        if self.is_halted {
            self.cycles += 1;
            return 1;
        }
        let mut execution_cycles = 0;
        if self.prefetch[1] != 0 {
            let value = self.prefetch[1];
            let decoded_instruction = self.decode_instruction(value);
            self.prefetch[1] = 0;
            self.executed_instruction_hex = value;
            execution_cycles += decoded_instruction.execute(self, memory) as u64;
        }

        if self.prefetch[1] == 0 {
            execution_cycles += self.advance_pipeline(memory) as u64;
            self.cycles += execution_cycles;
        }
        execution_cycles as u8
    }

    pub(crate) fn flush_pipeline(&mut self, memory: &mut GBAMemory) -> CYCLES {
        self.advance_pipeline(memory) + self.advance_pipeline(memory)
        //let pc = self.get_pc() as usize;
        //let (prefetch_1, prefetch_0) = {
        //    match self.get_instruction_mode() {
        //        InstructionMode::ARM => {
        //            self.set_pc(pc as u32 + 8);
        //            memory.readu32_double(pc)
        //        }
        //        InstructionMode::THUMB => {
        //            self.set_pc(pc as u32 + 4);
        //            (memory.readu16(pc).into(), memory.readu16(pc + 2).into())
        //        }
        //    }
        //};

        //self.prefetch[1] = prefetch_1.data;
        //self.prefetch[0] = prefetch_0.data;

        //prefetch_1.cycles + prefetch_0.cycles
    }

    pub(crate) fn advance_pipeline(&mut self, memory: &mut GBAMemory) -> CYCLES {
        self.prefetch[1] = self.prefetch[0];
        self.fetch_instruction(memory)
    }

    pub(crate) fn get_pc(&self) -> u32 {
        self.registers.active_registers[PC_REGISTER] & 0xFFFF_FFFE
    }

    pub(crate) fn set_pc(&mut self, address: WORD) {
        self.registers.active_registers[PC_REGISTER] = address & !1;
    }

    pub(crate) fn set_sp(&mut self, address: WORD) {
        self.set_register(13, address);
    }

    pub(crate) fn get_sp(&self) -> u32 {
        self.get_register(13)
    }

    pub(crate) fn get_cpsr(&self) -> PSR {
        self.cpsr
    }

    pub(crate) fn set_cpsr(&mut self, cpsr: PSR) {
        self.cpsr = cpsr;
        self.registers.update_registers(self.cpsr.mode());
    }

    pub(crate) fn disable_irq(&mut self) {
        self.cpsr.set_irq(false);
    }

    pub(crate) fn disable_fiq(&mut self) {
        self.cpsr.set_fiq(false);
    }

    pub(crate) fn pop_spsr(&mut self) {
        if let Some(spsr) = self.get_current_spsr() {
            self.cpsr = *spsr;
            self.set_instruction_mode(self.cpsr.instruction_mode());
            self.registers.update_registers(self.cpsr.mode());
        }
    }

    pub(crate) fn increment_pc(&mut self) {
        match self.get_instruction_mode() {
            InstructionMode::ARM => self.registers.active_registers[PC_REGISTER] += 4,
            InstructionMode::THUMB => self.registers.active_registers[PC_REGISTER] += 2,
        }
    }

    pub(crate) fn get_register(&self, register_num: REGISTER) -> WORD {
        self.registers.get_register(register_num as usize)
    }

    pub(crate) fn set_register(&mut self, register_num: REGISTER, value: WORD) {
        self.registers.set_register(register_num as usize, value);
    }

    pub(crate) fn set_flag(&mut self, flag: FlagsRegister) {
        self.cpsr.set_flag(flag);
    }

    pub(crate) fn reset_flag(&mut self, flag: FlagsRegister) {
        self.cpsr.reset_flag(flag);
    }

    pub(crate) fn get_flag(&self, flag: FlagsRegister) -> WORD {
        if self.cpsr.get_flag(flag) {
            1
        } else {
            0
        }
    }

    pub(crate) fn set_instruction_mode(&mut self, instruction_mode: InstructionMode) {
        self.cpsr.set_instruction_mode(instruction_mode);
    }

    pub(crate) fn get_instruction_mode(&self) -> InstructionMode {
        self.cpsr.instruction_mode()
    }

    pub(crate) fn set_mode(&mut self, mode: CPUMode) {
        self.cpsr.set_mode(mode);
        self.registers.update_registers(mode);
    }

    pub(crate) fn get_cpu_mode(&self) -> CPUMode {
        self.cpsr.mode()
    }

    pub(crate) fn get_current_spsr(&mut self) -> Option<&mut PSR> {
        match self.get_cpu_mode() {
            CPUMode::FIQ => Some(&mut self.spsr[0]),
            CPUMode::SVC => Some(&mut self.spsr[1]),
            CPUMode::ABT => Some(&mut self.spsr[2]),
            CPUMode::IRQ => Some(&mut self.spsr[3]),
            CPUMode::UND => Some(&mut self.spsr[4]),
            _ => None,
        }
    }

    pub(crate) fn set_flag_from_bit(&mut self, flag: FlagsRegister, bit: u8) {
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
        self.prefetch[0] = memory_fetch.data;
        self.increment_pc();

        memory_fetch.cycles
    }

    fn get_status(&self) -> Status {
        let mut status = Status::default();
        for i in 0..self.registers.active_registers.len() {
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

        write!(f, "{:08x} ", u32::from(self.cpsr))?;
        write!(f, "0\n")
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

    use crate::arm7tdmi::cpu::CPUMode;

    use super::CPU;

    #[test]
    fn it_sets_and_resets_the_corrects_flags() {
        let mut cpu = CPU::new();

        cpu.set_flag(super::FlagsRegister::C);
        cpu.set_flag(super::FlagsRegister::N);
        cpu.set_flag(super::FlagsRegister::Z);
        cpu.reset_flag(super::FlagsRegister::Z);

        assert_eq!(cpu.cpsr.get_flag(super::FlagsRegister::C), true);
        assert_eq!(cpu.cpsr.get_flag(super::FlagsRegister::N), true);
        assert_eq!(cpu.cpsr.get_flag(super::FlagsRegister::Z), false);
        assert_eq!(cpu.cpsr.get_flag(super::FlagsRegister::V), false);

        cpu.reset_flag(super::FlagsRegister::C);
        assert_eq!(cpu.cpsr.get_flag(super::FlagsRegister::C), false);
        assert_eq!(cpu.cpsr.get_flag(super::FlagsRegister::N), true);
    }

    #[test]
    fn cpu_starts_in_svc_mode() {
        let cpu = CPU::new();

        assert!(matches!(cpu.get_cpu_mode(), CPUMode::SVC));
    }
}
