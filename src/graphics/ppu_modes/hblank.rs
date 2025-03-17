pub(crate) use std::sync::{atomic::Ordering, Arc};

use crate::graphics::display::DisplayBuffer;
use crate::graphics::ppu::{PPUModes, HBLANK, HDRAW, PPU, VBLANK_FLAG, VDRAW};
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
                    *disp_stat |= VBLANK_FLAG;
                    display_buffer
                        .ready_to_render
                        .store(true, Ordering::Relaxed);
                    self.current_mode = PPUModes::VBLANK;
                } else {
                    self.current_line_objects.clear();
                    for i in 0..NUM_OAM_ENTRIES {
                        let oam = Oam::oam_read(memory, i);
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
}
