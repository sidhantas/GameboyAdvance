use std::{cmp::Reverse, collections::BinaryHeap, sync::Arc};

use crate::{
    graphics::{
        color_effects::color_effects_pipeline,
        display::DisplayBuffer,
        layers::{Layers, OBJPixel},
        pallete::{rgb555_to_rgb24, OBJPalleteData},
        ppu::{PPUModes, HBLANK_FLAG, HDRAW, PPU},
        wrappers::tile::Tile,
    },
    memory::{
        io_handlers::DISPCNT,
        memory::GBAMemory,
        oam::{OBJMode, Oam},
        wrappers::dispcnt::Dispcnt,
    },
};

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
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
    pub(crate) fn hdraw(&mut self, memory: &mut GBAMemory, display_buffer: &Arc<DisplayBuffer>) {
        let mut display_buffer = display_buffer.buffer.lock().unwrap();
        for _ in 0..HDRAW {
            let obj_pixel = self.get_obj_pixel();

            let enabled_layers = Layers::get_enabled_layers(
                self.x,
                self.y,
                &Dispcnt(memory.io_load(DISPCNT)),
                memory,
                obj_pixel,
            );

            display_buffer[(self.y * HDRAW + self.x) as usize] =
                rgb555_to_rgb24(color_effects_pipeline(memory, enabled_layers));

            self.x += 1;
        }
        self.current_mode = PPUModes::HBLANK;
    }

    fn get_obj_pixel(
        &mut self,
    ) -> Option<OBJPixel> {
        self.obj_buffer[self.x as usize]
    }

    pub(super) fn get_tile_coordinates(x: i32, y: i32) -> (i32, i32, i32, i32) {
        let tile_x = x / 8;
        let tile_y = y / 8;
        let pixel_x = x % 8;
        let pixel_y = y % 8;
        (tile_x, tile_y, pixel_x, pixel_y)
    }

    pub(super) fn transform_coordinates(
        memory: &GBAMemory,
        oam: &Oam,
        x: i32,
        y: i32,
    ) -> (i32, i32) {
        if let Some(affine_group) = oam.rotation_scaling_parameter() {
            let affine_parameters = memory.oam.get_affine_paramters(affine_group);
            return affine_parameters.transform_coordinates(x, y, oam);
        };

        (x, y)
    }
}
