use crate::memory::memory::GBAMemory;

use super::oam::Oam;

pub enum Tile<'a> {
    FourBit(&'a [u8; 32]),
    EightBit(&'a [u8; 64]),
}

impl<'a> Tile<'a> {
    pub fn get_tile_relative(
        memory: &'a GBAMemory,
        oam: &Oam,
        offset_x: u32,
        offset_y: u32,
    ) -> Self {
        let relative_tile = oam.tile_number() as u32 + offset_y * 0x20 + offset_x * 2;
        Self::get_tile_single(memory, oam, relative_tile as usize)
    }

    fn get_tile_single(memory: &'a GBAMemory, oam: &Oam, tile_num: usize) -> Self {
        if oam.color_pallete() == 0 {
            Self::FourBit(
                memory.vram[0x10000 + tile_num * 32..][..32]
                    .try_into()
                    .unwrap(),
            )
        } else {
            Self::EightBit(
                memory.vram[0x10000 + tile_num * 32..][..64]
                    .try_into()
                    .unwrap(),
            )
        }
    }
}
