use crate::{graphics::ppu::{PPUModes, PPU, VBLANK, VBLANK_FLAG, VDRAW}, memory::memory::{IOEvent, GBAMemory}};

impl PPU {
    pub(crate) fn vblank(&mut self, memory: &mut GBAMemory) {
        self.y += 1;

        if self.y >= (VDRAW + VBLANK) {
            self.y = 0;
            self.x = 0;
            
            memory.add_event(IOEvent::HDraw);
            self.current_mode = PPUModes::HDRAW;
        }
    }
}
