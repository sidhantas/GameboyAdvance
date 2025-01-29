use crate::{memory::{io_handlers::{DISPSTAT, IF, IO_BASE, VCOUNT}, memory::MemoryBus}, utils::bits::Bits};

const HDRAW: u64 = 240;
const HBLANK: u64 = 68;
const VDRAW: u64 = 160;
const VBLANK: u64 = 68;

const VBLANK_FLAG: u16 = 1 << 0;
const HBLANK_FLAG: u16 = 1 << 1;
const VCOUNTER_FLAG: u16 = 1 << 2;
const VBLANK_ENABLE: u16 = 1 << 3;
const HBLANK_ENABLE: u16 = 1 << 4;

#[derive(Default)]
pub struct PPU {
    pub x: u64,
    pub y: u64,
}

impl PPU {
    pub fn advance_ppu(&mut self, cycles: &mut u64, memory: &mut Box<dyn MemoryBus>) {
        let dots = *cycles / 4;
        *cycles %= 4;
        self.x += dots;
        let mut disp_stat = memory.readu16(IO_BASE + DISPSTAT).data;
        let mut interrupt_flags_register = memory.readu16(IO_BASE + IF).data;
        if self.x >= (HDRAW + HBLANK) {
            self.y += 1;
            self.x %= HDRAW + HBLANK;

            if self.y >= VDRAW && (disp_stat & VBLANK_ENABLE) > 0 {
                disp_stat |= VBLANK_FLAG;
                interrupt_flags_register |= VBLANK_FLAG;
            }

            if self.y >= (VDRAW + VBLANK) {
                self.y %= VDRAW + VBLANK;
            }
            memory.ppu_io_write(VCOUNT, self.y as u16);
        }
        memory.ppu_io_write(DISPSTAT, disp_stat);
        memory.ppu_io_write(IF, interrupt_flags_register);
    }
}

#[cfg(test)]
mod tests {
    use crate::{arm7tdmi::cpu::CPU, graphics::ppu::{HBLANK, HDRAW, VDRAW}, memory::{io_handlers::{DISPSTAT, IO_BASE}, memory::GBAMemory}};

    use super::VBLANK_ENABLE;

    #[test]
    fn ppu_sets_vblank_flag_when_in_vblank() {
        let memory = GBAMemory::new();
        let mut cpu = CPU::new(memory);
        cpu.memory.writeu16(IO_BASE + DISPSTAT, VBLANK_ENABLE); // Enable VBLANK
        assert_eq!(cpu.memory.readu16(IO_BASE + DISPSTAT).data, 0x8);

        for _ in 0..(VDRAW * (HDRAW + HBLANK) * 4) {
            cpu.execute_cpu_cycle();
        }

        assert_eq!(cpu.memory.readu16(IO_BASE + DISPSTAT).data, 0x9);

    }
}
