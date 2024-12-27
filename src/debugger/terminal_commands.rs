use std::{fmt::Display, sync::mpsc::SendError};

use crate::utils::utils::{try_parse_num, try_parse_reg, ParsingError};

use super::debugger::{BreakType, DebugCommands, Debugger};


pub enum TerminalCommandErrors {
    CouldNotFindCommand,
    NotEnoughArguments,
    CouldNotParse,
    NoCommandProvided,
    InvalidArgument(String),
    ChannelError(SendError<DebugCommands>),
}

impl Display for TerminalCommandErrors {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TerminalCommandErrors::CouldNotFindCommand => "Invalid Command".fmt(f),
            TerminalCommandErrors::NotEnoughArguments => "Not enough arguments provided".fmt(f),
            TerminalCommandErrors::CouldNotParse => "Unable to parse command".fmt(f),
            TerminalCommandErrors::NoCommandProvided => "No Command Provided".fmt(f),
            TerminalCommandErrors::InvalidArgument(arg) => write!(f, "Invalid argument provided: {}", arg),
            TerminalCommandErrors::ChannelError(err) => {
                write!(f, "Experienced an error with the channel: {}", err)
            }
        }
    }
}

pub struct TerminalCommand {
    pub name: &'static str,
    pub arguments: u8,
    pub description: &'static str,
    pub handler: fn(debugger: &mut Debugger, args: Vec<&str>) -> Result<String, TerminalCommandErrors>,
}

pub struct TerminalHistoryEntry {
    pub command: String,
    pub result: String,
}

pub const TERMINAL_COMMANDS: [TerminalCommand; 8] = [
    TerminalCommand {
        name: "next",
        arguments: 1,
        description: "Goes to the next instruction",
        handler: next_handler,
    },
    TerminalCommand {
        name: "quit",
        arguments: 0,
        description: "Closes the program",
        handler: quit_handler,
    },
    TerminalCommand {
        name: "break",
        arguments: 1,
        description: "Sets a breakpoint at specified address",
        handler: set_breakpoint_handler,
    },
    TerminalCommand {
        name: "delete",
        arguments: 1,
        description: "Deletes a breakpoint",
        handler: delete_breakpoint_handler,
    },
    TerminalCommand {
        name: "listb",
        arguments: 0,
        description: "Lists breakpoints",
        handler: list_breakpoint_handler,
    },
    TerminalCommand {
        name: "watchr",
        arguments: 2,
        description: "Sets a watch point on a register and a value",
        handler: set_watchpoint_handler,
    },
    TerminalCommand {
        name: "watcha",
        arguments: 2,
        description: "Sets a watch point on an address range",
        handler: set_watch_address_range_handler,
    },
    TerminalCommand {
        name: "mem",
        arguments: 1,
        description: "Sets start memory address",
        handler: set_mem_start,
    },
];

fn find_command(command: &str) -> Result<&TerminalCommand, TerminalCommandErrors> {
    for term_command in &TERMINAL_COMMANDS {
        if command == term_command.name {
            return Ok(term_command);
        }
    }
    Err(TerminalCommandErrors::CouldNotFindCommand)
}

pub fn parse_command(debugger: &mut Debugger) -> Result<String, TerminalCommandErrors> {
    let buff = debugger.terminal_buffer.clone();
    let mut split_command = buff.split_whitespace();
    let Some(command_name) = split_command.next() else {
        return Err(TerminalCommandErrors::NoCommandProvided);
    };
    let command = find_command(command_name)?;
    Ok((command.handler)(debugger, split_command.collect())?)

}

fn next_handler(debugger: &mut Debugger, args: Vec<&str>) -> Result<String, TerminalCommandErrors> {
    let num_executions = match args.get(0) {
        Some(value) => {
            let Ok(parsed_value) = value.parse::<u32>() else {
                return Err(TerminalCommandErrors::CouldNotParse.into());
            };
            parsed_value
        }
        None => 1,
    };
    if let Err(err) = debugger
        .cpu_sender
        .send(DebugCommands::Continue(num_executions))
    {
        return Err(TerminalCommandErrors::ChannelError(err));
    }

    Ok(String::new())
}

fn quit_handler(debugger: &mut Debugger, _args: Vec<&str>) -> Result<String, TerminalCommandErrors> {
    if let Err(err) = debugger
        .cpu_sender
        .send(DebugCommands::End)
    {
        return Err(TerminalCommandErrors::ChannelError(err));
    }
    debugger.end_debugger = true;
    Ok(String::new())
}

fn set_breakpoint_handler(debugger: &mut Debugger, args: Vec<&str>) -> Result<String, TerminalCommandErrors> {
    let breakpoint = match args.get(0) {
        Some(value) => {
            let Ok(parsed_value) = try_parse_num(value) else {
                return Err(TerminalCommandErrors::CouldNotParse.into());
            };
            parsed_value
        }
        None => return Err(TerminalCommandErrors::NotEnoughArguments),
    };
    if let Err(err) = debugger
        .cpu_sender
        .send(DebugCommands::SetBreakpoint(BreakType::Break(breakpoint)))
    {
        return Err(TerminalCommandErrors::ChannelError(err));
    }
    Ok(format!("Breakpoint set at address {:#x}", breakpoint))
}

fn delete_breakpoint_handler(debugger: &mut Debugger, args: Vec<&str>) -> Result<String, TerminalCommandErrors> {
    let breakpoint = match args.get(0) {
        Some(value) => {
            let Ok(parsed_value) = value.parse::<u32>() else {
                return Err(TerminalCommandErrors::CouldNotParse.into());
            };
            parsed_value
        }
        None => return Err(TerminalCommandErrors::NotEnoughArguments),
    };

    if breakpoint < 1 {
        return Err(TerminalCommandErrors::InvalidArgument(breakpoint.to_string()));
    }

    if let Err(err) = debugger
        .cpu_sender
        .send(DebugCommands::DeleteBreakpoint(breakpoint - 1))
    {
        return Err(TerminalCommandErrors::ChannelError(err));
    }
    Ok(format!("Breakpoint number {} removed", breakpoint))
}

fn list_breakpoint_handler(debugger: &mut Debugger, _args: Vec<&str>) -> Result<String, TerminalCommandErrors> {
    let mut breakpoint_list = String::new();
    let breakpoints = &debugger.cpu.lock().unwrap().breakpoints;
    if breakpoints.is_empty() {
        return Ok("No Breakpoints".into());
    }
    for (i, breakpoint) in breakpoints.into_iter().enumerate() {
        match breakpoint {
            BreakType::Break(bp) => breakpoint_list.push_str(format!("{}: break {:#x}\n", i + 1, bp).as_str()),
            BreakType::WatchRegister(reg, value) => breakpoint_list.push_str(format!("{}: watch r{reg} {:#x}\n", i + 1, value).as_str()),
            BreakType::WatchAddressRange(address1, address2) => breakpoint_list.push_str(format!("{}: watch {:#X} {:#X}\n", i + 1, address1, address2).as_str()),
        }
    }

    Ok(breakpoint_list)
}

impl From<ParsingError> for TerminalCommandErrors {
    fn from(_value: ParsingError) -> Self {
        Self::CouldNotParse
    }
}

fn set_watchpoint_handler(debugger: &mut Debugger, args: Vec<&str>) -> Result<String, TerminalCommandErrors> {
    if args.len() < 2 {
        return Err(TerminalCommandErrors::NotEnoughArguments);
    }
    let register = try_parse_reg(args[0])?;
    let value = try_parse_num(args[1])?;
    
    if let Err(err) = debugger
        .cpu_sender
        .send(DebugCommands::SetBreakpoint(BreakType::WatchRegister(register, value)))
    {
        return Err(TerminalCommandErrors::ChannelError(err));
    }
    Ok(format!("Watchpoint set for register r{register} with value {:#x}", value))
}

fn set_watch_address_range_handler(debugger: &mut Debugger, args: Vec<&str>) -> Result<String, TerminalCommandErrors> {
    if args.len() < 1 {
        return Err(TerminalCommandErrors::NotEnoughArguments);
    }
    let address1 = try_parse_num(args[0])?;
    let address2 = if args.len() >= 2 {
        try_parse_num(args[1]).unwrap_or(address1)
    } else {
        address1
    };
    
    if let Err(err) = debugger
        .cpu_sender
        .send(DebugCommands::SetBreakpoint(BreakType::WatchAddressRange(address1, address2)))
    {
        return Err(TerminalCommandErrors::ChannelError(err));
    }
    Ok(format!("Watchpoint set for range {:#x}-{:#x}", address1, address2))
}

fn set_mem_start(debugger: &mut Debugger, args: Vec<&str>) -> Result<String, TerminalCommandErrors> {
    if args.len() < 1 {
        return Err(TerminalCommandErrors::NotEnoughArguments);
    }
    let mem_start = try_parse_num(args[0])?;

    debugger.memory_start_address = mem_start;
    
    Ok(String::new())
}
