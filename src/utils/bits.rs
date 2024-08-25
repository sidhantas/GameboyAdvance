use crate::types::WORD;

pub trait Bits {
    fn bit_is_set(&self, bit: u8) -> bool;
} 

impl Bits for WORD {
    fn bit_is_set(&self, bit: u8) -> bool {
        assert!(bit < 32);
        return self >> bit & 0x01 != 0;
    }
}


pub fn sign_extend(word: WORD, sign_bit: u8) -> u32 {
    if word.bit_is_set(sign_bit) {
        let mut mask: u32 = u32::MAX;
        mask = mask << (sign_bit + 1);
        return word | mask;
    }
    word
}
