use std::mem::size_of;

use crate::{
    memory::memory::MemoryBus,
    types::{CYCLES, REGISTER, WORD},
    utils::{
        bits::{sign_extend, Bits},
        utils::print_vec,
    },
};

use super::cpu::{CPUMode, CPU, PC_REGISTER};

impl CPU {
    pub fn sdt_instruction_execution(&mut self, instruction: u32) -> CYCLES {
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

        cycles += self.advance_pipeline();

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
            self.ldr_instruction_execution(rd, access_address, is_byte_transfer)
        } else {
            self.str_instruction_execution(rd, access_address, is_byte_transfer)
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
    ) -> CYCLES {
        let data: WORD = self.get_register(rd);
        if byte_transfer {
            self.set_executed_instruction(format_args!("STRB {} [{:#X}]", rd, address));
            self.memory.write(address as usize, data as u8)
        } else {
            self.set_executed_instruction(format_args!("STR {} [{:#X}]", rd, address));
            self.memory.writeu32(address as usize, data)
        }
    }

    pub fn ldr_instruction_execution(
        &mut self,
        rd: REGISTER,
        address: u32,
        byte_transfer: bool,
    ) -> CYCLES {
        let mut cycles = 1;
        let data = {
            let memory_fetch = if byte_transfer {
                self.set_executed_instruction(format_args!("LDRB {} [{:#X}]", rd, address));
                self.memory.read(address as usize).into()
            } else {
                self.set_executed_instruction(format_args!("LDR {} [{:#X}]", rd, address));
                self.memory.readu32(address as usize)
            };
            cycles += memory_fetch.cycles;

            memory_fetch.data
        };

        self.set_register(rd, data);
        if rd as usize == PC_REGISTER {
            cycles += self.flush_pipeline();
        }

        cycles
    }

    pub fn hw_or_signed_data_transfer(&mut self, instruction: u32) -> CYCLES {
        let pre_indexed_addressing = instruction.bit_is_set(24);
        let add_offset = instruction.bit_is_set(23);
        let use_immediate_offset = instruction.bit_is_set(22);
        let write_back_address: bool = !pre_indexed_addressing || instruction.bit_is_set(21);
        let rd = (instruction & 0x0000_F000) >> 12;

        let mut cycles = 1;
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

        cycles += self.advance_pipeline();

        let access_address = if pre_indexed_addressing {
            offset_address
        } else {
            base_register_address
        };

        cycles += if instruction.bit_is_set(20) {
            let opcode = (instruction & 0x0000_0060) >> 5;
            match opcode {
                0b01 => self.ldrh_execution(rd, access_address),
                0b10 => self.ldrsb_execution(rd, access_address),
                0b11 => self.ldrsh_execution(rd, access_address),
                _ => panic!("Invalid Opcode"),
            }
        } else {
            self.strh_execution(rd, access_address)
        };

        if write_back_address {
            self.set_register(base_register, offset_address);
        }

        cycles
    }

    pub fn strh_execution(&mut self, rd: REGISTER, address: u32) -> CYCLES {
        let data: WORD = self.get_register(rd);
        let cycles = { self.memory.writeu16(address as usize, data as u16) };
        self.set_executed_instruction(format_args!("STRH {} [{:#X}]", rd, address));

        cycles
    }

    pub fn ldrsh_execution(&mut self, rd: REGISTER, address: u32) -> CYCLES {
        let mut cycles = 0;
        let memory_fetch = { self.memory.readu16(address as usize) };

        cycles += memory_fetch.cycles;
        let data = memory_fetch.data;

        self.set_register(rd, sign_extend(data.into(), 15));
        if rd as usize == PC_REGISTER {
            cycles += self.flush_pipeline();
        }
        self.set_executed_instruction(format_args!("LDRH {} [{:#X}]", rd, address));

        cycles
    }

    pub fn ldrsb_execution(&mut self, rd: REGISTER, address: u32) -> CYCLES {
        let mut cycles = 0;
        let memory_fetch = { self.memory.read(address as usize) };

        cycles += memory_fetch.cycles;
        let data = memory_fetch.data;

        self.set_register(rd, sign_extend(data.into(), 7));
        if rd as usize == PC_REGISTER {
            cycles += self.flush_pipeline();
        }
        self.set_executed_instruction(format_args!("LDRH {} [{:#X}]", rd, address));

        cycles
    }

    pub fn ldrh_execution(&mut self, rd: REGISTER, address: u32) -> CYCLES {
        let mut cycles = 1;
        let memory_fetch = { self.memory.readu16(address as usize) };

        cycles += memory_fetch.cycles;
        let data = memory_fetch.data;

        self.set_register(rd, data.into());
        if rd as usize == PC_REGISTER {
            cycles += self.flush_pipeline();
        }
        self.set_executed_instruction(format_args!("LDRH {} [{:#X}]", rd, address));

        cycles
    }

    pub fn block_dt_execution(&mut self, instruction: u32) -> CYCLES {
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

        cycles += self.advance_pipeline();

        cycles += match opcode {
            0b00000 => self.stmda_execution(base_address, &register_list, None),
            0b00001 => self.ldmda_execution(base_address, &register_list, None),
            0b00010 => self.stmda_execution(base_address, &register_list, Some(base_register)),
            0b00011 => self.ldmda_execution(base_address, &register_list, Some(base_register)),
            0b01000 => self.stmia_execution(base_address, &register_list, None),
            0b01001 => self.ldmia_execution(base_address, &register_list, None),
            0b01010 => self.stmia_execution(base_address, &register_list, Some(base_register)),
            0b01011 => self.ldmia_execution(base_address, &register_list, Some(base_register)),
            0b10000 => self.stmdb_execution(base_address, &register_list, None),
            0b10001 => self.ldmdb_execution(base_address, &register_list, None),
            0b10010 => self.stmdb_execution(base_address, &register_list, Some(base_register)),
            0b10011 => self.ldmdb_execution(base_address, &register_list, Some(base_register)),
            0b11000 => self.stmib_execution(base_address, &register_list, None),
            0b11001 => self.ldmib_execution(base_address, &register_list, None),
            0b11010 => self.stmib_execution(base_address, &register_list, Some(base_register)),
            0b11011 => self.ldmib_execution(base_address, &register_list, Some(base_register)),
            _ => todo!(),
        };

        cycles
    }

    pub fn stmia_execution(
        &mut self,
        base_address: usize,
        register_list: &Vec<REGISTER>,
        writeback_register: Option<REGISTER>,
    ) -> CYCLES {
        let mut cycles = 0;
        let mut curr_address = base_address;
        for register in register_list {
            let data = self.get_register(*register);
            cycles += self.memory.writeu32(curr_address, data);
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
    ) -> CYCLES {
        let mut cycles = 1;
        let mut curr_address = base_address;
        for register in register_list {
            let memory_fetch = self.memory.readu32(curr_address);
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
    ) -> CYCLES {
        let mut cycles = 0;
        let mut curr_address = base_address;
        for register in register_list {
            curr_address += size_of::<WORD>();
            let data = self.get_register(*register);
            cycles += self.memory.writeu32(curr_address, data);
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
    ) -> CYCLES {
        let mut cycles = 1;
        let mut curr_address = base_address;
        for register in register_list {
            curr_address += size_of::<WORD>();
            let memory_fetch = self.memory.readu32(curr_address);
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
    ) -> CYCLES {
        let base_address = base_address - register_list.len() * size_of::<WORD>();
        let cycles = self.stmia_execution(base_address, register_list, None);
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
    ) -> CYCLES {
        let base_address = base_address - register_list.len() * size_of::<WORD>();
        let cycles = self.ldmia_execution(base_address, register_list, None);
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
    ) -> CYCLES {
        let base_address = base_address - register_list.len() * size_of::<WORD>();
        let cycles = self.stmib_execution(base_address, register_list, None);
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
    ) -> CYCLES {
        let base_address = base_address - register_list.len() * size_of::<WORD>();
        let cycles = self.ldmib_execution(base_address, register_list, None);
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
    use crate::{
        arm7tdmi::cpu::CPU,
        memory::memory::{GBAMemory, MemoryBus},
    };

    #[test]
    fn ldr_should_return_data_at_specified_address() {
        let memory = GBAMemory::new();

        let mut cpu = CPU::new(memory);

        let value = 0xFABCD321;
        let address: u32 = 0x3000200;

        let _res = cpu.memory.writeu32(address as usize, value);

        cpu.set_register(1, address);

        cpu.prefetch[0] = Some(0xe5912000); // ldr r2, [r1]

        cpu.execute_cpu_cycle();
        cpu.execute_cpu_cycle();

        assert_eq!(cpu.get_register(2), value);
    }

    #[test]
    fn ldr_should_return_data_at_specified_address_plus_offset() {
        let memory = GBAMemory::new();

        let mut cpu = CPU::new(memory);

        let value = 0xFABCD321;
        let address: u32 = 0x3000200;

        let _res = cpu.memory.writeu32(address as usize + 8, value);

        cpu.set_register(1, address);

        cpu.prefetch[0] = Some(0xe5912008); // ldr r2, [r1, 8]

        cpu.execute_cpu_cycle();
        cpu.execute_cpu_cycle();

        assert_eq!(cpu.get_register(2), value);
    }

    #[test]
    fn ldr_should_return_data_at_specified_address_minus_offset() {
        let memory = GBAMemory::new();

        let mut cpu = CPU::new(memory);

        let value = 0xFABCD321;
        let address: u32 = 0x3000208;

        let _res = cpu.memory.writeu32(address as usize - 8, value);

        cpu.set_register(1, address);

        cpu.prefetch[0] = Some(0xe5112008); // ldr r2, [r1, -8]

        cpu.execute_cpu_cycle();
        cpu.execute_cpu_cycle();

        assert_eq!(cpu.get_register(2), value);
    }

    #[test]
    fn ldr_should_return_data_at_lsl_shifted_address() {
        let memory = GBAMemory::new();

        let mut cpu = CPU::new(memory);

        let value = 0xFABCD321;
        let address: u32 = 0x3000200;

        let _res = cpu.memory.writeu32(address as usize + 8, value);

        cpu.set_register(1, address);
        cpu.set_register(3, 4);

        cpu.prefetch[0] = Some(0xe7912083); //  ldr r2, [r1, r3, lsl 1]

        cpu.execute_cpu_cycle();
        cpu.execute_cpu_cycle();

        assert_eq!(cpu.get_register(2), value);
    }

    #[test]
    fn ldr_should_return_a_byte_at_address() {
        let memory = GBAMemory::new();

        let mut cpu = CPU::new(memory);

        let value = 0xFABCD321;
        let address: u32 = 0x3000200;

        let _res = cpu.memory.writeu32(address as usize, value);

        cpu.set_register(1, address);

        cpu.prefetch[0] = Some(0xe5d12000); //  ldrb r2, [r1]

        cpu.execute_cpu_cycle();
        cpu.execute_cpu_cycle();

        assert_eq!(cpu.get_register(2), value & 0xFF);

        cpu.prefetch[0] = Some(0xe5d12001); //  ldrb r2, [r1, 1]

        cpu.execute_cpu_cycle();
        cpu.execute_cpu_cycle();

        assert_eq!(cpu.get_register(2), (value & 0xFF00) >> 8);
    }

    #[test]
    fn ldr_should_rotate_value_when_not_word_aligned() {
        let memory = GBAMemory::new();

        let mut cpu = CPU::new(memory);

        let value = 0xFABCD321;
        let address: u32 = 0x3000202;

        let _res = cpu.memory.writeu32(0x3000200, value);

        cpu.set_register(1, address);

        cpu.prefetch[0] = Some(0xe5912000); // ldr r2, [r1]

        cpu.execute_cpu_cycle();
        cpu.execute_cpu_cycle();

        assert_eq!(cpu.get_register(2), 0xD321FABC);

        cpu.set_register(1, 0x3000203);

        cpu.prefetch[0] = Some(0xe5912000); // ldr r2, [r1]

        cpu.execute_cpu_cycle();
        cpu.execute_cpu_cycle();
        assert_eq!(cpu.get_register(2), 0xBCD321FA);
    }

    #[test]
    fn ldr_should_writeback_when_post_indexed() {
        let memory = GBAMemory::new();

        let mut cpu = CPU::new(memory);

        let value = 0xFABCD321;
        let address: u32 = 0x3000200;

        let _res = cpu.memory.writeu32(address as usize, value);

        cpu.set_register(1, address);

        cpu.prefetch[0] = Some(0xe4912004); // ldr r2, [r1], 4

        cpu.execute_cpu_cycle();
        cpu.execute_cpu_cycle();

        assert_eq!(cpu.get_register(2), value);
        assert_eq!(cpu.get_register(1), address + 4);
    }

    #[test]
    fn str_should_store_word_at_memory_address() {
        let memory = GBAMemory::new();

        let mut cpu = CPU::new(memory);

        let value = 0xFABCD321;
        let address: u32 = 0x3000200;

        cpu.set_register(1, address);
        cpu.set_register(2, value);

        cpu.prefetch[0] = Some(0xe5812000); // str r2, [r1]

        cpu.execute_cpu_cycle();
        cpu.execute_cpu_cycle();

        let stored_value = cpu.memory.readu32(address as usize).data;

        assert_eq!(value, stored_value);
    }

    #[test]
    fn str_should_store_byte_at_address() {
        let memory = GBAMemory::new();

        let mut cpu = CPU::new(memory);

        let value: u8 = 0x21;
        let address: u32 = 0x3000203;

        cpu.set_register(1, address);
        cpu.set_register(2, value.into());

        cpu.prefetch[0] = Some(0xe5c12000); // strb r2, [r1]

        cpu.execute_cpu_cycle();
        cpu.execute_cpu_cycle();

        let stored_value = cpu.memory.read(address as usize).data;

        assert_eq!(value, stored_value);
    }

    #[test]
    fn strh_should_store_hw_at_address() {
        let memory = GBAMemory::new();

        let mut cpu = CPU::new(memory);

        let value: u16 = 0x21;
        let address: u32 = 0x3000200;

        cpu.set_register(1, address);
        cpu.set_register(3, value.into());

        cpu.prefetch[0] = Some(0xe1c130b0); // strh r3, [r1]

        cpu.execute_cpu_cycle();
        cpu.execute_cpu_cycle();

        let stored_value = cpu.memory.readu16(address as usize).data;

        assert_eq!(value as u16, stored_value);
    }

    #[test]
    fn strh_should_only_store_bottom_half_of_register() {
        let memory = GBAMemory::new();

        let mut cpu = CPU::new(memory);

        let value: u32 = 0x1234_5678;
        let address: u32 = 0x3000200;

        cpu.set_register(1, address);
        cpu.set_register(3, value);

        cpu.prefetch[0] = Some(0xe1c130b0); // strh r3, [r1]

        cpu.execute_cpu_cycle();
        cpu.execute_cpu_cycle();

        let stored_value = cpu.memory.readu32(address as usize).data;

        assert_eq!(value & 0x0000_FFFF, stored_value);
    }

    #[test]
    fn ldrh_should_only_load_bottom_half_of_register() {
        let memory = GBAMemory::new();

        let mut cpu = CPU::new(memory);

        let value = 0xFABCD321;
        let address: u32 = 0x3000200;

        let _res = cpu.memory.writeu32(address as usize, value);

        cpu.set_register(1, address);
        cpu.prefetch[0] = Some(0xe1d130b0); // ldrh r3, [r1]

        cpu.execute_cpu_cycle();
        cpu.execute_cpu_cycle();

        assert_eq!(cpu.get_register(3), value & 0x0000_FFFF);
    }

    #[test]
    fn ldrsh_should_return_a_signed_hw() {
        let memory = GBAMemory::new();

        let mut cpu = CPU::new(memory);

        let value = 0x0000_FABC;
        let address: u32 = 0x3000200;

        let _res = cpu.memory.writeu32(address as usize, value);

        cpu.set_register(1, address);
        cpu.prefetch[0] = Some(0xe1d130f0); // ldrsh r3, [r1]

        cpu.execute_cpu_cycle();
        cpu.execute_cpu_cycle();

        assert_eq!(cpu.get_register(3), value | 0xFFFF_0000);
    }

    #[test]
    fn ldrsh_should_return_a_signed_byte() {
        let memory = GBAMemory::new();

        let mut cpu = CPU::new(memory);

        let value = 0x0000_0081;
        let address: u32 = 0x3000200;

        let _res = cpu.memory.writeu32(address as usize, value);

        cpu.set_register(1, address);
        cpu.prefetch[0] = Some(0xe1d130d0); // ldrsb r3, [r1]

        cpu.execute_cpu_cycle();
        cpu.execute_cpu_cycle();

        assert_eq!(cpu.get_register(3), value | 0xFFFF_FF00);
    }

    #[test]
    fn ldm_should_load_multiple_registers() {
        let memory = GBAMemory::new();

        let mut cpu = CPU::new(memory);

        let value = 0x0000_0081;
        let address: u32 = 0x3000200;

        cpu.set_register(5, address);

        cpu.memory.writeu32(address as usize, value);

        cpu.memory.writeu32(address as usize + 4, 0x55);

        cpu.prefetch[0] = Some(0xe8950003); // ldmia r5, {r0, r1}

        cpu.execute_cpu_cycle();
        cpu.execute_cpu_cycle();

        assert_eq!(cpu.get_register(0), value);
        assert_eq!(cpu.get_register(1), 0x55);
    }

    #[test]
    fn ldmib_should_load_multiple_registers_and_modify_base_register() {
        let memory = GBAMemory::new();

        let mut cpu = CPU::new(memory);

        let value = 0x0000_0081;
        let address: u32 = 0x3000200;

        cpu.set_register(5, address);

        cpu.memory.writeu32(address as usize + 4, value);

        cpu.memory.writeu32(address as usize + 8, 0x55);

        cpu.prefetch[0] = Some(0xe9b500c0); // ldmib r5!, {r6, r7}

        cpu.execute_cpu_cycle();
        cpu.execute_cpu_cycle();

        assert_eq!(cpu.get_register(6), value);
        assert_eq!(cpu.get_register(7), 0x55);
        assert_eq!(cpu.get_register(5), address + 8);
    }

    #[test]
    fn ldmda_should_load_multiple_registers_and_modify_base_register() {
        let memory = GBAMemory::new();

        let mut cpu = CPU::new(memory);

        let value = 0x0000_0081;
        let address: u32 = 0x3000200;

        cpu.set_register(5, address);

        cpu.memory.writeu32(address as usize, value);

        cpu.memory.writeu32(address as usize - 4, 0x55);

        cpu.prefetch[0] = Some(0xe83500c0); // ldmda r5!, {r6, r7}

        cpu.execute_cpu_cycle();
        cpu.execute_cpu_cycle();

        assert_eq!(cpu.get_register(6), 0x55);
        assert_eq!(cpu.get_register(7), value);
        assert_eq!(cpu.get_register(5), address - 8);
    }

    #[test]
    fn ldmdb_should_load_multiple_registers_and_modify_base_register() {
        let memory = GBAMemory::new();

        let mut cpu = CPU::new(memory);

        let value = 0x0000_0081;
        let address: u32 = 0x3000200;

        cpu.set_register(5, address);

        cpu.memory.writeu32(address as usize - 4, value);

        cpu.memory.writeu32(address as usize - 8, 0x55);

        cpu.prefetch[0] = Some(0xe93500c0); // ldmdb r5!, {r6, r7}

        cpu.execute_cpu_cycle();
        cpu.execute_cpu_cycle();

        assert_eq!(cpu.get_register(6), 0x55);
        assert_eq!(cpu.get_register(7), value);
        assert_eq!(cpu.get_register(5), address - 8);
    }

    #[test]
    fn stm_should_store_multiple_registers() {
        let memory = GBAMemory::new();

        let mut cpu = CPU::new(memory);

        let address: u32 = 0x3000200;

        cpu.set_register(5, address);
        cpu.set_register(6, 123);
        cpu.set_register(7, 456);

        cpu.prefetch[0] = Some(0xe88500c0); // stm r5, {r6, r7}

        cpu.execute_cpu_cycle();
        cpu.execute_cpu_cycle();

        assert_eq!(cpu.memory.readu32(address as usize).data, 123);
        assert_eq!(cpu.memory.readu32(address as usize + 4).data, 456);
    }

    #[test]
    fn stmib_should_store_multiple_registers_and_writeback() {
        let memory = GBAMemory::new();

        let mut cpu = CPU::new(memory);

        let address: u32 = 0x3000200;

        cpu.set_register(5, address);
        cpu.set_register(6, 123);
        cpu.set_register(7, 456);

        cpu.prefetch[0] = Some(0xe9a500c0); // stmib r5!, {r6, r7}

        cpu.execute_cpu_cycle();
        cpu.execute_cpu_cycle();

        assert_eq!(cpu.memory.readu32(address as usize + 4).data, 123);
        assert_eq!(cpu.memory.readu32(address as usize + 8).data, 456);
        assert_eq!(cpu.get_register(5), address + 8);
    }

    #[test]
    fn stmdb_should_store_multiple_registers_and_writeback() {
        let memory = GBAMemory::new();

        let mut cpu = CPU::new(memory);

        let address: u32 = 0x3000200;

        cpu.set_register(5, address);
        cpu.set_register(6, 123);
        cpu.set_register(7, 456);

        cpu.prefetch[0] = Some(0xe92500c0); // stmdb r5!, {r6, r7}

        cpu.execute_cpu_cycle();
        cpu.execute_cpu_cycle();

        assert_eq!(cpu.memory.readu32((address - 4) as usize).data, 456);
        assert_eq!(cpu.memory.readu32((address - 8) as usize).data, 123);
        assert_eq!(cpu.get_register(5), address - 8);
    }

    #[test]
    fn stmda_should_store_multiple_registers_and_writeback() {
        let memory = GBAMemory::new();

        let mut cpu = CPU::new(memory);

        let address: u32 = 0x3000200;

        cpu.set_register(5, address);
        cpu.set_register(6, 123);
        cpu.set_register(7, 456);

        cpu.prefetch[0] = Some(0xe82500c0); // stmda r5!, {r6, r7}

        cpu.execute_cpu_cycle();
        cpu.execute_cpu_cycle();

        assert_eq!(cpu.memory.readu32((address) as usize).data, 456);
        assert_eq!(cpu.memory.readu32((address - 4) as usize).data, 123);
        assert_eq!(cpu.get_register(5), address - 8);
    }
}
