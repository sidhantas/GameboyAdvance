use std::{
    fmt::Display,
    sync::{Arc, Mutex},
};

use crate::{arm7tdmi::cpu::CPU, types::REGISTER};

#[derive(PartialEq)]
pub enum BreakType {
    Break(u32),
    WatchRegister(REGISTER, u32),
    WatchAddress(usize),
}

pub struct Breakpoint {
    pub break_type: BreakType,
    pub prev_value: u32,
}

impl Breakpoint {
    pub fn new(break_type: BreakType, cpu: Arc<Mutex<CPU>>) -> Self {
        let cpu = cpu.lock().unwrap();
        let current_value = match break_type {
            BreakType::WatchAddress(address) => {
                cpu.memory.readu32(address).data
            }
            _ => 0
        };

        Self {
            break_type,
            prev_value: current_value,
        }
    }
}

impl Display for BreakType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BreakType::Break(breakpoint) => write!(f, "PC == {}", breakpoint),
            BreakType::WatchRegister(register, value) => {
                write!(f, "r{} == {}", register, value)
            }
            BreakType::WatchAddress(address) => write!(f, "address == {}", address),
        }
    }
}
