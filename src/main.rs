use arm7tdmi::decoder::InstructionDecoder;
use types::ARMInstruction;

mod memory;
mod types;
mod arm7tdmi;
mod utils;

fn main() {
    
    let instruction: ARMInstruction = 0xE;
    instruction.decode_instruction();
}
