use crate::memory::memory::CPUCallbacks;
use crate::{arm7tdmi::cpu::CPU, memory::memory::GBAMemory};

use crate::graphics::ppu::PPU;

pub struct GBA {
    pub cpu: CPU,
    pub memory: GBAMemory,
    pub ppu: PPU,
}

impl GBA {
    #[cfg(test)]
    pub fn new_no_bios() -> Self {
        Self {
            memory: GBAMemory::new(),
            cpu: CPU::new(),
            ppu: PPU::default()
        }
    }
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

        for command in self.memory.cpu_commands.drain(..) {
            match command {
                CPUCallbacks::Halt => self.cpu.halt(),
                CPUCallbacks::RaiseIrq => {
                    self.cpu.interrupt_triggered = true;
                },
                _ => panic!("{:#?}", command)
            }
        }
    }

}
