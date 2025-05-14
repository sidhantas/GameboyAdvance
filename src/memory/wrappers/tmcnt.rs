use crate::utils::bits::Bits;

pub struct TMCntH(pub u16);

impl TMCntH {
    pub fn prescaler_value(&self) -> u32 {
        match self.0 & 0b11 {
            0 => 1,
            1 => 64,
            2 => 256,
            3 => 1024,
            _ => unreachable!(),
        }
    }

    pub fn count_up_timing(&self) -> bool {
        self.0.bit_is_set(2)
    }

    pub fn timer_irq_enable(&self) -> bool {
        self.0.bit_is_set(6)
    }

    pub fn timer_enabled(&self) -> bool {
        self.0.bit_is_set(7)
    }
}
