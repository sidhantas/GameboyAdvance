use crate::debugger::{breakpoints::{BreakType, Breakpoint}, debugger::Debugger};

use super::memory::MemoryBus;

pub struct DebuggerMemory {
    pub breakpoint_checker: Box<dyn Fn(usize) -> ()>,
    pub memory: Box<dyn MemoryBus>,
}

impl DebuggerMemory {
    pub fn new(memory: Box<dyn MemoryBus>, trigger_breakpoint: Box<dyn Fn(usize) -> ()>) -> Box<dyn MemoryBus> {
        Box::new(Self {
            memory,
            breakpoint_checker: trigger_breakpoint
        })
    }
}

impl MemoryBus for DebuggerMemory {
    fn read(&self, address: usize) -> super::memory::MemoryFetch<u8> {
        (self.breakpoint_checker)(address);
        self.memory.read(address)
    }

    fn readu16(&self, address: usize) -> super::memory::MemoryFetch<u16> {
        (self.breakpoint_checker)(address);
        self.memory.readu16(address)
    }

    fn readu32(&self, address: usize) -> super::memory::MemoryFetch<u32> {
        (self.breakpoint_checker)(address);
        self.memory.readu32(address)
    }

    fn write(&mut self, address: usize, value: u8) -> crate::types::CYCLES {
        (self.breakpoint_checker)(address);
        self.memory.write(address, value)
    }

    fn writeu16(&mut self, address: usize, value: u16) -> crate::types::CYCLES {
        (self.breakpoint_checker)(address);
        self.memory.writeu16(address, value)
    }

    fn writeu32(&mut self, address: usize, value: u32) -> crate::types::CYCLES {
        (self.breakpoint_checker)(address);
        self.memory.writeu32(address, value)
    }
}
