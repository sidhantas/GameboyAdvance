use std::sync::Arc;

use crate::graphics::display::{DisplayBuffer, CANVAS_AREA};
use crate::graphics::ppu::PPU;
use crate::memory::memory::CPUCallbacks;
use crate::{arm7tdmi::cpu::CPU, memory::memory::GBAMemory};

pub struct GBA {
    pub cpu: CPU,
    pub memory: GBAMemory,
    pub ppu: PPU,
    display_buffer: Arc<DisplayBuffer>,
}

impl GBA {
    #[cfg(test)]
    pub fn new_no_bios() -> Self {
        Self {
            memory: GBAMemory::new(),
            cpu: CPU::new(),
            ppu: PPU::default(),
            display_buffer: Arc::new(DisplayBuffer::new()),
        }
    }

    pub fn new(bios: String, rom: String, display_buffer: Arc<DisplayBuffer>) -> Self {
        let mut memory = GBAMemory::new();
        memory.initialize_bios(bios).unwrap();
        memory.initialize_rom(rom).unwrap();
        let mut gba = Self {
            memory,
            cpu: CPU::new(),
            ppu: PPU::default(),
            display_buffer,
        };
        gba.cpu.flush_pipeline(&mut gba.memory);
        gba
    }

    pub fn step(&mut self) {
        let cpu_cycles = self.cpu.execute_cpu_cycle(&mut self.memory);
        self.ppu.advance_ppu(
            cpu_cycles,
            &mut self.memory,
            &self.display_buffer,
        );
        if let Some(mut timers) = self.memory.timers.take() {
            timers.tick(cpu_cycles.into(), &mut self.memory);
            self.memory.timers.replace(timers);
        }
        for command in self.memory.cpu_commands.drain(..) {
            match command {
                CPUCallbacks::Halt => self.cpu.halt(),
                CPUCallbacks::RaiseIrq => {
                    self.cpu.interrupt_triggered = true;
                }
                _ => panic!("{:#?}", command),
            }
        }
    }
}
