use crate::utils::bits::Bits;

pub struct Dispcnt(pub u16);

impl Dispcnt {
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

    pub fn bg0_enabled(&self) -> bool {
        self.0.bit_is_set(8)
    }

    pub fn bg1_enabled(&self) -> bool {
        self.0.bit_is_set(9)
    }

    pub fn bg2_enabled(&self) -> bool {
        self.0.bit_is_set(10)
    }

    pub fn bg3_enabled(&self) -> bool {
        self.0.bit_is_set(11)
    }

    pub fn obj_enabled(&self) -> bool {
        self.0.bit_is_set(12)
    }
}

pub struct EnabledBackgrounds {
    enabled_backgrounds: u16,
    count: usize,
}

impl Iterator for EnabledBackgrounds {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        while self.count < 4 {
            let current_count = self.count;
            let current_enabled_backgounds = self.enabled_backgrounds;
            self.enabled_backgrounds >>= 1;
            self.count += 1;
            if current_enabled_backgounds & 0b1 > 0 {
                return Some(current_count);
            }
        }

        None
    }
}

#[cfg(test)]
mod test {
    use super::Dispcnt;

    #[test]
    fn test_gets_enabled_backgrounds() {
        let dispcnt = Dispcnt(0x9802);

        for layer in dispcnt.enabled_backgrounds() {
            dbg!(layer);
        }

        assert_eq!(1, 1);
    }
}
