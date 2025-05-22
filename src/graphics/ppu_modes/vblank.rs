use crate::graphics::ppu::{PPUModes, HBLANK, HDRAW, PPU, VBLANK, VBLANK_FLAG, VDRAW};

impl PPU {
    pub(crate) fn vblank(&mut self, disp_stat: &mut u16) {
        self.y += 1;

        if self.y >= (VDRAW + VBLANK) {
            self.y = 0;
            self.x = 0;
            self.current_mode = PPUModes::HDRAW;
            *disp_stat &= !VBLANK_FLAG;
        }
    }
}
