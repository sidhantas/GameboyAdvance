use std::panic;
use std::thread;

use debugger::debugger::start_debugger;
use getopts::Options;
use memory::memory::{GBAMemory, MemoryBus};
use std::env;
mod arm7tdmi;
mod debugger;
mod graphics;
mod memory;
mod types;
mod utils;

fn main() -> Result<(), std::io::Error> {
    let args: Vec<String> = env::args().collect();

    let mut opts = Options::new();
    opts.optopt("b", "bios", "set bios", "BIOS");
    opts.optopt("g", "game", "set game rom", "ROM");
    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(_) => {
            panic!("Invalid Argument")
        }
    };

    let bios = matches.opt_str("b").unwrap_or(String::from("gba_bios.bin"));
    let rom = matches.opt_str("g").unwrap();

    //let display_memory = memory.clone();

    thread::scope(move |scope| {
        scope.spawn(move || start_debugger(bios, rom));
        //start_display(display_memory);
    });

    Ok(())
}
