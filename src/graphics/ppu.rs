use std::sync::{Arc, Mutex};

use crate::memory::Memory;


struct Pixel {
    bg_mode: u8,
    display_frame_select: u8,
    h_blank_interval_free: u8,
    obj_char_vram_mapping: u8,
}

impl Pixel {
    pub fn new(memory: &Memory) {

    }
}

#[repr(u32)]
enum DisplayAddresses {
    DISPCNT = 0x4000_0000,
    DISPSTAT = 0x4000_0004,
    VCOUNT = 0x4000_0006,
    BG0CNT = 0x4000_0008
}

pub struct PPU {
    memory: Arc<Mutex<Memory>>,
    frame_buffer: Vec<u8>
}

