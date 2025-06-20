use std::{
    mem::size_of,
    ops::{BitAnd, Shl, Shr},
};

use num_traits::{Bounded, PrimInt, Unsigned};

use crate::types::{BYTE, HWORD, WORD};

//pub trait Bits2
//where
//    Self: Shr<u8, Output = Self>,
//    Self: BitAnd<Self, Output = Self>,
//    Self: BitOrAssign<Self>,
//    Self: PartialEq<Self>,
//    Self: Sized,
//    Self: PrimInt,
//    Self: Unsigned,
//    Self: Copy
//{
//    fn twos_complement(self) -> Self {
//        !self + Self::one()
//    }
//    fn bit_is_set(&self, bit: u8) -> bool {
//        assert!(bit < (size_of::<Self>() * 8) as u8);
//        let shift = self.shr(bit);
//        let and = shift.bitand(Self::one());
//        and != Self::zero()
//    }
//    fn set_bit(&mut self, bit: u8) {
//        assert!(bit < (size_of::<Self>() * 8) as u8);
//        *self |= Self::one() << bit as usize;
//    }
//
//    fn get_bit(&self, bit: u8) -> Self {
//        assert!(bit < (size_of::<Self>() * 8) as u8);
//        let shift = self.shr(bit);
//        let and = shift.bitand(Self::one());
//        return and;
//    }
//}
//
//impl Bits2 for HWORD {}

pub trait Bits: PrimInt + Shr<u8> + BitAnd + Eq {
    fn twos_complement(self) -> Self;
    fn bit_is_set(&self, bit: u8) -> bool;
    fn set_bit(&mut self, bit: u8);
    fn reset_bit(&mut self, bit: u8);
    fn get_bit(self, bit: u8) -> Self;
}
impl Bits for u64 {
    fn bit_is_set(&self, bit: u8) -> bool {
        return self >> bit & 0x01 != 0;
    }

    fn set_bit(&mut self, bit: u8) {
        *self |= 1 << bit;
    }

    fn reset_bit(&mut self, bit: u8) {
        *self &= !(1 << bit);
    }

    fn get_bit(self, bit: u8) -> u64 {
        return (self >> bit & 0x01) as u64;
    }

    fn twos_complement(self) -> u64 {
        return !self + 1;
    }
}

impl Bits for WORD {
    fn bit_is_set(&self, bit: u8) -> bool {
        return self >> bit & 0x01 != 0;
    }

    fn set_bit(&mut self, bit: u8) {
        *self |= 1 << bit;
    }

    fn reset_bit(&mut self, bit: u8) {
        *self &= !(1 << bit);
    }

    fn get_bit(self, bit: u8) -> WORD {
        return (self >> bit & 0x01) as WORD;
    }

    fn twos_complement(self) -> WORD {
        return !self + 1;
    }
}

impl Bits for HWORD {
    fn bit_is_set(&self, bit: u8) -> bool {
        return self >> bit & 0x01 != 0;
    }

    fn set_bit(&mut self, bit: u8) {
        *self |= 1 << bit;
    }

    fn reset_bit(&mut self, bit: u8) {
        *self &= !(1 << bit);
    }

    fn get_bit(self, bit: u8) -> Self {
        return (self >> bit & 0x01) as Self;
    }

    fn twos_complement(self) -> Self {
        return !self + 1;
    }
}

impl Bits for BYTE {
    fn bit_is_set(&self, bit: u8) -> bool {
        return self >> bit & 0x01 != 0;
    }

    fn set_bit(&mut self, bit: u8) {
        *self |= 1 << bit;
    }

    fn reset_bit(&mut self, bit: u8) {
        *self &= !(1 << bit);
    }

    fn get_bit(self, bit: u8) -> BYTE {
        return (self >> bit & 0x01) as BYTE;
    }

    fn twos_complement(self) -> BYTE {
        return !self + 1;
    }
}

pub fn sign_extend<T>(word: T, sign_bit: u8) -> T
where
    T: Bounded + Shl<u8, Output = T> + Bits,
{
    if word.bit_is_set(sign_bit) {
        let mut mask = T::max_value();
        mask = mask.shl(sign_bit + 1);
        return word | mask;
    }
    word
}

pub fn fixed88_point_to_floating_point(mut fixed88: u16) -> f32 {
    let mut float: u32 = 0;

    if fixed88 == 0 {
        return 0.;
    }

    if fixed88.bit_is_set(15) {
        float.set_bit(31);
    }

    let mut exponent: i32 = 6;
    for _ in 0..15 {
        if fixed88.bit_is_set(14) {
            break;
        }
        exponent -= 1;
        fixed88 <<= 1;
    }
    float |= ((exponent + 127) as u32) << 23;

    fixed88 <<= 1;
    float |= ((fixed88 & 0x7FFF) as u32) << 8;

    f32::from_bits(float)
}

#[cfg(test)]
mod fixed88_tests {
    use rstest::rstest;

    use super::fixed88_point_to_floating_point;

    #[rstest]
    #[case(0b101100, 0.171875)]
    #[case(0b1000000000101100, -0.171875)]
    #[case(0b00, 0.)]
    #[case(0xb8, 0.71875)]
    #[case(0x0100, 1.)]
    #[case(0x0040, 0.25)]
    pub fn converts_fixed_point_to_f32(#[case] fixed88: u16, #[case] expected_output: f32) {
        let result = dbg!(fixed88_point_to_floating_point(fixed88));
        println!("0b{:0>32b}", result.to_bits());
        assert_eq!(result, expected_output);
    }
}


