use super::breakpoints::{BreakType, Breakpoint};
use crossterm::{
    event::{self, read, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::{
    io::{self, Stdout},
    sync::{
        mpsc::{Receiver, Sender},
        Arc, Mutex,
    },
    thread,
    time::Duration,
};
use tui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::Style,
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame, Terminal,
};

use crate::{arm7tdmi::cpu::{CPUMode, FlagsRegister, InstructionMode, CPU}, memory::{debugger_memory::DebuggerMemory, memory::{GBAMemory, MemoryBus}}};

use super::terminal_commands::{parse_command, TerminalHistoryEntry};

pub enum DebugCommands {
    Continue(u32),
    SetBreakpoint(BreakType),
    DeleteBreakpoint(u32),
    End,
}

pub struct Debugger {
    pub memory_start_address: u32,
    pub cpu_sender: Sender<DebugCommands>,
    pub terminal_buffer: String,
    pub terminal_history: Vec<TerminalHistoryEntry>,
    pub end_debugger: bool,
    pub cpu: Arc<Mutex<CPU>>,
    pub breakpoints: Vec<Breakpoint>,
}

impl Debugger {
    fn new(cpu: Arc<Mutex<CPU>>, cpu_sender: Sender<DebugCommands>) -> Self {
        let mut gba_memory = GBAMemory::new();
        let debugger_memory = DebuggerMemory::new(&mut gba_memory);

        Self {
            memory_start_address: 0x0000000,
            cpu_sender,
            terminal_buffer: String::new(),
            terminal_history: Vec::new(),
            end_debugger: false,
            cpu,
            breakpoints: Vec::new(),
        }
    }
}

pub fn start_debugger(
    cpu: Arc<Mutex<CPU>>,
    cpu_sender: Sender<DebugCommands>,
    debug_receiver: Receiver<DebugCommands>,
) -> Result<(), std::io::Error> {

    enable_raw_mode()?;
    execute!(io::stdout(), EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(io::stdout());
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;
    let mut terminal_enabled = false;

    let mut debugger = Debugger::new(cpu.clone(), cpu_sender.clone());
    while !debugger.end_debugger {
        if let Ok(DebugCommands::End) = debug_receiver.try_recv() {
            debugger.end_debugger = true;
        }
        loop {
            if event::poll(Duration::from_millis(10))? {
                if let Event::Key(event) = read()? {
                    if event.modifiers == KeyModifiers::CONTROL {
                        match event.code {
                            KeyCode::Char('t') => {
                                terminal_enabled = !terminal_enabled;
                            }
                            KeyCode::Char('c') => {
                                cpu_sender.send(DebugCommands::End).unwrap();
                                debugger.end_debugger = true;
                            }
                            KeyCode::Char('w') => {
                                if terminal_enabled {
                                    debugger.terminal_buffer.clear();
                                }
                            }
                            _ => (),
                        }
                        continue;
                    }
                    if terminal_enabled {
                        match event.code {
                            KeyCode::Backspace => {
                                debugger.terminal_buffer.pop();
                            }
                            KeyCode::Char(c) => debugger.terminal_buffer.push(c),
                            KeyCode::Enter => {
                                match parse_command(&mut debugger) {
                                    Ok(res) => {
                                        debugger.terminal_history.push(TerminalHistoryEntry {
                                            command: debugger.terminal_buffer.clone(),
                                            result: res,
                                        })
                                    }
                                    Err(err) => {
                                        debugger.terminal_history.push(TerminalHistoryEntry {
                                            command: debugger.terminal_buffer.clone(),
                                            result: err.to_string(),
                                        })
                                    }
                                };
                                debugger.terminal_buffer.clear();
                            }
                            _ => {}
                        }
                    } else {
                        match event.code {
                            KeyCode::Char('M') => debugger.memory_start_address -= 0x100,
                            KeyCode::Char('m') => debugger.memory_start_address += 0x100,
                            _ => {}
                        }
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
            let memory_chunk = horizontal_chunks_1[0];
            let terminal_chunk = horizontal_chunks_1[1];

            {
                let cpu = &cpu.lock().unwrap();
                draw_cpu(f, cpu_chunk, cpu).unwrap();
                draw_registers(f, register_chunk, 0, cpu).unwrap();
                draw_registers(f, register_chunk_2, 10, cpu).unwrap();
                draw_cpsr(f, flags_chunk, cpu).unwrap();
                draw_memory(f, memory_chunk, cpu, &debugger).unwrap();
                draw_terminal(f, terminal_chunk, terminal_enabled, &debugger).unwrap();
            }
        }) else {
            break;
        };
        thread::sleep(Duration::from_millis(100));
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
            Constraint::Length(2),
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

    let executed_instruction =
        Paragraph::new(format!("executed inst:\n{}", cpu.executed_instruction))
            .alignment(tui::layout::Alignment::Center)
            .wrap(Wrap { trim: true });

    let inst_mode = Paragraph::new(format!("{:#x}", cpu.executed_instruction_hex))
        .alignment(tui::layout::Alignment::Center)
        .wrap(Wrap { trim: true });

    f.render_widget(block, cpu_chunk);
    f.render_widget(pc, sections[1]);
    f.render_widget(sp, sections[2]);
    f.render_widget(instruction, sections[3]);
    f.render_widget(decoded_instruction, sections[4]);
    f.render_widget(executed_instruction, sections[5]);
    f.render_widget(inst_mode, sections[6]);

    Ok(())
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

        let register_value = Paragraph::new(format!("{:#x}", cpu.get_register(i as u32 - 1)))
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
        .constraints([Constraint::Length(1); 8])
        .split(flags_sections[0]);

    let flag_values = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1); 8])
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
        Paragraph::new(format!("{:08x}", cpu.cpsr)).alignment(Alignment::Center),
        flag_values[7],
    );
    f.render_widget(block, flags_chunk);

    Ok(())
}

fn draw_memory(
    f: &mut Frame<'_, CrosstermBackend<Stdout>>,
    memory_chunk: Rect,
    cpu: &CPU,
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
            let value = cpu
                .memory
                .read((start_address + ((row as u32 - 2) * 0x10) + (column as u32 - 1)) as usize)
                .data;

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
    terminal_enabled: bool,
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
    if terminal_enabled {
        let input = Paragraph::new(format!(" > {}", debugger.terminal_buffer));
        f.render_widget(input, terminal_sections[1]);
    }

    let mut output = String::new();
    for entry in &debugger.terminal_history {
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
