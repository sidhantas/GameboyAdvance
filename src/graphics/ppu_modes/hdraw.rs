use std::sync::Arc;

use crate::{
    graphics::{
        color_effects::color_effects_pipeline,
        display::DisplayBuffer,
        pallete::{rgb555_to_rgb24, BGPalleteData, OBJPaletteData},
        ppu::{PPUModes, HBLANK_FLAG, HDRAW, PPU},
        wrappers::{
            oam::{OBJMode, Oam},
            tile::Tile,
        },
    },
    memory::{
        io_handlers::DISPCNT,
        memory::GBAMemory,
        wrappers::{bgcnt::BGCnt, dispcnt::Dispcnt},
    },
};

#[derive(Default)]
pub struct RGBComponents {
    pub r: u16,
    pub g: u16,
    pub b: u16,
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

pub struct OBJPixel {
    pub priority: u16,
    pub pixel: RGBComponents,
    pub is_semi_transparent: bool,
}

pub struct BGPixel {
    pub priority: u16,
    pub pixel: RGBComponents,
    pub layer: u16
}

impl PPU {
    pub(crate) fn hdraw(
        &mut self,
        mut dots: u32,
        memory: &mut GBAMemory,
        disp_stat: &mut u16,
        display_buffer: &Arc<DisplayBuffer>,
    ) -> u32 {
        let dispcnt = memory.io_load(DISPCNT);
        let mut display_buffer = display_buffer.buffer.lock().unwrap();
        while dots > 0 {
            if self.x >= HDRAW {
                *disp_stat |= HBLANK_FLAG;
                self.current_mode = PPUModes::HBLANK;
                return dots;
            }
            let bg_pixel = self.get_background_pixel(memory, &Dispcnt(dispcnt));
            let obj_pixel = self.get_obj_pixel(memory);

            display_buffer[(self.y * HDRAW + self.x) as usize] =
                rgb555_to_rgb24(color_effects_pipeline(memory, obj_pixel, bg_pixel));

            dots -= 1;
            self.x += 1;
        }

        return 0;
    }

    fn get_background_pixel(&self, memory: &GBAMemory, dispcnt: &Dispcnt) -> BGPixel {
        let tile_x = self.x / 8;
        let tile_y = self.y / 8;
        let pixel_x = self.x % 8;
        let pixel_y = self.y % 8;

        let highest_priority_bg = self.get_highest_priority_bgcnt(memory, dispcnt);
        let pallete_region = &memory.pallete_ram[0x00..][..0x200].try_into().unwrap();
        let pallete = BGPalleteData(pallete_region);

        let Some(highest_priority_bg) = highest_priority_bg else {
            return BGPixel {
                priority: 3,
                pixel: pallete
                    .get_bg_color(0, 0, 1, true)
                    .unwrap_or(RGBComponents {
                        r: 0xFF,
                        g: 0,
                        b: 0,
                    }),
                layer: 4
            };
        };

        let tile = match dispcnt.get_bg_mode() {
            0x2 => Tile::get_tile_relative_bg(
                memory,
                &highest_priority_bg,
                dispcnt,
                tile_y as usize,
                tile_x as usize,
            ),
            _ => {
                return BGPixel {
                    priority: 3,
                    pixel: Default::default(),
                    layer: 4
                }
            }
        };

        BGPixel {
            priority: highest_priority_bg.priority(),
            pixel: pallete
                .get_pixel_from_tile(&tile, pixel_x as usize, pixel_y as usize)
                .unwrap_or(RGBComponents {
                    r: 0x1F,
                    g: 0x1F,
                    b: 0x1F,
                }),
        }
    }

    #[inline]
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
