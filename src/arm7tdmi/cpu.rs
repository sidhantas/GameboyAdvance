use std::
    sync::{
        mpsc::{
            Receiver,
            TryRecvError::{Disconnected, Empty},
        },
        Arc, Mutex,
    }
;

use crate::{
    debugger::{breakpoints::BreakType, debugger::DebugCommands},
    memory::memory::GBAMemory,
    types::*,
    utils::bits::Bits,
};


pub const PC_REGISTER: usize = 15;
pub const LINK_REGISTER: u32 = 14;

pub enum InstructionMode {
    ARM,
    THUMB,
}

#[derive(PartialEq)]
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

pub struct CPU {
    registers: [WORD; 16],
    registers_fiq: [WORD; 8],
    registers_svc: [WORD; 2],
    registers_abt: [WORD; 2],
    registers_irq: [WORD; 2],
    registers_und: [WORD; 2],
    pub memory: GBAMemory,
    pub prefetch: [Option<WORD>; 2],
    pub executed_instruction_hex: ARMByteCode,
    pub executed_instruction: String,
    pub cpsr: WORD,
    pub spsr: [WORD; 5],
    pub breakpoints: Vec<BreakType>,
}

pub fn cpu_thread(cpu: Arc<Mutex<CPU>>, rx: Receiver<DebugCommands>) {
    let mut instructions_left = 0;
    loop {
        loop {
            match rx.try_recv() {
                Ok(data) => match data {
                    DebugCommands::End => {
                        return;
                    }
                    DebugCommands::Continue(num) => {
                        instructions_left += num;
                    }
                    DebugCommands::SetBreakpoint(breakpoint) => {
                        cpu.lock().unwrap().breakpoints.push(breakpoint);
                    }
                    DebugCommands::DeleteBreakpoint(breakpoint_num) => {
                        cpu.lock()
                            .unwrap()
                            .breakpoints
                            .remove(breakpoint_num as usize);
                    }
                },
                Err(Disconnected) => return,
                Err(Empty) => {
                    break;
                }
            }
        }
        let mut cpu = cpu.lock().unwrap();
        while instructions_left > 0 {
            cpu.execute_cpu_cycle();
            let pc = cpu.get_pc();
            for breakpoint in &cpu.breakpoints {
                match *breakpoint {
                    BreakType::Break(break_pc) if break_pc == pc => instructions_left = 0,
                    BreakType::WatchRegister(register, value) if cpu.get_register(register) == value => {
                        instructions_left = 0
                    }
                    _ => {}
                }
            }
        }
    }
}

impl CPU {
    pub fn execute_cpu_cycle(&mut self) {
        if let Some(value) = self.prefetch[1] {
            let decoded_instruction = self.decode_instruction(value);
            self.executed_instruction_hex = self.prefetch[1].unwrap_or(0x0);
            self.prefetch[1] = None;
            ((decoded_instruction.executable)(self, decoded_instruction.instruction));
        }

        if let None = self.prefetch[1] {
            // refill pipeline if decoded instruction doesn't advance the pipeline
            self.advance_pipeline();
        }
    }

    pub fn new(memory: GBAMemory) -> CPU {
        CPU {
            registers: [0; 16],
            executed_instruction_hex: 0,
            memory,
            executed_instruction: String::from(""),
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
            breakpoints: Vec::new(),
        }
    }

    pub fn flush_pipeline(&mut self) -> CYCLES {
        let mut cycles = 0;
        self.prefetch[0] = None;
        self.prefetch[1] = None;

        cycles += self.advance_pipeline();
        cycles += self.advance_pipeline();

        cycles
    }

    pub fn advance_pipeline(&mut self) -> CYCLES {
        self.prefetch[1] = self.prefetch[0];
        self.fetch_instruction()
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
            _ => panic!("Impossible cpsr value"),
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

    fn fetch_instruction(&mut self) -> CYCLES {
        let mut cycles = 0;
        let memory_fetch = {
            match self.get_instruction_mode() {
                InstructionMode::ARM => self.memory.readu32(self.get_pc() as usize),
                InstructionMode::THUMB => self.memory
                    .readu16(self.get_pc() as usize)
                    .into(),
            }
        };
        cycles += memory_fetch.cycles;
        self.prefetch[0] = Some(memory_fetch.data);
        self.increment_pc();

        cycles
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
                if operand_register_value.bit_is_set((32 - shift_amount) as u8) {
                    self.set_flag(FlagsRegister::C);
                } else {
                    self.reset_flag(FlagsRegister::C);
                }
                operand_register_value << shift_amount
            }
            // Logical shift right
            0x01 => {
                if operand_register_value.bit_is_set((shift_amount - 1) as u8) {
                    self.set_flag(FlagsRegister::C);
                } else {
                    self.reset_flag(FlagsRegister::C);
                }
                operand_register_value >> shift_amount
            }
            // Arithmetic shift right
            0x02 => {
                if operand_register_value.bit_is_set((shift_amount - 1) as u8) {
                    self.set_flag(FlagsRegister::C);
                } else {
                    self.reset_flag(FlagsRegister::C);
                }
                (operand_register_value as i32 >> shift_amount) as u32
            }
            // Rotate Right
            0x03 => {
                if set_flags {
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
}

#[cfg(test)]
mod cpu_tests {
    

    use crate::{arm7tdmi::cpu::CPUMode, memory::memory::GBAMemory, utils::bits::Bits};

    use super::CPU;

    #[test]
    fn it_sets_and_resets_the_corrects_flags() {
        let memory = GBAMemory::new();
        
        let mut cpu = CPU::new(memory);

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
        let memory = GBAMemory::new();
        
        let cpu = CPU::new(memory);

        assert!(matches!(cpu.get_cpu_mode(), CPUMode::SVC));
    }
}
