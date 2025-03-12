use super::wrappers::tile::Tile;

pub struct OBJPalleteData<'a>(pub &'a [u8; 0x200]);

impl<'a> OBJPalleteData<'a> {
    pub fn get_pixel_from_tile(&self, tile: &Tile, x: usize, y: usize) -> Option<u32> {
        match tile {
            Tile::FourBit { tile, pallete_num } => {
                let palette_index: usize = (tile[y * 4 + x / 2] >> ((x & 0x1) * 4)).into();
                let palette_index = palette_index & 0xF;
                self.get_obj_color(palette_index, *pallete_num, 0)
            },
            Tile::EightBit { tile } => {
                let palette_index: usize = tile[y * 8 + x].into();
                self.get_obj_color(palette_index, 0, 1)
            }
        }
    }

    pub fn get_obj_color(
        &self,
        palette_index: usize,
        pallete_num: usize,
        pallete_mode: usize,
    ) -> Option<u32> {
        if palette_index == 0 {
            return None;
        }
        let index = match pallete_mode {
            0 => pallete_num * 32 + palette_index * 2,
            1 => palette_index * 2,
            _ => panic!(),
        };

        let color = rgb555_to_rgb24(u16::from_le_bytes(self.0[index..][..2].try_into().unwrap()));

        return Some(color);
    }
}

pub struct BGPalleteData<'a>(pub &'a [u8; 0x200]);

impl<'a> BGPalleteData<'a> {
    pub fn get_pixel_from_tile(&self, tile: &Tile, x: usize, y: usize) -> Option<u32> {
        match tile {
            Tile::FourBit { .. } => Some(0x0000FFFF),
            Tile::EightBit { tile } => {
                let palette_index: usize = tile[y * 8 + x].into();
                self.get_bg_color(palette_index, 0, 1, false)
            }
        }
    }

    pub fn get_bg_color(
        &self,
        palette_index: usize,
        pallete_num: usize,
        pallete_mode: usize,
        is_backdrop: bool,
    ) -> Option<u32> {
        if palette_index == 0 && is_backdrop == false {
            return None;
        }
        let index = match pallete_mode {
            0 => pallete_num * 32 + palette_index * 2,
            1 => palette_index * 2,
            _ => panic!(),
        };

        let color = rgb555_to_rgb24(u16::from_le_bytes(self.0[index..][..2].try_into().unwrap()));

        return Some(color);
    }
}


fn rgb555_to_rgb24(rgb555: u16) -> u32 {
    let r5 = (rgb555 >> 10) & 0x1F;
    let g5 = (rgb555 >> 5) & 0x1F;
    let b5 = rgb555 & 0x1F;
    let r8: u32 = (r5 * 8).into();
    let g8: u32 = (g5 * 8).into();
    let b8: u32 = (b5 * 8).into();

    return r8 << 16 | g8 << 8 | b8;
}
