use crate::types::*;

fn get_condition_bits(instruction: WORD) -> WORD {
    return (instruction & 0xF0000000) >> 28; 
}

pub fn decode_arm_instruction(instruction: WORD) -> String {
    println!("{:#x}", get_condition_bits(0xEA000018));
}

