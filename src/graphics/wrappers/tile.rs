use crate::memory::{
    memory::GBAMemory,
    oam::Oam,
    wrappers::{
        bgcnt::{BGCnt, MAP_SIZE_BYTES, TILE_DATA_SIZE_BYTES},
        dispcnt::Dispcnt,
    },
};

pub enum Tile<'a> {
    FourBit {
        tile: &'a [u8; 32],
        pallete_num: usize,
    },
    EightBit {
        tile: &'a [u8; 64],
    },
}

impl<'a> Tile<'a> {
    pub fn get_tile_relative_obj(
        memory: &'a GBAMemory,
        oam: &Oam,
        tile_x: i32,
        tile_y: i32,
    ) -> Self {
        let relative_tile = oam.tile_number() as i32 + tile_y * 0x20 + tile_x * 2;
        Self::get_tile_single_obj(memory, oam, relative_tile as usize)
    }

    fn get_tile_single_obj(memory: &'a GBAMemory, object: &Oam, tile_num: usize) -> Self {
        if object.color_pallete() == 0 {
            Self::FourBit {
                tile: memory.vram.memory[0x10000 + tile_num * 32..][..32]
                    .try_into()
                    .unwrap(),
                pallete_num: object.pallete_number(),
            }
        } else {
            Self::EightBit {
                tile: memory.vram.memory[0x10000 + tile_num * 32..][..64]
                    .try_into()
                    .unwrap(),
            }
        }
    }

    pub fn get_tile_relative_bg(
        memory: &'a GBAMemory,
        bgcnt: &BGCnt,
        y: usize,
        x: usize,
    ) -> Self {
        const BYTES_PER_ENTRY: usize = 2;
        const BYTES_PER_MAP_ROW: usize = 32 * BYTES_PER_ENTRY;

        let map_address = bgcnt.map_data_base() * MAP_SIZE_BYTES;
        let relative_map_address = map_address + y * BYTES_PER_MAP_ROW + x * BYTES_PER_ENTRY;
        let text_bg_screen_entry = u16::from_le_bytes(
            memory.vram.memory[relative_map_address..][..2]
                .try_into()
                .unwrap(),
        );
        let text_bg_screen_entry = BGEntry(&text_bg_screen_entry);

        let tile_num = text_bg_screen_entry.tile_number();
        let tile_data_start = bgcnt.tile_data_base() * TILE_DATA_SIZE_BYTES;

        if bgcnt.color_pallete() == 0 {
            Self::FourBit {
                tile: memory.vram.memory[tile_data_start + tile_num * 32..][..32]
                    .try_into()
                    .unwrap(),
                pallete_num: text_bg_screen_entry.pallete_num(),
            }
        } else {
            Self::EightBit {
                tile: memory.vram.memory[tile_data_start + tile_num * 32..][..64]
                    .try_into()
                    .unwrap(),
            }
        }
    }
}

struct BGEntry<'a>(pub &'a u16);

impl<'a> BGEntry<'a> {
    pub fn tile_number(&self) -> usize {
        (self.0 & 0x1FF).into()
    }

    pub fn pallete_num(&self) -> usize {
        ((self.0 >> 12) & 0xF).into()
    }
}
