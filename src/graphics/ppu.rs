use crate::memory::{io_handlers::VCOUNT, memory::MemoryBus};

const HDRAW: u32 = 240;
const HBLANK: u32 = 68;
const VDRAW: u32 = 160;
const VBLANK: u32 = 68;

#[derive(Default)]
pub struct PPU {
    pub x: u64,
    pub y: u64,
}


impl PPU {
    pub fn advance_ppu(&mut self, cycles: &mut u64, memory: &mut Box<dyn MemoryBus>) {
        let dots = *cycles / 4;
        *cycles %= 4;
        self.x += dots;
        if self.x > (HDRAW + HBLANK) as u64 {
            self.y += 1;
            self.x %= (HDRAW + HBLANK) as u64;

            if self.y > (VDRAW + VBLANK) as u64 {
                self.y %= (VDRAW + VBLANK) as u64;
            }
            memory.ppu_io_write(VCOUNT, self.y as u16);
        }

    }
}
