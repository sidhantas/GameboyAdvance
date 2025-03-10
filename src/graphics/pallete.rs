use super::{oam::Oam, tile::Tile};

pub struct PalleteData<'a>(pub &'a [u8; 0x400]);

impl<'a> PalleteData<'a> {
    pub fn get_pixel_from_tile(&self, oam: &Oam, tile: &Tile, x: usize, y: usize) -> Option<u32> {
        match tile {
            Tile::FourBit(_) => Some(0xFFFF0000),
            Tile::EightBit(tile) => {
                let palette_index: usize = tile[y * 8 + x].into();
                self.get_obj_color(palette_index, oam.pallete_number(), oam.color_pallete())
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

        let color = Self::rgb555_to_rgb24(u16::from_le_bytes(
            self.0[0x200 + index..][..2].try_into().unwrap(),
        ));

        return Some(color);
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
}
