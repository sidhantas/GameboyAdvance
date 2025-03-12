use std::sync::MutexGuard;

use crate::memory::{
    io_handlers::{BG0CNT, DISPCNT, DISPSTAT, IO_BASE, VCOUNT},
    memory::GBAMemory,
    wrappers::{
        bgcnt::{BGCnt, MAP_SIZE_BYTES, TILE_DATA_SIZE_BYTES},
        dispcnt::Dispcnt,
    },
};

use super::{
    background::{self, Background},
    display::{self, CANVAS_AREA},
    pallete::{BGPalleteData, OBJPalleteData},
    wrappers::{
        oam::{Oam, NUM_OAM_ENTRIES},
        tile::Tile,
    },
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
            let background_pixel =
                self.get_background_pixel(memory, &Dispcnt(&dispcnt));
            let obj_pixel = self.get_obj_pixel(memory);

            if let Some(pixel) = obj_pixel {
                display_buffer[(self.y * HDRAW + self.x) as usize] = pixel;
            } else {
                display_buffer[(self.y * HDRAW + self.x) as usize] = background_pixel;
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

    #[inline]
    fn get_background_pixel(&self, memory: &GBAMemory, dispcnt: &Dispcnt) -> u32 {
        let tile_x = self.x / 8;
        let tile_y = self.y / 8;
        let pixel_x = self.x % 8;
        let pixel_y = self.y % 8;

        let highest_priority_bg = self.get_highest_priority_bgcnt(memory, dispcnt);
        let pallete_region = &memory.pallete_ram[0x00..][..0x200].try_into().unwrap();
        let pallete = BGPalleteData(pallete_region);

        let Some(highest_priority_bg) = highest_priority_bg else {
            return pallete.get_bg_color(0, 0, 1, true).unwrap_or(0xFFFF0000);
        };

        let tile = match dispcnt.get_bg_mode() {
            0x2 => Tile::get_tile_relative_bg(
                memory,
                &highest_priority_bg,
                dispcnt,
                tile_y as usize,
                tile_x as usize,
            ),
            _ => return 0xFFFF0000,
        };



        return pallete
            .get_pixel_from_tile(&tile, pixel_x as usize, pixel_y as usize)
            .unwrap_or(0xFFFFFFFF);
    }

    #[inline]
    fn get_obj_pixel(&self, memory: &GBAMemory) -> Option<u32> {
        for obj in &self.current_line_objects {
            let oam = Self::oam_read(memory, *obj);
            if oam.x() < self.x && self.x <= oam.x() + oam.width() {
                let tile_x = (self.x - oam.x()) / 8;
                let tile_y = (self.y - oam.y()) / 8;
                let pixel_x = (self.x - oam.x()) % 8;
                let pixel_y = (self.y - oam.y()) % 8;
                let tile = Tile::get_tile_relative_obj(memory, &oam, tile_x, tile_y);

                let pallete_region = &memory.pallete_ram[0x200..][..0x200].try_into().unwrap();
                let pallete = OBJPalleteData(pallete_region);
                return pallete.get_pixel_from_tile(
                    &tile,
                    pixel_x as usize,
                    pixel_y as usize,
                );
            }
        }
        return None;
    }

    fn oam_read<'a>(memory: &'a GBAMemory, oam_num: usize) -> Oam<'a> {
        let oam_slice: &[u8; 6] = memory.oam[oam_num * 0x08..][..6].try_into().unwrap();
        let oam_slice: &[u16; 3] = unsafe { oam_slice.align_to::<u16>().1.try_into().unwrap() };

        return Oam(oam_slice);
    }

    fn get_highest_priority_bgcnt(&self, memory: &GBAMemory, dispcnt: &Dispcnt) -> Option<BGCnt> {
        let enabled_backgrounds = dispcnt.enabled_backgrounds();
        let mut highest_priority_background: Option<BGCnt> = None;

        for layer in enabled_backgrounds {
            let new_bgcnt = BGCnt(memory.io_load(0x08 + 2 * layer));

            if let Some(current_bgcnt) = highest_priority_background {
                if current_bgcnt.priority() > new_bgcnt.priority() {
                    highest_priority_background = Some(new_bgcnt);
                }
            } else {
                highest_priority_background = Some(new_bgcnt)
            }
        }
        highest_priority_background
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
