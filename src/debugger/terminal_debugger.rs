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
    arm7tdmi::cpu::PC_REGISTER,
    debugger::terminal_commands::PPUToDisplayCommands,
    gba::{GBA, KILL_SIGNAL},
    graphics::display::DisplayBuffer,
};

static COMMANDS_EXECUTED: AtomicUsize = AtomicUsize::new(0);
static mut BREAK: bool = false;

pub(crate) fn breaker() {
    unsafe { BREAK = true };
}

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
    )
    .with_breaker(breaker);

    let mut command = String::new();
    let mut last_command = None;

    std::panic::set_hook(Box::new(|panic_info| {
        println!("GBA Paniced: {panic_info}");
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
        let parse_command = parse_command(&command, &last_command);
        last_command = parse_command.clone();
        if KILL_SIGNAL.killed() {
            break;
        }

        match parse_command {
            Some(command) => match command {
                Commands::Exit => {
                    break;
                }
                Commands::Next(steps) => {
                    let now = Instant::now();
                    for _ in 0..steps {
                        gba.step();
                        COMMANDS_EXECUTED.fetch_add(1, Ordering::AcqRel);
                        let region = gba.cpu.get_register(PC_REGISTER as u32) >> 24;
                        //if region != 0x0 && region != 0x8 {
                        //    println!(
                        //        "PC went out of bounds after {}",
                        //        COMMANDS_EXECUTED.load(Ordering::Acquire)
                        //    );
                        //    break;
                        //}

                        unsafe {
                            if BREAK {
                                println!(
                                    "Loop Broke {}",
                                    COMMANDS_EXECUTED.load(Ordering::Acquire)
                                );
                                BREAK = false;
                                break;
                            }
                        }
                    }

                    gba.get_status();

                    println!(
                        "Completed {} instructions in {}ms",
                        steps,
                        Instant::now().duration_since(now).as_millis()
                    );
                }
                Commands::Break(address) => todo!(),
                Commands::Continue => loop {
                    gba.step();
                    unsafe {
                        if BREAK {
                            BREAK = false;
                            break;
                        }
                    }
                },
                Commands::Read(address) => {
                    println!("{:#x}", gba.memory.read_privilegedu32(address));
                }
            },
            None => println!("Invalid Command"),
        };
    }
    Ok(())
}

#[derive(Clone)]
enum Commands {
    Exit,
    Next(u32),
    Break(usize),
    Read(usize),
    Continue,
}

fn parse_command(command_string: &String, last_command: &Option<Commands>) -> Option<Commands> {
    use Commands::*;
    let mut parts = command_string.split_ascii_whitespace();
    let Some(action) = parts.next() else {
        return Some(Next(1));
    };

    Some(match action {
        "exit" | "quit" | "q" => Exit,
        "next" | "n" => {
            let Some(count) = parts.next() else {
                return Some(Next(1));
            };
            Next(str::parse::<u32>(count).ok()?)
        }
        "break" | "b" => {
            let Some(count) = parts.next() else {
                return None;
            };
            Break(str::parse::<usize>(count).ok()?)
        }
        "read" | "r" => {
            let Some(address) = parts.next() else {
                return None;
            };
            Read(usize::from_str_radix(address, 16).ok()?)
        }
        "continue" | "c" => {
            Continue
        }
        _ => return None,
    })
}
