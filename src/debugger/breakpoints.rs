use std::
    fmt::Display
;

use crate::types::REGISTER;

#[derive(PartialEq, Debug, Clone)]
pub enum BreakType {
    Break(u32),
    WatchRegister(REGISTER, u32),
    WatchAddress(usize),
}

pub struct TriggeredWatchpoint {
    address: usize,
    value: u32
}

#[derive(Clone)]
pub struct Breakpoint {
    pub break_type: BreakType,
}

impl Breakpoint {
    pub fn new(break_type: BreakType) -> Self {
        Self {
            break_type,
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
