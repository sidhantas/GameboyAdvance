use std::{sync::{mpsc, Arc, Mutex}, thread};

use arm7tdmi::cpu::{start_cpu, CPU};
use debugger::debugger::start_debugger;

mod arm7tdmi;
mod debugger;
mod memory;
mod types;
mod utils;

fn main() -> Result<(), std::io::Error> {

    let cpu = Arc::new(Mutex::new(CPU::initialize()));
    let (tx, rx) = mpsc::channel();

    ctrlc::set_handler(move || tx.send(true).unwrap()).unwrap();

    thread::scope(|scope| {
        let debug_cpu = Arc::clone(&cpu);
        scope.spawn(move || start_cpu(cpu, rx));
        scope.spawn(move || start_debugger(debug_cpu));
    });

    Ok(())
}
