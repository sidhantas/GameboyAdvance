use crate::{graphics::ppu::{PPU, PPUModes, VBLANK, VBLANK_FLAG, VDRAW}, memory::memory::{CPUEvent, CPUEventType, GBAMemory}};

impl PPU {
    pub(crate) fn vblank(&mut self, memory: &mut GBAMemory) {
        self.y += 1;

        if self.y >= (VDRAW + VBLANK) {
            self.y = 0;
            self.x = 0;
            
            memory.add_event(CPUEvent::new(0, CPUEventType::VBlank));
            self.current_mode = PPUModes::HDRAW;
        }
    }
}
