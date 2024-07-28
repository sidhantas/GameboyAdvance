use std::{sync::{Arc, Mutex}, thread};

use arm7tdmi::cpu::{start_cpu, CPU};
use debugger::debugger::start_debugger;
use memory::Memory;

mod arm7tdmi;
mod debugger;
mod memory;
mod types;
mod utils;

fn main() -> Result<(), std::io::Error> {
    let mut cpu = CPU::initialize();

    thread::scope(|scope| {
        let arc_cpu = Arc::new(cpu);
        let debug_cpu = Arc::clone(&arc_cpu);
        scope.spawn(|| start_cpu(arc_cpu));
        scope.spawn(|| start_debugger(debug_cpu));
    });

    Ok(())
}
