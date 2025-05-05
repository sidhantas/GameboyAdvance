use std::{cmp::Ordering, sync::Arc};

use crate::{
    graphics::{
        color_effects::color_effects_pipeline,
        display::DisplayBuffer,
        layers::Layers,
        pallete::{rgb555_to_rgb24, BGPalleteData, OBJPaletteData},
        ppu::{PPUModes, HBLANK_FLAG, HDRAW, PPU},
        wrappers::{
            oam::{OBJMode, Oam},
            tile::Tile,
        },
    },
    memory::{
        memory::GBAMemory,
        wrappers::{bgcnt::BGCnt, dispcnt::Dispcnt},
    },
};

#[derive(Default, Clone, Copy, PartialEq, Eq)]
pub struct RGBComponents {
    pub r: u16,
    pub g: u16,
    pub b: u16,
}

impl RGBComponents {
    pub fn backdrop() -> Self {
        Self {
            r: 0x1f,
            g: 0x1f,
            b: 0x1f,
        }
    }
}

impl From<u16> for RGBComponents {
    fn from(value: u16) -> Self {
        Self {
            r: (value >> 10) & 0x1F,
            g: (value >> 5) & 0x1F,
            b: value & 0x1F,
        }
    }
}

#[derive(Clone, Copy)]
pub struct OBJPixel {
    pub priority: u16,
    pub pixel: RGBComponents,
    pub is_semi_transparent: bool,
}

#[derive(Clone, Copy)]
pub struct BGPixel {
    pub priority: u16,
    pub pixel: RGBComponents,
}

impl PartialEq for BGPixel {
    fn eq(&self, other: &Self) -> bool {
        self.priority == other.priority
    }
}

impl PartialOrd for BGPixel {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        if self.priority < other.priority {
            return Some(Ordering::Less);
        } else if self.priority == other.priority {
            return Some(Ordering::Equal);
        } else if self.priority > other.priority {
            return Some(Ordering::Greater);
        }
        None
    }
}

impl BGPixel {
    pub fn backdrop() -> Self {
        Self {
            priority: 3,
            pixel: RGBComponents::backdrop(),
        }
    }
}

impl PPU {
    pub(crate) fn hdraw(
        &mut self,
        mut dots: u32,
        memory: &mut GBAMemory,
        disp_stat: &mut u16,
        display_buffer: &Arc<DisplayBuffer>,
    ) -> u32 {
        let mut display_buffer = display_buffer.buffer.lock().unwrap();
        while dots > 0 {
            if self.x >= HDRAW {
                *disp_stat |= HBLANK_FLAG;
                self.current_mode = PPUModes::HBLANK;
                return dots;
            }
            let obj_pixel = self.get_obj_pixel(memory);

            let enabled_layers = Layers::get_enabled_layers(self.x, self.y, memory, obj_pixel);

            display_buffer[(self.y * HDRAW + self.x) as usize] =
                rgb555_to_rgb24(color_effects_pipeline(memory, enabled_layers));

            dots -= 1;
            self.x += 1;
        }

        return 0;
    }

    fn get_obj_pixel(&self, memory: &GBAMemory) -> Option<OBJPixel> {
        for obj in &self.current_line_objects {
            let oam = Oam::oam_read(memory, *obj);
            if oam.x() < self.x && self.x <= oam.x() + oam.width() {
                let tile_x = (self.x - oam.x()) / 8;
                let tile_y = (self.y - oam.y()) / 8;
                let pixel_x = (self.x - oam.x()) % 8;
                let pixel_y = (self.y - oam.y()) % 8;
                let tile = Tile::get_tile_relative_obj(memory, &oam, tile_x, tile_y);

                let pallete_region = &memory.pallete_ram[0x200..][..0x200].try_into().unwrap();
                let pallete = OBJPaletteData(pallete_region);
                if let Some(pixel) =
                    pallete.get_pixel_from_tile(&tile, pixel_x as usize, pixel_y as usize)
                {
                    return Some(OBJPixel {
                        priority: oam.priority(),
                        pixel,
                        is_semi_transparent: matches!(oam.obj_mode(), OBJMode::SemiTransparent),
                    });
                }
            }
        }
        return None;
    }
}
