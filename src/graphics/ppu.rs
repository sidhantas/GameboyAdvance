use std::sync::MutexGuard;

use num_traits::pow;

use crate::memory::{
    io_handlers::{DISPCNT, DISPSTAT, IO_BASE, VCOUNT},
    memory::GBAMemory,
};

use super::display::CANVAS_AREA;

pub const HDRAW: u32 = 240;
const HBLANK: u32 = 68;
pub const VDRAW: u32 = 160;
const VBLANK: u32 = 68;

const VBLANK_FLAG: u16 = 1 << 0;
const HBLANK_FLAG: u16 = 1 << 1;
const VCOUNTER_FLAG: u16 = 1 << 2;
const VBLANK_ENABLE: u16 = 1 << 3;
const HBLANK_ENABLE: u16 = 1 << 4;

#[derive(Default, Debug)]
pub struct PPU {
    usable_cycles: u32,
    pub x: u32,
    pub y: u32,
}

impl PPU {
    pub fn advance_ppu(
        &mut self,
        cycles: u8,
        memory: &mut GBAMemory,
        display_buffer: &mut MutexGuard<'_, [u32; CANVAS_AREA]>,
    ) {
        self.usable_cycles += cycles as u32;
        let dots = self.usable_cycles / 4;
        if dots < 1 {
            return;
        }

        // get background pixels with priority
        // get window pixels with priority
        // get obj pixels with priority
        // overlay on top of each other

        self.update_display(dots, memory, display_buffer);
        self.usable_cycles %= 4;
        self.update_dots(memory);
    }

    #[inline(always)]
    fn get_background_pixel(&self, bg_mode: u16) -> u32 {
        // placeholder
        match bg_mode {
            0x2 => pow(self.x * self.y, 2),
            _ => 0,
        }
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

    fn update_display(
        &mut self,
        mut dots: u32,
        memory: &mut GBAMemory,
        display_buffer: &mut MutexGuard<'_, [u32; CANVAS_AREA]>,
    ) {
        while dots > 0 {
            self.x += dots;
            dots -= 1;
        }
        if self.x >= HDRAW || self.y >= VDRAW {
            return;
        }
        let dispcnt = memory.io_load(DISPCNT);
        let bg_mode = Self::get_mode(dispcnt);
        display_buffer[(self.y * HDRAW + self.x) as usize] = self.get_background_pixel(bg_mode)
    }

    #[inline(always)]
    fn update_dots(&mut self, memory: &mut GBAMemory) {
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
    fn get_mode(dispcnt: u16) -> u16 {
        dispcnt & 0x7
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
