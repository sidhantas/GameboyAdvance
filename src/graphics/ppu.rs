use crate::memory::{
    io_handlers::{DISPSTAT, IO_BASE, VCOUNT},
    memory::GBAMemory,
};

const HDRAW: u64 = 240;
const HBLANK: u64 = 68;
const VDRAW: u64 = 160;
const VBLANK: u64 = 68;

const VBLANK_FLAG: u16 = 1 << 0;
const HBLANK_FLAG: u16 = 1 << 1;
const VCOUNTER_FLAG: u16 = 1 << 2;
const VBLANK_ENABLE: u16 = 1 << 3;
const HBLANK_ENABLE: u16 = 1 << 4;

#[derive(Default, Debug)]
pub struct PPU {
    usable_cycles: u64,
    pub x: u64,
    pub y: u64,
}

impl PPU {
    pub fn advance_ppu(&mut self, cycles: u8, memory: &mut GBAMemory) {
        self.usable_cycles += cycles as u64;
        let dots = self.usable_cycles / 4;
        if dots < 1 {
            return;
        }
        self.usable_cycles %= 4;
        self.x += dots;
        let mut disp_stat = memory.readu16(IO_BASE + DISPSTAT).data;
        if self.x >= (HDRAW + HBLANK) {
            self.y += 1;
            self.x %= HDRAW + HBLANK;

            self.check_vblank(&mut disp_stat);

            memory.ppu_io_write(VCOUNT, self.y as u16);
        }
        memory.ppu_io_write(DISPSTAT, disp_stat);
    }

    #[inline(always)]
    fn check_vblank(&mut self, disp_stat: &mut u16) {
        if self.y == VDRAW {
            *disp_stat |= VBLANK_FLAG;
        } else if self.y < VDRAW {
            *disp_stat &= !VBLANK_FLAG;
        }
        self.y %= VDRAW + VBLANK;
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        gba::GBA,
        graphics::ppu::{HBLANK, HDRAW, VDRAW},
        memory::io_handlers::{DISPSTAT, IO_BASE},
    };

    use super::VBLANK_ENABLE;

    #[test]
    fn ppu_sets_vblank_flag_when_in_vblank() {
        let mut gba = GBA::new_no_bios();
        gba.memory.writeu16(IO_BASE + DISPSTAT, VBLANK_ENABLE); // Enable VBLANK
        assert_eq!(gba.memory.readu16(IO_BASE + DISPSTAT).data, 0x8);

        for _ in 0..(VDRAW * (HDRAW + HBLANK) * 4) {
            gba.step();
        }

        assert_eq!(gba.memory.readu16(IO_BASE + DISPSTAT).data, 0x9);
    }
}
