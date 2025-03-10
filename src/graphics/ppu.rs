use core::slice;
use std::{default, sync::MutexGuard};

use num_traits::pow;

use crate::{
    memory::{
        io_handlers::{DISPCNT, DISPSTAT, IO_BASE, VCOUNT},
        memory::GBAMemory,
    },
    utils::bits::Bits,
};

use super::{
    display::CANVAS_AREA,
    oam::{NUM_OAM_ENTRIES, OAM},
    pallete::PalleteData,
};

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
enum PPUModes {
    #[default]
    HDRAW,
    HBLANK,
    VBLANK,
}

#[derive(Debug)]
pub struct PPU {
    usable_cycles: u32,
    available_dots: u32,
    current_mode: PPUModes,
    pub x: u32,
    pub y: u32,
    current_line_objects: Vec<usize>,
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
        display_buffer: &mut MutexGuard<'_, [u32; CANVAS_AREA]>,
    ) {
        self.usable_cycles += cycles as u32;
        self.available_dots += self.usable_cycles / 4;
        self.usable_cycles %= 4;
        if self.available_dots < 1 {
            return;
        }

        let mut dispstat = memory.readu16(IO_BASE + DISPSTAT).data;

        self.available_dots = match self.current_mode {
            PPUModes::HDRAW => self.hdraw(self.available_dots, memory, display_buffer),
            PPUModes::HBLANK => self.hblank(self.available_dots, memory, &mut dispstat),
            PPUModes::VBLANK => self.vblank(self.available_dots, &mut dispstat),
        };

        // get background pixels with priority
        // get window pixels with priority
        // get obj pixels with priority
        // overlay on top of each other
        memory.ppu_io_write(DISPSTAT, dispstat);
        memory.ppu_io_write(VCOUNT, self.y as u16);
    }

    fn hdraw(
        &mut self,
        mut dots: u32,
        memory: &mut GBAMemory,
        display_buffer: &mut MutexGuard<'_, [u32; CANVAS_AREA]>,
    ) -> u32 {
        let dispcnt = memory.io_load(DISPCNT);
        while dots > 0 {
            if self.x >= HDRAW {
                self.current_mode = PPUModes::HBLANK;
                return dots;
            }
            display_buffer[(self.y * HDRAW + self.x) as usize] = self.get_background_pixel(dispcnt);
            for obj in &self.current_line_objects {
                let oam = Self::oam_read(memory, *obj);
                if oam.x() < self.x && self.x <= oam.x() + oam.width() {
                    let offset_x = (self.x - oam.x()) / 8;
                    let offset_y = (self.y - oam.y()) / 8;
                    let pixel_x = (self.x - oam.x()) % 8;
                    let pixel_y = (self.y - oam.y()) % 8;
                    let tile = Self::get_tile_relative(memory, oam.tile_number(), offset_x, offset_y);
                    let pixel_color = tile[(pixel_y * 8 + pixel_x) as usize];

                    let pallete_region = &memory.bgram[0x00..][..0x400].try_into().unwrap();
                    let pallete = PalleteData(pallete_region);
                    if let Some(color) = pallete.get_obj_color(
                        pixel_color as usize,
                        oam.pallete_number().into(),
                        oam.color_pallete().into(),
                    ) {
                        display_buffer[(self.y * HDRAW + self.x) as usize] = color;
                    };
                }
            }

            dots -= 1;
            self.x += 1;
        }

        return 0;
    }

    fn hblank(&mut self, mut dots: u32, memory: &mut GBAMemory, disp_stat: &mut u16) -> u32 {
        while dots > 0 {
            if self.x >= HDRAW + HBLANK {
                self.y += 1;
                self.x = 0;
                if self.y >= VDRAW {
                    *disp_stat |= VBLANK_FLAG;
                    self.current_mode = PPUModes::VBLANK;
                } else {
                    self.current_line_objects.clear();
                    for i in 0..NUM_OAM_ENTRIES {
                        let oam = Self::oam_read(memory, i);
                        if oam.y() < self.y
                            && self.y <= oam.y() + oam.height()
                            && !oam.obj_disabled()
                        {
                            self.current_line_objects.push(i);
                        }
                    }
                    self.current_mode = PPUModes::HDRAW;
                }
                return dots;
            }
            self.x += 1;
            dots -= 1;
        }
        return 0;
    }

    fn vblank(&mut self, mut dots: u32, disp_stat: &mut u16) -> u32 {
        while dots > 0 {
            if self.x >= HDRAW + HBLANK {
                self.y += 1;
                self.x = 0;

                if self.y >= VDRAW + VBLANK {
                    self.y = 0;
                    self.current_mode = PPUModes::HDRAW;
                    *disp_stat &= !VBLANK_FLAG;
                    return dots;
                }
            }
            self.x += 1;
            dots -= 1;
        }
        return 0;
    }

    #[inline(always)]
    fn get_background_pixel(&self, dispcnt: u16) -> u32 {
        // placeholder
        match dispcnt & 0x3 {
            0x2 => 0xFFFFFFFF,
            _ => 0,
        }
    }

    fn oam_read<'a>(memory: &'a GBAMemory, oam_num: usize) -> OAM<'a> {
        let oam_slice: &[u8; 6] = memory.oam[oam_num * 0x08..][..6].try_into().unwrap();
        let oam_slice: &[u16; 3] = unsafe { oam_slice.align_to::<u16>().1.try_into().unwrap() };

        return OAM(oam_slice);
    }

    fn get_tile_relative<'a>(memory: &'a GBAMemory, tile_num: usize, offset_x: u32, offset_y: u32) -> &'a [u8; 64] {
        let relative_tile = tile_num as u32 + offset_y * 0x20 + offset_x * 2;
        Self::get_tile_single(memory, relative_tile as usize)
    }

    fn get_tile_single<'a>(memory: &'a GBAMemory, tile_num: usize) -> &'a [u8; 64] {
        memory.vram[0x10000 + tile_num * 32..][..64]
            .try_into()
            .unwrap()
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
