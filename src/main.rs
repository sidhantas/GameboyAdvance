mod memory;
mod types;
mod arm7tdmi;
mod utils;
use arm7tdmi::decoder::InstructionDecoder;

use crate::memory::{Memory, AccessFlags};
use crate::arm7tdmi::decoder::ARMINSTRUCTION;

fn main() {
    let instruction: ARMINSTRUCTION = 0xEA000018;
    instruction.decode_instruction();
}
