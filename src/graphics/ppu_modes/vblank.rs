use crate::graphics::ppu::{PPUModes, HBLANK, HDRAW, PPU, VBLANK, VBLANK_FLAG, VDRAW};

impl PPU {
    pub(crate) fn vblank(&mut self, mut dots: u32, disp_stat: &mut u16) -> u32 {
        while dots > 0 {
            if self.x >= HDRAW + HBLANK {
                self.y += 1;
                self.x = 0;

                let var_name = VDRAW + VBLANK;
                if self.y >= var_name {
                    self.y = 0;
                    self.current_mode = PPUModes::HDRAW;
                    *disp_stat &= !VBLANK_FLAG;
                    return dots;
                }
            }
            self.x += 1;
            dots -= 1;
        }
        return 0;
    }
}
