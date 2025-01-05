use std::panic;
use std::{
    sync::{Arc, Mutex},
    thread,
};

use arm7tdmi::cpu::CPU;
use debugger::debugger::start_debugger;
use getopts::Options;
use memory::memory::{GBAMemory, MemoryBus};
use std::env;
mod arm7tdmi;
mod debugger;
mod memory;
mod types;
mod utils;
mod graphics;

fn main() -> Result<(), std::io::Error> {
    let args: Vec<String> = env::args().collect();

    let mut opts = Options::new();
    opts.optopt("b", "bios", "set bios", "BIOS");
    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(_) => {
            panic!("Invalid Argument")
        }
    };

    let bios = matches.opt_str("b").unwrap_or(String::from("gba_bios.bin"));

    let mut memory = GBAMemory::new();
    memory
        .initialize_bios(bios)
        .expect("Unable to initialize bios for CPU");

    //let display_memory = memory.clone();
    let cpu = Arc::new(Mutex::new(CPU::new(memory)));

    thread::scope(move |scope| {
        let debug_cpu = Arc::clone(&cpu);
        scope.spawn(move || start_debugger(debug_cpu));
        //start_display(display_memory);
    });

    Ok(())
}
