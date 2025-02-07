use std::panic;
use std::sync::{Arc, Mutex};
use std::thread;

use debugger::debugger::start_debugger;
use gameboy_advance::graphics::display::CANVAS_AREA;
use getopts::Options;
use graphics::display::start_display;
use std::env;
mod arm7tdmi;
mod debugger;
mod gba;
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

    let pixel_buffer = Arc::new(Mutex::new([0u32; CANVAS_AREA]));
    let gba_pixel_buff = pixel_buffer.clone();
    thread::scope(move |scope| {
        scope.spawn(move || start_debugger(bios, rom, gba_pixel_buff));
        start_display(pixel_buffer.clone());
    });

    Ok(())
}
