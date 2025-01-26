use super::memory::{DebuggerMemoryBus, MemoryBus, MemoryBusNoPanic, MemoryError, MemoryFetch};

pub struct DebuggerMemory {
    catch_memory_error: Box<dyn Fn(MemoryError) -> ()>,
    breakpoint_checker: Box<dyn Fn(usize) -> ()>,
    pub memory: Box<dyn DebuggerMemoryBus>,
}


impl DebuggerMemory {
    pub fn new(
        memory: Box<dyn DebuggerMemoryBus>,
        breakpoint_checker: Box<dyn Fn(usize) -> ()>,
        catch_memory_error: Box<dyn Fn(MemoryError) -> ()>,
    ) -> Box<DebuggerMemory> {
        Box::new(Self {
            memory,
            breakpoint_checker,
            catch_memory_error
        })
    }
}

impl MemoryBusNoPanic for DebuggerMemory {
    fn try_read(
        &self,
        address: usize,
    ) -> Result<super::memory::MemoryFetch<u8>, super::memory::MemoryError> {
        (self.breakpoint_checker)(address);
        self.memory.try_read(address)
    }

    fn try_readu16(
        &self,
        address: usize,
    ) -> Result<super::memory::MemoryFetch<u16>, super::memory::MemoryError> {
        (self.breakpoint_checker)(address);
        self.memory.try_readu16(address)
    }

    fn try_readu32(
        &self,
        address: usize,
    ) -> Result<super::memory::MemoryFetch<u32>, super::memory::MemoryError> {
        (self.breakpoint_checker)(address);
        self.memory.try_readu32(address)
    }

    fn try_write(
        &mut self,
        address: usize,
        value: u8,
    ) -> Result<crate::types::CYCLES, super::memory::MemoryError> {
        (self.breakpoint_checker)(address);
        self.memory.try_write(address, value)
    }

    fn try_writeu16(
        &mut self,
        address: usize,
        value: u16,
    ) -> Result<crate::types::CYCLES, super::memory::MemoryError> {
        (self.breakpoint_checker)(address);
        self.memory.try_writeu16(address, value)
    }

    fn try_writeu32(
        &mut self,
        address: usize,
        value: u32,
    ) -> Result<crate::types::CYCLES, super::memory::MemoryError> {
        (self.breakpoint_checker)(address);
        self.memory.try_writeu32(address, value)
    }
}

impl MemoryBus for DebuggerMemory {
    fn read(&self, address: usize) -> super::memory::MemoryFetch<u8> {
        (self.breakpoint_checker)(address);
        self.memory.try_read(address).unwrap_or_else(|err| {
            (self.catch_memory_error)(err);
            MemoryFetch {
                data: 0,
                cycles: 0
            }
        })
    }

    fn readu16(&self, address: usize) -> super::memory::MemoryFetch<u16> {
        (self.breakpoint_checker)(address);
        self.memory.try_readu16(address).unwrap_or_else(|err| {
            (self.catch_memory_error)(err);
            MemoryFetch {
                data: 0,
                cycles: 0
            }
        })
    }

    fn readu32(&self, address: usize) -> super::memory::MemoryFetch<u32> {
        (self.breakpoint_checker)(address);
        self.memory.try_readu32(address).unwrap_or_else(|err| {
            (self.catch_memory_error)(err);
            MemoryFetch {
                data: 0,
                cycles: 0
            }
        })

    }

    fn write(&mut self, address: usize, value: u8) -> crate::types::CYCLES {
        (self.breakpoint_checker)(address);
        self.memory.try_write(address, value).unwrap_or_else(|err| {
            (self.catch_memory_error)(err);
            0
        })

    }

    fn writeu16(&mut self, address: usize, value: u16) -> crate::types::CYCLES {
        (self.breakpoint_checker)(address);
        self.memory.try_writeu16(address, value).unwrap_or_else(|err| {
            (self.catch_memory_error)(err);
            0
        })

    }

    fn writeu32(&mut self, address: usize, value: u32) -> crate::types::CYCLES {
        (self.breakpoint_checker)(address);
        self.memory.try_writeu32(address, value).unwrap_or_else(|err| {
            (self.catch_memory_error)(err);
            0
        })

    }
    
    fn ppu_io_write(&mut self, address: usize, value: u16) {
        self.memory.ppu_io_write(address, value)
    }
}
