use std::sync::Arc;

use crate::{
    graphics::{
        color_effects::color_effects_pipeline,
        display::DisplayBuffer,
        layers::{Layers, OBJPixel},
        pallete::{rgb555_to_rgb24, OBJPaletteData},
        ppu::{PPUModes, HBLANK_FLAG, HDRAW, PPU},
        wrappers::{
            oam::{OBJMode, Oam}, rotation_and_scaling::AffineParameters, tile::Tile
        },
    },
    memory::{io_handlers::DISPCNT, memory::GBAMemory, wrappers::dispcnt::Dispcnt},
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

            let enabled_layers = Layers::get_enabled_layers(
                self.x,
                self.y,
                &Dispcnt(memory.io_load(DISPCNT)),
                memory,
                obj_pixel,
            );

            display_buffer[(self.y * HDRAW + self.x) as usize] =
                rgb555_to_rgb24(color_effects_pipeline(memory, enabled_layers));

            dots -= 1;
            self.x += 1;
        }

        return 0;
    }

    fn get_obj_pixel(&self, memory: &GBAMemory) -> Option<OBJPixel> {
        let mut highest_prio_obj: Option<OBJPixel> = None;
        for obj in &self.current_line_objects {
            let oam = Oam::oam_read(memory, *obj);
            let normalized_x = self.x - oam.view_x();
            let normalized_y = self.y - oam.view_y();
            let (transform_x, transform_y) = self.transform_coordinates(memory, &oam, normalized_x, normalized_y);
            if 0 < transform_x
                && transform_x <= oam.width()
                && 0 < transform_y
                && transform_y <= oam.height()
            {
                let (tile_x, tile_y, pixel_x, pixel_y) = self.get_tile_coordinates(&oam, transform_x, transform_y);
                let tile = Tile::get_tile_relative_obj(memory, &oam, tile_x, tile_y);

                let pallete_region = &memory.pallete_ram[0x200..][..0x200].try_into().unwrap();
                let pallete = OBJPaletteData(pallete_region);
                if let Some(pixel) =
                    pallete.get_pixel_from_tile(&tile, pixel_x as usize, pixel_y as usize)
                {
                    let obj = OBJPixel {
                        priority: oam.priority(),
                        pixel,
                        is_semi_transparent: matches!(oam.obj_mode(), OBJMode::SemiTransparent),
                    };
                    highest_prio_obj = highest_prio_obj.map_or(Some(obj), |current_obj| {
                        if current_obj.priority > oam.priority() {
                            Some(obj)
                        } else {
                            Some(current_obj)
                        }
                    });
                }
            }
        }
        return highest_prio_obj;
    }

    fn get_tile_coordinates(&self, oam: &Oam<'_>, x: i32, y: i32) -> (i32, i32, i32, i32) {
        let tile_x = x / 8;
        let tile_y = y / 8;
        let pixel_x = x % 8;
        let pixel_y = y % 8;
        (tile_x, tile_y, pixel_x, pixel_y)
    }

    fn transform_coordinates(&self, memory: &GBAMemory, oam: &Oam, x: i32, y: i32) -> (i32, i32) {
        let Some(affine_parameters) = AffineParameters::create_parameters(memory, oam) else {
            return (x, y)
        };

        affine_parameters.transform_coordinates(x, y, oam)

    }
}
