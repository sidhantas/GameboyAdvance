use std::cmp::min;

use crate::utils::bits::Bits;

pub struct BldCnt<'a>(pub &'a u16);
pub struct BldAlpha(pub u16);
pub struct BldY<'a>(pub &'a u16);

pub enum BlendMode {
    BldOff,
    BldAlpha,
    BldWhite,
    BldBlack,
}

impl BldAlpha {
    pub fn eva(&self) -> u16 {
        min(self.0 & 0x1F, 16)
    }

    pub fn evb(&self) -> u16 {
        min((self.0 >> 8) & 0x1F, 16)
    }
}

impl<'a> BldCnt<'a> {
    pub fn bld_mode(&self) -> BlendMode {
        match (self.0 >> 5) & 0x3 {
            0 => BlendMode::BldOff,
            1 => BlendMode::BldAlpha,
            2 => BlendMode::BldWhite,
            3 => BlendMode::BldBlack,
            _ => unreachable!(),
        }
    }
    pub fn get_target_layers_a(&self) -> u16 {
        self.0 & 0x3F
    }

    pub fn get_target_layers_b(&self) -> u16 {
        (self.0 >> 7) & 0x3F
    }

    pub fn target_a_bg0_enabled(&self) -> bool {
        self.0.bit_is_set(0)
    }
    pub fn target_a_bg1_enabled(&self) -> bool {
        self.0.bit_is_set(1)
    }
    pub fn target_a_bg2_enabled(&self) -> bool {
        self.0.bit_is_set(2)
    }
    pub fn target_a_bg3_enabled(&self) -> bool {
        self.0.bit_is_set(3)
    }
    pub fn target_a_obj_enabled(&self) -> bool {
        self.0.bit_is_set(4)
    }
    pub fn target_a_bd_enabled(&self) -> bool {
        self.0.bit_is_set(5)
    }
    pub fn target_b_bg0_enabled(&self) -> bool {
        self.0.bit_is_set(8)
    }
    pub fn target_b_bg1_enabled(&self) -> bool {
        self.0.bit_is_set(9)
    }
    pub fn target_b_bg2_enabled(&self) -> bool {
        self.0.bit_is_set(10)
    }
    pub fn target_b_bg3_enabled(&self) -> bool {
        self.0.bit_is_set(11)
    }
    pub fn target_b_obj_enabled(&self) -> bool {
        self.0.bit_is_set(12)
    }
    pub fn target_b_bd_enabled(&self) -> bool {
        self.0.bit_is_set(13)
    }
}

impl<'a> BldY<'a> {
    pub fn evy_coefficient(&self) -> u16 {
        min(self.0 & 0x1F, 16)
    }
}
