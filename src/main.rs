mod memory;
mod types;
mod arm7tdmi;
mod utils;
use crate::memory::{Memory, AccessFlags};
use crate::arm7tdmi::decoder::decode_arm_instruction;

fn main() {
    dbg!(decode_arm_instruction(0xEA000018));
}
