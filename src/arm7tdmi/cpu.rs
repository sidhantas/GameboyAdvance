use std::{
    sync::{
        mpsc::{
            Receiver,
            TryRecvError::{Disconnected, Empty},
        },
        Arc, Mutex
    }, thread, time::Duration
};

use crate::{debugger::debugger::DebugCommands, memory::{AccessFlags, Memory}, types::{ARMByteCode, WORD}};

use super::instructions::ARMDecodedInstruction;

const PC_REGISTER: usize = 15;

pub enum InstructionMode {
    ARM,
    THUMB
}

pub enum CPUMode {
    USER,
    FIQ, // Fast Interrupt
    SVC, // Supervisor
    ABT, // Abort
    UND // Undefined
}

pub struct CPU {
    registers: [u32; 31],
    pub inst_mode: InstructionMode,
    pub cpu_mode: CPUMode,
    memory: Arc<Mutex<Memory>>,
    pub decoded_instruction: ARMDecodedInstruction,
    pub fetched_instruction: ARMByteCode,
    pub executed_instruction: String,
}

pub fn cpu_thread(cpu: Arc<Mutex<CPU>>, rx: Receiver<DebugCommands>) {
    let mut instructions_left = 0; 
    loop {
        match rx.try_recv() {
            Ok(data) => {
                match data {
                    DebugCommands::End => {
                        break;
                    }
                    DebugCommands::Continue => {
                        instructions_left += 1;
                    }

                }
            }
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
            let executable = self.decoded_instruction.executable;
            let instruction = self.decoded_instruction.instruction;
            executable(self, instruction);
            self.decode_instruction(self.fetched_instruction);
            self.fetch_instruction();
    }

    pub fn new(memory: Arc<Mutex<Memory>>) -> CPU {
        CPU {
            registers: [0; 31],
            inst_mode: InstructionMode::ARM,
            cpu_mode: CPUMode::USER,
            fetched_instruction: 0,
            decoded_instruction: ARMDecodedInstruction {
                instruction: 0,
                executable: CPU::arm_nop
            },
            memory,
            executed_instruction: String::from("")
        }
    }

    pub fn flush_pipeline(&mut self) {
        self.decoded_instruction = ARMDecodedInstruction {
            instruction: 0,
            executable: CPU::arm_nop
        };
        self.fetched_instruction = 0;
    }

    #[inline(always)]
    pub fn get_pc(&self) -> u32 {
        self.registers[PC_REGISTER]
    }

    #[inline(always)]
    pub fn set_pc(&mut self, address: WORD) {
        self.registers[PC_REGISTER] = address;
    }

    #[inline(always)]
    pub fn get_sp(&self) -> u32 {
        self.registers[13]
    }

    #[inline(always)]
    pub fn increment_pc(&mut self) {
        self.registers[PC_REGISTER] += 4;
    }

    fn fetch_instruction(&mut self) {
        self.fetched_instruction = self.memory.lock().unwrap()
            .readu32(self.get_pc() as usize, AccessFlags::User)
            .unwrap_or_else(|_| panic!("Unable to access memory at {:#04x}", self.get_pc()));
        self.increment_pc();
    }
}
