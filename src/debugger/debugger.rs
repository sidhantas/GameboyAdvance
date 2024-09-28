use crossterm::{
    event::{self, read, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::{
    io::{self, Stdout},
    sync::{mpsc::Sender, Arc, Mutex},
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

use crate::{
    arm7tdmi::{
        cpu::{FlagsRegister, InstructionMode, CPU},
        instructions::ARMDecodedInstruction,
    },
    memory::{self, AccessFlags},
};

pub enum DebugCommands {
    Continue,
    End,
}

pub fn start_debugger(
    cpu: Arc<Mutex<CPU>>,
    cpu_sender: Sender<DebugCommands>,
) -> Result<(), std::io::Error> {
    enable_raw_mode()?;
    execute!(io::stdout(), EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(io::stdout());
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;
    let mut end_debugger = false;

    let mut memory_start_address: u32 = 0;

    while !end_debugger {
        if event::poll(Duration::from_millis(100))? {
            match read()? {
                Event::Key(event) => {
                    if event.modifiers == KeyModifiers::CONTROL && event.code == KeyCode::Char('c')
                    {
                        end_debugger = true;
                        cpu_sender.send(DebugCommands::End).unwrap();
                    }
                    else if event.code == KeyCode::Char('n') {
                        cpu_sender.send(DebugCommands::Continue).unwrap();
                    }
                    else if event.code == KeyCode::Char('b') {
                        memory_start_address -= 0x100;
                    }
                    else if event.code == KeyCode::Char('m') {
                        memory_start_address += 0x100;
                    }
                }
                _ => {}
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
                        Constraint::Length(50),
                    ]
                    .as_ref(),
                )
                .split(vertical_chunks[0]);

            let horizontal_chunks_1 = Layout::default()
                .direction(Direction::Horizontal)
                .margin(1)
                .constraints([Constraint::Length(110), Constraint::Length(50)].as_ref())
                .split(vertical_chunks[1]);

            let cpu_chunk = horizontal_chunks[0];
            let register_chunk = horizontal_chunks[1];
            let flags_chunk = horizontal_chunks[2];
            let memory_chunk = horizontal_chunks_1[0];

            draw_cpu(f, cpu_chunk, &cpu.lock().unwrap()).unwrap();
            draw_registers(f, register_chunk, &cpu.lock().unwrap()).unwrap();
            draw_cpsr(f, flags_chunk, &cpu.lock().unwrap()).unwrap();
            draw_memory(f, memory_chunk, &cpu.lock().unwrap(), memory_start_address).unwrap();
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
            Constraint::Length(2),
            Constraint::Length(2),
        ])
        .split(cpu_chunk);

    let pc = Paragraph::new(format!("PC: {:#04x}", cpu.get_pc()))
        .alignment(tui::layout::Alignment::Center)
        .wrap(Wrap { trim: true });

    let sp = Paragraph::new(format!("SP: {:#04x}", cpu.get_sp()))
        .alignment(tui::layout::Alignment::Center)
        .wrap(Wrap { trim: true });

    let instruction = Paragraph::new(format!("fetched inst:\n{:#010x}", cpu.fetched_instruction))
        .alignment(tui::layout::Alignment::Center)
        .wrap(Wrap { trim: true });

    let decoded_instruction = Paragraph::new(format!(
        "decoded inst:\n{:#010x}",
        cpu.decoded_instruction
            .unwrap_or(ARMDecodedInstruction {
                ..Default::default()
            })
            .instruction
    ))
    .alignment(tui::layout::Alignment::Center)
    .wrap(Wrap { trim: true });

    let executed_instruction =
        Paragraph::new(format!("executed inst:\n{}", cpu.executed_instruction))
            .alignment(tui::layout::Alignment::Center)
            .wrap(Wrap { trim: true });

    let inst_mode_text = match cpu.inst_mode {
        InstructionMode::ARM => "ARM",
        InstructionMode::THUMB => "THUMB",

    };
    let inst_mode = 
        Paragraph::new(format!("Instruction Mode:\n{}", inst_mode_text))
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
    cpu: &CPU,
) -> Result<(), std::io::Error> {
    let register_sections = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(20), Constraint::Percentage(80)])
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

    for i in 1..register_names.len() {
        let register_name = Paragraph::new(format!("{}", i - 1))
            .alignment(tui::layout::Alignment::Center)
            .wrap(Wrap { trim: true });
        f.render_widget(register_name, register_names[i]);
    }

    for i in 1..register_names.len() {
        let register_value = Paragraph::new(format!("{}", cpu.get_register(i as u32 - 1)))
            .alignment(tui::layout::Alignment::Center)
            .wrap(Wrap { trim: true });
        f.render_widget(register_value, register_values[i]);
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
        .constraints([Constraint::Percentage(20), Constraint::Percentage(80)])
        .split(flags_chunk);

    let flag_names = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1); 5])
        .split(flags_sections[0]);

    let flag_values = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1); 5])
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
        Paragraph::new(format!("{}", cpu.get_flag(FlagsRegister::N))).alignment(Alignment::Center),
        flag_values[1],
    );
    f.render_widget(
        Paragraph::new(format!("{}", cpu.get_flag(FlagsRegister::C))).alignment(Alignment::Center),
        flag_values[2],
    );
    f.render_widget(
        Paragraph::new(format!("{}", cpu.get_flag(FlagsRegister::Z))).alignment(Alignment::Center),
        flag_values[3],
    );
    f.render_widget(
        Paragraph::new(format!("{}", cpu.get_flag(FlagsRegister::V))).alignment(Alignment::Center),
        flag_values[4],
    );
    f.render_widget(block, flags_chunk);

    Ok(())
}

fn draw_memory(
    f: &mut Frame<'_, CrosstermBackend<Stdout>>,
    memory_chunk: Rect,
    cpu: &CPU,
    start_address: u32,
) -> Result<(), std::io::Error> {
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

    let memory = cpu.memory.lock().unwrap();
    for column in 1..memory_grid.len() {
        for row in 2..memory_grid[column].len() {
            let value = memory
                .read(
                    (start_address + ((row as u32 - 2) * 0x10) + (column as u32 - 1)) as usize,
                    AccessFlags::Privileged,
                )
                .unwrap_or(0x00);

            let widget = Paragraph::new(format!("0x{:0>2x}", value))
                .style(Style::default().fg(if value > 0 { tui::style::Color::Blue } else {tui::style::Color::White}))
                .alignment(Alignment::Center);
            f.render_widget(widget, memory_grid[column][row]);
        }
    }

    let border = Block::default().borders(Borders::ALL);

    f.render_widget(border, memory_sections[0]);
    f.render_widget(block, memory_chunk);
    Ok(())
}
