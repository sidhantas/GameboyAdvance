use crate::debugger::terminal_commands::PPUToDisplayCommands;
use crate::memory::io_handlers::{DISPSTAT, VCOUNT};
use crate::memory::memory::GBAMemory;
use crate::memory::oam::Oam;
use std::cmp::Reverse;
use std::collections::BinaryHeap;
use std::sync::mpsc::Sender;
use std::sync::Arc;

use super::display::DisplayBuffer;
use super::layers::OBJPixel;

pub const HDRAW: i32 = 240;
pub const HBLANK: i32 = 68;
pub const VDRAW: i32 = 160;
pub const VBLANK: i32 = 68;

pub(super) const VBLANK_FLAG: u16 = 1 << 0;
pub(super) const HBLANK_FLAG: u16 = 1 << 1;
pub(super) const VCOUNTER_FLAG: u16 = 1 << 2;
pub(super) const VBLANK_ENABLE: u16 = 1 << 3;
pub(super) const HBLANK_ENABLE: u16 = 1 << 4;

#[derive(Default, Debug)]
pub(crate) enum PPUModes {
    #[default]
    HDRAW,
    HBLANK,
    VBLANK,
}

#[derive(Debug)]
pub struct PPU {
    usable_cycles: u32,
    available_dots: u32,
    pub(super) current_mode: PPUModes,
    pub x: i32,
    pub y: i32,
    pub obj_buffer: [Option<OBJPixel>; HDRAW as usize],
    pub(super) active_objects: Vec<Oam>,
    pub show_borders: bool,
    pub(super) ppu_to_display_sender: Sender<PPUToDisplayCommands>,
}

impl PPU {
    pub fn new(ppu_to_display_sender: Sender<PPUToDisplayCommands>) -> Self {
        Self {
            usable_cycles: 0,
            available_dots: 0,
            current_mode: PPUModes::HDRAW,
            x: 0,
            y: 0,
            active_objects: Vec::new(),
            obj_buffer: [(); HDRAW as usize].map(|_| None),
            show_borders: false,
            ppu_to_display_sender,
        }
    }

    pub fn reset(&mut self) {
        self.usable_cycles = 0;
        self.available_dots = 0;
        self.current_mode = PPUModes::HDRAW;
        self.x = 0;
        self.y = 0;
    }
    pub fn advance_ppu(
        &mut self,
        cycles: u8,
        memory: &mut GBAMemory,
        display_buffer: &Arc<DisplayBuffer>,
    ) {
        self.available_dots += cycles as u32;

        let mut dispstat = memory.ioram.io_load(DISPSTAT);
        match self.current_mode {
            PPUModes::HDRAW => {
                if self.available_dots < 4 * HDRAW as u32 {
                    // accumulate enough dots to draw entire lin
                    return;
                }
                self.available_dots -= 4 * HDRAW as u32;
                self.hdraw(memory, display_buffer);
            }
            PPUModes::HBLANK => {
                dispstat |= HBLANK_FLAG;
                if self.available_dots < 4 * HBLANK as u32 {
                    // accumulate enough dots to draw entire lin
                    return;
                }
                self.available_dots -= 4 * HBLANK as u32;
                self.hblank(memory);
                dispstat &= !HBLANK_FLAG;
            }
            PPUModes::VBLANK => {
                dispstat |= VBLANK_FLAG;
                if self.available_dots < 4 * (HBLANK + HDRAW) as u32 {
                    // accumulate enough dots to draw entire lin
                    return;
                }
                self.available_dots -= 4 * (HBLANK + HDRAW) as u32;
                self.vblank(&mut dispstat);
            }
        };

        // get background pixels with priority
        // get window pixels with priority
        // get obj pixels with priority
        // overlay on top of each other
        memory.ppu_io_write(DISPSTAT, dispstat);
        memory.ppu_io_write(VCOUNT, self.y as u16);
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        gba::GBA,
        graphics::ppu::{HBLANK, HDRAW, VDRAW},
        memory::io_handlers::DISPSTAT,
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
