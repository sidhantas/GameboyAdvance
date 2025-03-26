use core::panic;
use std::fs::{remove_file, File, OpenOptions};

use crate::{memory::memory::GBAMemory, types::*, utils::bits::Bits};

use super::cpsr::PSR;

pub const PC_REGISTER: usize = 15;
pub const LINK_REGISTER: u32 = 14;
pub const STACK_POINTER: u32 = 13;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum InstructionMode {
    ARM,
    THUMB,
}

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum CPUMode {
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
pub enum FlagsRegister {
    N = 31,
    Z = 30,
    C = 29,
    V = 28,
}

//#[derive(Default, Debug)]
//struct Status {
//    pub instruction_count: usize,
//    pub registers: [WORD; 16],
//    pub cpsr: PSR,
//    pub cycles: u64,
//}

#[derive(Debug)]
pub struct CPU {
    registers: [WORD; 16],
    registers_fiq: [WORD; 16],
    registers_svc: [WORD; 16],
    registers_abt: [WORD; 16],
    registers_irq: [WORD; 16],
    registers_und: [WORD; 16],
    pub is_halted: bool,
    pub prefetch: [Option<WORD>; 2],
    pub executed_instruction_hex: ARMByteCode,
    pub executed_instruction: String,
    cpsr: PSR,
    pub spsr: [PSR; 5],
    pub output_file: File,
    pub cycles: u64,
    //status_history: VecDeque<Status>,
    pub interrupt_triggered: bool,
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
            cpsr: PSR::new_cpsr(),
            spsr: [PSR::new_spsr(); 5],
            registers_fiq: [0; 16],
            registers_svc: [0; 16],
            registers_abt: [0; 16],
            registers_irq: [0; 16],
            registers_und: [0; 16],
            output_file: OpenOptions::new()
                .create(true)
                .write(true)
                .open(OUTPUT_FILE)
                .unwrap(),
            cycles: 0,
            //status_history: VecDeque::with_capacity(HISTORY_SIZE),
            is_halted: false,
            interrupt_triggered: false,
        };
        cpu
    }

    pub fn execute_cpu_cycle(&mut self, memory: &mut GBAMemory) -> CYCLES {
        self.set_executed_instruction(format_args!(""));
        //if self.status_history.len() >= HISTORY_SIZE {
        //    self.status_history.pop_front();
        //}
        //unsafe {
        //    INSTRUCTION_COUNT += 1;
        //}
        //self.status_history.push_back(self.get_status());
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
                ((decoded_instruction.executable)(self, decoded_instruction.instruction, memory))
                    as u64;
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

    pub fn get_cpsr(&self) -> PSR {
        self.cpsr
    }

    pub fn set_cpsr(&mut self, cpsr: PSR) {
        self.cpsr = cpsr;
    }

    pub fn disable_irq(&mut self) {
        self.cpsr.irq_disabled = true;
    }

    pub fn disable_fiq(&mut self) {
        self.cpsr.fiq_disabled = true;
    }

    pub fn pop_spsr(&mut self) {
        if let Some(spsr) = self.get_current_spsr() {
            self.cpsr = *spsr;
        }
    }

    pub fn increment_pc(&mut self) {
        match self.get_instruction_mode() {
            InstructionMode::ARM => self.registers[PC_REGISTER] += 4,
            InstructionMode::THUMB => self.registers[PC_REGISTER] += 2,
        }
    }

    pub fn get_register(&self, register_num: REGISTER) -> WORD {
        let cpu_mode = self.get_cpu_mode();
        if let CPUMode::FIQ = cpu_mode {
            if register_num >= 8 && register_num < 15 {
                return self.registers_fiq[register_num as usize];
            }
        }

        if register_num < 13 || register_num == 15 {
            return self.registers[register_num as usize];
        }

        match cpu_mode {
            CPUMode::FIQ => unreachable!(), // Shouldn't happen
            CPUMode::USER | CPUMode::SYS => self.registers[register_num as usize],
            CPUMode::SVC => self.registers_svc[(register_num) as usize],
            CPUMode::UND => self.registers_und[(register_num) as usize],
            CPUMode::IRQ => self.registers_irq[(register_num) as usize],
            CPUMode::ABT => self.registers_abt[(register_num) as usize],
            CPUMode::INVALID(_) => todo!(),
        }
    }

    pub fn set_register(&mut self, register_num: REGISTER, value: WORD) {
        if register_num < 8 || register_num == 15 {
            self.registers[register_num as usize] = value;
            return;
        }

        let cpu_mode = self.get_cpu_mode();

        if let CPUMode::FIQ = cpu_mode {
            if register_num >= 8 && register_num < 15 {
                self.registers_fiq[register_num as usize] = value;
                return;
            }
        }

        if register_num < 13 || register_num == 15 {
            self.registers[register_num as usize] = value;
            return;
        }

        match cpu_mode {
            CPUMode::FIQ => unreachable!(), // Shouldn't happen
            CPUMode::USER | CPUMode::SYS => self.registers[register_num as usize] = value,
            CPUMode::SVC => self.registers_svc[(register_num) as usize] = value,
            CPUMode::UND => self.registers_und[(register_num) as usize] = value,
            CPUMode::IRQ => self.registers_irq[(register_num) as usize] = value,
            CPUMode::ABT => self.registers_abt[(register_num) as usize] = value,
            CPUMode::INVALID(_) => todo!(),
        };
    }

    #[inline(always)]
    pub fn set_flag(&mut self, flag: FlagsRegister) {
        self.cpsr.set_flag(flag);
    }

    #[inline(always)]
    pub fn reset_flag(&mut self, flag: FlagsRegister) {
        self.cpsr.reset_flag(flag);
    }

    #[inline(always)]
    pub fn get_flag(&self, flag: FlagsRegister) -> WORD {
        if self.cpsr.get_flag(flag) {
            1
        } else {
            0
        }
    }

    #[inline(always)]
    pub fn set_instruction_mode(&mut self, instruction_mode: InstructionMode) {
        self.cpsr.instruction_mode = instruction_mode;
    }

    #[inline(always)]
    pub fn get_instruction_mode(&self) -> InstructionMode {
        self.cpsr.instruction_mode
    }

    pub fn set_mode(&mut self, mode: CPUMode) {
        self.cpsr.mode = mode;
    }

    pub fn get_cpu_mode(&self) -> CPUMode {
        self.cpsr.mode
    }

    pub fn get_current_spsr(&mut self) -> Option<&mut PSR> {
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

    //fn get_status(&self) -> Status {
    //    let mut status = Status::default();
    //    for i in 0..self.registers.len() {
    //        status.registers[i] = self.get_register(i as REGISTER);
    //    }
    //    status.cpsr = self.cpsr;
    //    status.cycles = 0;
    //    unsafe {
    //        status.instruction_count = INSTRUCTION_COUNT;
    //    }
    //    status
    //}
}

//impl Display for Status {
//    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//        write!(f, "{} ", self.instruction_count)?;
//        for i in self.registers {
//            write!(f, "{:08x} ", i)?;
//        }
//
//        write!(f, "{:08x} ", self.cpsr)?;
//        write!(f, "{}\n", self.cycles)
//    }
//}

//impl Drop for CPU {
//    fn drop(&mut self) {
//        for i in self.status_history.iter().skip(1) {
//            self.output_file.write(format!("{}", i).as_bytes()).unwrap();
//        }
//    }
//}

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
