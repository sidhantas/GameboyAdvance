use std::panic;
use std::{
    sync::{mpsc, Arc, Mutex},
    thread,
};

use arm7tdmi::cpu::{cpu_thread, CPU};
use debugger::debugger::start_debugger;
use getopts::Options;
use memory::memory::GBAMemory;
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
    let (cpu_tx, cpu_rx) = mpsc::channel();
    let (debug_tx, debug_rx) = mpsc::channel();

    thread::scope(move |scope| {
        let debug_cpu = Arc::clone(&cpu);
        let debug_cpu_sender = cpu_tx.clone();
        scope.spawn(move || cpu_thread(cpu, cpu_rx));
        scope.spawn(move || start_debugger(debug_cpu, debug_cpu_sender, debug_rx));
        //start_display(display_memory);
    });

    Ok(())
}
