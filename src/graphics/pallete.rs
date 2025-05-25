use super::{ppu_modes::hdraw::RGBComponents, wrappers::tile::Tile};

pub struct OBJPalleteData(pub [u8; 0x200]);

impl OBJPalleteData {
    pub fn get_pixel_from_tile(&self, tile: &Tile, x: usize, y: usize) -> Option<RGBComponents> {
        match tile {
            Tile::FourBit { tile, pallete_num } => {
                let palette_index = tile[y * 4 + x / 2];
                let palette_index = palette_index >> ((x & 0x1) * 4);
                let palette_index: usize = (palette_index & 0xF).into();
                self.get_obj_color(palette_index, *pallete_num, 0)
            }
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
    ) -> Option<RGBComponents> {
        if palette_index == 0 {
            return None;
        }
        let index = match pallete_mode {
            0 => pallete_num * 32 + palette_index * 2,
            1 => palette_index * 2,
            _ => panic!(),
        };

        let color = u16::from_le_bytes(self.0[index..][..2].try_into().unwrap());

        return Some(color.into());
    }
}

pub struct BGPalleteData<'a>(pub &'a [u8; 0x200]);

impl<'a> BGPalleteData<'a> {
    pub fn get_pixel_from_tile(&self, tile: &Tile, x: usize, y: usize) -> Option<RGBComponents> {
        match tile {
            Tile::FourBit { .. } => Some(RGBComponents {
                r: 0,
                g: 0,
                b: 0xFF,
            }),
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
    ) -> Option<RGBComponents> {
        if palette_index == 0 && is_backdrop == false {
            return None;
        }
        let index = match pallete_mode {
            0 => pallete_num * 32 + palette_index * 2,
            1 => palette_index * 2,
            _ => panic!(),
        };

        let color = u16::from_le_bytes(self.0[index..][..2].try_into().unwrap());

        return Some(color.into());
    }
}

pub fn rgb555_to_rgb24(rgb555: RGBComponents) -> u32 {
    let r8: u32 = (rgb555.r * 8).into();
    let g8: u32 = (rgb555.g * 8).into();
    let b8: u32 = (rgb555.b * 8).into();

    return r8 << 16 | g8 << 8 | b8;
}
