use super::{
    breakpoints::{BreakType, Breakpoint, TriggeredWatchpoints},
    terminal_commands::PPUToDisplayCommands,
};
use crossterm::{
    event::{
        self, read, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent, KeyModifiers,
    },
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::{
    cell::RefCell,
    io::{self, Stdout},
    rc::Rc,
    sync::{mpsc::Sender, Arc},
    time::Duration,
};
use tui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::Style,
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame, Terminal,
};

use crate::{
    arm7tdmi::{
        cpu::{CPUMode, FlagsRegister, InstructionMode, CPU},
        instruction_table::{
            condition_code_as_str, DecodeARMInstructionToString, DecodeThumbInstructionToString,
            Instruction,
        },
        thumb::alu::ThumbFullAdder,
    },
    gba::{GBA, KILL_SIGNAL},
    graphics::display::DisplayBuffer,
    memory::{
        io_handlers::{DISPCNT, VCOUNT},
        memory::GBAMemory,
    },
    utils::bits::Bits,
};

use super::terminal_commands::{parse_command, TerminalHistoryEntry};

pub struct Debugger {
    pub memory_start_address: u32,
    pub terminal_buffer: String,
    pub terminal_history: Vec<TerminalHistoryEntry>,
    pub terminal_enabled: bool,
    pub end_debugger: bool,
    pub gba: GBA,
    pub breakpoints: Option<Vec<Breakpoint>>,
    pub triggered_watchpoints: Rc<RefCell<Vec<TriggeredWatchpoints>>>,
}

impl Debugger {
    pub fn new(
        bios: String,
        rom: String,
        pixel_buffer: Arc<DisplayBuffer>,
        ppu_to_display_sender: Sender<PPUToDisplayCommands>,
    ) -> Self {
        let breakpoints = Some(Vec::<Breakpoint>::new());
        let triggered_watchpoints = Rc::new(RefCell::new(Vec::<TriggeredWatchpoints>::new()));

        let mut gba = GBA::new(bios, rom, pixel_buffer, ppu_to_display_sender);
        gba.memory.breakpoint_checker = Some(Box::new(|memory: &GBAMemory, address: usize| {
            let Some(memory_breakpoints) = memory.breakpoints.as_ref() else {
                return;
            };
            let triggered_breakpoints = &memory.triggered_breakpoints;
            for watchpoint in memory_breakpoints.iter() {
                if let BreakType::WatchAddress(low, high) = watchpoint.break_type {
                    if low <= address && address <= high {
                        triggered_breakpoints
                            .borrow_mut()
                            .push(TriggeredWatchpoints::Address(address));
                    }
                }
            }
        }));

        Self {
            memory_start_address: 0x0000000,
            terminal_buffer: String::new(),
            terminal_history: Vec::new(),
            terminal_enabled: true,
            end_debugger: false,
            gba,
            breakpoints,
            triggered_watchpoints,
        }
    }
}

pub fn start_debugger(
    bios: String,
    rom: String,
    pixel_buffer: Arc<DisplayBuffer>,
    ppu_to_display_sender: Sender<PPUToDisplayCommands>,
) -> Result<(), std::io::Error> {
    enable_raw_mode()?;
    execute!(io::stdout(), EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(io::stdout());
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    let debugger = &mut Debugger::new(bios, rom, pixel_buffer, ppu_to_display_sender);

    while !debugger.end_debugger {
        if KILL_SIGNAL.killed() {
            debugger.end_debugger = true;
        }
        loop {
            if event::poll(Duration::from_millis(10))? {
                if let Event::Key(event) = read()? {
                    if event.modifiers == KeyModifiers::CONTROL {
                        handle_control_events(debugger, event);
                    } else if debugger.terminal_enabled {
                        handle_terminal_events(debugger, event);
                    } else {
                        handle_normal_mode_events(debugger, event);
                    }
                }
            } else {
                break;
            }
        }

        let Ok(_) = terminal.draw(|f| {
            let vertical_chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(1)
                .constraints(
                    [
                        Constraint::Length(15),
                        Constraint::Length(21),
                        Constraint::Length(15),
                    ]
                    .as_ref(),
                )
                .split(f.size());

            let horizontal_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .margin(1)
                .constraints(
                    [
                        Constraint::Length(20),
                        Constraint::Length(20),
                        Constraint::Length(20),
                        Constraint::Length(20),
                        Constraint::Length(20),
                        Constraint::Length(50),
                    ]
                    .as_ref(),
                )
                .split(vertical_chunks[0]);

            let horizontal_chunks_1 = Layout::default()
                .direction(Direction::Horizontal)
                .margin(1)
                .constraints(
                    [
                        Constraint::Length(110),
                        Constraint::Length(40),
                        Constraint::Length(50),
                    ]
                    .as_ref(),
                )
                .split(vertical_chunks[1]);

            let cpu_chunk = horizontal_chunks[0];
            let register_chunk = horizontal_chunks[1];
            let register_chunk_2 = horizontal_chunks[2];
            let flags_chunk = horizontal_chunks[3];
            let ppu_chunk = horizontal_chunks[4];
            let memory_chunk = horizontal_chunks_1[0];
            let terminal_chunk = horizontal_chunks_1[1];

            {
                let cpu = &debugger.gba;
                draw_cpu(f, cpu_chunk, &cpu.cpu).unwrap();
                draw_ppu(f, ppu_chunk, cpu).unwrap();
                draw_registers(f, register_chunk, 0, &cpu.cpu).unwrap();
                draw_registers(f, register_chunk_2, 10, &cpu.cpu).unwrap();
                draw_cpsr(f, flags_chunk, &cpu.cpu).unwrap();
                draw_memory(f, memory_chunk, &cpu, &debugger).unwrap();
                draw_terminal(f, terminal_chunk, &debugger).unwrap();
            }
        }) else {
            break;
        };
    }

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}

fn draw_ppu(
    f: &mut Frame<'_, CrosstermBackend<Stdout>>,
    ppu_chunk: Rect,
    cpu: &GBA,
) -> Result<(), std::io::Error> {
    let block = Block::default()
        .title("PPU")
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL);

    let flags_sections = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(ppu_chunk);

    let ppu_regs = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1); 8])
        .split(flags_sections[0]);

    let ppu_values = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1); 8])
        .split(flags_sections[1]);

    f.render_widget(
        Paragraph::new(format!("VCOUNT")).alignment(Alignment::Center),
        ppu_regs[1],
    );

    f.render_widget(
        Paragraph::new(format!("{}", cpu.memory.io_load(VCOUNT))).alignment(Alignment::Center),
        ppu_values[1],
    );

    f.render_widget(
        Paragraph::new(format!("X")).alignment(Alignment::Center),
        ppu_regs[2],
    );

    f.render_widget(
        Paragraph::new(format!("{}", cpu.ppu.x)).alignment(Alignment::Center),
        ppu_values[2],
    );

    f.render_widget(
        Paragraph::new(format!("Y")).alignment(Alignment::Center),
        ppu_regs[3],
    );

    f.render_widget(
        Paragraph::new(format!("{}", cpu.ppu.y)).alignment(Alignment::Center),
        ppu_values[3],
    );

    f.render_widget(
        Paragraph::new(format!("1D")).alignment(Alignment::Center),
        ppu_regs[4],
    );

    f.render_widget(
        Paragraph::new(format!("{}", cpu.memory.io_load(DISPCNT).bit_is_set(6)))
            .alignment(Alignment::Center),
        ppu_values[4],
    );

    f.render_widget(block, ppu_chunk);

    Ok(())
}
fn draw_cpu(
    f: &mut Frame<'_, CrosstermBackend<Stdout>>,
    cpu_chunk: Rect,
    cpu: &CPU,
) -> Result<(), std::io::Error> {
    let block = Block::default()
        .title("CPU")
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL);

    let sections = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(2),
            Constraint::Length(2),
            Constraint::Length(3),
            Constraint::Length(1),
            Constraint::Length(1),
        ])
        .split(cpu_chunk);

    let pc = Paragraph::new(format!("PC: {:#04x}", cpu.get_pc()))
        .alignment(tui::layout::Alignment::Center)
        .wrap(Wrap { trim: true });

    let sp = Paragraph::new(format!("SP: {:#04x}", cpu.get_sp()))
        .alignment(tui::layout::Alignment::Center)
        .wrap(Wrap { trim: true });

    let instruction = Paragraph::new(format!(
        "fetched inst:\n{:#010x}",
        cpu.prefetch[0].unwrap_or(0)
    ))
    .alignment(tui::layout::Alignment::Center)
    .wrap(Wrap { trim: true });

    let decoded_instruction = Paragraph::new(format!(
        "decoded inst:\n{:#010x}",
        cpu.prefetch[1].unwrap_or(0)
    ))
    .alignment(tui::layout::Alignment::Center)
    .wrap(Wrap { trim: true });

    let executed_instruction_decode =
        cpu.decode_instruction((cpu.executed_instruction_hex & !0xF0000000) | 0b1110 << 28);
    let condition_code = condition_code_as_str((cpu.executed_instruction_hex & 0xF0000000) >> 28);
    let executed_instruction_print = match executed_instruction_decode {
        Instruction::ALUInstruction(data_processing_instruction) => {
            &data_processing_instruction.instruction_to_string(condition_code)
        }
        Instruction::MRS(data_processing_instruction) => &data_processing_instruction
            .instruction_to_string(condition_code),
        Instruction::MSR(data_processing_instruction) => &data_processing_instruction
            .instruction_to_string(condition_code),
        Instruction::ThumbFullAdder(full_adder) => &full_adder.instruction_to_string(),
        Instruction::ThumbMoveShiftedRegister(instruction) => &instruction.instruction_to_string(),
        Instruction::ThumbAluInstruction(instruction) => &instruction.instruction_to_string(),
        Instruction::ThumbArithmeticImmInstruction(instruction) => {
            &instruction.instruction_to_string()
        }
        Instruction::ThumbHiRegisterInstruction(instruction) => {
            &instruction.instruction_to_string()
        }
        Instruction::ThumbBx(instruction) => &instruction.instruction_to_string(),
        Instruction::ThumbAdr(instruction) => &instruction.instruction_to_string(),
        Instruction::ThumbAddToSp(instruction) => &instruction.instruction_to_string(),
        Instruction::SdtInstruction(instruction) => &instruction.instruction_to_string(condition_code),
        Instruction::SignedAndHwDtInstruction(instruction) => &instruction.instruction_to_string(condition_code),
        Instruction::Funcpointer(_) => &cpu.executed_instruction,
    };

    let executed_instruction =
        Paragraph::new(format!("executed inst:\n{}", executed_instruction_print))
            .alignment(tui::layout::Alignment::Center)
            .wrap(Wrap { trim: true });

    let inst_mode = Paragraph::new(format!("{:#010X}", cpu.executed_instruction_hex))
        .alignment(tui::layout::Alignment::Center)
        .wrap(Wrap { trim: true });

    let cycles = Paragraph::new(format!("{}", cpu.cycles))
        .alignment(tui::layout::Alignment::Center)
        .wrap(Wrap { trim: true });

    f.render_widget(block, cpu_chunk);
    f.render_widget(pc, sections[1]);
    f.render_widget(sp, sections[2]);
    f.render_widget(instruction, sections[3]);
    f.render_widget(decoded_instruction, sections[4]);
    f.render_widget(executed_instruction, sections[5]);
    f.render_widget(inst_mode, sections[6]);
    f.render_widget(cycles, sections[7]);

    Ok(())
}

fn handle_control_events(debugger: &mut Debugger, event: KeyEvent) {
    match event.code {
        KeyCode::Char('t') => {
            debugger.terminal_enabled = !debugger.terminal_enabled;
        }
        KeyCode::Char('c') => {
            KILL_SIGNAL.kill();
            debugger.end_debugger = true;
        }
        KeyCode::Char('w') => {
            if debugger.terminal_enabled {
                debugger.terminal_buffer.clear();
            }
        }
        _ => (),
    }
}

fn handle_terminal_events(debugger: &mut Debugger, event: KeyEvent) {
    match event.code {
        KeyCode::Backspace => {
            debugger.terminal_buffer.pop();
        }
        KeyCode::Char(c) => debugger.terminal_buffer.push(c),
        KeyCode::Enter => {
            match parse_command(debugger) {
                Ok(res) => {
                    let mut history_command = String::new();
                    std::mem::swap(&mut debugger.terminal_buffer, &mut history_command);
                    debugger.terminal_history.push(TerminalHistoryEntry {
                        command: history_command,
                        result: res,
                    })
                }
                Err(err) => {
                    let mut history_command = String::new();
                    std::mem::swap(&mut debugger.terminal_buffer, &mut history_command);
                    debugger.terminal_history.push(TerminalHistoryEntry {
                        command: history_command,
                        result: err.to_string(),
                    })
                }
            };
        }
        _ => {}
    }
}

fn handle_normal_mode_events(debugger: &mut Debugger, event: KeyEvent) {
    match event.code {
        KeyCode::Char('n') => debugger.gba.step(),
        KeyCode::Char('M') => debugger.memory_start_address -= 0x100,
        KeyCode::Char('m') => debugger.memory_start_address += 0x100,
        _ => {}
    }
}

fn draw_registers(
    f: &mut Frame<'_, CrosstermBackend<Stdout>>,
    register_chunk: Rect,
    start: usize,
    cpu: &CPU,
) -> Result<(), std::io::Error> {
    let register_sections = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
        .split(register_chunk);

    let block2 = Block::default()
        .title("Registers")
        .title_alignment(tui::layout::Alignment::Center)
        .borders(Borders::ALL);

    let register_names = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1); 15])
        .split(register_sections[0]);

    let register_values = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1); 15])
        .split(register_sections[1]);

    let get_register_suffix = |cpu_mode: CPUMode, register_num: usize| {
        if register_num < 8 || register_num == 15 {
            return "";
        }
        match cpu_mode {
            CPUMode::FIQ => "_fiq",
            CPUMode::USER | CPUMode::SYS => "",
            _ if register_num < 13 => "",
            CPUMode::SVC => "_svc",
            CPUMode::UND => "_und",
            CPUMode::IRQ => "_irq",
            CPUMode::ABT => "_abt",
            CPUMode::INVALID(_) => "",
        }
    };

    for i in (start + 1)..(start + register_names.len()) {
        if i - 1 > 15 {
            break;
        }
        let register_name = Paragraph::new(format!(
            "{}{}",
            i - 1,
            get_register_suffix(cpu.get_cpu_mode(), i - 1)
        ))
        .alignment(tui::layout::Alignment::Center)
        .wrap(Wrap { trim: true });
        f.render_widget(register_name, register_names[i - start]);

        let register_value = Paragraph::new(format!("{:#X}", cpu.get_register(i as u32 - 1)))
            .alignment(tui::layout::Alignment::Center)
            .wrap(Wrap { trim: true });
        f.render_widget(register_value, register_values[i - start]);
    }

    f.render_widget(block2, register_chunk);
    Ok(())
}

fn draw_cpsr(
    f: &mut Frame<'_, CrosstermBackend<Stdout>>,
    flags_chunk: Rect,
    cpu: &CPU,
) -> Result<(), std::io::Error> {
    let block = Block::default()
        .title("CPSR")
        .title_alignment(tui::layout::Alignment::Center)
        .borders(Borders::ALL);

    let flags_sections = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(flags_chunk);

    let flag_names = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1); 10])
        .split(flags_sections[0]);

    let flag_values = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1); 10])
        .split(flags_sections[1]);

    f.render_widget(
        Paragraph::new(format!("N")).alignment(Alignment::Center),
        flag_names[1],
    );
    f.render_widget(
        Paragraph::new(format!("Z")).alignment(Alignment::Center),
        flag_names[2],
    );
    f.render_widget(
        Paragraph::new(format!("C")).alignment(Alignment::Center),
        flag_names[3],
    );
    f.render_widget(
        Paragraph::new(format!("V")).alignment(Alignment::Center),
        flag_names[4],
    );

    f.render_widget(
        Paragraph::new(format!("OP MODE")).alignment(Alignment::Center),
        flag_names[5],
    );

    f.render_widget(
        Paragraph::new(format!("I MODE")).alignment(Alignment::Center),
        flag_names[6],
    );
    f.render_widget(
        Paragraph::new(format!("CPSR")).alignment(Alignment::Center),
        flag_names[7],
    );

    f.render_widget(
        Paragraph::new(format!("{}", cpu.get_flag(FlagsRegister::N))).alignment(Alignment::Center),
        flag_values[1],
    );
    f.render_widget(
        Paragraph::new(format!("{}", cpu.get_flag(FlagsRegister::Z))).alignment(Alignment::Center),
        flag_values[2],
    );
    f.render_widget(
        Paragraph::new(format!("{}", cpu.get_flag(FlagsRegister::C))).alignment(Alignment::Center),
        flag_values[3],
    );
    f.render_widget(
        Paragraph::new(format!("{}", cpu.get_flag(FlagsRegister::V))).alignment(Alignment::Center),
        flag_values[4],
    );
    f.render_widget(
        Paragraph::new(format!(
            "{}",
            match cpu.get_cpu_mode() {
                CPUMode::FIQ => "FIQ",
                CPUMode::USER => "USER",
                CPUMode::IRQ => "IRQ",
                CPUMode::SVC => "SVC",
                CPUMode::ABT => "ABT",
                CPUMode::UND => "UND",
                CPUMode::SYS => "SYS",
                CPUMode::INVALID(_) => "INVLD",
            }
        ))
        .alignment(Alignment::Center),
        flag_values[5],
    );
    f.render_widget(
        Paragraph::new(format!(
            "{}",
            match cpu.get_instruction_mode() {
                InstructionMode::ARM => "ARM",
                InstructionMode::THUMB => "THUMB",
            }
        ))
        .alignment(Alignment::Center),
        flag_values[6],
    );
    f.render_widget(
        Paragraph::new(format!("{:x}", u32::from(cpu.get_cpsr()))).alignment(Alignment::Center),
        flag_values[7],
    );
    f.render_widget(block, flags_chunk);

    Ok(())
}

fn draw_memory(
    f: &mut Frame<'_, CrosstermBackend<Stdout>>,
    memory_chunk: Rect,
    cpu: &GBA,
    debugger: &Debugger,
) -> Result<(), std::io::Error> {
    let start_address = debugger.memory_start_address;
    let block = Block::default()
        .title("Memory")
        .title_alignment(tui::layout::Alignment::Center)
        .borders(Borders::ALL);

    let memory_sections = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(13),
            Constraint::Length(6),
            Constraint::Length(6),
            Constraint::Length(6),
            Constraint::Length(6),
            Constraint::Length(6),
            Constraint::Length(6),
            Constraint::Length(6),
            Constraint::Length(6),
            Constraint::Length(6),
            Constraint::Length(6),
            Constraint::Length(6),
            Constraint::Length(6),
            Constraint::Length(6),
            Constraint::Length(6),
            Constraint::Length(6),
            Constraint::Length(6),
            Constraint::Length(6),
        ])
        .split(memory_chunk);

    let memory_grid: Vec<Vec<Rect>> = memory_sections
        .clone()
        .into_iter()
        .map(|memory_section| -> Vec<Rect> {
            Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Length(1); 18])
                .split(memory_section)
        })
        .collect();

    for i in 2..memory_grid[0].len() {
        f.render_widget(
            Paragraph::new(format!("0x{:0>8x}", start_address + (i as u32 - 2) * 0x10))
                .alignment(Alignment::Center),
            memory_grid[0][i],
        );
    }

    for column in 1..memory_grid.len() {
        f.render_widget(
            Paragraph::new(format!("0x{:0>2x}", column as u32 - 1)).alignment(Alignment::Center),
            memory_grid[column][1],
        );
    }

    for column in 1..memory_grid.len() {
        for row in 2..memory_grid[column].len() {
            let value = cpu.memory.read_raw(
                (start_address + ((row as u32 - 2) * 0x10) + (column as u32 - 1)) as usize,
            );

            let widget = Paragraph::new(format!("0x{:0>2x}", value))
                .style(Style::default().fg(if value > 0 {
                    tui::style::Color::Blue
                } else {
                    tui::style::Color::White
                }))
                .alignment(Alignment::Center);
            f.render_widget(widget, memory_grid[column][row]);
        }
    }

    let border = Block::default().borders(Borders::ALL);

    f.render_widget(border, memory_sections[0]);
    f.render_widget(block, memory_chunk);
    Ok(())
}

fn draw_terminal(
    f: &mut Frame<'_, CrosstermBackend<Stdout>>,
    terminal_chunk: Rect,
    debugger: &Debugger,
) -> Result<(), std::io::Error> {
    let block = Block::default()
        .title("Terminal")
        .title_alignment(tui::layout::Alignment::Center)
        .borders(Borders::ALL);

    let terminal_sections = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(85), Constraint::Length(3)])
        .split(terminal_chunk);

    let border = Block::default().borders(Borders::ALL);
    if debugger.terminal_enabled {
        let input = Paragraph::new(format!(" > {}", debugger.terminal_buffer));
        f.render_widget(input, terminal_sections[1]);
    }

    let mut output = String::new();
    for entry in debugger.terminal_history.iter() {
        output.push_str(&format!("> {}", entry.command));
        output.push_str("\n");
        if !&entry.result.is_empty() {
            output.push_str(&entry.result);
            output.push_str("\n");
        }
    }

    let num_lines = output.split("\n").collect::<Vec<&str>>().len();
    let output: Vec<&str> = output
        .split("\n")
        .skip(if num_lines + 1 > terminal_sections[0].height as usize {
            num_lines + 1 - terminal_sections[0].height as usize
        } else {
            0
        })
        .collect();
    let output = Paragraph::new(output.join("\n")).block(border);

    f.render_widget(output, terminal_sections[0]);
    f.render_widget(block, terminal_chunk);
    Ok(())
}
