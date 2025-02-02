use crate::arm7tdmi::interrupts::Exceptions;
use crate::memory::io_handlers::{IE, IF, IME, IO_BASE};
use crate::memory::memory::CPUCallbacks;
use crate::utils::bits::Bits;
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
        self.check_interrupts();
        self.ppu
            .advance_ppu(cpu_cycles, &mut self.memory);

        for command in self.memory.cpu_commands.drain(..) {
            match command {
                CPUCallbacks::HALT => self.cpu.halt(),
                _ => panic!("{:#?}", command)
            }
        }
    }

    fn check_interrupts(&mut self) {
        let ime = self.memory.readu16(IO_BASE + IME).data;
        let interrupt_flags_register = self.memory.readu16(IO_BASE + IF).data;
        let interrupt_enable_register = self.memory.readu16(IO_BASE + IE).data;

        if (interrupt_flags_register & interrupt_enable_register) > 0
            && ime > 0
            && !self.cpu.cpsr.bit_is_set(7)
        {
            self.cpu.raise_exception(Exceptions::IRQ, &mut self.memory);
        }

    }
}
