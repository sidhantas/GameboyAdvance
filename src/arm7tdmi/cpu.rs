use std::sync::{
    mpsc::{
        Receiver,
        TryRecvError::{Disconnected, Empty},
    },
    Arc, Mutex,
};

use crate::{
    debugger::debugger::DebugCommands,
    memory::{AccessFlags, Memory},
    types::{ARMByteCode, REGISTER, WORD},
    utils::bits::Bits,
};

use super::{alu::ALUInstruction, instructions::{ALUOperation, ARMDecodedInstruction}};

pub const PC_REGISTER: usize = 15;
pub const LINK_REGISTER: u32 = 14;

pub enum InstructionMode {
    ARM,
    THUMB,
}

pub enum CPUMode {
    USER,
    FIQ, // Fast Interrupt
    SVC, // Supervisor
    ABT, // Abort
    UND, // Undefined
}

#[repr(u8)]
pub enum FlagsRegister {
    C = 31,
    N = 30,
    Z = 29,
    V = 28,
}

pub struct CPU {
    registers: [WORD; 31],
    pub inst_mode: InstructionMode,
    pub cpu_mode: CPUMode,
    memory: Arc<Mutex<Memory>>,
    pub decoded_instruction: ARMDecodedInstruction,
    pub fetched_instruction: ARMByteCode,
    pub executed_instruction: String,
    pub alu_executable: ALUInstruction,
    pub cpsr: WORD,
    pub spsr: [WORD; 5],
}

pub fn cpu_thread(cpu: Arc<Mutex<CPU>>, rx: Receiver<DebugCommands>) {
    let mut instructions_left = 0;
    loop {
        match rx.try_recv() {
            Ok(data) => match data {
                DebugCommands::End => {
                    break;
                }
                DebugCommands::Continue => {
                    instructions_left += 1;
                }
            },
            Err(Disconnected) => break,
            Err(Empty) => {}
        }
        if instructions_left > 0 {
            let mut cpu = cpu.lock().unwrap();
            cpu.execute_cpu_cycle();
            instructions_left -= 1;
        }
    }
}

impl CPU {
    pub fn execute_cpu_cycle(&mut self) {
        if self.alu_executable.instruction != 0 {
            // if the alu has an instruction at the beginning
            // of a cycle that means that it is stalled on getting
            // a register so it needs 2 clocks to complete an instruction
            self.execute_alu_operation();
        }
        let executable = self.decoded_instruction.executable;
        let instruction = self.decoded_instruction.instruction;
        executable(self, instruction);
        self.decode_instruction(self.fetched_instruction);
        self.fetch_instruction();
    }

    pub fn execute_alu_operation(&mut self) {
        let operand2 = self.decode_operand2(
            self.alu_executable.instruction,
            self.alu_executable.set_flags,
        );
        (self.alu_executable.operation)(
            self,
            self.alu_executable.rd,
            self.get_register(self.alu_executable.rn),
            operand2,
            self.alu_executable.set_flags,
        );
        self.alu_executable = Default::default();
        if self.alu_executable.rd == 15 {
            self.flush_pipeline();
        }
    }

    pub fn new(memory: Arc<Mutex<Memory>>) -> CPU {
        CPU {
            registers: [0; 31],
            inst_mode: InstructionMode::ARM,
            cpu_mode: CPUMode::USER,
            fetched_instruction: 0,
            decoded_instruction: ARMDecodedInstruction {
                ..Default::default()
            },
            memory,
            executed_instruction: String::from(""),
            cpsr: 0,
            spsr: [0; 5],
            alu_executable: ALUInstruction {
                ..Default::default()
            },
        }
    }

    pub fn flush_pipeline(&mut self) {
        self.decoded_instruction = ARMDecodedInstruction {
            ..Default::default()
        };
        self.fetched_instruction = 0;
    }

    pub fn get_pc(&self) -> u32 {
        self.registers[PC_REGISTER]
    }

    pub fn set_pc(&mut self, address: WORD) {
        self.registers[PC_REGISTER] = address;
    }

    pub fn get_sp(&self) -> u32 {
        self.registers[13]
    }

    pub fn increment_pc(&mut self) {
        self.registers[PC_REGISTER] += 4;
    }

    pub fn get_register(&self, register_num: REGISTER) -> WORD {
        assert!(register_num < 16);
        self.registers[register_num as usize]
    }

    pub fn set_register(&mut self, register_num: REGISTER, value: WORD) {
        assert!(register_num < 16);
        self.registers[register_num as usize] = value;
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
    pub fn get_flag(&mut self, flag: FlagsRegister) -> WORD {
        self.cpsr.get_bit(flag as u8)
    }

    pub fn set_flag_from_bit(&mut self, flag: FlagsRegister, bit: u8) {
        assert!(bit == 0 || bit == 1);
        if bit == 0 {
            self.reset_flag(flag);
            return;
        }
        self.set_flag(flag);
    }

    fn fetch_instruction(&mut self) {
        self.fetched_instruction = self
            .memory
            .lock()
            .unwrap()
            .readu32(self.get_pc() as usize, AccessFlags::User)
            .unwrap_or_else(|_| panic!("Unable to access memory at {:#04x}", self.get_pc()));
        self.increment_pc();
    }

    fn decode_operand2(&mut self, instruction: ARMByteCode, set_flags: bool) -> u32 {
        let shift_amount;
        if instruction.bit_is_set(25) {
            // operand 2 is immediate
            shift_amount = ((instruction & 0x0000_0F00) >> 8) * 2;
            let immediate = instruction & 0x0000_00FF;

            return immediate.rotate_right(shift_amount);
        }
        let shift_type = (instruction & 0x0000_0060) >> 5;
        let operand_register = instruction & 0x0000_000F;
        let operand_register_value = self.get_register(operand_register);

        if instruction.bit_is_set(4) {
            // shift by register
            let shift_register = (instruction & 0x0000_0F00) >> 8;
            shift_amount = self.get_register(shift_register);
        } else {
            shift_amount = (instruction & 0x0000_0F80) >> 7;
            if shift_amount == 0 {
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
    use std::sync::{Arc, Mutex};

    use crate::{memory::Memory, utils::bits::Bits};

    use super::CPU;

    #[test]
    fn it_sets_and_resets_the_corrects_flags() {
        let memory = Memory::new().unwrap();
        let memory = Arc::new(Mutex::new(memory));
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
}
