use crate::memory::memory::MemoryBus;
use crate::{arm7tdmi::cpu::CPU, memory::memory::GBAMemory};

use crate::graphics::ppu::PPU;

pub struct GBA {
    pub cpu: CPU,
    pub memory: Box<dyn MemoryBus>,
    pub ppu: PPU,
}


impl GBA {
    pub fn new(bios: String, rom: String) -> Self {
        let mut memory = GBAMemory::new();
        memory.initialize_bios(bios).unwrap();
        memory.initialize_rom(rom).unwrap();
        let mut gba = Self {
            memory,
            cpu: CPU::new(),
            ppu: PPU::default()
        };
        gba.cpu.flush_pipeline(&mut gba.memory);
        gba
    }

    pub fn step(&mut self) {
        let cpu_cycles = self.cpu.execute_cpu_cycle(&mut self.memory);
        self.ppu
            .advance_ppu(cpu_cycles, &mut self.memory);
    }
}
