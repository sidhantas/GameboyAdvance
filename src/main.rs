use std::panic;
use std::sync::mpsc::sync_channel;
use std::sync::Arc;
use std::thread;

use debugger::debugger::start_debugger;
use getopts::Options;
use graphics::display::{start_display, DisplayBuffer};
use std::env;
mod arm7tdmi;
mod debugger;
mod gba;
mod graphics;
mod io;
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

    let pixel_buffer = Arc::new(DisplayBuffer::new());
    let gba_pixel_buff = pixel_buffer.clone();
    let (ppu_to_display_send, ppu_to_display_recv) = sync_channel(1);
    thread::scope(move |scope| {
        scope.spawn(move || start_debugger(bios, rom, gba_pixel_buff, ppu_to_display_send));
        start_display(pixel_buffer.clone(), ppu_to_display_recv);
    });

    Ok(())
}
