use std::{
    sync::{
        mpsc::{
            Receiver,
            TryRecvError::{Disconnected, Empty},
        },
        Arc, Mutex
    },
    thread,
    time::Duration,
};

use crate::memory::{AccessFlags, Memory};

const PC_REGISTER: usize = 15;

pub struct CPU {
    registers: [u32; 31],
    pub fetched_instruction: u32,
}

pub fn start_cpu(cpu: Arc<Mutex<CPU>>, rx: Receiver<bool>) {
    let mut memory = Memory::initialize().expect("Unable to initialize memory for CPU");
    memory
        .initialize_bios(String::from("gba_bios.bin"))
        .expect("Unable to initialize bios for CPU");

    loop {
        match rx.try_recv() {
            Ok(data) => {
                if data == true {
                    break;
                }
            }
            Err(Disconnected) => break,
            Err(Empty) => {}
        }
        {
            let mut cpu = cpu.lock().unwrap();
            cpu.fetch_instruction(&memory);
            cpu.increment_pc();
        }
        thread::sleep(Duration::from_millis(500));
    }
}

impl CPU {
    pub fn initialize() -> CPU {
        CPU {
            registers: [0; 31],
            fetched_instruction: 0,
        }
    }

    #[inline(always)]
    pub fn get_pc(&self) -> u32 {
        self.registers[PC_REGISTER]
    }

    #[inline(always)]
    pub fn get_sp(&self) -> u32 {
        self.registers[13]
    }

    #[inline(always)]
    pub fn increment_pc(&mut self) {
        self.registers[PC_REGISTER] += 4;
    }

    fn fetch_instruction(&mut self, memory: &Memory) {
        self.fetched_instruction = memory
            .readu32(self.get_pc() as usize, AccessFlags::User)
            .unwrap_or_else(|_| panic!("Unable to access memory at {:#04x}", self.get_pc()));
    }
}
