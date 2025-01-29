use std::mem::size_of;

use crate::types::{BYTE, HWORD, WORD};

pub trait Bits {
    fn twos_complement(self) -> Self;
    fn bit_is_set(&self, bit: u8) -> bool;
    fn set_bit(&mut self, bit: u8);
    fn reset_bit(&mut self, bit: u8);
    fn get_bit(self, bit: u8) -> Self;
}

impl Bits for WORD {
    fn bit_is_set(&self, bit: u8) -> bool {
        assert!(bit < 32);
        return self >> bit & 0x01 != 0;
    }

    fn set_bit(&mut self, bit: u8) {
        assert!(bit < 32);
        *self |= 1 << bit;
    }

    fn reset_bit(&mut self, bit: u8) {
        assert!(bit < 32);
        *self &= !(1 << bit);
    }

    fn get_bit(self, bit: u8) -> WORD {
        assert!(bit < 32);
        return (self >> bit & 0x01) as WORD;
    }
    
    fn twos_complement(self) -> WORD {
        return !self + 1
    }
}

impl Bits for HWORD {
    fn bit_is_set(&self, bit: u8) -> bool {
        assert!(bit < size_of::<Self>() as u8);
        return self >> bit & 0x01 != 0;
    }

    fn set_bit(&mut self, bit: u8) {
        assert!(bit < size_of::<Self>() as u8);
        *self |= 1 << bit;
    }

    fn reset_bit(&mut self, bit: u8) {
        assert!(bit < size_of::<Self>() as u8);
        *self &= !(1 << bit);
    }

    fn get_bit(self, bit: u8) -> Self {
        assert!(bit < size_of::<Self>() as u8);
        return (self >> bit & 0x01) as Self;
    }
    
    fn twos_complement(self) -> Self {
        return !self + 1
    }
}

impl Bits for BYTE {
    fn bit_is_set(&self, bit: u8) -> bool {
        assert!(bit < 8);
        return self >> bit & 0x01 != 0;
    }

    fn set_bit(&mut self, bit: u8) {
        assert!(bit < 8);
        *self |= 1 << bit;
    }

    fn reset_bit(&mut self, bit: u8) {
        assert!(bit < 8);
        *self &= !(1 << bit);
    }

    fn get_bit(self, bit: u8) -> BYTE {
        assert!(bit < 8);
        return (self >> bit & 0x01) as BYTE;
    }
    
    fn twos_complement(self) -> BYTE {
        return !self + 1
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
