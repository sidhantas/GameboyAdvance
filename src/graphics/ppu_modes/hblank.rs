use crate::debugger::terminal_commands::PPUToDisplayCommands::{self, Render, RenderWithBorders};
use crate::graphics::layers::OBJPixel;
use crate::graphics::pallete::OBJPalleteData;
use crate::graphics::ppu::{PPUModes, HDRAW, PPU, VDRAW};
use crate::graphics::wrappers::tile::Tile;
use crate::memory::io_handlers::DISPCNT;
use crate::memory::memory::GBAMemory;
use crate::memory::oam::{OBJMode, NUM_OAM_ENTRIES};
use crate::memory::wrappers::dispcnt::Dispcnt;

impl PPU {
    pub(crate) fn hblank(&mut self, memory: &mut GBAMemory) {
        self.y += 1;
        self.x = 0;
        if self.y < VDRAW {
            let dispcnt = Dispcnt(memory.io_load(DISPCNT));
            self.obj_selection(memory);
            self.update_oam_objects(memory, dispcnt);
            self.current_mode = PPUModes::HDRAW;
            return;
        }
        self.current_mode = PPUModes::VBLANK;
        self.start_display_rendering(memory);
    }

    fn start_display_rendering(&mut self, memory: &mut GBAMemory) {
        match self.show_borders {
            true => self.send_command(RenderWithBorders(memory.get_oam_borders())),
            false => self.send_command(Render),
        }
    }

    fn send_command(&mut self, command: PPUToDisplayCommands) {
        self.ppu_to_display_sender.send(command).unwrap()
    }

    fn obj_selection(&mut self, memory: &mut GBAMemory) {
        self.obj_buffer.fill(None);
        self.obj_window.fill(false);
        if memory.oam.is_dirty {
            self.active_objects.clear();
            for i in 0..NUM_OAM_ENTRIES {
                let object = memory.oam.oam_read(i);
                if !object.obj_disabled() && object.y() + object.view_height() >= 0 {
                    self.active_objects.push(object);
                }
                memory.oam.is_dirty = false;
            }
        }
    }
    pub fn update_oam_objects(&mut self, memory: &mut GBAMemory, dispcnt: Dispcnt) {
        let pallete_region = memory.pallete_ram.memory[0x200..][..0x200]
            .try_into()
            .unwrap();
        let pallete = OBJPalleteData(pallete_region);
        for object in &self.active_objects {
            if !(object.y() <= self.y && self.y < object.y() + object.view_height()) {
                continue;
            }
            if object.obj_mode() == OBJMode::OBJWindow {
                for i in object.x()..object.x() + object.view_width() {
                    if i < 0 || i >= HDRAW || self.obj_window[i as usize] {
                        continue;
                    };
                    let offset_x = i - object.x();
                    let offset_y = self.y - object.y();
                    let (transform_x, transform_y) =
                        Self::transform_coordinates(memory, &object, offset_x, offset_y);
                    if transform_x < 0
                        || transform_x > object.width()
                        || transform_y <= 0
                        || transform_y >= object.height()
                    {
                        continue;
                    }
                    let (tile_x, tile_y, pixel_x, pixel_y) =
                        Self::get_tile_coordinates(transform_x, transform_y);
                    let tile = Tile::get_tile_relative_obj(memory, &object, tile_x, tile_y);
                    self.obj_window[i as usize] = pallete
                        .get_pixel_from_tile(&tile, pixel_x as usize, pixel_y as usize)
                        .is_some();
                }
                continue;
            }
            for i in object.x()..object.x() + object.view_width() {
                if i < 0 || i >= HDRAW {
                    continue;
                };
                if let Some(current_obj) = self.obj_buffer[i as usize] {
                    if current_obj.priority <= object.priority() {
                        continue;
                    }
                }
                let offset_x = i - object.x();
                let offset_y = self.y - object.y();
                let (transform_x, transform_y) =
                    Self::transform_coordinates(memory, &object, offset_x, offset_y);
                if transform_x < 0
                    || transform_x > object.width()
                    || transform_y <= 0
                    || transform_y >= object.height()
                {
                    continue;
                }
                let (tile_x, tile_y, pixel_x, pixel_y) =
                    Self::get_tile_coordinates(transform_x, transform_y);
                let tile = Tile::get_tile_relative_obj(memory, &object, tile_x, tile_y);
                self.obj_buffer[i as usize] = pallete
                    .get_pixel_from_tile(&tile, pixel_x as usize, pixel_y as usize)
                    .map(|pixel| OBJPixel {
                        priority: object.priority(),
                        pixel,
                        mode: object.obj_mode(),
                    });
            }
        }
    }
}
