use std::thread;

use arm7tdmi::cpu::CPU;
use debugger::debugger::start_debugger;
use memory::Memory;

mod memory;
mod types;
mod arm7tdmi;
mod utils;
mod debugger;

fn main() -> Result<(), std::io::Error> {
    let mut memory = Memory::initialize().unwrap();
    let cpu = CPU::initialize();
    memory.initialize_bios(String::from("gba_bios.bin"))?;

    let debugger = thread::spawn(|| {
        start_debugger(cpu)
    });

    let _ = debugger.join().unwrap();

    Ok(())
}
