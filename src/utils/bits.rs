use crate::types::WORD;

pub trait Bits {
    fn and_mask(&self, high_bit: u8, low_bit: u8) -> Self;
} 
impl Bits for WORD {
    fn and_mask(&self, high_bit: u8, low_bit: u8) -> Self {
        self.clone()
    }
}
