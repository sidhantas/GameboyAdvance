use crossterm::{
    event::{self, read, Event, KeyCode, KeyModifiers},
    terminal::{disable_raw_mode, enable_raw_mode},
};
use std::{
    io::{self, Stdout},
    thread,
    time::Duration,
};
use tui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    widgets::{Block, Borders, Paragraph},
    Frame, Terminal,
};

use crate::arm7tdmi::cpu::CPU;

pub fn start_debugger(cpu: CPU) -> Result<(), std::io::Error> {
    let backend = CrosstermBackend::new(io::stdout());
    let mut terminal = Terminal::new(backend)?;
    enable_raw_mode()?;
    terminal.clear()?;
    let mut end_debugger = false;

    while !end_debugger {
        if event::poll(Duration::from_millis(100))? {
            match read()? {
                Event::FocusGained => println!("FocusGained"),
                Event::FocusLost => println!("FocusLost"),
                Event::Key(event) => {
                    if event.modifiers == KeyModifiers::CONTROL && event.code == KeyCode::Char('c')
                    {
                        end_debugger = true;
                    }
                }
                Event::Mouse(event) => println!("{:?}", event),
                Event::Resize(width, height) => println!("New size {}x{}", width, height),
                _ => {}
            }
        }
        terminal.draw(|f| {
            let vertical_chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(1)
                .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
                .split(f.size());

            let horizontal_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .margin(1)
                .constraints([Constraint::Percentage(10), Constraint::Percentage(50)].as_ref())
                .split(vertical_chunks[0]);

            draw_cpu(f, &horizontal_chunks, &cpu).unwrap();
        })?;
        thread::sleep(Duration::from_millis(100));
    }

    terminal.clear()?;
    disable_raw_mode()?;

    Ok(())
}

fn draw_cpu(
    f: &mut Frame<'_, CrosstermBackend<Stdout>>,
    chunks: &Vec<Rect>,
    cpu: &CPU,
) -> Result<(), std::io::Error> {
    let block = Block::default()
        .title("CPU")
        .title_alignment(tui::layout::Alignment::Center)
        .borders(Borders::ALL);

    let sections = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
        ])
        .split(chunks[0]);

    let pc = Paragraph::new(format!("PC: {:#04x}", cpu.get_pc()))
        .alignment(tui::layout::Alignment::Center);

    let sp = Paragraph::new(format!("SP: {:#04x}", cpu.get_pc()))
        .alignment(tui::layout::Alignment::Center);

    f.render_widget(block, chunks[0]);
    f.render_widget(pc, sections[1]);
    f.render_widget(sp, sections[2]);
    Ok(())
}