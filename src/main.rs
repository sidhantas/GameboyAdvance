use std::{
    sync::{mpsc, Arc, Mutex},
    thread,
};

use arm7tdmi::cpu::{cpu_thread, CPU};
use debugger::debugger::{start_debugger, DebugCommands};
use getopts::Options;
use memory::Memory;
use std::env;

mod arm7tdmi;
mod debugger;
mod memory;
mod types;
mod utils;

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

    let mut memory = Memory::new().expect("Unable to initialize memory for CPU");
    memory
        .initialize_bios(bios)
        .expect("Unable to initialize bios for CPU");

    let memory = Arc::new(Mutex::new(memory));

    let cpu = Arc::new(Mutex::new(CPU::new(memory)));
    let (tx, rx) = mpsc::channel();

    let ctrlc_tx = tx.clone();

    ctrlc::set_handler(move || ctrlc_tx.send(DebugCommands::End).unwrap()).unwrap();

    thread::scope(|scope| {
        let debug_cpu = Arc::clone(&cpu);
        scope.spawn(move || cpu_thread(cpu, rx));
        scope.spawn(move || start_debugger(debug_cpu, tx));
    });

    Ok(())
}
