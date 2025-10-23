use std::{fmt::Display, iter::Enumerate, mem::size_of};

use crate::{
    arm7tdmi::{
        arm::alu::{Shift, ShiftType},
        cpu::{CPU, PC_REGISTER},
        instruction_table::{DecodeARMInstructionToString, Execute, Operand},
    },
    memory::memory::GBAMemory,
    types::{CYCLES, REGISTER, WORD},
    utils::{
        bits::{sign_extend, Bits},
        instruction_to_string::{print_register, print_shifted_operand},
        utils::print_vec,
    },
};

pub struct SdtInstruction(pub u32);

pub enum SdtOpcode {
    Load(LoadOpcodes),
    Store(StoreOpcodes),
}

pub enum LoadOpcodes {
    LDR,
    LDRB,
}

pub enum StoreOpcodes {
    STR,
    STRB,
}

impl Display for SdtOpcode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let opcode = match self {
            SdtOpcode::Load(LoadOpcodes::LDR) => "ldr",
            SdtOpcode::Load(LoadOpcodes::LDRB) => "ldrb",
            SdtOpcode::Store(StoreOpcodes::STR) => "str",
            SdtOpcode::Store(StoreOpcodes::STRB) => "strb",
        };

        write!(f, "{}", opcode)
    }
}

enum SdtOffset {
    Imm(u32),
    ShiftedRegister(REGISTER, Shift),
}

impl SdtInstruction {
    fn rd(&self) -> REGISTER {
        (self.0 & 0x0000_F000) >> 12
    }

    fn rn(&self) -> REGISTER {
        (self.0 & 0x000F_0000) >> 16
    }

    fn add_offset(&self) -> bool {
        self.0.bit_is_set(23)
    }

    fn pre_indexed_addressing(&self) -> bool {
        self.0.bit_is_set(24)
    }

    fn write_back_address(&self, pre_indexed_addressing: bool) -> bool {
        !pre_indexed_addressing || self.0.bit_is_set(21)
    }

    fn opcode(&self) -> SdtOpcode {
        match self.0.get_bit(20) << 1 | self.0.get_bit(22) {
            0b00 => SdtOpcode::Store(StoreOpcodes::STR),
            0b01 => SdtOpcode::Store(StoreOpcodes::STRB),
            0b10 => SdtOpcode::Load(LoadOpcodes::LDR),
            0b11 => SdtOpcode::Load(LoadOpcodes::LDRB),
            _ => unreachable!(),
        }
    }

    fn offset(&self) -> SdtOffset {
        if self.0.bit_is_set(25) {
            let shift_amount = (self.0 & 0x0000_0F80) >> 7;
            if shift_amount == 0 {
                return match (self.0 >> 5) & 0b11 {
                    0b00 => SdtOffset::ShiftedRegister(
                        self.0 & 0xF,
                        Shift(ShiftType::LSL, Operand::Immediate(0)),
                    ),
                    0b01 => SdtOffset::ShiftedRegister(
                        self.0 & 0xF,
                        Shift(ShiftType::LSR, Operand::Immediate(32)),
                    ),
                    0b10 => SdtOffset::ShiftedRegister(
                        self.0 & 0xF,
                        Shift(ShiftType::ASR, Operand::Immediate(32)),
                    ),
                    0b11 => SdtOffset::ShiftedRegister(
                        self.0 & 0xF,
                        Shift(ShiftType::RRX, Operand::Immediate(1)),
                    ),
                    _ => unreachable!(),
                };
            } else {
                let shift_type = match (self.0 >> 5) & 0b11 {
                    0b00 => ShiftType::LSL,
                    0b01 => ShiftType::LSR,
                    0b10 => ShiftType::ASR,
                    0b11 => ShiftType::ROR,
                    _ => unreachable!(),
                };
                return SdtOffset::ShiftedRegister(
                    self.0 & 0xF,
                    Shift(shift_type, Operand::Immediate(shift_amount)),
                );
            }
        } else {
            SdtOffset::Imm(self.0 & 0x0000_0fff)
        }
    }
}

impl SdtOpcode {
    #[inline]
    pub fn execute(&self, cpu: &mut CPU, memory: &mut GBAMemory, rd: REGISTER, access_address: usize) -> CYCLES {
        let mut cycles = 0;
        match self {
            SdtOpcode::Load(load_opcode) => {
                let data = match load_opcode {
                    LoadOpcodes::LDR => memory.readu32(access_address),
                    LoadOpcodes::LDRB => memory.read(access_address).into(),
                };
                cpu.set_register(rd, data.data);
                if rd == PC_REGISTER as u32 {
                    cycles += cpu.flush_pipeline(memory)
                }
                cycles += data.cycles + 1;
            }
            SdtOpcode::Store(store) => {
                let mut data = cpu.get_register(rd);
                if rd == PC_REGISTER as u32 {
                    data += 4;
                }
                cycles += match store {
                    StoreOpcodes::STR => memory.writeu32(access_address, data),
                    StoreOpcodes::STRB => memory.write(access_address, data as u8),
                };
            }
        };
        cycles
    }
}

impl Execute for SdtInstruction {
    fn execute(self, cpu: &mut CPU, memory: &mut GBAMemory) -> CYCLES {
        let mut cycles = 0;

        let offset = match self.offset() {
            SdtOffset::Imm(imm) => imm,
            SdtOffset::ShiftedRegister(register, shift) => {
                cpu.execute_register_shift(&mut cycles, memory, register, shift, false)
            }
        };

        let base_register_address = cpu.get_register(self.rn());

        let offset_address = if self.add_offset() {
            base_register_address + offset
        } else {
            base_register_address - offset
        };

        let pre_indexed_addressing = self.pre_indexed_addressing();
        let access_address = if pre_indexed_addressing {
            offset_address
        } else {
            base_register_address
        } as usize;

        cycles += self.opcode().execute(cpu, memory, self.rd(), access_address);

        if self.write_back_address(pre_indexed_addressing) {
            cpu.set_register(self.rn(), offset_address);
        }
        cycles
    }
}

impl DecodeARMInstructionToString for SdtInstruction {
    fn instruction_to_string(&self, condition_code: &str) -> String {
        let opcode = match self.opcode() {
            SdtOpcode::Load(LoadOpcodes::LDR) => "ldr",
            SdtOpcode::Load(LoadOpcodes::LDRB) => "ldrb",
            SdtOpcode::Store(StoreOpcodes::STR) => "str",
            SdtOpcode::Store(StoreOpcodes::STRB) => "strb",
        };

        let add_offset = if self.add_offset() { "" } else { "-" };

        let offset_address = match self.offset() {
            SdtOffset::Imm(0) => "".into(),
            SdtOffset::Imm(imm) => format!(", {}{}", add_offset, Operand::Immediate(imm)),
            SdtOffset::ShiftedRegister(reg, shift) => format!(
                ", {}{}, ",
                add_offset,
                print_shifted_operand(&Operand::Register(reg), &shift)
            ),
        };

        let access_address = if self.pre_indexed_addressing() {
            let write_back = if self.write_back_address(true) {
                "!"
            } else {
                ""
            };
            format!(
                "[{}{offset_address}]{write_back}",
                print_register(&self.rn())
            )
        } else {
            format!("[{}]{offset_address}", print_register(&self.rn()))
        };

        format!(
            "{opcode}{condition_code} {} {access_address}",
            print_register(&self.rd())
        )
    }
}

pub struct SignedAndHwDtInstruction(pub u32);

pub enum SignedAndHwDtOpcodes {
    Load(SignedAndHwDtLoadOpcodes),
    STRH,
}

pub enum SignedAndHwDtLoadOpcodes {
    LDRH,
    LDRSB,
    LDRSH,
}

impl Display for SignedAndHwDtOpcodes {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let opcode = match self {
            SignedAndHwDtOpcodes::Load(SignedAndHwDtLoadOpcodes::LDRH) => "ldrh",
            SignedAndHwDtOpcodes::Load(SignedAndHwDtLoadOpcodes::LDRSH) => "ldrsh",
            SignedAndHwDtOpcodes::Load(SignedAndHwDtLoadOpcodes::LDRSB) => "ldrsb",
            SignedAndHwDtOpcodes::STRH => "strh",
        };

        write!(f, "{opcode}")
    }
}

impl SignedAndHwDtInstruction {
    fn add_offset(&self) -> bool {
        self.0.bit_is_set(23)
    }

    fn pre_indexed_addressing(&self) -> bool {
        self.0.bit_is_set(24)
    }

    fn rn(&self) -> REGISTER {
        (self.0 & 0x000F_0000) >> 16
    }

    fn rd(&self) -> REGISTER {
        (self.0 & 0x0000_F000) >> 12
    }

    fn offset(&self) -> Operand {
        if self.0.bit_is_set(22) {
            Operand::Immediate((self.0 & 0x0000_000F) | ((self.0 >> 4) & 0x0000_00F0))
        } else {
            Operand::Register(self.0 & 0x0000_000F)
        }
    }

    fn opcode(&self) -> SignedAndHwDtOpcodes {
        if self.0.bit_is_set(20) {
            match (self.0 >> 5) & 0b11 {
                0b01 => SignedAndHwDtOpcodes::Load(SignedAndHwDtLoadOpcodes::LDRH),
                0b10 => SignedAndHwDtOpcodes::Load(SignedAndHwDtLoadOpcodes::LDRSB),
                0b11 => SignedAndHwDtOpcodes::Load(SignedAndHwDtLoadOpcodes::LDRSH),
                _ => panic!(),
            }
        } else {
            SignedAndHwDtOpcodes::STRH
        }
    }

    fn write_back_address(&self, pre_indexed_addressing: bool) -> bool {
        !pre_indexed_addressing || self.0.bit_is_set(21)
    }
}

impl SignedAndHwDtOpcodes {
    pub fn execute(&self, cpu: &mut CPU, memory: &mut GBAMemory, rd: REGISTER, access_address: usize) -> CYCLES {
        let mut cycles = 0;
        match self {
            SignedAndHwDtOpcodes::Load(load_opcode) => {
                cycles += 1;
                let data: u32 = match load_opcode {
                    SignedAndHwDtLoadOpcodes::LDRH => {
                        let load = memory.readu16(access_address);
                        cycles += load.cycles;
                        load.data.into()
                    }
                    SignedAndHwDtLoadOpcodes::LDRSB => {
                        let load = memory.read(access_address);
                        cycles += load.cycles;
                        sign_extend(load.data.into(), 7)
                    }
                    SignedAndHwDtLoadOpcodes::LDRSH => {
                        let load = memory.readu16(access_address);
                        cycles += load.cycles;
                        sign_extend(load.data.into(), 15)
                    }
                };

                cpu.set_register(rd, data)
            }
            SignedAndHwDtOpcodes::STRH => {
                let mut data: WORD = cpu.get_register(rd);
                if rd == PC_REGISTER as u32 {
                    data += 4
                }
                cycles += memory.writeu16(access_address as usize, data as u16);
            }
        };

        cycles
    }
}

impl Execute for SignedAndHwDtInstruction {
    fn execute(self, cpu: &mut CPU, memory: &mut GBAMemory) -> CYCLES {
        let mut cycles = 0;
        let offset = match self.offset() {
            Operand::Register(reg) => cpu.get_register(reg),
            Operand::Immediate(imm) => imm,
        };

        let base_register = self.rn();
        let base_register_address = cpu.get_register(base_register);

        let offset_address = if self.add_offset() {
            base_register_address + offset
        } else {
            base_register_address - offset
        };

        let pre_indexed_addressing = self.pre_indexed_addressing();
        let address = if pre_indexed_addressing {
            offset_address
        } else {
            base_register_address
        } as usize;

        cycles += self.opcode().execute(cpu, memory, self.rd(), address);

        if self.write_back_address(pre_indexed_addressing) {
            cpu.set_register(base_register, offset_address);
        }

        cycles
    }
}

impl DecodeARMInstructionToString for SignedAndHwDtInstruction {
    fn instruction_to_string(&self, condition_code: &str) -> String {
        let opcode = match self.opcode() {
            SignedAndHwDtOpcodes::Load(SignedAndHwDtLoadOpcodes::LDRH) => "ldrh",
            SignedAndHwDtOpcodes::Load(SignedAndHwDtLoadOpcodes::LDRSB) => "ldrsb",
            SignedAndHwDtOpcodes::Load(SignedAndHwDtLoadOpcodes::LDRSH) => "ldrsh",
            SignedAndHwDtOpcodes::STRH => "strh",
        };

        let add_offset = if self.add_offset() { "" } else { "-" };

        let offset_address = match self.offset() {
            Operand::Immediate(0) => "".into(),
            operand => format!("{add_offset}{operand}"),
        };

        let access_address = if self.pre_indexed_addressing() {
            let write_back = if self.write_back_address(true) {
                "!"
            } else {
                ""
            };
            format!(
                "[{}{offset_address}]{write_back}",
                print_register(&self.rn())
            )
        } else {
            format!("[{}]{offset_address}", print_register(&self.rn()))
        };

        format!(
            "{opcode}{condition_code} {} {access_address}",
            print_register(&self.rd())
        )
    }
}

pub struct BlockDTInstruction(pub u32);

pub enum BlockDTOpcodes {
    STM,
    LDM,
}

impl BlockDTInstruction {
    fn rn(&self) -> REGISTER {
        (self.0 & 0x000F_0000) >> 16
    }

    fn pre_add(&self) -> bool {
        self.0.bit_is_set(24)
    }

    fn add_to_base(&self) -> bool {
        self.0.bit_is_set(23)
    }

    fn s_bit(&self) -> bool {
        self.0.bit_is_set(22)
    }

    fn write_back(&self) -> bool {
        self.0.bit_is_set(21)
    }

    fn opcode(&self) -> BlockDTOpcodes {
        match self.0.bit_is_set(20) {
            false => BlockDTOpcodes::STM,
            true => BlockDTOpcodes::LDM,
        }
    }

    fn register_list(&self) -> impl Iterator<Item = REGISTER> {
        RegisterList {
            list: self.0 & 0xFFFF,
            i: 0,
        }
    }
}

pub struct RegisterList {
    pub(crate) list: u32,
    pub(crate) i: u32,
}

impl Iterator for RegisterList {
    type Item = u32;

    fn next(&mut self) -> Option<Self::Item> {
        while !self.list.bit_is_set(self.i as u8) {
            if self.i == 16 {
                return None;
            }
            self.i += 1;
        }
        let register = self.i;
        self.i += 1;
        Some(register)
    }
}

impl Execute for BlockDTInstruction {
    fn execute(self, cpu: &mut CPU, memory: &mut GBAMemory) -> CYCLES {
        let mut cycles = 0;

        let base_address = cpu.get_register(self.rn()) as usize;
        cycles += cpu.advance_pipeline(memory);

        if self.s_bit() {
            todo!("Implement S bit");
        }
        match (self.opcode(), self.pre_add(), self.add_to_base()) {
            (BlockDTOpcodes::STM, true, true) => {
                let mut curr_address = base_address;
                for register in self.register_list() {
                    curr_address += size_of::<WORD>();
                    let data = cpu.get_register(register);
                    cycles += memory.writeu32(curr_address, data);
                }
                if self.write_back() {
                    cpu.set_register(self.rn(), curr_address as u32);
                }
            }
            (BlockDTOpcodes::STM, true, false) => {
                let base_address = base_address - self.register_list().count() * size_of::<WORD>();
                let mut curr_address = base_address;
                for register in self.register_list() {
                    let data = cpu.get_register(register);
                    cycles += memory.writeu32(curr_address, data);
                    curr_address += size_of::<WORD>();
                }
                if self.write_back() {
                    cpu.set_register(self.rn(), base_address as u32);
                }
            }
            (BlockDTOpcodes::STM, false, true) => {
                let mut curr_address = base_address;
                for register in self.register_list() {
                    let data = cpu.get_register(register);
                    cycles += memory.writeu32(curr_address, data);
                    curr_address += size_of::<WORD>();
                }
                if self.write_back() {
                    cpu.set_register(self.rn(), curr_address as u32);
                }
            }
            (BlockDTOpcodes::STM, false, false) => {
                let base_address = base_address - self.register_list().count() * size_of::<WORD>();
                let mut curr_address = base_address;
                for register in self.register_list() {
                    curr_address += size_of::<WORD>();
                    let data = cpu.get_register(register);
                    cycles += memory.writeu32(curr_address, data);
                }
                if self.write_back() {
                    cpu.set_register(self.rn(), base_address as u32);
                }
            }
            (BlockDTOpcodes::LDM, true, true) => {
                cycles += 1;
                let mut curr_address = base_address;
                for register in self.register_list() {
                    curr_address += size_of::<WORD>();
                    let memory_fetch = memory.readu32(curr_address);
                    cycles += memory_fetch.cycles;
                    let data = memory_fetch.data;
                    cpu.set_register(register, data);
                }
                if self.write_back() {
                    cpu.set_register(self.rn(), curr_address as u32);
                }
            }
            (BlockDTOpcodes::LDM, true, false) => {
                cycles += 1;
                let base_address = base_address - self.register_list().count() * size_of::<WORD>();
                let mut curr_address = base_address;
                for register in self.register_list() {
                    let memory_fetch = memory.readu32(curr_address);
                    curr_address += size_of::<WORD>();
                    cycles += memory_fetch.cycles;
                    let data = memory_fetch.data;
                    cpu.set_register(register, data);
                }
                if self.write_back() {
                    cpu.set_register(self.rn(), base_address as u32);
                }
            }
            (BlockDTOpcodes::LDM, false, true) => {
                cycles += 1;
                let mut curr_address = base_address;
                for register in self.register_list() {
                    let memory_fetch = memory.readu32(curr_address);
                    cycles += memory_fetch.cycles;
                    let data = memory_fetch.data;
                    cpu.set_register(register, data);
                    curr_address += size_of::<WORD>();
                }
                if self.write_back() {
                    cpu.set_register(self.rn(), curr_address as u32);
                }
            }
            (BlockDTOpcodes::LDM, false, false) => {
                cycles += 1;
                let base_address = base_address - self.register_list().count() * size_of::<WORD>();
                let mut curr_address = base_address;
                for register in self.register_list() {
                    curr_address += size_of::<WORD>();
                    let memory_fetch = memory.readu32(curr_address);
                    cycles += memory_fetch.cycles;
                    let data = memory_fetch.data;
                    cpu.set_register(register, data);
                }
                if self.write_back() {
                    cpu.set_register(self.rn(), base_address as u32);
                }
            }
        };

        cycles
    }
}

impl DecodeARMInstructionToString for BlockDTInstruction {
    fn instruction_to_string(&self, condition_code: &str) -> String {
        let opcode = match (self.opcode(), self.pre_add(), self.add_to_base()) {
            (BlockDTOpcodes::STM, true, true) => "stmib",
            (BlockDTOpcodes::STM, true, false) => "push",
            (BlockDTOpcodes::STM, false, true) => "stmia",
            (BlockDTOpcodes::STM, false, false) => "stmda",
            (BlockDTOpcodes::LDM, true, true) => "ldmib",
            (BlockDTOpcodes::LDM, true, false) => "ldmdb",
            (BlockDTOpcodes::LDM, false, true) => "pop",
            (BlockDTOpcodes::LDM, false, false) => "ldmda",
        };

        let mut rlist = Vec::new();

        for register in self.register_list() {
            rlist.push(print_register(&register));
        }

        let rlist = format!("{{{}}}", rlist.join(","));

        let write_back = if self.write_back() { "!" } else { "" };

        let s_bit = if self.s_bit() { "^" } else { "" };

        format!(
            "{opcode}{condition_code}{} {}, {rlist}{}",
            write_back,
            print_register(&self.rn()),
            s_bit
        )
    }
}

impl CPU {
    pub fn stmia_execution(
        &mut self,
        base_address: usize,
        register_list: &Vec<REGISTER>,
        writeback_register: Option<REGISTER>,
        memory: &mut GBAMemory,
    ) -> CYCLES {
        let mut cycles = 0;
        let mut curr_address = base_address;
        for register in register_list {
            let data = self.get_register(*register);
            cycles += memory.writeu32(curr_address, data);
            curr_address += size_of::<WORD>();
        }
        if let Some(reg) = writeback_register {
            self.set_register(reg, curr_address as u32);
        }
        self.set_executed_instruction(format_args!(
            "STMIA [{:#X}], {}",
            base_address,
            print_vec(register_list)
        ));
        cycles
    }

    pub fn ldmia_execution(
        &mut self,
        base_address: usize,
        register_list: &Vec<REGISTER>,
        writeback_register: Option<REGISTER>,
        memory: &mut GBAMemory,
    ) -> CYCLES {
        let mut cycles = 1;
        let mut curr_address = base_address;
        for register in register_list {
            let memory_fetch = memory.readu32(curr_address);
            cycles += memory_fetch.cycles;
            let data = memory_fetch.data;
            self.set_register(*register, data);
            curr_address += size_of::<WORD>();
        }
        if let Some(reg) = writeback_register {
            self.set_register(reg, curr_address as u32);
        }
        self.set_executed_instruction(format_args!(
            "LDMIA [{:#X}], {}",
            base_address,
            print_vec(register_list)
        ));
        cycles
    }

    pub fn stmdb_execution(
        &mut self,
        base_address: usize,
        register_list: &Vec<REGISTER>,
        writeback_register: Option<REGISTER>,
        memory: &mut GBAMemory,
    ) -> CYCLES {
        let base_address = base_address - register_list.len() * size_of::<WORD>();
        let cycles = self.stmia_execution(base_address, register_list, None, memory);
        self.set_executed_instruction(format_args!(
            "STMDB [{:#X}], {}",
            base_address,
            print_vec(register_list)
        ));
        if let Some(reg) = writeback_register {
            self.set_register(reg, base_address as u32);
        }

        cycles
    }
}

#[cfg(test)]
mod sdt_tests {
    use crate::gba::GBA;

    #[test]
    fn ldr_should_return_data_at_specified_address() {
        let mut gba = GBA::new_no_bios();

        let value = 0xFABCD321;
        let address: u32 = 0x3000200;

        let _res = gba.memory.writeu32(address as usize, value);

        gba.cpu.set_register(1, address);

        gba.cpu.prefetch[0] = Some(0xe5912000); // ldr r2, [r1]

        gba.step();
        gba.step();

        assert_eq!(gba.cpu.get_register(2), value);
    }

    #[test]
    fn ldr_should_return_data_at_specified_address_plus_offset() {
        let mut gba = GBA::new_no_bios();

        let value = 0xFABCD321;
        let address: u32 = 0x3000200;

        let _res = gba.memory.writeu32(address as usize + 8, value);

        gba.cpu.set_register(1, address);

        gba.cpu.prefetch[0] = Some(0xe5912008); // ldr r2, [r1, 8]

        gba.step();
        gba.step();

        assert_eq!(gba.cpu.get_register(2), value);
    }

    #[test]
    fn ldr_should_return_data_at_specified_address_minus_offset() {
        let mut gba = GBA::new_no_bios();

        let value = 0xFABCD321;
        let address: u32 = 0x3000208;

        let _res = gba.memory.writeu32(address as usize - 8, value);

        gba.cpu.set_register(1, address);

        gba.cpu.prefetch[0] = Some(0xe5112008); // ldr r2, [r1, -8]

        gba.step();
        gba.step();

        assert_eq!(gba.cpu.get_register(2), value);
    }

    #[test]
    fn ldr_should_return_data_at_lsl_shifted_address() {
        let mut gba = GBA::new_no_bios();

        let value = 0xFABCD321;
        let address: u32 = 0x3000200;

        let _res = gba.memory.writeu32(address as usize + 8, value);

        gba.cpu.set_register(1, address);
        gba.cpu.set_register(3, 4);

        gba.cpu.prefetch[0] = Some(0xe7912083); //  ldr r2, [r1, r3, lsl 1]

        gba.step();
        gba.step();

        assert_eq!(gba.cpu.get_register(2), value);
    }

    #[test]
    fn ldr_should_return_a_byte_at_address() {
        let mut gba = GBA::new_no_bios();

        let value = 0xFABCD321;
        let address: u32 = 0x3000200;

        let _res = gba.memory.writeu32(address as usize, value);

        gba.cpu.set_register(1, address);

        gba.cpu.prefetch[0] = Some(0xe5d12000); //  ldrb r2, [r1]

        gba.step();
        gba.step();

        assert_eq!(gba.cpu.get_register(2), value & 0xFF);

        gba.cpu.prefetch[0] = Some(0xe5d12001); //  ldrb r2, [r1, 1]

        gba.step();
        gba.step();

        assert_eq!(gba.cpu.get_register(2), (value & 0xFF00) >> 8);
    }

    #[test]
    fn ldr_should_rotate_value_when_not_word_aligned() {
        let mut gba = GBA::new_no_bios();

        let value = 0xFABCD321;
        let address: u32 = 0x3000202;

        let _res = gba.memory.writeu32(0x3000200, value);

        gba.cpu.set_register(1, address);

        gba.cpu.prefetch[0] = Some(0xe5912000); // ldr r2, [r1]

        gba.step();
        gba.step();

        assert_eq!(gba.cpu.get_register(2), 0xD321FABC);

        gba.cpu.set_register(1, 0x3000203);

        gba.cpu.prefetch[0] = Some(0xe5912000); // ldr r2, [r1]

        gba.step();
        gba.step();
        assert_eq!(gba.cpu.get_register(2), 0xBCD321FA);
    }

    #[test]
    fn ldr_should_writeback_when_post_indexed() {
        let mut gba = GBA::new_no_bios();

        let value = 0xFABCD321;
        let address: u32 = 0x3000200;

        let _res = gba.memory.writeu32(address as usize, value);

        gba.cpu.set_register(1, address);

        gba.cpu.prefetch[0] = Some(0xe4912004); // ldr r2, [r1], 4

        gba.step();
        gba.step();

        assert_eq!(gba.cpu.get_register(2), value);
        assert_eq!(gba.cpu.get_register(1), address + 4);
    }

    #[test]
    fn str_should_store_word_at_memory_address() {
        let mut gba = GBA::new_no_bios();

        let value = 0xFABCD321;
        let address: u32 = 0x3000200;

        gba.cpu.set_register(1, address);
        gba.cpu.set_register(2, value);

        gba.cpu.prefetch[0] = Some(0xe5812000); // str r2, [r1]

        gba.step();
        gba.step();

        let stored_value = gba.memory.readu32(address as usize).data;

        assert_eq!(value, stored_value);
    }

    #[test]
    fn str_should_store_byte_at_address() {
        let mut gba = GBA::new_no_bios();

        let value: u8 = 0x21;
        let address: u32 = 0x3000203;

        gba.cpu.set_register(1, address);
        gba.cpu.set_register(2, value.into());

        gba.cpu.prefetch[0] = Some(0xe5c12000); // strb r2, [r1]

        gba.step();
        gba.step();

        let stored_value = gba.memory.read(address as usize).data;

        assert_eq!(value, stored_value);
    }

    #[test]
    fn strh_should_store_hw_at_address() {
        let mut gba = GBA::new_no_bios();

        let value: u16 = 0x21;
        let address: u32 = 0x3000200;

        gba.cpu.set_register(1, address);
        gba.cpu.set_register(3, value.into());

        gba.cpu.prefetch[0] = Some(0xe1c130b0); // strh r3, [r1]

        gba.step();
        gba.step();

        let stored_value = gba.memory.readu16(address as usize).data;

        assert_eq!(value as u16, stored_value);
    }

    #[test]
    fn strh_should_only_store_bottom_half_of_register() {
        let mut gba = GBA::new_no_bios();

        let value: u32 = 0x1234_5678;
        let address: u32 = 0x3000200;

        gba.cpu.set_register(1, address);
        gba.cpu.set_register(3, value);

        gba.cpu.prefetch[0] = Some(0xe1c130b0); // strh r3, [r1]

        gba.step();
        gba.step();

        let stored_value = gba.memory.readu32(address as usize).data;

        assert_eq!(value & 0x0000_FFFF, stored_value);
    }

    #[test]
    fn ldrh_should_only_load_bottom_half_of_register() {
        let mut gba = GBA::new_no_bios();

        let value = 0xFABCD321;
        let address: u32 = 0x3000200;

        let _res = gba.memory.writeu32(address as usize, value);

        gba.cpu.set_register(1, address);
        gba.cpu.prefetch[0] = Some(0xe1d130b0); // ldrh r3, [r1]

        gba.step();
        gba.step();

        assert_eq!(gba.cpu.get_register(3), value & 0x0000_FFFF);
    }

    #[test]
    fn ldrsh_should_return_a_signed_hw() {
        let mut gba = GBA::new_no_bios();

        let value = 0x0000_FABC;
        let address: u32 = 0x3000200;

        let _res = gba.memory.writeu32(address as usize, value);

        gba.cpu.set_register(1, address);
        gba.cpu.prefetch[0] = Some(0xe1d130f0); // ldrsh r3, [r1]

        gba.step();
        gba.step();

        assert_eq!(gba.cpu.get_register(3), value | 0xFFFF_0000);
    }

    #[test]
    fn ldrsh_should_return_a_signed_byte() {
        let mut gba = GBA::new_no_bios();

        let value = 0x0000_0081;
        let address: u32 = 0x3000200;

        let _res = gba.memory.writeu32(address as usize, value);

        gba.cpu.set_register(1, address);
        gba.cpu.prefetch[0] = Some(0xe1d130d0); // ldrsb r3, [r1]

        gba.step();
        gba.step();

        assert_eq!(gba.cpu.get_register(3), value | 0xFFFF_FF00);
    }

    #[test]
    fn ldm_should_load_multiple_registers() {
        let mut gba = GBA::new_no_bios();

        let value = 0x0000_0081;
        let address: u32 = 0x3000200;

        gba.cpu.set_register(5, address);

        gba.memory.writeu32(address as usize, value);

        gba.memory.writeu32(address as usize + 4, 0x55);

        gba.cpu.prefetch[0] = Some(0xe8950003); // ldmia r5, {r0, r1}

        gba.step();
        gba.step();

        assert_eq!(gba.cpu.get_register(0), value);
        assert_eq!(gba.cpu.get_register(1), 0x55);
    }

    #[test]
    fn ldmib_should_load_multiple_registers_and_modify_base_register() {
        let mut gba = GBA::new_no_bios();

        let value = 0x0000_0081;
        let address: u32 = 0x3000200;

        gba.cpu.set_register(5, address);

        gba.memory.writeu32(address as usize + 4, value);

        gba.memory.writeu32(address as usize + 8, 0x55);

        gba.cpu.prefetch[0] = Some(0xe9b500c0); // ldmib r5!, {r6, r7}

        gba.step();
        gba.step();

        assert_eq!(gba.cpu.get_register(6), value);
        assert_eq!(gba.cpu.get_register(7), 0x55);
        assert_eq!(gba.cpu.get_register(5), address + 8);
    }

    #[test]
    fn ldmda_should_load_multiple_registers_and_modify_base_register() {
        let mut gba = GBA::new_no_bios();

        let value = 0x0000_0081;
        let address: u32 = 0x3000200;

        gba.cpu.set_register(5, address);

        gba.memory.writeu32(address as usize, value);

        gba.memory.writeu32(address as usize - 4, 0x55);

        gba.cpu.prefetch[0] = Some(0xe83500c0); // ldmda r5!, {r6, r7}

        gba.step();
        gba.step();

        assert_eq!(gba.cpu.get_register(6), 0x55);
        assert_eq!(gba.cpu.get_register(7), value);
        assert_eq!(gba.cpu.get_register(5), address - 8);
    }

    #[test]
    fn ldmdb_should_load_multiple_registers_and_modify_base_register() {
        let mut gba = GBA::new_no_bios();

        let value = 0x0000_0081;
        let address: u32 = 0x3000200;

        gba.cpu.set_register(5, address);

        gba.memory.writeu32(address as usize - 4, value);

        gba.memory.writeu32(address as usize - 8, 0x55);

        gba.cpu.prefetch[0] = Some(0xe93500c0); // ldmdb r5!, {r6, r7}

        gba.step();
        gba.step();

        assert_eq!(gba.cpu.get_register(6), 0x55);
        assert_eq!(gba.cpu.get_register(7), value);
        assert_eq!(gba.cpu.get_register(5), address - 8);
    }

    #[test]
    fn stm_should_store_multiple_registers() {
        let mut gba = GBA::new_no_bios();

        let address: u32 = 0x3000200;

        gba.cpu.set_register(5, address);
        gba.cpu.set_register(6, 123);
        gba.cpu.set_register(7, 456);

        gba.cpu.prefetch[0] = Some(0xe88500c0); // stm r5, {r6, r7}

        gba.step();
        gba.step();

        assert_eq!(gba.memory.readu32(address as usize).data, 123);
        assert_eq!(gba.memory.readu32(address as usize + 4).data, 456);
    }

    #[test]
    fn stmib_should_store_multiple_registers_and_writeback() {
        let mut gba = GBA::new_no_bios();

        let address: u32 = 0x3000200;

        gba.cpu.set_register(5, address);
        gba.cpu.set_register(6, 123);
        gba.cpu.set_register(7, 456);

        gba.cpu.prefetch[0] = Some(0xe9a500c0); // stmib r5!, {r6, r7}

        gba.step();
        gba.step();

        assert_eq!(gba.memory.readu32(address as usize + 4).data, 123);
        assert_eq!(gba.memory.readu32(address as usize + 8).data, 456);
        assert_eq!(gba.cpu.get_register(5), address + 8);
    }

    #[test]
    fn stmdb_should_store_multiple_registers_and_writeback() {
        let mut gba = GBA::new_no_bios();

        let address: u32 = 0x3000200;

        gba.cpu.set_register(5, address);
        gba.cpu.set_register(6, 123);
        gba.cpu.set_register(7, 456);

        gba.cpu.prefetch[0] = Some(0xe92500c0); // stmdb r5!, {r6, r7}

        gba.step();
        gba.step();

        assert_eq!(gba.memory.readu32((address - 4) as usize).data, 456);
        assert_eq!(gba.memory.readu32((address - 8) as usize).data, 123);
        assert_eq!(gba.cpu.get_register(5), address - 8);
    }

    #[test]
    fn stmda_should_store_multiple_registers_and_writeback() {
        let mut gba = GBA::new_no_bios();

        let address: u32 = 0x3000200;

        gba.cpu.set_register(5, address);
        gba.cpu.set_register(6, 123);
        gba.cpu.set_register(7, 456);

        gba.cpu.prefetch[0] = Some(0xe82500c0); // stmda r5!, {r6, r7}

        gba.step();
        gba.step();

        assert_eq!(gba.memory.readu32((address) as usize).data, 456);
        assert_eq!(gba.memory.readu32((address - 4) as usize).data, 123);
        assert_eq!(gba.cpu.get_register(5), address - 8);
    }
}
