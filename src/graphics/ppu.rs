use crate::memory::io_handlers::{DISPSTAT, VCOUNT};
use crate::memory::memory::GBAMemory;
use std::sync::Arc;

use super::display::DisplayBuffer;

pub const HDRAW: u32 = 240;
pub const HBLANK: u32 = 68;
pub const VDRAW: u32 = 160;
pub const VBLANK: u32 = 68;

pub(super) const VBLANK_FLAG: u16 = 1 << 0;
pub(super) const HBLANK_FLAG: u16 = 1 << 1;
pub(super) const VCOUNTER_FLAG: u16 = 1 << 2;
pub(super) const VBLANK_ENABLE: u16 = 1 << 3;
pub(super) const HBLANK_ENABLE: u16 = 1 << 4;

#[derive(Default, Debug)]
pub(crate) enum PPUModes {
    #[default]
    HDRAW,
    HBLANK,
    VBLANK,
}

#[derive(Debug)]
pub struct PPU {
    usable_cycles: u32,
    available_dots: u32,
    pub(super) current_mode: PPUModes,
    pub x: u32,
    pub y: u32,
    pub(super) current_line_objects: Vec<usize>,
}

impl Default for PPU {
    fn default() -> Self {
        Self {
            usable_cycles: 0,
            available_dots: 0,
            current_mode: PPUModes::HDRAW,
            x: 0,
            y: 0,
            current_line_objects: Vec::new(),
        }
    }
}

impl PPU {
    pub fn advance_ppu(
        &mut self,
        cycles: u8,
        memory: &mut GBAMemory,
        display_buffer: &Arc<DisplayBuffer>,
    ) {
        self.usable_cycles += cycles as u32;
        self.available_dots += self.usable_cycles / 4;
        self.usable_cycles %= 4;
        if self.available_dots < 1 {
            return;
        }

        let mut dispstat = memory.io_load(DISPSTAT);

        self.available_dots = match self.current_mode {
            PPUModes::HDRAW => {
                self.hdraw(self.available_dots, memory, &mut dispstat, display_buffer)
            }
            PPUModes::HBLANK => {
                self.hblank(self.available_dots, memory, &mut dispstat, display_buffer)
            }
            PPUModes::VBLANK => self.vblank(self.available_dots, &mut dispstat),
        };

        // get background pixels with priority
        // get window pixels with priority
        // get obj pixels with priority
        // overlay on top of each other
        memory.ppu_io_write(DISPSTAT, dispstat);
        memory.ppu_io_write(VCOUNT, self.y as u16);
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

        for _ in 0..((VDRAW + 1) * (HDRAW + HBLANK) * 4) {
            gba.step();
        }

        assert_eq!(gba.memory.readu16(IO_BASE + DISPSTAT).data, 0x9);
    }
}
