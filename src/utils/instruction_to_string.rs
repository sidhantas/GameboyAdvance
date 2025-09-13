use std::{fmt::Display, ops::Deref};

use num_traits::PrimInt;

use crate::{
    arm7tdmi::{
        arm::alu::{
            ArithmeticInstruction, DataProcessingInstruction, LogicalInstruction, PSRRegister,
            Shift, ShiftType,
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
                    "{instruction}{condition_code}{} {}{} {}",
                    if *set_flags { "s" } else { "" },
                    print_option_register(rd),
                    print_register(rn),
                    print_shifted_operand(op2, shift)
                )
            }
            DataProcessingInstruction::Logical(instruction, rd, rn, op2, shift, set_flags) => {
                format!(
                    "{instruction}{condition_code}{} {}{}{}",
                    if *set_flags { "s" } else { "" },
                    print_option_register(rd),
                    print_option_register(rn),
                    print_shifted_operand(op2, shift)
                )
            }
            DataProcessingInstruction::MSR(psr_register, write_flags, write_control, op2, shift) => {
                let psr_register = match psr_register {
                    PSRRegister::SPSR => "spsr",
                    PSRRegister::CPSR => "cpsr",
                };
                let flags = |write_flags: bool, write_control: bool| {
                    if !write_control && !write_flags {
                        return "".into();
                    }
                    format!(
                        "_{}{}",
                        if write_flags { "f" } else { "" },
                        if write_control { "c" } else { "" }
                    )
                };

                format!(
                    "msr{condition_code} {}{} {}",
                    psr_register,
                    flags(*write_flags, *write_control),
                    print_shifted_operand(op2, &Shift(ShiftType::ROR, Operand::Immediate(*shift)))
                )
            }
            DataProcessingInstruction::MRS(rd, psr_register) => {
                let psr_register = match psr_register {
                    PSRRegister::SPSR => "spsr",
                    PSRRegister::CPSR => "cpsr",
                };
                format!("mrs{condition_code} {} {}", print_register(rd), psr_register)
            },
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

fn print_shifted_operand(operand2: &Operand, shift: &Shift) -> String {
    match operand2 {
        Operand::Register(_) => {
            if let Shift(ShiftType::LSL, Operand::Immediate(0)) = shift{
                return format!("{operand2}");
            };
            return format!("{operand2}{shift}")
        },
        Operand::Immediate(imm) => {
            let Shift(ShiftType::ROR, Operand::Immediate(shift_amount)) = shift else {
                panic!("Invalid Shift")
            };
            let imm = imm.rotate_right(*shift_amount);
            return format!("#{imm}");
        },
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
            Some(2),
            Operand::Register(3),
            Shift(ShiftType::LSL, Operand::Immediate(100)),
            true,
        );

        println!("{}", instruction.instruction_to_string(""))
    }
}
