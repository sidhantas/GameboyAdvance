use std::{fmt::Display, ops::Deref};

use crate::{
    arm7tdmi::{
        arm::alu::{
            ArithmeticInstruction, DataProcessingInstruction, LogicalInstruction, Shift, ShiftType,
        },
        instruction_table::{DecodeARMInstructionToString, Operand},
    },
    types::REGISTER,
};

impl DecodeARMInstructionToString for DataProcessingInstruction {
    fn instruction_to_string(&self, condition_code: &str) -> String {
        match self {
            DataProcessingInstruction::Arithmetic(instruction, rd, rn, op2, shift, set_flags) => {
                format!(
                    "{instruction}{condition_code}{} {}{}{op2}{shift}",
                    if *set_flags { "s" } else { "" },
                    print_option_register(rd),
                    print_register(rn),
                )
            }
            DataProcessingInstruction::Logical(instruction, rd, rn, op2, shift, set_flags) => {
                format!(
                    "{instruction}{condition_code}{} {}{}{op2}{shift}",
                    if *set_flags { "s" } else { "" },
                    print_option_register(rd),
                    print_register(rn),
                )
            }
            DataProcessingInstruction::MSR(_, _, _, _, _) => todo!(),
            DataProcessingInstruction::MRS(_, _) => todo!(),
        }
    }
}
impl Display for ArithmeticInstruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let op = match self {
            ArithmeticInstruction::Sub => "sub",
            ArithmeticInstruction::Rsb => "rsb",
            ArithmeticInstruction::Add => "add",
            ArithmeticInstruction::Adc => "adc",
            ArithmeticInstruction::Sbc => "sbc",
            ArithmeticInstruction::Rsc => "rsc",
            ArithmeticInstruction::Cmp => "cmp",
            ArithmeticInstruction::Cmn => "cmn",
        };
        write!(f, "{}", op)
    }
}
impl Display for LogicalInstruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let op = match self {
            LogicalInstruction::And => "and",
            LogicalInstruction::Eor => "eor",
            LogicalInstruction::Tst => "tst",
            LogicalInstruction::Teq => "teq",
            LogicalInstruction::Orr => "orr",
            LogicalInstruction::Mov => "mov",
            LogicalInstruction::Bic => "bic",
            LogicalInstruction::Mvn => "mvn",
        };
        write!(f, "{}", op)
    }
}
impl Display for Shift {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if matches!(self, Shift(ShiftType::LSL, Operand::Immeidate(0))) {
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

fn print_option_register(register: &Option<REGISTER>) -> String {
    (*register).map_or("".into(), |reg| print_register(&reg))
}

fn print_register(register: &REGISTER) -> String {
    let register = *register;
    match register {
        11 => "fp".into(),
        12 => "ip".into(),
        13 => "sp".into(),
        14 => "lr".into(),
        15 => "pc".into(),
        _ => format!("r{register}"),
    }
}

#[cfg(test)]
mod test_printing {
    use crate::arm7tdmi::{
        arm::alu::{DataProcessingInstruction, LogicalInstruction, Shift, ShiftType},
        instruction_table::{DecodeARMInstructionToString, Operand},
    };

    #[test]
    fn test_printing_an_instruction() {
        let instruction = DataProcessingInstruction::Logical(
            LogicalInstruction::And,
            Some(1),
            2,
            Operand::Register(3),
            Shift(ShiftType::LSL, Operand::Immeidate(100)),
            true,
        );

        println!("{}", instruction.instruction_to_string(""))
    }
}
