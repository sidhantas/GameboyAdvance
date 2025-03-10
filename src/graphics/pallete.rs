
pub struct PalleteData<'a>(pub &'a [u8; 0x400]);


impl<'a> PalleteData<'a> {
    pub fn get_obj_color(&self, color: usize, pallete_num: usize, pallete_mode: usize) -> Option<u32> {
        if color == 0 {
            return None;
        }
        let index = match pallete_mode {
            0 => pallete_num * 32 + color * 2,
            1 => color * 2,
            _ => panic!(),
        };

        let color = Self::scale_up_rgb555_to_rgb24(u16::from_le_bytes(
            self.0[0x200 + index..][..2].try_into().unwrap(),
        ));

        return Some(color);
    }

    fn scale_up_rgb555_to_rgb24(rgb555: u16) -> u32 {
        let r5 = (rgb555 >> 10) & 0x1F;
        let g5 = (rgb555 >> 5) & 0x1F;
        let b5 = rgb555 & 0x1F;
        let r8: u32 = (r5 * 8).into();
        let g8: u32 = (g5 * 8).into();
        let b8: u32 = (b5 * 8).into();

        return r8 << 16 | g8 << 8 | b8;
    }
}
