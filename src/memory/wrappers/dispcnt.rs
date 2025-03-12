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

    pub fn enabled_backgrounds(&self) -> EnabledBackgrounds {
        EnabledBackgrounds {
            enabled_backgrounds: (self.0 >> 8) & 0xF,
            count: 0,
        }
    }
}

pub struct EnabledBackgrounds {
    enabled_backgrounds: u16,
    count: usize,
}

impl Iterator for EnabledBackgrounds {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        while self.enabled_backgrounds & 0b1 == 0 {
            if self.count >= 4 {
                return None;
            }
            self.enabled_backgrounds >>= 1;
            self.count += 1;
        }

        Some(self.count)
    }
}
