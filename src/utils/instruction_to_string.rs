use std::fmt::Display;

use crate::{
    arm7tdmi::{
        arm::alu::{Shift, ShiftType},
        instruction_table::{DecodeThumbInstructionToString, Operand},
        thumb::alu::{
            ThumbALUInstruction, ThumbArithmeticInstruction, ThumbFullAdder,
            ThumbFullAdderOperations, ThumbLogicalInstruction, ThumbShiftInstruction,
        },
    },
    types::REGISTER,
};

impl Display for Shift {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if matches!(self, Shift(ShiftType::LSL, Operand::Immediate(0))) {
            return Ok(());
        }
        write!(f, ", {} {}", self.0, self.1)
    }
}

impl Display for ShiftType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ShiftType::LSL => write!(f, "lsl"),
            ShiftType::LSR => write!(f, "lsr"),
            ShiftType::ASR => write!(f, "asr"),
            ShiftType::ROR => write!(f, "ror"),
            ShiftType::RRX => write!(f, "rrx"),
        }
    }
}

impl Display for Operand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Operand::Register(reg) => write!(f, "{}", print_register(reg)),
            Operand::Immediate(imm) => write!(f, "#{imm}"),
        }
    }
}


fn print_option_register(register: &Option<REGISTER>) -> String {
    (*register).map_or("".into(), |reg| {
        let mut print = print_register(&reg);
        print.push(' ');
        print
    })
}

pub fn print_register(register: &REGISTER) -> String {
    let register = *register;
    match register {
        13 => "sp".into(),
        14 => "lr".into(),
        15 => "pc".into(),
        _ => format!("r{register}"),
    }
}

pub fn print_shifted_operand(operand2: &Operand, shift: &Shift) -> String {
    match operand2 {
        Operand::Register(_) => {
            if let Shift(ShiftType::LSL, Operand::Immediate(0)) = shift {
                return format!("{operand2}");
            };
            return format!("{operand2}{shift}");
        }
        Operand::Immediate(imm) => {
            let Shift(ShiftType::ROR, Operand::Immediate(shift_amount)) = shift else {
                panic!("Invalid Shift")
            };
            let imm = imm.rotate_right(*shift_amount);
            return format!("#{imm}");
        }
    }
}
