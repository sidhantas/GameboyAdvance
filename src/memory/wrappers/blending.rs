use std::cmp::min;

use crate::utils::bits::Bits;

pub(crate) struct BldCnt<'a>(pub(crate) &'a u16);
pub(crate) struct BldAlpha(pub(crate) u16);
pub(crate) struct BldY<'a>(pub(crate) &'a u16);

pub(crate) enum BlendMode {
    BldOff,
    BldAlpha,
    BldWhite,
    BldBlack,
}

impl BldAlpha {
    pub(crate) fn eva(&self) -> u16 {
        min(self.0 & 0x1F, 16)
    }

    pub(crate) fn evb(&self) -> u16 {
        min((self.0 >> 8) & 0x1F, 16)
    }
}

impl<'a> BldCnt<'a> {
    pub(crate) fn bld_mode(&self) -> BlendMode {
        match (self.0 >> 5) & 0x3 {
            0 => BlendMode::BldOff,
            1 => BlendMode::BldAlpha,
            2 => BlendMode::BldWhite,
            3 => BlendMode::BldBlack,
            _ => unreachable!(),
        }
    }
    pub(crate) fn target_a_bg0_enabled(&self) -> bool {
        self.0.bit_is_set(0)
    }
    pub(crate) fn target_a_bg1_enabled(&self) -> bool {
        self.0.bit_is_set(1)
    }
    pub(crate) fn target_a_bg2_enabled(&self) -> bool {
        self.0.bit_is_set(2)
    }
    pub(crate) fn target_a_bg3_enabled(&self) -> bool {
        self.0.bit_is_set(3)
    }
    pub(crate) fn target_a_obj_enabled(&self) -> bool {
        self.0.bit_is_set(4)
    }
    pub(crate) fn target_a_bd_enabled(&self) -> bool {
        self.0.bit_is_set(5)
    }
    pub(crate) fn target_b_bg0_enabled(&self) -> bool {
        self.0.bit_is_set(8)
    }
    pub(crate) fn target_b_bg1_enabled(&self) -> bool {
        self.0.bit_is_set(9)
    }
    pub(crate) fn target_b_bg2_enabled(&self) -> bool {
        self.0.bit_is_set(10)
    }
    pub(crate) fn target_b_bg3_enabled(&self) -> bool {
        self.0.bit_is_set(11)
    }
    pub(crate) fn target_b_obj_enabled(&self) -> bool {
        self.0.bit_is_set(12)
    }
    pub(crate) fn target_b_bd_enabled(&self) -> bool {
        self.0.bit_is_set(13)
    }
}

impl<'a> BldY<'a> {
    pub(crate) fn evy_coefficient(&self) -> u16 {
        min(self.0 & 0x1F, 16)
    }
}
