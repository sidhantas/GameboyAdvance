use std::fmt::Display;

use crate::{memory::memory::MemoryError, types::REGISTER};

#[derive(PartialEq, Debug, Clone)]
pub(crate) enum BreakType {
    Break(u32),
    WatchRegister(REGISTER, u32),
    WatchAddress(usize, usize),
}

#[derive(Debug)]
pub(crate) enum TriggeredWatchpoints {
    Address(usize),
    Error(MemoryError),
}

#[derive(Clone)]
pub(crate) struct Breakpoint {
    pub(crate) break_type: BreakType,
}

impl Breakpoint {
    pub(crate) fn new(break_type: BreakType) -> Self {
        Self { break_type }
    }
}

impl Display for BreakType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BreakType::Break(breakpoint) => write!(f, "PC == {}", breakpoint),
            BreakType::WatchRegister(register, value) => {
                write!(f, "r{} == {}", register, value)
            }
            BreakType::WatchAddress(address, address1) => write!(f, "address == {}", address),
        }
    }
}
