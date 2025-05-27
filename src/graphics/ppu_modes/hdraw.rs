use std::{cmp::Reverse, collections::BinaryHeap, fmt::Display, sync::Arc};

use crate::{
    graphics::{
        color_effects::color_effects_pipeline,
        display::DisplayBuffer,
        layers::{DisplayContext, Layers, OBJPixel},
        pallete::{rgb555_to_rgb24, OBJPalleteData},
        ppu::{PPUModes, HBLANK_FLAG, HDRAW, PPU},
        wrappers::tile::Tile,
    },
    memory::{
        io_handlers::{DISPCNT, WINOUT},
        memory::GBAMemory,
        oam::{OBJMode, Oam},
        wrappers::{dispcnt::Dispcnt, window::WinOut},
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
            r: value & 0x1F,
            g: (value >> 5) & 0x1F,
            b: (value >> 10) & 0x1F,
        }
    }
}

impl PPU {
    pub(crate) fn hdraw(&mut self, memory: &mut GBAMemory, display_buffer: &Arc<DisplayBuffer>) {
        let mut display_buffer = display_buffer.buffer.lock().unwrap();
        let dispcnt = Dispcnt(memory.io_load(DISPCNT));
        let winout = WinOut(memory.io_load(WINOUT));
        let default_context = DisplayContext {
            bg0_enabled: dispcnt.bg0_enabled(),
            bg1_enabled: dispcnt.bg1_enabled(),
            bg2_enabled: dispcnt.bg2_enabled(),
            bg3_enabled: dispcnt.bg3_enabled(),
            obj_enabled: dispcnt.obj_enabled(),
            color_special_effects_enabled: true,
        };
        let winout_context = DisplayContext {
            bg0_enabled: winout.bg0_enabled() && default_context.bg0_enabled,
            bg1_enabled: winout.bg1_enabled() && default_context.bg1_enabled,
            bg2_enabled: winout.bg2_enabled() && default_context.bg2_enabled,
            bg3_enabled: winout.bg3_enabled() && default_context.bg3_enabled,
            obj_enabled: winout.obj_enabled() && default_context.obj_enabled,
            color_special_effects_enabled: winout.color_special_effects_enabled(),
        };
        let obj_window_context = DisplayContext {
            bg0_enabled: winout.obj_window_bg0_enabled() && default_context.bg0_enabled,
            bg1_enabled: winout.obj_window_bg1_enabled() && default_context.bg1_enabled,
            bg2_enabled: winout.obj_window_bg2_enabled() && default_context.bg2_enabled,
            bg3_enabled: winout.obj_window_bg3_enabled() && default_context.bg3_enabled,
            obj_enabled: winout.obj_window_obj_enabled() && default_context.obj_enabled,
            color_special_effects_enabled: winout.obj_window_color_special_effects_enabled(),
        };
        for x in 0..HDRAW {
            let obj_pixel = self.get_obj_pixel(x as usize);
            let current_context = if dispcnt.winout_enabled() {
                if self.obj_window[x as usize]
                    && dispcnt.obj_enabled()
                    && dispcnt.obj_window_enabled()
                {
                    &obj_window_context
                } else {
                    &winout_context
                }
            } else {
                &default_context
            };

            let enabled_layers = Layers::get_enabled_layers(
                self.x,
                self.y,
                memory,
                obj_pixel,
                dispcnt.get_bg_mode(),
                current_context,
            );

            display_buffer[(self.y * HDRAW + x) as usize] =
                rgb555_to_rgb24(color_effects_pipeline(memory, enabled_layers));
        }
        self.current_mode = PPUModes::HBLANK;
    }

    fn get_obj_pixel(&mut self, x: usize) -> Option<OBJPixel> {
        self.obj_buffer[x]
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
