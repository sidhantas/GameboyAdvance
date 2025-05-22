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
        memory: &mut GBAMemory,
        disp_stat: &mut u16,
        display_buffer: &Arc<DisplayBuffer>,
    ) {
        let mut display_buffer = display_buffer.buffer.lock().unwrap();
        let pallete_region = unsafe {
            &memory.pallete_ram.memory[0x200..][..0x200]
                .try_into()
                .unwrap_unchecked()
        };
        let pallete = OBJPalleteData(pallete_region);
        for _ in 0..HDRAW {
            self.current_line_objects.update_active_objects();
            let obj_pixel = if let Some(mut active_objects) = self.active_objects.take() {
                let obj_pixel = self.get_obj_pixel(memory, &mut active_objects, &pallete);
                self.active_objects.replace(active_objects);
                obj_pixel
            } else {
                None
            };

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
        *disp_stat |= HBLANK_FLAG;
        self.current_mode = PPUModes::HBLANK;
    }

    fn get_obj_pixel(
        &mut self,
        memory: &GBAMemory,
        active_object_heap: &mut BinaryHeap<Reverse<usize>>,
        pallete: &OBJPalleteData,
    ) -> Option<OBJPixel> {
        self.current_line_objects.active_objects(active_object_heap);
        while let Some(oam_num) = active_object_heap.pop() {
            let oam = self.current_line_objects.get_oam(oam_num.0);
            let offset_x = self.x - oam.x();
            let offset_y = self.y - oam.y();
            let (transform_x, transform_y) =
                self.transform_coordinates(memory, &oam, offset_x, offset_y);
            if 0 < transform_x
                && transform_x <= oam.width()
                && 0 < transform_y
                && transform_y < oam.height()
            {
                let (tile_x, tile_y, pixel_x, pixel_y) =
                    self.get_tile_coordinates(transform_x, transform_y);
                let tile = Tile::get_tile_relative_obj(memory, &oam, tile_x, tile_y);

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

    fn get_tile_coordinates(&self, x: i32, y: i32) -> (i32, i32, i32, i32) {
        let tile_x = x / 8;
        let tile_y = y / 8;
        let pixel_x = x % 8;
        let pixel_y = y % 8;
        (tile_x, tile_y, pixel_x, pixel_y)
    }

    fn transform_coordinates(&self, memory: &GBAMemory, oam: &Oam, x: i32, y: i32) -> (i32, i32) {
        if let Some(affine_group) = oam.rotation_scaling_parameter() {
            let affine_parameters = memory.oam.get_affine_paramters(affine_group);
            return affine_parameters.transform_coordinates(x, y, oam);
        };

        (x, y)
    }
}
