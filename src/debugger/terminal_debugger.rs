use std::{
    io::{self, stdout, Write},
    panic::catch_unwind,
    sync::{
        atomic::{AtomicUsize, Ordering},
        mpsc::Sender,
        Arc,
    },
    time::Instant,
};

use crate::{
    debugger::terminal_commands::PPUToDisplayCommands,
    gba::{GBA, KILL_SIGNAL},
    graphics::display::DisplayBuffer,
};

static COMMANDS_EXECUTED: AtomicUsize = AtomicUsize::new(0);

pub(crate) fn start_debugger(
    bios: String,
    rom: String,
    pixel_buffer: Arc<DisplayBuffer>,
    ppu_to_display_sender: Sender<PPUToDisplayCommands>,
) -> Result<(), std::io::Error> {
    let mut gba = GBA::new(
        bios.clone(),
        rom.clone(),
        pixel_buffer.clone(),
        ppu_to_display_sender.clone(),
    );

    let mut command = String::new();

    std::panic::set_hook(Box::new(|_| {
        println!("GBA Paniced");
        println!(
            "Last Successful Instruction: {}",
            COMMANDS_EXECUTED.load(Ordering::Relaxed)
        );
        let bt = std::backtrace::Backtrace::capture();
        println!("{}", bt);
    }));

    loop {
        print!(">> ");
        let _ = stdout().flush();
        command.clear();
        let _ = io::stdin().read_line(&mut command);
        if KILL_SIGNAL.killed() {
            break;
        }
        match parse_command(&command) {
            Some(command) => match command {
                Commands::Exit => {
                    break;
                }
                Commands::Next(steps) => {
                    let now = Instant::now();
                    for _ in 0..steps {
                        gba.step();
                        COMMANDS_EXECUTED.fetch_add(1, Ordering::AcqRel);
                    }


                    gba.get_status();

                    println!(
                        "Completed {} instructions in {}ms",
                        steps,
                        Instant::now().duration_since(now).as_millis()
                    );
                }
            },
            None => println!("Invalid Command"),
        };
    }
    Ok(())
}

enum Commands {
    Exit,
    Next(u32),
}

fn parse_command(command_string: &String) -> Option<Commands> {
    use Commands::*;
    let mut parts = command_string.split_ascii_whitespace();
    let Some(action) = parts.next() else {
        return None;
    };

    Some(match action {
        "exit" | "quit" => Exit,
        "next" | "n" => {
            let Some(count) = parts.next() else {
                return Some(Next(1));
            };
            Next(str::parse::<u32>(count).ok()?)
        }
        _ => return None,
    })
}
