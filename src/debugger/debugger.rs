use crossterm::{
    event::{self, read, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnableLineWrap, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::{
    io::{self, Stdout},
    sync::{mpsc::Sender, Arc, Mutex},
    thread,
    time::Duration,
};
use tui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame, Terminal,
};

use crate::arm7tdmi::cpu::CPU;

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

    while !end_debugger {
        if event::poll(Duration::from_millis(100))? {
            match read()? {
                Event::Key(event) => {
                    if event.modifiers == KeyModifiers::CONTROL && event.code == KeyCode::Char('c')
                    {
                        end_debugger = true;
                        cpu_sender.send(DebugCommands::End).unwrap();
                    }

                    if event.code == KeyCode::Char('n') {
                        cpu_sender.send(DebugCommands::Continue).unwrap();
                    }
                }
                _ => {}
            }
        }

        let Ok(_) = terminal.draw(|f| {
            let vertical_chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(1)
                .constraints([Constraint::Length(15), Constraint::Percentage(50)].as_ref())
                .split(f.size());

            let horizontal_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .margin(1)
                .constraints([Constraint::Length(20), Constraint::Percentage(50)].as_ref())
                .split(vertical_chunks[0]);

            draw_cpu(f, &horizontal_chunks, &cpu.lock().unwrap()).unwrap();
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
            Constraint::Length(2),
            Constraint::Length(2),
            Constraint::Length(2),
        ])
        .split(chunks[0]);

    let pc = Paragraph::new(format!("PC: {:#04x}", cpu.get_pc()))
        .alignment(tui::layout::Alignment::Center).wrap(Wrap {trim: true});

    let sp = Paragraph::new(format!("SP: {:#04x}", cpu.get_sp()))
        .alignment(tui::layout::Alignment::Center).wrap(Wrap {trim: true});

    let instruction = Paragraph::new(format!("fetched inst:\n{:#08x}", cpu.fetched_instruction))
        .alignment(tui::layout::Alignment::Center).wrap(Wrap {trim: true});
    let decoded_instruction = Paragraph::new(format!(
        "decoded inst:\n{:#08x}",
        cpu.decoded_instruction.instruction
    ))
    .alignment(tui::layout::Alignment::Center).wrap(Wrap {trim: true});

    let executed_instruction =
        Paragraph::new(format!("executed inst:\n{}", cpu.executed_instruction))
            .alignment(tui::layout::Alignment::Center).wrap(Wrap {trim: true});

    f.render_widget(block, chunks[0]);
    f.render_widget(pc, sections[1]);
    f.render_widget(sp, sections[2]);
    f.render_widget(instruction, sections[3]);
    f.render_widget(decoded_instruction, sections[4]);
    f.render_widget(executed_instruction, sections[5]);
    Ok(())
}
