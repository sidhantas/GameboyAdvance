use crate::utils::bits::Bits;

pub(crate) const MAP_SIZE_BYTES: usize = 0x800;
pub(crate) const TILE_DATA_SIZE_BYTES: usize = 0x4000;

#[derive(Clone, Copy)]
pub(crate) struct BGCnt(pub(crate) u16);

impl BGCnt {
    pub(crate) fn priority(&self) -> u16 {
        self.0 & 0b11
    }

    pub(crate) fn tile_data_base(&self) -> usize {
        ((self.0 >> 2) & 0b11).into()
    }

    pub(crate) fn mosaic(&self) -> bool {
        self.0.bit_is_set(6)
    }

    pub(crate) fn color_pallete(&self) -> usize {
        self.0.get_bit(7).into()
    }

    pub(crate) fn map_data_base(&self) -> usize {
        ((self.0 >> 14) & 0b11) as usize
    }
}
