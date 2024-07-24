use crate::types::WORD;

pub trait Bits {
    fn bit_is_set(&self, bit: u8) -> bool;
} 

impl Bits for WORD {
    fn bit_is_set(&self, bit: u8) -> bool {
        return self >> bit & 0x01 != 0;
    }
}
