use std::fmt::Display;

use crate::{
    arm7tdmi::{
        arm::alu::ArithmeticInstruction,
        cpu::{FlagsRegister, InstructionMode, CPU, PC_REGISTER},
        instruction_table::{DecodeThumbInstructionToString, Execute, Operand},
        thumb,
    },
    memory::memory::GBAMemory,
    types::{CYCLES, REGISTER},
    utils::{
        bits::{sign_extend, Bits},
        instruction_to_string::print_register,
    },
};

pub struct ThumbFullAdder(pub u32);

impl ThumbFullAdder {
    fn full_adder_operation(&self) -> (ThumbFullAdderOperations, Operand) {
        let opcode = (self.0 & 0x0600) >> 9;
        let operand2 = (self.0 & 0x01C0) >> 6;
        match opcode {
            0 => (ThumbFullAdderOperations::Add, Operand::Register(operand2)),
            1 => (ThumbFullAdderOperations::Sub, Operand::Register(operand2)),
            2 => (ThumbFullAdderOperations::Add, Operand::Immediate(operand2)),
            3 => (ThumbFullAdderOperations::Sub, Operand::Immediate(operand2)),
            _ => unreachable!(),
        }
    }

    fn rd(&self) -> REGISTER {
        self.0 & 0x0007
    }
    fn rs(&self) -> REGISTER {
        (self.0 & 0x0038) >> 3
    }
}

pub enum ThumbFullAdderOperations {
    Add,
    Sub,
}

impl Execute for ThumbFullAdder {
    fn execute(self, cpu: &mut CPU, _memory: &mut GBAMemory) -> CYCLES {
        let rs_val = cpu.get_register(self.rs());
        let (operation, op2) = self.full_adder_operation();
        let op2 = match op2 {
            Operand::Register(reg) => cpu.get_register(reg),
            Operand::Immediate(imm) => imm,
        };

        let result = match operation {
            ThumbFullAdderOperations::Add => {
                let result = rs_val + op2;
                cpu.set_arithmetic_flags(result, rs_val, op2, 0, true);
                result
            }
            ThumbFullAdderOperations::Sub => {
                let result = rs_val - op2;
                cpu.set_arithmetic_flags(result, rs_val, !op2, 1, true);
                result
            }
        };

        cpu.set_register(self.rd(), result);

        0
    }
}

impl DecodeThumbInstructionToString for ThumbFullAdder {
    fn instruction_to_string(&self) -> String {
        let (operation, op2) = self.full_adder_operation();
        format!("{}s {}, {}, {}", operation, self.rd(), self.rs(), op2)
    }
}

impl Display for ThumbFullAdderOperations {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ThumbFullAdderOperations::Add => write!(f, "add"),
            ThumbFullAdderOperations::Sub => write!(f, "sub"),
        }
    }
}

pub enum ThumbALUInstruction {
    Logical(ThumbLogicalInstruction),
    Arithmetic(ThumbArithmeticInstruction),
    Shift(ThumbShiftInstruction),
}

pub struct ThumbALUOperation(pub u32);

impl ThumbALUOperation {
    fn rd(&self) -> REGISTER {
        self.0 & 0x0007
    }

    fn rs(&self) -> REGISTER {
        (self.0 & 0x0038) >> 3
    }

    fn opcode(&self) -> ThumbALUInstruction {
        match (self.0 & 0x03C0) >> 6 {
            0x0 => ThumbALUInstruction::Logical(ThumbLogicalInstruction::And),
            0x1 => ThumbALUInstruction::Logical(ThumbLogicalInstruction::Eor),
            0x2 => ThumbALUInstruction::Shift(ThumbShiftInstruction::Lsl),
            0x3 => ThumbALUInstruction::Shift(ThumbShiftInstruction::Lsr),
            0x4 => ThumbALUInstruction::Shift(ThumbShiftInstruction::Asr),
            0x5 => ThumbALUInstruction::Arithmetic(ThumbArithmeticInstruction::Adc),
            0x6 => ThumbALUInstruction::Arithmetic(ThumbArithmeticInstruction::Sbc),
            0x7 => ThumbALUInstruction::Shift(ThumbShiftInstruction::Ror),
            0x8 => ThumbALUInstruction::Logical(ThumbLogicalInstruction::Tst),
            0x9 => ThumbALUInstruction::Arithmetic(ThumbArithmeticInstruction::Neg),
            0xa => ThumbALUInstruction::Arithmetic(ThumbArithmeticInstruction::Cmp),
            0xb => ThumbALUInstruction::Arithmetic(ThumbArithmeticInstruction::Cmn),
            0xc => ThumbALUInstruction::Logical(ThumbLogicalInstruction::Orr),
            0xd => ThumbALUInstruction::Arithmetic(ThumbArithmeticInstruction::Mul),
            0xe => ThumbALUInstruction::Logical(ThumbLogicalInstruction::Bic),
            0xf => ThumbALUInstruction::Logical(ThumbLogicalInstruction::Mvn),
            _ => unreachable!(),
        }
    }
}

pub enum ThumbArithmeticInstruction {
    Adc,
    Sbc,
    Neg,
    Cmp,
    Cmn,
    Mul,
}

pub enum ThumbLogicalInstruction {
    And,
    Eor,
    Tst,
    Orr,
    Bic,
    Mvn,
}

pub enum ThumbShiftInstruction {
    Lsl,
    Lsr,
    Asr,
    Ror,
}

impl Execute for ThumbALUOperation {
    fn execute(self, cpu: &mut CPU, _memory: &mut GBAMemory) -> CYCLES {
        match self.opcode() {
            ThumbALUInstruction::Arithmetic(thumb_arithmetic_instruction) => cpu
                .execute_thumb_arithmetic_instruction(
                    thumb_arithmetic_instruction,
                    self.rd(),
                    self.rs(),
                ),
            ThumbALUInstruction::Logical(thumb_logical_instruction) => cpu
                .execute_thumb_logical_instruction(thumb_logical_instruction, self.rd(), self.rs()),
            ThumbALUInstruction::Shift(thumb_shift_instruction) => {
                cpu.execute_thumb_shift_instruction(thumb_shift_instruction, self.rd(), self.rs())
            }
        }
    }
}

impl DecodeThumbInstructionToString for ThumbALUOperation {
    fn instruction_to_string(&self) -> String {
        let rd = self.rd();
        let rs = self.rs();

        let opcode = match self.opcode() {
            ThumbALUInstruction::Logical(thumb_logical_instruction) => {
                match thumb_logical_instruction {
                    ThumbLogicalInstruction::And => "ands",
                    ThumbLogicalInstruction::Eor => "eors",
                    ThumbLogicalInstruction::Tst => "tst",
                    ThumbLogicalInstruction::Orr => "orrs",
                    ThumbLogicalInstruction::Bic => "bics",
                    ThumbLogicalInstruction::Mvn => "mvns",
                }
            }
            ThumbALUInstruction::Arithmetic(thumb_arithmetic_instruction) => {
                match thumb_arithmetic_instruction {
                    ThumbArithmeticInstruction::Adc => "adcs",
                    ThumbArithmeticInstruction::Sbc => "sbcs",
                    ThumbArithmeticInstruction::Neg => "negs",
                    ThumbArithmeticInstruction::Cmp => "cmp",
                    ThumbArithmeticInstruction::Cmn => "cmn",
                    ThumbArithmeticInstruction::Mul => "muls",
                }
            }
            ThumbALUInstruction::Shift(thumb_shift_instruction) => match thumb_shift_instruction {
                ThumbShiftInstruction::Lsl => "lsls",
                ThumbShiftInstruction::Lsr => "lsrs",
                ThumbShiftInstruction::Asr => "asrs",
                ThumbShiftInstruction::Ror => "rors",
            },
        };

        format!("{opcode} {rd}, {rs}")
    }
}

impl CPU {
    fn execute_thumb_arithmetic_instruction(
        &mut self,
        instruction: ThumbArithmeticInstruction,
        rd: REGISTER,
        rs: REGISTER,
    ) -> CYCLES {
        let rs_val = self.get_register(rs);
        let rd_val = self.get_register(rd);
        let mut cycles = 0;
        match instruction {
            ThumbArithmeticInstruction::Adc => {
                let carry = self.get_flag(FlagsRegister::C);
                let result = rd_val + rs_val + carry;
                self.set_arithmetic_flags(result, rd_val, rs_val, carry, true);
                self.set_register(rd, result);
            }
            ThumbArithmeticInstruction::Sbc => {
                let carry = self.get_flag(FlagsRegister::C);
                let result = rd_val - rs_val + carry - 1;
                self.set_arithmetic_flags(result, rd_val, !rs_val, carry, true);
                self.set_register(rd, result);
            }
            ThumbArithmeticInstruction::Neg => {
                let result = 0 - rs_val;
                self.set_arithmetic_flags(result, 0, !rs_val, 1, true);
                self.set_register(rd, result);
            }
            ThumbArithmeticInstruction::Cmp => {
                let result = rd_val - rs_val;
                self.set_arithmetic_flags(result, rd_val, !rs_val, 1, true);
            }
            ThumbArithmeticInstruction::Cmn => {
                let result = rd_val + rs_val;
                self.set_arithmetic_flags(result, rd_val, rs_val, 0, true);
            }
            ThumbArithmeticInstruction::Mul => {
                let multiplier = self.get_register(rd);
                cycles += if multiplier & 0xFFFF_FF00 == 0
                    || multiplier & 0xFFFF_FF00 == 0xFFFF_FF00
                {
                    1
                } else if multiplier & 0xFFFF_0000 == 0 || multiplier & 0xFFFF_0000 == 0xFFFF_0000 {
                    2
                } else if multiplier & 0xFF00_0000 == 0 || multiplier & 0xFF00_0000 == 0xFFFF_0000 {
                    3
                } else {
                    4
                };
                self.thumb_mul(rd, rd_val, rs_val, true);
            }
        };

        cycles
    }

    fn execute_thumb_logical_instruction(
        &mut self,
        instruction: ThumbLogicalInstruction,
        rd: REGISTER,
        rs: REGISTER,
    ) -> CYCLES {
        let rs_val = self.get_register(rs);
        let rd_val = self.get_register(rd);

        let result = match instruction {
            ThumbLogicalInstruction::And => {
                let result = rd_val & rs_val;
                self.set_register(rd, result);
                result
            }
            ThumbLogicalInstruction::Eor => {
                let result = rd_val ^ rs_val;
                self.set_register(rd, result);
                result
            }
            ThumbLogicalInstruction::Tst => rd_val & rs_val,
            ThumbLogicalInstruction::Orr => {
                let result = rd_val | rs_val;
                self.set_register(rd, result);
                result
            }
            ThumbLogicalInstruction::Bic => {
                let result = rd_val & !rs_val;
                self.set_register(rd, result);
                result
            }
            ThumbLogicalInstruction::Mvn => {
                let result = !rs_val;
                self.set_register(rd, result);
                result
            }
        };

        self.set_logical_flags(result, true);

        0
    }

    fn execute_thumb_shift_instruction(
        &mut self,
        instruction: ThumbShiftInstruction,
        rd: REGISTER,
        rs: REGISTER,
    ) -> CYCLES {
        let mut shift_amount = self.get_register(rs) & 0xFF;
        let mut rd_val = self.get_register(rd);

        if rs == PC_REGISTER as u32 {
            shift_amount += 2;
        }

        if rd == PC_REGISTER as u32 {
            rd_val += 2;
        }

        let result = match instruction {
            ThumbShiftInstruction::Lsl => {
                if shift_amount == 0 {
                    self.shifter_output = self.get_flag(FlagsRegister::C);
                    rd_val
                } else if shift_amount < 32 {
                    self.shifter_output = rd_val.get_bit(32 - shift_amount as u8);
                    rd_val << shift_amount
                } else if shift_amount == 32 {
                    self.shifter_output = rd_val & 0x1;
                    0
                } else {
                    self.shifter_output = 0;
                    0
                }
            }
            ThumbShiftInstruction::Lsr => {
                if shift_amount == 0 {
                    self.shifter_output = self.get_flag(FlagsRegister::C);
                    rd_val
                } else if shift_amount < 32 {
                    self.shifter_output = rd_val.get_bit((shift_amount as u8) - 1);
                    rd_val >> shift_amount
                } else if shift_amount == 32 {
                    self.shifter_output = rd_val >> 31;
                    0
                } else {
                    self.shifter_output = 0;
                    0
                }
            }
            ThumbShiftInstruction::Asr => {
                if shift_amount == 0 {
                    self.shifter_output = self.get_flag(FlagsRegister::C);
                    rd_val
                } else if shift_amount < 32 {
                    self.shifter_output = rd_val.get_bit((shift_amount as u8) - 1);
                    ((rd_val as i32) >> shift_amount) as u32
                } else if (rd_val >> 31) > 0 {
                    self.shifter_output = 1;
                    u32::MAX
                } else {
                    self.shifter_output = 0;
                    0
                }
            }
            ThumbShiftInstruction::Ror => {
                let rotate_amount = shift_amount & 0x1F;
                if shift_amount == 0 {
                    self.shifter_output = self.get_flag(FlagsRegister::C);
                    rd_val
                } else if rotate_amount > 0 {
                    self.shifter_output = rd_val.get_bit((rotate_amount as u8) - 1);
                    rd_val.rotate_right(rotate_amount)
                } else {
                    self.shifter_output = rd_val >> 31;
                    rd_val
                }
            }
        };

        self.set_register(rd, result);
        self.set_arm_logical_flags(result, true);

        1
    }
}

pub struct ThumbMoveShiftedRegister(pub u32);

enum ThumbMoveShiftedRegisterOperations {
    LSL,
    LSR,
    ASR,
}

impl ThumbMoveShiftedRegister {
    fn opcode(&self) -> ThumbMoveShiftedRegisterOperations {
        match (self.0 & 0x1800) >> 11 {
            0 => ThumbMoveShiftedRegisterOperations::LSL,
            1 => ThumbMoveShiftedRegisterOperations::LSR,
            2 => ThumbMoveShiftedRegisterOperations::ASR,
            _ => panic!(),
        }
    }

    fn rs(&self) -> REGISTER {
        (self.0 & 0x0038) >> 3
    }

    fn rd(&self) -> REGISTER {
        self.0 & 0x0007
    }

    fn offset(&self) -> u32 {
        (self.0 & 0x07C0) >> 6
    }
}

impl Execute for ThumbMoveShiftedRegister {
    fn execute(self, cpu: &mut CPU, _memory: &mut GBAMemory) -> CYCLES {
        let rs_val = cpu.get_register(self.rs());
        match self.opcode() {
            ThumbMoveShiftedRegisterOperations::LSL => {
                cpu.thumb_lsl(self.rd(), rs_val, self.offset(), true)
            }
            ThumbMoveShiftedRegisterOperations::LSR => {
                cpu.thumb_lsr(self.rd(), rs_val, self.offset(), true)
            }
            ThumbMoveShiftedRegisterOperations::ASR => {
                cpu.thumb_asr(self.rd(), rs_val, self.offset(), true)
            }
        }

        0
    }
}

impl DecodeThumbInstructionToString for ThumbMoveShiftedRegister {
    fn instruction_to_string(&self) -> String {
        let opcode = match self.opcode() {
            ThumbMoveShiftedRegisterOperations::LSL => "lsls",
            ThumbMoveShiftedRegisterOperations::LSR => "lsrs",
            ThumbMoveShiftedRegisterOperations::ASR => "asrs",
        };
        format!("{opcode} {}, {}, #{}", self.rd(), self.rs(), self.offset())
    }
}

pub struct ThumbArithmeticImmInstruction(pub u32);

enum ThumbArithmeticImmOperations {
    Mov,
    Cmp,
    Add,
    Sub,
}

impl ThumbArithmeticImmInstruction {
    fn opcode(&self) -> ThumbArithmeticImmOperations {
        match (self.0 & 0x1800) >> 11 {
            0b00 => ThumbArithmeticImmOperations::Mov,
            0b01 => ThumbArithmeticImmOperations::Cmp,
            0b10 => ThumbArithmeticImmOperations::Add,
            0b11 => ThumbArithmeticImmOperations::Sub,
            _ => unreachable!(),
        }
    }

    fn rd(&self) -> u32 {
        (self.0 & 0x0700) >> 8
    }

    fn imm(&self) -> u32 {
        self.0 & 0x00FF
    }
}

impl DecodeThumbInstructionToString for ThumbArithmeticImmInstruction {
    fn instruction_to_string(&self) -> String {
        let opcode = match self.opcode() {
            ThumbArithmeticImmOperations::Mov => "movs",
            ThumbArithmeticImmOperations::Add => "adds",
            ThumbArithmeticImmOperations::Cmp => "cmp",
            ThumbArithmeticImmOperations::Sub => "subs",
        };

        format!(
            "{opcode} {}, {}",
            print_register(&self.rd()),
            Operand::Immediate(self.imm())
        )
    }
}

impl Execute for ThumbArithmeticImmInstruction {
    fn execute(self, cpu: &mut CPU, _memory: &mut GBAMemory) -> CYCLES {
        let rd = self.rd();
        let imm = self.imm();
        match self.opcode() {
            ThumbArithmeticImmOperations::Mov => cpu.thumb_move_imm(rd, imm),
            ThumbArithmeticImmOperations::Add => cpu.thumb_add_imm(rd, imm),
            ThumbArithmeticImmOperations::Cmp => cpu.thumb_cmp_imm(rd, imm),
            ThumbArithmeticImmOperations::Sub => cpu.thumb_sub_imm(rd, imm),
        };

        0
    }
}

pub struct ThumbHiRegInstruction(pub u32);

pub enum ThumbHiRegOperations {
    Add,
    Cmp,
    Mov,
}
impl ThumbHiRegInstruction {
    fn opcode(&self) -> ThumbHiRegOperations {
        match (self.0 & 0x0300) >> 8 {
            0b00 => ThumbHiRegOperations::Add,
            0b01 => ThumbHiRegOperations::Cmp,
            0b10 => ThumbHiRegOperations::Mov,
            _ => panic!(),
        }
    }

    fn rd(&self) -> REGISTER {
        (self.0.get_bit(7) << 3) | (self.0 & 0x0007)
    }

    fn rs(&self) -> REGISTER {
        (self.0.get_bit(6) << 3) | ((self.0 & 0x0038) >> 3)
    }
}

impl Execute for ThumbHiRegInstruction {
    fn execute(self, cpu: &mut CPU, memory: &mut GBAMemory) -> CYCLES {
        let mut cycles = 0;
        let rd = self.rd();
        let rs = self.rs();

        match self.opcode() {
            ThumbHiRegOperations::Add => {
                cpu.arm_add(rd, cpu.get_register(rd), cpu.get_register(rs), false)
            }
            ThumbHiRegOperations::Cmp => {
                cpu.arm_cmp(rd, cpu.get_register(rd), cpu.get_register(rs), true)
            }
            ThumbHiRegOperations::Mov => {
                cpu.arm_mov(rd, cpu.get_register(rd), cpu.get_register(rs), false)
            }
        };

        if rd == PC_REGISTER as u32 {
            cycles += cpu.flush_pipeline(memory);
        }

        cycles
    }
}

impl DecodeThumbInstructionToString for ThumbHiRegInstruction {
    fn instruction_to_string(&self) -> String {
        let opcode = match self.opcode() {
            ThumbHiRegOperations::Add => "add",
            ThumbHiRegOperations::Cmp => "cmp",
            ThumbHiRegOperations::Mov => "mov",
        };

        format!(
            "{opcode} {}, {}",
            print_register(&self.rd()),
            print_register(&self.rs())
        )
    }
}

pub struct ThumbBx(pub u32);

impl ThumbBx {
    fn rs(&self) -> REGISTER {
        (self.0.get_bit(6) << 3) | ((self.0 & 0x0038) >> 3)
    }
}

impl Execute for ThumbBx {
    fn execute(self, cpu: &mut CPU, memory: &mut GBAMemory) -> CYCLES {
        let mut cycles = 1;
        let rs = self.rs();
        let mut destination = cpu.get_register(rs);
        if destination.bit_is_set(0) {
            cpu.set_instruction_mode(InstructionMode::THUMB);
        } else {
            destination &= !2; // arm instructions must be word aligned
            cpu.set_instruction_mode(InstructionMode::ARM);
        };

        cpu.set_pc(destination & !1); // bit 0 is forced to 0 before storing
        cycles += cpu.flush_pipeline(memory);

        cycles
    }
}

impl DecodeThumbInstructionToString for ThumbBx {
    fn instruction_to_string(&self) -> String {
        format!("bx {}", print_register(&self.rs()))
    }
}

pub struct ThumbAdr(pub u32);

enum ThumbAdrRegister {
    PC,
    SP,
}

impl ThumbAdr {
    fn register(&self) -> ThumbAdrRegister {
        match self.0.get_bit(11) {
            0b0 => ThumbAdrRegister::PC,
            0b1 => ThumbAdrRegister::SP,
            _ => unreachable!(),
        }
    }

    fn rd(&self) -> REGISTER {
        (self.0 & 0x0700) >> 8
    }

    fn imm(&self) -> u32 {
        (self.0 & 0x00FF) * 4
    }
}

impl Execute for ThumbAdr {
    fn execute(self, cpu: &mut CPU, _memory: &mut GBAMemory) -> CYCLES {
        let rd = self.rd();
        let imm = self.imm();

        let result = match self.register() {
            ThumbAdrRegister::PC => (cpu.get_pc() & !2) + imm,
            ThumbAdrRegister::SP => cpu.get_sp() + imm,
        };

        cpu.set_register(rd, result);

        0
    }
}

impl DecodeThumbInstructionToString for ThumbAdr {
    fn instruction_to_string(&self) -> String {
        let source_reg = match self.register() {
            ThumbAdrRegister::PC => "pc",
            ThumbAdrRegister::SP => "sp",
        };
        format!(
            "adr {}, {source_reg}, {}",
            print_register(&self.rd()),
            Operand::Immediate(self.imm())
        )
    }
}

pub struct ThumbAddToSp(pub u32);

enum ThumbAddToSpOpcodes {
    Add,
    Sub,
}

impl ThumbAddToSp {
    fn opcode(&self) -> ThumbAddToSpOpcodes {
        match self.0.get_bit(7) {
            0b0 => ThumbAddToSpOpcodes::Add,
            0b1 => ThumbAddToSpOpcodes::Sub,
            _ => panic!(),
        }
    }

    fn imm(&self) -> u32 {
        (self.0 & 0x007F) * 4
    }
}

impl Execute for ThumbAddToSp {
    fn execute(self, cpu: &mut CPU, _memory: &mut GBAMemory) -> CYCLES {
        let imm = self.imm();

        let result = match self.opcode() {
            ThumbAddToSpOpcodes::Add => cpu.get_sp() + imm,
            ThumbAddToSpOpcodes::Sub => cpu.get_sp() - imm,
        };

        cpu.set_sp(result);

        0
    }
}

impl DecodeThumbInstructionToString for ThumbAddToSp {
    fn instruction_to_string(&self) -> String {
        let opcode = match self.opcode() {
            ThumbAddToSpOpcodes::Add => "add",
            ThumbAddToSpOpcodes::Sub => "sub",
        };
        format!("{opcode}, sp, {}", Operand::Immediate(self.imm()))
    }
}

impl CPU {
    fn thumb_lsl(&mut self, rd: REGISTER, rs_val: u32, offset: u32, set_flags: bool) {
        let offset = offset & 0xFF;
        let result = rs_val << offset;
        self.set_register(rd, result);

        if set_flags {
            if offset > 0 {
                if rs_val.bit_is_set(32 - offset as u8) {
                    self.set_flag(FlagsRegister::C)
                } else {
                    self.reset_flag(FlagsRegister::C)
                }
            }

            if result == 0 {
                self.set_flag(FlagsRegister::Z);
            } else {
                self.reset_flag(FlagsRegister::Z);
            }

            if result.bit_is_set(31) {
                self.set_flag(FlagsRegister::N);
            } else {
                self.reset_flag(FlagsRegister::N);
            }
        }
    }

    fn thumb_lsr(&mut self, rd: REGISTER, rs_val: u32, offset: u32, set_flags: bool) {
        let offset = offset & 0xFF;
        if offset == 0 {
            // LSR#32
            if set_flags {
                self.set_flag(FlagsRegister::Z);
                self.reset_flag(FlagsRegister::N);
                self.set_flag_from_bit(FlagsRegister::C, rs_val.get_bit(31) as u8);
            }
            self.set_register(rd, 0);
            return;
        }
        let result = rs_val >> offset;

        if set_flags {
            if rs_val.bit_is_set((offset - 1) as u8) {
                self.set_flag(FlagsRegister::C);
            } else {
                self.reset_flag(FlagsRegister::C);
            }

            if result == 0 {
                self.set_flag(FlagsRegister::Z);
            } else {
                self.reset_flag(FlagsRegister::Z);
            }

            if result.bit_is_set(31) {
                self.set_flag(FlagsRegister::N);
            } else {
                self.reset_flag(FlagsRegister::N);
            }
        }

        self.set_register(rd, result);
    }

    fn thumb_asr(&mut self, rd: REGISTER, rs_val: u32, offset: u32, set_flags: bool) {
        let offset = offset & 0xFF;
        if offset == 0 {
            if rs_val.bit_is_set(31) {
                if set_flags {
                    self.set_flag(FlagsRegister::C);
                    self.set_flag(FlagsRegister::N);
                    self.reset_flag(FlagsRegister::Z);
                }
                self.set_register(rd, u32::MAX);
            } else {
                if set_flags {
                    self.reset_flag(FlagsRegister::C);
                    self.set_flag(FlagsRegister::Z);
                    self.reset_flag(FlagsRegister::N);
                }
                self.set_register(rd, 0);
            }

            return;
        }

        let result = (rs_val as i32 >> offset) as u32;
        if set_flags {
            if rs_val.bit_is_set((offset - 1) as u8) {
                self.set_flag(FlagsRegister::C);
            } else {
                self.reset_flag(FlagsRegister::C);
            }

            if result == 0 {
                self.set_flag(FlagsRegister::Z);
            } else {
                self.reset_flag(FlagsRegister::Z);
            }

            if result.bit_is_set(31) {
                self.set_flag(FlagsRegister::N);
            } else {
                self.reset_flag(FlagsRegister::N);
            }
        }
        self.set_register(rd, result);
    }

    fn thumb_move_imm(&mut self, rd: REGISTER, imm: u32) {
        self.reset_flag(FlagsRegister::N);
        if imm == 0 {
            self.set_flag(FlagsRegister::Z);
        } else {
            self.reset_flag(FlagsRegister::Z);
        }
        self.set_register(rd, imm.into());
    }

    fn thumb_cmp_imm(&mut self, rd: REGISTER, imm: u32) {
        let minuend = self.get_register(rd);
        let result = minuend - imm as u32;
        self.set_arithmetic_flags(result, minuend, !(imm as u32), 1, true);
    }

    fn thumb_add_imm(&mut self, rd: REGISTER, imm: u32) {
        let addend1 = self.get_register(rd);
        let result = addend1 + imm as u32;
        self.set_arithmetic_flags(result, addend1, imm as u32, 0, true);
        self.set_register(rd, result);
    }

    fn thumb_sub_imm(&mut self, rd: REGISTER, imm: u32) {
        let minuend = self.get_register(rd);
        let imm = !(imm as u32);
        let result = minuend + imm + 1;
        self.set_arithmetic_flags(result, minuend, imm, 1, true);
        self.set_register(rd, result);
    }

    fn thumb_mul(&mut self, rd: REGISTER, operand1: u32, operand2: u32, set_flags: bool) {
        let result = (operand1 as u64 * operand2 as u64) as u32;
        if set_flags {
            self.set_flag_from_bit(FlagsRegister::N, result.get_bit(31) as u8);
            if result == 0 {
                self.set_flag(FlagsRegister::Z);
            } else {
                self.reset_flag(FlagsRegister::Z);
            }
        }
        self.set_register(rd, result);
    }
}

#[cfg(test)]
mod thumb_add_and_subtract_tests {

    use rstest::rstest;

    use crate::{
        arm7tdmi::{
            cpu::{FlagsRegister, InstructionMode},
            instruction_table::{Execute, Operand},
            thumb::alu::{ThumbFullAdder, ThumbFullAdderOperations},
        },
        gba::GBA,
    };

    #[rstest]
    #[case(ThumbFullAdderOperations::Add, 20, 43, 63)]
    #[case(ThumbFullAdderOperations::Add, 0xFFFFFFFF, 1, 0)]
    #[case(ThumbFullAdderOperations::Sub, 0xFFFFFFFF, 1, 0xFFFFFFFE)]
    fn able_to_execute_full_adder_instructions_with_register_op2(
        #[case] operation: ThumbFullAdderOperations,
        #[case] register2_val: u32,
        #[case] register1_val: u32,
        #[case] expected_result: u32,
    ) {
        let mut gba = GBA::new_no_bios();
        gba.cpu.set_register(1, register1_val);
        gba.cpu.set_register(2, register2_val);
        let instruction = ThumbFullAdder(operation, 3, 2, Operand::Register(1));

        instruction.execute(&mut gba.cpu, &mut gba.memory);

        assert_eq!(gba.cpu.get_register(3), expected_result);
    }

    #[test]
    fn should_add_two_registers_together() {
        let mut gba = GBA::new_no_bios();
        gba.cpu.set_instruction_mode(InstructionMode::THUMB);

        gba.cpu.set_register(1, 20);
        gba.cpu.set_register(2, 43);
        gba.cpu.prefetch[0] = Some(0x1888); // adds r0, r1, r2
        gba.step();
        gba.step();

        assert_eq!(gba.cpu.get_register(0), 63);
        assert!(gba.cpu.get_flag(FlagsRegister::C) == 0);
        assert!(gba.cpu.get_flag(FlagsRegister::N) == 0);
        assert!(gba.cpu.get_flag(FlagsRegister::Z) == 0);
        assert!(gba.cpu.get_flag(FlagsRegister::V) == 0);
    }

    #[test]
    fn should_add_two_registers_together_and_set_n_flag() {
        let mut gba = GBA::new_no_bios();
        gba.cpu.set_instruction_mode(InstructionMode::THUMB);

        gba.cpu.set_register(1, 20);
        gba.cpu.set_register(2, (-43 as i32) as u32);
        gba.cpu.prefetch[0] = Some(0x1888); // adds r0, r1, r2
        gba.step();
        gba.step();

        assert_eq!(gba.cpu.get_register(0), (-23 as i32) as u32);
        assert!(gba.cpu.get_flag(FlagsRegister::C) == 0);
        assert!(gba.cpu.get_flag(FlagsRegister::N) == 1);
        assert!(gba.cpu.get_flag(FlagsRegister::Z) == 0);
        assert!(gba.cpu.get_flag(FlagsRegister::V) == 0);
    }

    #[test]
    fn should_add_two_registers_together_and_set_z_and_c_flag() {
        let mut gba = GBA::new_no_bios();
        gba.cpu.set_instruction_mode(InstructionMode::THUMB);

        gba.cpu.set_register(1, 20);
        gba.cpu.set_register(2, (-20 as i32) as u32);
        gba.cpu.prefetch[0] = Some(0x1888); // adds r0, r1, r2
        gba.step();
        gba.step();

        assert_eq!(gba.cpu.get_register(0), 0);
        assert!(gba.cpu.get_flag(FlagsRegister::C) == 1);
        assert!(gba.cpu.get_flag(FlagsRegister::N) == 0);
        assert!(gba.cpu.get_flag(FlagsRegister::Z) == 1);
        assert!(gba.cpu.get_flag(FlagsRegister::V) == 0);
    }

    #[test]
    fn should_add_two_registers_together_and_set_z_flag() {
        let mut gba = GBA::new_no_bios();
        gba.cpu.set_instruction_mode(InstructionMode::THUMB);

        gba.cpu.set_register(1, 0);
        gba.cpu.set_register(2, 0);
        gba.cpu.prefetch[0] = Some(0x1888); // adds r0, r1, r2
        gba.step();
        gba.step();

        assert_eq!(gba.cpu.get_register(0), 0);
        assert!(gba.cpu.get_flag(FlagsRegister::C) == 0);
        assert!(gba.cpu.get_flag(FlagsRegister::N) == 0);
        assert!(gba.cpu.get_flag(FlagsRegister::Z) == 1);
        assert!(gba.cpu.get_flag(FlagsRegister::V) == 0);
    }

    #[test]
    fn should_add_two_registers_together_and_set_v_flag() {
        let mut gba = GBA::new_no_bios();
        gba.cpu.set_instruction_mode(InstructionMode::THUMB);

        gba.cpu.set_register(1, 0x1);
        gba.cpu.set_register(2, 0x7FFF_FFFF);
        gba.cpu.prefetch[0] = Some(0x1888); // adds r0, r1, r2
        gba.step();
        gba.step();

        assert_eq!(gba.cpu.get_register(0), 0x8000_0000);
        assert!(gba.cpu.get_flag(FlagsRegister::C) == 0);
        assert!(gba.cpu.get_flag(FlagsRegister::N) == 1);
        assert!(gba.cpu.get_flag(FlagsRegister::Z) == 0);
        assert!(gba.cpu.get_flag(FlagsRegister::V) == 1);
    }

    #[test]
    fn should_add_register_and_immediate_and_set_n_flag() {
        let mut gba = GBA::new_no_bios();
        gba.cpu.set_instruction_mode(InstructionMode::THUMB);

        gba.cpu.set_register(1, -10 as i32 as u32);
        gba.cpu.prefetch[0] = Some(0x1d48); // adds r0, r1, 5
        gba.step();
        gba.step();

        assert_eq!(gba.cpu.get_register(0), -5 as i32 as u32);
        assert!(gba.cpu.get_flag(FlagsRegister::C) == 0);
        assert!(gba.cpu.get_flag(FlagsRegister::N) == 1);
        assert!(gba.cpu.get_flag(FlagsRegister::Z) == 0);
        assert!(gba.cpu.get_flag(FlagsRegister::V) == 0);
    }

    #[test]
    fn should_add_register_and_immediate_and_set_z_flag() {
        let mut gba = GBA::new_no_bios();
        gba.cpu.set_instruction_mode(InstructionMode::THUMB);

        gba.cpu.set_register(1, -5 as i32 as u32);
        gba.cpu.prefetch[0] = Some(0x1d48); // adds r0, r1, 5
        gba.step();
        gba.step();

        assert_eq!(gba.cpu.get_register(0), 0 as i32 as u32);
        assert!(gba.cpu.get_flag(FlagsRegister::C) == 1);
        assert!(gba.cpu.get_flag(FlagsRegister::N) == 0);
        assert!(gba.cpu.get_flag(FlagsRegister::Z) == 1);
        assert!(gba.cpu.get_flag(FlagsRegister::V) == 0);
    }

    #[test]
    fn should_add_register_and_immediate_and_set_v_flag() {
        let mut gba = GBA::new_no_bios();
        gba.cpu.set_instruction_mode(InstructionMode::THUMB);

        gba.cpu.set_register(1, 0x7FFF_FFFF);
        gba.cpu.prefetch[0] = Some(0x1d48); // adds r0, r1, 5
        gba.step();
        gba.step();

        assert_eq!(gba.cpu.get_register(0), 0x8000_0004);
        assert!(gba.cpu.get_flag(FlagsRegister::C) == 0);
        assert!(gba.cpu.get_flag(FlagsRegister::N) == 1);
        assert!(gba.cpu.get_flag(FlagsRegister::Z) == 0);
        assert!(gba.cpu.get_flag(FlagsRegister::V) == 1);
    }

    #[test]
    fn should_add_register_and_immediate_and_set_c_flag() {
        let mut gba = GBA::new_no_bios();
        gba.cpu.set_instruction_mode(InstructionMode::THUMB);

        gba.cpu.set_register(1, 0xFFFF_FFFF);
        gba.cpu.prefetch[0] = Some(0x1d48); // adds r0, r1, 5
        gba.step();
        gba.step();

        assert_eq!(gba.cpu.get_register(0), 0x0000_0004);
        assert!(gba.cpu.get_flag(FlagsRegister::C) == 1);
        assert!(gba.cpu.get_flag(FlagsRegister::N) == 0);
        assert!(gba.cpu.get_flag(FlagsRegister::Z) == 0);
        assert!(gba.cpu.get_flag(FlagsRegister::V) == 0);
    }

    #[test]
    fn should_sub_two_registers() {
        let mut gba = GBA::new_no_bios();
        gba.cpu.set_instruction_mode(InstructionMode::THUMB);

        gba.cpu.set_register(1, 50);
        gba.cpu.set_register(2, 20);
        gba.cpu.prefetch[0] = Some(0x1a88); // subs r0, r1, r2
        gba.step();
        gba.step();

        assert_eq!(gba.cpu.get_register(0), 30);
        assert!(gba.cpu.get_flag(FlagsRegister::C) == 1);
        assert!(gba.cpu.get_flag(FlagsRegister::N) == 0);
        assert!(gba.cpu.get_flag(FlagsRegister::Z) == 0);
        assert!(gba.cpu.get_flag(FlagsRegister::V) == 0);
    }

    #[test]
    fn should_sub_two_registers_and_reset_c_flag() {
        let mut gba = GBA::new_no_bios();
        gba.cpu.set_instruction_mode(InstructionMode::THUMB);

        gba.cpu.set_register(1, 25);
        gba.cpu.set_register(2, 50);
        gba.cpu.prefetch[0] = Some(0x1a88); // subs r0, r1, r2
        gba.step();
        gba.step();

        assert_eq!(gba.cpu.get_register(0), -25 as i32 as u32);
        assert_eq!(gba.cpu.get_flag(FlagsRegister::C), 0);
        assert_eq!(gba.cpu.get_flag(FlagsRegister::N), 1);
        assert_eq!(gba.cpu.get_flag(FlagsRegister::Z), 0);
        assert_eq!(gba.cpu.get_flag(FlagsRegister::V), 0);
    }
}

#[cfg(test)]
mod thumb_move_shifted_register_tests {

    use crate::{
        arm7tdmi::cpu::{FlagsRegister, InstructionMode, CPU},
        gba::GBA,
        memory::memory::GBAMemory,
    };

    #[test]
    fn should_left_shift_a_register_and_set_c_flag() {
        let mut gba = GBA::new_no_bios();
        gba.cpu.set_instruction_mode(InstructionMode::THUMB);

        gba.cpu.set_register(1, 0x0F00_0000);
        gba.cpu.prefetch[0] = Some(0x0148); // lsls r0, r1, 5
        gba.step();
        gba.step();

        assert_eq!(gba.cpu.get_register(0), 0x0F00_0000 << 5);
        assert_eq!(gba.cpu.get_flag(FlagsRegister::C), 1);
        assert_eq!(gba.cpu.get_flag(FlagsRegister::N), 1);
        assert_eq!(gba.cpu.get_flag(FlagsRegister::Z), 0);
    }

    #[test]
    fn should_not_left_shift_register_and_not_change_c_flag() {
        let mut gba = GBA::new_no_bios();
        gba.cpu.set_instruction_mode(InstructionMode::THUMB);

        gba.cpu.set_register(1, 0x0F00_0000);
        gba.cpu.set_flag(FlagsRegister::C);
        gba.cpu.prefetch[0] = Some(0x0008); // lsls r0, r1, 0
        gba.step();
        gba.step();

        assert_eq!(gba.cpu.get_register(0), 0x0F00_0000);
        assert_eq!(gba.cpu.get_flag(FlagsRegister::C), 1);
        assert_eq!(gba.cpu.get_flag(FlagsRegister::N), 0);
        assert_eq!(gba.cpu.get_flag(FlagsRegister::Z), 0);
    }

    #[test]
    fn should_lsl_register_and_not_affect_v_flag() {
        let mut gba = GBA::new_no_bios();
        gba.cpu.set_instruction_mode(InstructionMode::THUMB);

        gba.cpu.set_register(1, 0xF000_0000);
        gba.cpu.set_flag(FlagsRegister::V);
        gba.cpu.prefetch[0] = Some(0x0148); // lsls r0, r1, 5
        gba.step();
        gba.step();

        assert_eq!(gba.cpu.get_register(0), 0x0);
        assert_eq!(gba.cpu.get_flag(FlagsRegister::C), 0);
        assert_eq!(gba.cpu.get_flag(FlagsRegister::N), 0);
        assert_eq!(gba.cpu.get_flag(FlagsRegister::Z), 1);
        assert_eq!(gba.cpu.get_flag(FlagsRegister::V), 1);
    }

    #[test]
    fn should_asr_register_and_set_c_flag() {
        let mut gba = GBA::new_no_bios();
        gba.cpu.set_instruction_mode(InstructionMode::THUMB);

        gba.cpu.set_register(1, 0x0000_008F);
        gba.cpu.prefetch[0] = Some(0x1108); // asrs r0, r1, 4
        gba.step();
        gba.step();

        assert_eq!(gba.cpu.get_register(0), 0x0000_0008);
        assert_eq!(gba.cpu.get_flag(FlagsRegister::C), 1);
        assert_eq!(gba.cpu.get_flag(FlagsRegister::N), 0);
        assert_eq!(gba.cpu.get_flag(FlagsRegister::Z), 0);
    }

    #[test]
    fn should_asr_register_and_maintain_sign() {
        let mut gba = GBA::new_no_bios();
        gba.cpu.set_instruction_mode(InstructionMode::THUMB);

        gba.cpu.set_register(1, 0x8000_008F);
        gba.cpu.prefetch[0] = Some(0x1108); // asrs r0, r1, 4
        gba.step();
        gba.step();

        assert_eq!(gba.cpu.get_register(0), 0xF800_0008);
        assert_eq!(gba.cpu.get_flag(FlagsRegister::C), 1);
        assert_eq!(gba.cpu.get_flag(FlagsRegister::N), 1);
        assert_eq!(gba.cpu.get_flag(FlagsRegister::Z), 0);
    }

    #[test]
    fn should_asr_register_and_set_all_ones() {
        let mut gba = GBA::new_no_bios();
        gba.cpu.set_instruction_mode(InstructionMode::THUMB);

        gba.cpu.set_register(1, 0x8000_0000);
        gba.cpu.prefetch[0] = Some(0x1008); // asrs r0, r1, 32
        gba.step();
        gba.step();

        assert_eq!(gba.cpu.get_register(0), 0xFFFF_FFFF);
        assert_eq!(gba.cpu.get_flag(FlagsRegister::C), 1);
        assert_eq!(gba.cpu.get_flag(FlagsRegister::N), 1);
        assert_eq!(gba.cpu.get_flag(FlagsRegister::Z), 0);
    }

    #[test]
    fn should_lsr_register() {
        let mut gba = GBA::new_no_bios();
        gba.cpu.set_instruction_mode(InstructionMode::THUMB);

        gba.cpu.set_register(1, 0x8000_008F);
        gba.cpu.prefetch[0] = Some(0x0a88); // lsrs r0, r1, 10
        gba.step();
        gba.step();

        assert_eq!(gba.cpu.get_register(0), 0x8000_008F >> 10);
        assert_eq!(gba.cpu.get_flag(FlagsRegister::C), 0);
        assert_eq!(gba.cpu.get_flag(FlagsRegister::N), 0);
        assert_eq!(gba.cpu.get_flag(FlagsRegister::Z), 0);
    }

    #[test]
    fn should_lsr_register_and_clear_register() {
        let mut gba = GBA::new_no_bios();
        gba.cpu.set_instruction_mode(InstructionMode::THUMB);

        gba.cpu.set_register(1, 0x8000_008F);
        gba.cpu.prefetch[0] = Some(0x0808); // lsrs r0, r1, 32
        gba.step();
        gba.step();

        assert_eq!(gba.cpu.get_register(0), 0);
        assert_eq!(gba.cpu.get_flag(FlagsRegister::C), 1);
        assert_eq!(gba.cpu.get_flag(FlagsRegister::N), 0);
        assert_eq!(gba.cpu.get_flag(FlagsRegister::Z), 1);
    }
}

#[cfg(test)]
mod thumb_move_compare_add_subtract_tests {

    use crate::{
        arm7tdmi::cpu::{FlagsRegister, InstructionMode, CPU},
        gba::GBA,
        memory::memory::GBAMemory,
    };

    #[test]
    fn should_move_immediate_into_r0() {
        let mut gba = GBA::new_no_bios();
        gba.cpu.set_instruction_mode(InstructionMode::THUMB);

        gba.cpu.prefetch[0] = Some(0x200f); // movs r0, 15
        gba.step();
        gba.step();

        assert_eq!(gba.cpu.get_register(0), 15);
        assert_eq!(gba.cpu.get_flag(FlagsRegister::C), 0);
        assert_eq!(gba.cpu.get_flag(FlagsRegister::N), 0);
        assert_eq!(gba.cpu.get_flag(FlagsRegister::Z), 0);
    }

    #[test]
    fn should_move_immediate_into_r0_and_not_set_n_flag() {
        let mut gba = GBA::new_no_bios();
        gba.cpu.set_instruction_mode(InstructionMode::THUMB);

        gba.cpu.prefetch[0] = Some(0x2096); // movs r0, 150
        gba.step();
        gba.step();

        assert_eq!(gba.cpu.get_register(0), 150);
        assert_eq!(gba.cpu.get_flag(FlagsRegister::C), 0);
        assert_eq!(gba.cpu.get_flag(FlagsRegister::N), 0);
        assert_eq!(gba.cpu.get_flag(FlagsRegister::Z), 0);
    }

    #[test]
    fn should_move_immediate_into_r0_and_set_z_flag() {
        let mut gba = GBA::new_no_bios();
        gba.cpu.set_instruction_mode(InstructionMode::THUMB);

        gba.cpu.prefetch[0] = Some(0x2000); // movs r0, 0
        gba.step();
        gba.step();

        assert_eq!(gba.cpu.get_register(0), 0);
        assert_eq!(gba.cpu.get_flag(FlagsRegister::C), 0);
        assert_eq!(gba.cpu.get_flag(FlagsRegister::N), 0);
        assert_eq!(gba.cpu.get_flag(FlagsRegister::Z), 1);
    }

    #[test]
    fn should_sub_imm_from_r0_and_set_z_flag() {
        let mut gba = GBA::new_no_bios();
        gba.cpu.set_instruction_mode(InstructionMode::THUMB);

        gba.cpu.set_register(0, 15);
        gba.cpu.prefetch[0] = Some(0x380f); // subs r0, 15
        gba.step();
        gba.step();

        assert_eq!(gba.cpu.get_register(0), 0);
        assert_eq!(gba.cpu.get_flag(FlagsRegister::C), 1);
        assert_eq!(gba.cpu.get_flag(FlagsRegister::N), 0);
        assert_eq!(gba.cpu.get_flag(FlagsRegister::Z), 1);
    }

    #[test]
    fn should_add_imm_to_r0_and_set_n_flag() {
        let mut gba = GBA::new_no_bios();
        gba.cpu.set_instruction_mode(InstructionMode::THUMB);

        gba.cpu.set_register(0, 0x7FFF_FFFF);
        gba.cpu.prefetch[0] = Some(0x300f); // adds r0, 15
        gba.step();
        gba.step();

        assert_eq!(gba.cpu.get_register(0), 0x8000_000E);
        assert_eq!(gba.cpu.get_flag(FlagsRegister::C), 0);
        assert_eq!(gba.cpu.get_flag(FlagsRegister::N), 1);
        assert_eq!(gba.cpu.get_flag(FlagsRegister::Z), 0);
    }
}

#[cfg(test)]
mod thumb_alu_operations_tests {

    use crate::{
        arm7tdmi::cpu::{FlagsRegister, InstructionMode, CPU},
        gba::GBA,
        memory::memory::GBAMemory,
    };

    #[test]
    fn should_and_two_numbers_together() {
        let mut gba = GBA::new_no_bios();
        gba.cpu.set_instruction_mode(InstructionMode::THUMB);

        gba.cpu.set_register(0, 0x8123_2344);
        gba.cpu.set_register(1, 0x8000_2344);
        gba.cpu.prefetch[0] = Some(0x4008); // ands r0, r1
        gba.step();
        gba.step();

        assert_eq!(gba.cpu.get_register(0), 0x8000_2344);
        assert_eq!(gba.cpu.get_flag(FlagsRegister::N), 1);
        assert_eq!(gba.cpu.get_flag(FlagsRegister::Z), 0);
    }

    #[test]
    fn should_eor_two_numbers_together() {
        let mut gba = GBA::new_no_bios();
        gba.cpu.set_instruction_mode(InstructionMode::THUMB);

        gba.cpu.set_register(0, 0x1010_1010);
        gba.cpu.set_register(1, 0x0101_0101);
        gba.cpu.prefetch[0] = Some(0x4048); // eors r0, r1
        gba.step();
        gba.step();

        assert_eq!(gba.cpu.get_register(0), 0x1111_1111);
        assert_eq!(gba.cpu.get_flag(FlagsRegister::N), 0);
        assert_eq!(gba.cpu.get_flag(FlagsRegister::Z), 0);
    }

    #[test]
    fn should_lsl_rd_by_5() {
        let mut gba = GBA::new_no_bios();
        gba.cpu.set_instruction_mode(InstructionMode::THUMB);

        gba.cpu.set_register(0, 0x0F11_1230);
        gba.cpu.set_register(1, 5);
        gba.cpu.prefetch[0] = Some(0x4088); // lsl r0, r1
        gba.step();
        gba.step();

        assert_eq!(gba.cpu.get_register(0), 0xE222_4600);
        assert_eq!(gba.cpu.get_flag(FlagsRegister::N), 1);
        assert_eq!(gba.cpu.get_flag(FlagsRegister::C), 1);
        assert_eq!(gba.cpu.get_flag(FlagsRegister::Z), 0);
    }

    #[test]
    fn should_lsr_rd_by_0() {
        let mut gba = GBA::new_no_bios();
        gba.cpu.set_instruction_mode(InstructionMode::THUMB);

        gba.cpu.set_register(0, 0x0F11_1230);
        gba.cpu.set_register(1, 0);
        gba.cpu.prefetch[0] = Some(0x40c8); // lsr r0, r1
        gba.step();
        gba.step();

        assert_eq!(gba.cpu.get_register(0), 0x0F11_1230);
        assert_eq!(gba.cpu.get_flag(FlagsRegister::N), 0);
        assert_eq!(gba.cpu.get_flag(FlagsRegister::C), 0);
        assert_eq!(gba.cpu.get_flag(FlagsRegister::Z), 0);
    }
}

#[cfg(test)]
mod thumb_hi_reg_operations {

    use crate::{
        arm7tdmi::cpu::{FlagsRegister, InstructionMode, CPU},
        gba::GBA,
        memory::memory::GBAMemory,
    };

    #[test]
    fn should_add_two_regs_together_and_not_affect_flags() {
        let mut gba = GBA::new_no_bios();
        gba.cpu.set_instruction_mode(InstructionMode::THUMB);

        gba.cpu.set_register(0, 20);
        gba.cpu.set_register(11, 15);
        gba.cpu.set_flag(FlagsRegister::N);
        gba.cpu.prefetch[1] = Some(0x4458); // add r0, r11
        gba.step();

        assert_eq!(gba.cpu.get_register(0), 35);
        assert_eq!(gba.cpu.get_flag(FlagsRegister::N), 1);
        assert_eq!(gba.cpu.get_flag(FlagsRegister::C), 0);
        assert_eq!(gba.cpu.get_flag(FlagsRegister::Z), 0);
    }

    #[test]
    fn should_cmp_registers_and_set_flags() {
        let mut gba = GBA::new_no_bios();
        gba.cpu.set_instruction_mode(InstructionMode::THUMB);

        gba.cpu.set_register(0, 20);
        gba.cpu.set_register(11, 20);
        gba.cpu.set_flag(FlagsRegister::N);
        gba.cpu.prefetch[0] = Some(0x4558); // cmp r0, r11
        gba.step();
        gba.step();

        assert_eq!(gba.cpu.get_register(0), 20);
        assert_eq!(gba.cpu.get_flag(FlagsRegister::N), 0);
        assert_eq!(gba.cpu.get_flag(FlagsRegister::C), 1);
        assert_eq!(gba.cpu.get_flag(FlagsRegister::Z), 1);
    }

    #[test]
    fn should_mov_register() {
        let mut gba = GBA::new_no_bios();
        gba.cpu.set_instruction_mode(InstructionMode::THUMB);

        gba.cpu.set_register(0, 20);
        gba.cpu.set_register(11, 55);
        gba.cpu.set_flag(FlagsRegister::N);
        gba.cpu.prefetch[0] = Some(0x4658); // cmp r0, r11
        gba.step();
        gba.step();

        assert_eq!(gba.cpu.get_register(0), 55);
    }
}

#[cfg(test)]
mod thumb_bx_tests {

    use crate::{
        arm7tdmi::cpu::{InstructionMode, CPU},
        gba::GBA,
        memory::memory::GBAMemory,
    };

    #[test]
    fn should_switch_to_arm_mode_and_align_address() {
        let mut gba = GBA::new_no_bios();
        gba.cpu.set_instruction_mode(InstructionMode::THUMB);

        gba.cpu.set_register(5, 0x16);
        gba.cpu.prefetch[0] = Some(0x4728); // bx r5
        gba.step();
        gba.step();

        assert_eq!(gba.cpu.get_pc(), 0x1C);
        assert!(matches!(
            gba.cpu.get_instruction_mode(),
            InstructionMode::ARM
        ));
    }

    #[test]
    fn should_switch_to_arm_mode_when_pc_operand() {
        let mut gba = GBA::new_no_bios();
        gba.cpu.set_instruction_mode(InstructionMode::THUMB);

        gba.cpu.set_pc(0x16);
        gba.cpu.prefetch[0] = Some(0x4778); // bx r15
        gba.step();
        gba.step();

        assert_eq!(gba.cpu.get_pc(), 0x20);
        assert!(matches!(
            gba.cpu.get_instruction_mode(),
            InstructionMode::ARM
        ));
    }
}

#[cfg(test)]
mod get_relative_address_tests {

    use crate::{
        arm7tdmi::cpu::{InstructionMode, CPU},
        gba::GBA,
        memory::memory::GBAMemory,
    };

    #[test]
    fn should_add_12_to_pc() {
        let mut gba = GBA::new_no_bios();
        gba.cpu.set_instruction_mode(InstructionMode::THUMB);

        gba.cpu.prefetch[0] = Some(0xa503); // add r5, pc, 12
        gba.cpu.set_pc(2);
        gba.step();
        gba.step();

        assert_eq!(gba.cpu.get_pc(), 6);
        assert_eq!(gba.cpu.get_register(5), 16);
    }

    #[test]
    fn should_add_16_to_sp_and_store_in_r5() {
        let mut gba = GBA::new_no_bios();
        gba.cpu.set_instruction_mode(InstructionMode::THUMB);

        gba.cpu.prefetch[0] = Some(0xad04); // add r5, sp, 16
        gba.cpu.set_sp(2);
        gba.step();
        gba.step();

        assert_eq!(gba.cpu.get_register(5), 18);
    }

    #[test]
    fn should_add_500_to_sp() {
        let mut gba = GBA::new_no_bios();
        gba.cpu.set_instruction_mode(InstructionMode::THUMB);

        gba.cpu.prefetch[0] = Some(0xb07d); // add sp, 500
        gba.cpu.set_sp(2);
        gba.step();
        gba.step();

        assert_eq!(gba.cpu.get_sp(), 502);
    }

    #[test]
    fn should_sub_500_to_sp() {
        let mut gba = GBA::new_no_bios();
        gba.cpu.set_instruction_mode(InstructionMode::THUMB);

        gba.cpu.prefetch[0] = Some(0xb0fd); // add sp, 500
        gba.cpu.set_sp(2);
        gba.step();
        gba.step();

        assert_eq!(gba.cpu.get_sp(), (2 - 500) as i32 as u32);
    }
}
