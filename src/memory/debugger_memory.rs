use std::{fmt::Debug, rc::Rc};

use crate::debugger::breakpoints::{BreakType, Breakpoint};

use super::memory::MemoryBus;

struct DebuggerMemory {
    memory: Box<dyn MemoryBus>,
}

impl DebuggerMemory {
    fn new(memory: Box<dyn MemoryBus>, breakpoints: Rc<Vec<Breakpoint>>) -> Box<dyn MemoryBus> {
        Box::new(Self {
            memory,
        })
    }
}

impl MemoryBus for DebuggerMemory {
    fn initialize_bios(&mut self, filename: String) -> Result<(), std::io::Error> {
        Ok(())
    }

    fn read(&self, address: usize) -> super::memory::MemoryFetch<u8> {
        todo!();
    }

    fn readu16(&self, address: usize) -> super::memory::MemoryFetch<u16> {
        todo!()
    }

    fn readu32(&self, address: usize) -> super::memory::MemoryFetch<u32> {
        todo!()
    }

    fn write(&mut self, address: usize, value: u8) -> crate::types::CYCLES {
        todo!()
    }

    fn writeu16(&mut self, address: usize, value: u16) -> crate::types::CYCLES {
        todo!()
    }

    fn writeu32(&mut self, address: usize, value: u32) -> crate::types::CYCLES {
        todo!()
    }
}
