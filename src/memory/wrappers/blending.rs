use std::cmp::min;

pub struct BldCnt<'a>(pub &'a u16);
pub struct BldAlpha<'a>(pub &'a u16);
pub struct BldY<'a>(pub &'a u16);

pub enum BlendMode {
    BldOff,
    BldAlpha,
    BldWhite,
    BldBlack,
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
}

impl<'a> BldY<'a> {
    pub fn evy_coefficient(&self) -> u16 {
        min(self.0 & 0x1F, 16)
    }
}
