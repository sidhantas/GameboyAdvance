pub(crate) use std::sync::{atomic::Ordering, Arc};

use crate::debugger::terminal_commands::PPUToDisplayCommands::{Render, RenderWithBorders};
use crate::graphics::display::DisplayBuffer;
use crate::graphics::ppu::{PPUModes, HBLANK, HBLANK_FLAG, HDRAW, PPU, VBLANK_FLAG, VDRAW};
use crate::graphics::wrappers::oam::{Oam, NUM_OAM_ENTRIES};
use crate::memory::memory::GBAMemory;

impl PPU {
    pub(crate) fn hblank(
        &mut self,
        mut dots: u32,
        memory: &mut GBAMemory,
        disp_stat: &mut u16,
        display_buffer: &Arc<DisplayBuffer>,
    ) -> u32 {
        while dots > 0 {
            if self.x >= HDRAW + HBLANK {
                self.y += 1;
                self.x = 0;
                if self.y >= VDRAW {
                    *disp_stat &= !HBLANK_FLAG;
                    *disp_stat |= VBLANK_FLAG;
                    if self.show_borders {
                        self.ppu_to_display_sender.send(RenderWithBorders(memory.get_oam_borders())).unwrap()
                    } else {
                        self.ppu_to_display_sender.send(Render).unwrap();
                    }
                    self.current_mode = PPUModes::VBLANK;
                } else {
                    self.obj_selection(memory);
                    self.current_mode = PPUModes::HDRAW;
                }
                return dots;
            }
            self.x += 1;
            dots -= 1;
        }
        return 0;
    }

    fn obj_selection(&mut self, memory: &mut GBAMemory) {
        self.current_line_objects.clear();
        for i in 0..NUM_OAM_ENTRIES {
            let oam = Oam::oam_read(memory, i);
            if (oam.y() < self.y
                && self.y < oam.y() + oam.height())
                && !oam.obj_disabled()
            {
                self.current_line_objects.push(i);
            }
        }
    }
}
