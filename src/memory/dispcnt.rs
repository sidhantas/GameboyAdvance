use crate::utils::bits::Bits;

pub struct Dispcnt<'a>(pub &'a u16);


impl<'a> Dispcnt<'a> {
    pub fn get_bg_mode(&self) -> u16 {
        self.0 & 0b111
    }

    pub fn display_frame_select(&self) -> bool {
        self.0.bit_is_set(4)
    }

    pub fn h_blank_interval_free(&self) -> bool {
        self.0.bit_is_set(5)
    }

    pub fn one_dimensional_character_mapping(&self) -> bool {
        self.0.bit_is_set(6)
    }

    pub fn forced_blank(&self) -> bool {
        self.0.bit_is_set(7)
    }
}
