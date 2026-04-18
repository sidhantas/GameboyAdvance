use crate::utils::bits::Bits;

#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct TMCntH(pub(crate) u16);

impl TMCntH {
    pub(crate) fn prescaler_value(&self) -> u32 {
        match self.0 & 0b11 {
            0 => 1,
            1 => 64,
            2 => 256,
            3 => 1024,
            _ => unreachable!(),
        }
    }

    pub(crate) fn count_up_timing(&self) -> bool {
        self.0.bit_is_set(2)
    }

    pub(crate) fn timer_irq_enable(&self) -> bool {
        self.0.bit_is_set(6)
    }

    pub(crate) fn timer_enabled(&self) -> bool {
        self.0.bit_is_set(7)
    }
}
