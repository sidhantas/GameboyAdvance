use std::{intrinsics::offset, mem::size_of};

use crate::{
    arm7tdmi::{
        arm::alu::{Shift, ShiftType},
        cpu::{CPUMode, FlagsRegister, CPU, PC_REGISTER},
        instruction_table::{Execute, Operand},
    },
    memory::memory::GBAMemory,
    types::{CYCLES, REGISTER, WORD},
    utils::{
        bits::{sign_extend, Bits},
        utils::print_vec,
    },
};

pub struct SdtInstruction(pub u32);

enum SdtOpcode {
    LDR,
    LDRB,
    STR,
    STRB,
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
            0b00 => SdtOpcode::STR,
            0b01 => SdtOpcode::STRB,
            0b10 => SdtOpcode::LDR,
            0b11 => SdtOpcode::LDRB,
            _ => unreachable!(),
        }
    }

    fn offset(&self) -> SdtOffset {
        if self.0.bit_is_set(25) {
            let shift_amount = (self.0 & 0x0000_0F80) >> 7;
            if shift_amount == 0 {
                return match (self.0 >> 5) & 0b11 {
                    0b00 => SdtOffset::ShiftedRegister(
                        self.0 & 0x7,
                        Shift(ShiftType::LSL, Operand::Immediate(0)),
                    ),
                    0b01 => SdtOffset::ShiftedRegister(
                        self.0 & 0x7,
                        Shift(ShiftType::LSR, Operand::Immediate(32)),
                    ),
                    0b10 => SdtOffset::ShiftedRegister(
                        self.0 & 0x7,
                        Shift(ShiftType::ASR, Operand::Immediate(32)),
                    ),
                    0b11 => SdtOffset::ShiftedRegister(
                        self.0 & 0x7,
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
                    self.0 & 0x7,
                    Shift(shift_type, Operand::Immediate(shift_amount)),
                );
            }
        } else {
            SdtOffset::Imm(self.0 & 0x0000_0fff)
        }
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

        let offset_address;
        if self.add_offset() {
            offset_address = base_register_address + offset;
        } else {
            offset_address = base_register_address - offset;
        }

        let access_address = if self.pre_indexed_addressing() {
            offset_address
        } else {
            base_register_address
        } as usize;

        let memory_fetch = match self.opcode() {
            SdtOpcode::LDR => memory.readu32(access_address),
            SdtOpcode::LDRB => memory.read(access_address).into(),
            SdtOpcode::STR => todo!(),
            SdtOpcode::STRB => todo!(),
        };
        cycles
    }
}

impl CPU {
    pub fn sdt_instruction_execution(
        &mut self,
        instruction: u32,
        memory: &mut GBAMemory,
    ) -> CYCLES {
        let mut cycles = 0;
        let offset;
        let offset_address;

        let use_register_offset: bool = instruction.bit_is_set(25);
        let add_offset: bool = instruction.bit_is_set(23);
        let pre_indexed_addressing: bool = instruction.bit_is_set(24);
        let write_back_address: bool = !pre_indexed_addressing || instruction.bit_is_set(21);
        let rd = (instruction & 0x0000_F000) >> 12;
        let force_non_privileged_access: bool =
            pre_indexed_addressing && instruction.bit_is_set(21);
        let is_byte_transfer: bool = instruction.bit_is_set(22);

        if use_register_offset {
            let offset_register = instruction & 0x0000_000F;
            let offset_register_value = self.get_register(offset_register);
            let shift_amount = (instruction & 0x0000_0F80) >> 7;
            offset = self.decode_shifted_register(
                instruction,
                shift_amount,
                offset_register_value,
                false,
            );
        } else {
            offset = instruction & 0x0000_0fff;
        }

        let base_register = (instruction & 0x000F_0000) >> 16;
        let base_register_address = self.get_register(base_register);

        if add_offset {
            offset_address = base_register_address + offset;
        } else {
            offset_address = base_register_address - offset;
        }

        cycles += self.advance_pipeline(memory);

        let access_address = if pre_indexed_addressing {
            offset_address
        } else {
            base_register_address
        };

        let old_cpu_mode = self.get_cpu_mode();
        if force_non_privileged_access {
            self.set_mode(CPUMode::USER);
        }

        cycles += if instruction.bit_is_set(20) {
            self.ldr_instruction_execution(rd, access_address, is_byte_transfer, memory)
        } else {
            self.str_instruction_execution(rd, access_address, is_byte_transfer, memory)
        };

        self.set_mode(old_cpu_mode);

        if write_back_address {
            self.set_register(base_register, offset_address);
        }

        cycles
    }

    pub fn str_instruction_execution(
        &mut self,
        rd: REGISTER,
        address: u32,
        byte_transfer: bool,
        memory: &mut GBAMemory,
    ) -> CYCLES {
        let data: WORD = self.get_register(rd);
        if byte_transfer {
            self.set_executed_instruction(format_args!("STRB {} [{:#X}]", rd, address));
            memory.write(address as usize, data as u8)
        } else {
            self.set_executed_instruction(format_args!("STR {} [{:#X}]", rd, address));
            memory.writeu32(address as usize, data)
        }
    }

    pub fn ldr_instruction_execution(
        &mut self,
        rd: REGISTER,
        address: u32,
        byte_transfer: bool,
        memory: &mut GBAMemory,
    ) -> CYCLES {
        let mut cycles = 1;
        let data = {
            let memory_fetch = if byte_transfer {
                self.set_executed_instruction(format_args!("LDRB {} [{:#X}]", rd, address));
                memory.read(address as usize).into()
            } else {
                self.set_executed_instruction(format_args!("LDR {} [{:#X}]", rd, address));
                memory.readu32(address as usize)
            };
            cycles += memory_fetch.cycles;

            memory_fetch.data
        };

        self.set_register(rd, data);
        if rd as usize == PC_REGISTER {
            cycles += self.flush_pipeline(memory);
        }

        cycles
    }

    pub fn hw_or_signed_data_transfer(
        &mut self,
        instruction: u32,
        memory: &mut GBAMemory,
    ) -> CYCLES {
        let pre_indexed_addressing = instruction.bit_is_set(24);
        let add_offset = instruction.bit_is_set(23);
        let use_immediate_offset = instruction.bit_is_set(22);
        let write_back_address: bool = !pre_indexed_addressing || instruction.bit_is_set(21);
        let rd = (instruction & 0x0000_F000) >> 12;

        let mut cycles = 0;
        let offset;
        let offset_address;

        if use_immediate_offset {
            offset = instruction & 0x0000_000F;
        } else {
            let offset_register = instruction & 0x0000_000F;
            offset = self.get_register(offset_register);
        }

        let base_register = (instruction & 0x000F_0000) >> 16;
        let base_register_address = self.get_register(base_register);

        if add_offset {
            offset_address = base_register_address + offset;
        } else {
            offset_address = base_register_address - offset;
        }

        cycles += self.advance_pipeline(memory);

        let access_address = if pre_indexed_addressing {
            offset_address
        } else {
            base_register_address
        };

        cycles += if instruction.bit_is_set(20) {
            let opcode = (instruction & 0x0000_0060) >> 5;
            match opcode {
                0b01 => self.ldrh_execution(rd, access_address, memory),
                0b10 => self.ldrsb_execution(rd, access_address, memory),
                0b11 => self.ldrsh_execution(rd, access_address, memory),
                _ => panic!("Invalid Opcode"),
            }
        } else {
            self.strh_execution(rd, access_address, memory)
        };

        if write_back_address {
            self.set_register(base_register, offset_address);
        }

        cycles
    }

    pub fn strh_execution(&mut self, rd: REGISTER, address: u32, memory: &mut GBAMemory) -> CYCLES {
        let data: WORD = self.get_register(rd);
        let cycles = { memory.writeu16(address as usize, data as u16) };
        self.set_executed_instruction(format_args!("STRH {} [{:#X}]", rd, address));

        cycles
    }

    pub fn ldrsh_execution(
        &mut self,
        rd: REGISTER,
        address: u32,
        memory: &mut GBAMemory,
    ) -> CYCLES {
        let mut cycles = 1;
        let memory_fetch = { memory.readu16(address as usize) };

        cycles += memory_fetch.cycles;
        let data = memory_fetch.data;

        self.set_register(rd, sign_extend(data.into(), 15));
        if rd as usize == PC_REGISTER {
            cycles += self.flush_pipeline(memory);
        }
        self.set_executed_instruction(format_args!("LDRH {} [{:#X}]", rd, address));

        cycles
    }

    pub fn ldrsb_execution(
        &mut self,
        rd: REGISTER,
        address: u32,
        memory: &mut GBAMemory,
    ) -> CYCLES {
        let mut cycles = 1;
        let memory_fetch = { memory.read(address as usize) };

        cycles += memory_fetch.cycles;
        let data = memory_fetch.data;

        self.set_register(rd, sign_extend(data.into(), 7));
        if rd as usize == PC_REGISTER {
            cycles += self.flush_pipeline(memory);
        }
        self.set_executed_instruction(format_args!("LDRH {} [{:#X}]", rd, address));

        cycles
    }

    pub fn ldrh_execution(&mut self, rd: REGISTER, address: u32, memory: &mut GBAMemory) -> CYCLES {
        let mut cycles = 1;
        let memory_fetch = { memory.readu16(address as usize) };

        cycles += memory_fetch.cycles;
        let data = memory_fetch.data;

        self.set_register(rd, data.into());
        if rd as usize == PC_REGISTER {
            cycles += self.flush_pipeline(memory);
        }
        self.set_executed_instruction(format_args!("LDRH {} [{:#X}]", rd, address));

        cycles
    }

    pub fn block_dt_execution(&mut self, instruction: u32, memory: &mut GBAMemory) -> CYCLES {
        let mut cycles = 0;
        if instruction.bit_is_set(22) {
            todo!("Implement S bit");
        }

        let opcode = (instruction & 0x01F0_0000) >> 20;

        let base_register = (instruction & 0x000F_0000) >> 16;
        let base_address = self.get_register(base_register) as usize;

        let mut register_list: Vec<REGISTER> = Vec::with_capacity(15);
        for i in 0..16 {
            if instruction.bit_is_set(i as u8) {
                register_list.push(i as u32);
            }
        }

        cycles += self.advance_pipeline(memory);

        cycles += match opcode {
            0b00000 => self.stmda_execution(base_address, &register_list, None, memory),
            0b00001 => self.ldmda_execution(base_address, &register_list, None, memory),
            0b00010 => {
                self.stmda_execution(base_address, &register_list, Some(base_register), memory)
            }
            0b00011 => {
                self.ldmda_execution(base_address, &register_list, Some(base_register), memory)
            }
            0b01000 => self.stmia_execution(base_address, &register_list, None, memory),
            0b01001 => self.ldmia_execution(base_address, &register_list, None, memory),
            0b01010 => {
                self.stmia_execution(base_address, &register_list, Some(base_register), memory)
            }
            0b01011 => {
                self.ldmia_execution(base_address, &register_list, Some(base_register), memory)
            }
            0b10000 => self.stmdb_execution(base_address, &register_list, None, memory),
            0b10001 => self.ldmdb_execution(base_address, &register_list, None, memory),
            0b10010 => {
                self.stmdb_execution(base_address, &register_list, Some(base_register), memory)
            }
            0b10011 => {
                self.ldmdb_execution(base_address, &register_list, Some(base_register), memory)
            }
            0b11000 => self.stmib_execution(base_address, &register_list, None, memory),
            0b11001 => self.ldmib_execution(base_address, &register_list, None, memory),
            0b11010 => {
                self.stmib_execution(base_address, &register_list, Some(base_register), memory)
            }
            0b11011 => {
                self.ldmib_execution(base_address, &register_list, Some(base_register), memory)
            }
            _ => todo!(),
        };

        cycles
    }

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

    fn stmib_execution(
        &mut self,
        base_address: usize,
        register_list: &Vec<REGISTER>,
        writeback_register: Option<REGISTER>,
        memory: &mut GBAMemory,
    ) -> CYCLES {
        let mut cycles = 0;
        let mut curr_address = base_address;
        for register in register_list {
            curr_address += size_of::<WORD>();
            let data = self.get_register(*register);
            cycles += memory.writeu32(curr_address, data);
        }
        if let Some(reg) = writeback_register {
            self.set_register(reg, curr_address as u32);
        }
        self.set_executed_instruction(format_args!(
            "STMIB [{:#X}], {}",
            base_address,
            print_vec(register_list)
        ));
        cycles
    }

    fn ldmib_execution(
        &mut self,
        base_address: usize,
        register_list: &Vec<REGISTER>,
        writeback_register: Option<REGISTER>,
        memory: &mut GBAMemory,
    ) -> CYCLES {
        let mut cycles = 1;
        let mut curr_address = base_address;
        for register in register_list {
            curr_address += size_of::<WORD>();
            let memory_fetch = memory.readu32(curr_address);
            cycles += memory_fetch.cycles;
            let data = memory_fetch.data;
            self.set_register(*register, data);
        }
        if let Some(reg) = writeback_register {
            self.set_register(reg, curr_address as u32);
        }
        self.set_executed_instruction(format_args!(
            "LDMIB [{:#X}], {}",
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

    fn ldmdb_execution(
        &mut self,
        base_address: usize,
        register_list: &Vec<REGISTER>,
        writeback_register: Option<REGISTER>,
        memory: &mut GBAMemory,
    ) -> CYCLES {
        let base_address = base_address - register_list.len() * size_of::<WORD>();
        let cycles = self.ldmia_execution(base_address, register_list, None, memory);
        self.set_executed_instruction(format_args!(
            "LDMDB [{:#X}], {}",
            base_address,
            print_vec(register_list)
        ));
        if let Some(reg) = writeback_register {
            self.set_register(reg, base_address as u32);
        }

        cycles
    }

    fn stmda_execution(
        &mut self,
        base_address: usize,
        register_list: &Vec<REGISTER>,
        writeback_register: Option<REGISTER>,
        memory: &mut GBAMemory,
    ) -> CYCLES {
        let base_address = base_address - register_list.len() * size_of::<WORD>();
        let cycles = self.stmib_execution(base_address, register_list, None, memory);
        self.set_executed_instruction(format_args!(
            "STMDA [{:#X}], {}",
            base_address,
            print_vec(register_list)
        ));

        if let Some(reg) = writeback_register {
            self.set_register(reg, base_address as u32);
        }

        cycles
    }

    fn ldmda_execution(
        &mut self,
        base_address: usize,
        register_list: &Vec<REGISTER>,
        writeback_register: Option<REGISTER>,
        memory: &mut GBAMemory,
    ) -> CYCLES {
        let base_address = base_address - register_list.len() * size_of::<WORD>();
        let cycles = self.ldmib_execution(base_address, register_list, None, memory);
        self.set_executed_instruction(format_args!(
            "LDMDA [{:#X}], {}",
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
    use crate::{arm7tdmi::cpu::CPU, gba::GBA, memory::memory::GBAMemory};

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
