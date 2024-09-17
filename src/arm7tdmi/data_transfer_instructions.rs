use std::mem::size_of;

use crate::{
    memory::AccessFlags,
    types::{CYCLES, REGISTER, WORD},
    utils::bits::{sign_extend, Bits},
};

use super::cpu::{CPU, PC_REGISTER};

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

        let access_mode: AccessFlags = if force_non_privileged_access {
            AccessFlags::User
        } else {
            self.get_access_mode()
        };

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

        cycles += if instruction.bit_is_set(20) {
            self.ldr_instruction_execution(rd, access_address, is_byte_transfer, access_mode)
        } else {
            self.str_instruction_execution(rd, access_address, is_byte_transfer, access_mode)
        };

        if write_back_address {
            self.set_register(base_register, offset_address);
        }

        cycles
    }

    fn str_instruction_execution(
        &mut self,
        rd: REGISTER,
        address: u32,
        byte_transfer: bool,
        access_flag: AccessFlags,
    ) -> CYCLES {
        let data: WORD = self.get_register(rd);
        {
            let mut memory = self.memory.lock().unwrap();
            if byte_transfer {
                memory
                    .write(address as usize, data as u8, access_flag)
                    .unwrap();
            } else {
                memory
                    .writeu32(address as usize, data, access_flag)
                    .unwrap();
            }
        }
        self.set_executed_instruction(format!("STR {} [{:#x}]", rd, address));
        1
    }

    fn ldr_instruction_execution(
        &mut self,
        rd: REGISTER,
        address: u32,
        byte_transfer: bool,
        access_flag: AccessFlags,
    ) -> CYCLES {
        let mut cycles = 2;
        let data: WORD;
        {
            let memory = self.memory.lock().unwrap();
            data = if byte_transfer {
                memory.read(address as usize, access_flag).unwrap().into()
            } else {
                let mut word = memory.readu32(address as usize, access_flag).unwrap();
                if word % 4 != 0 {
                    word = word.rotate_right(8 * address & 0b11);
                }
                word
            }
        }

        self.set_register(rd, data);
        if rd as usize == PC_REGISTER {
            cycles += self.flush_pipeline();
        }
        self.set_executed_instruction(format!("LDR {} [{:#x}]", rd, address));

        cycles
    }

    pub fn hw_or_signed_data_transfer(&mut self, instruction: u32) -> CYCLES {
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

    fn strh_execution(&mut self, rd: REGISTER, address: u32) -> CYCLES {
        let data: WORD = self.get_register(rd);
        {
            let mut memory = self.memory.lock().unwrap();
            memory
                .writeu16(address as usize, data as u16, self.get_access_mode())
                .unwrap();
        }
        self.set_executed_instruction(format!("STRH {} [{:#x}]", rd, address));
        1
    }

    fn ldrsh_execution(&mut self, rd: REGISTER, address: u32) -> CYCLES {
        let mut cycles = 2;
        let data = {
            let memory = self.memory.lock().unwrap();
            memory
                .readu16(address as usize, self.get_access_mode())
                .unwrap()
        };

        self.set_register(rd, sign_extend(data.into(), 15));
        if rd as usize == PC_REGISTER {
            cycles += self.flush_pipeline();
        }
        self.set_executed_instruction(format!("LDRH {} [{:#x}]", rd, address));

        cycles
    }

    fn ldrsb_execution(&mut self, rd: REGISTER, address: u32) -> CYCLES {
        let mut cycles = 2;
        let data = {
            let memory = self.memory.lock().unwrap();
            memory
                .read(address as usize, self.get_access_mode())
                .unwrap()
        };

        self.set_register(rd, sign_extend(data.into(), 7));
        if rd as usize == PC_REGISTER {
            cycles += self.flush_pipeline();
        }
        self.set_executed_instruction(format!("LDRH {} [{:#x}]", rd, address));

        cycles
    }
    fn ldrh_execution(&mut self, rd: REGISTER, address: u32) -> CYCLES {
        let mut cycles = 2;
        let data = {
            let memory = self.memory.lock().unwrap();
            memory
                .readu16(address as usize, self.get_access_mode())
                .unwrap()
        };

        self.set_register(rd, data.into());
        if rd as usize == PC_REGISTER {
            cycles += self.flush_pipeline();
        }
        self.set_executed_instruction(format!("LDRH {} [{:#x}]", rd, address));

        cycles
    }

    pub fn block_dt_execution(&mut self, instruction: u32) -> CYCLES {
        if instruction.bit_is_set(22) {
            todo!("Implement S bit");
        }

        let pre_indexed_addressing = instruction.bit_is_set(24);
        let add_offset = instruction.bit_is_set(23);
        let write_back_address: bool = instruction.bit_is_set(21);

        let base_register = (instruction & 0x000F_0000) >> 16;
        let base_address = self.get_register(base_register);

        let mut register_list: Vec<REGISTER> = Vec::new();
        for i in 0..PC_REGISTER {
            if instruction.bit_is_set(i as u8) {
                register_list.push(i as u32);
            }
        }

        if add_offset {
            if instruction.bit_is_set(20) {
                self.ldm_execution(
                    base_address as usize,
                    pre_indexed_addressing,
                    &register_list,
                );
            } else {
                self.stm_execution(
                    base_address as usize,
                    pre_indexed_addressing,
                    &register_list,
                );
            }
            if write_back_address {
                self.set_register(
                    base_register,
                    base_address + register_list.len() as u32 * size_of::<WORD>() as u32,
                );
            }
        } else {
            let base_address = base_address - register_list.len() as u32 * size_of::<WORD>() as u32;
            if instruction.bit_is_set(20) {
                self.ldm_execution(
                    base_address as usize,
                    !pre_indexed_addressing,
                    &register_list,
                );
            } else {
                self.stm_execution(
                    base_address as usize,
                    !pre_indexed_addressing,
                    &register_list,
                );
            }
            if write_back_address {
                self.set_register(base_register, base_address);
            }
        };

        3
    }

    fn stm_execution(
        &mut self,
        base_address: usize,
        pre_indexed_addressing: bool,
        register_list: &Vec<REGISTER>,
    ) -> CYCLES {
        let mut curr_address = base_address;
        for i in 0..register_list.len() {
            if pre_indexed_addressing {
                curr_address += size_of::<WORD>()
            }
            let data = self.get_register(register_list[i] as u32);
            {
                let mut memory = self.memory.lock().unwrap();
                memory
                    .writeu32(curr_address, data, self.get_access_mode())
                    .unwrap()
            };
            if !pre_indexed_addressing {
                curr_address += size_of::<WORD>()
            }
        }

        1
    }

    fn ldm_execution(
        &mut self,
        base_address: usize,
        pre_indexed_addressing: bool,
        register_list: &Vec<REGISTER>,
    ) -> CYCLES {
        let mut curr_address = base_address;
        for i in 0..register_list.len() {
            if pre_indexed_addressing {
                curr_address += size_of::<WORD>()
            }
            let data = {
                let memory = self.memory.lock().unwrap();
                memory
                    .readu32(curr_address, self.get_access_mode())
                    .unwrap()
            };
            if !pre_indexed_addressing {
                curr_address += size_of::<WORD>()
            }
            self.set_register(register_list[i] as u32, data);
        }

        1
    }
}

#[cfg(test)]
mod sdt_tests {
    use std::sync::{Arc, Mutex};

    use crate::{
        arm7tdmi::cpu::CPU,
        memory::{AccessFlags, Memory},
    };

    #[test]
    fn ldr_should_return_data_at_specified_address() {
        let memory = Memory::new().unwrap();
        let cpu_memory = Arc::new(Mutex::new(memory));
        let mem = Arc::clone(&cpu_memory);
        let mut cpu = CPU::new(cpu_memory);

        let value = 0xFABCD321;
        let address: u32 = 0x200;

        let _res = mem
            .lock()
            .unwrap()
            .writeu32(address as usize, value, AccessFlags::User);

        cpu.set_register(1, address);

        cpu.fetched_instruction = 0xe5912000; // ldr r2, [r1]

        cpu.execute_cpu_cycle();
        cpu.execute_cpu_cycle();

        assert_eq!(cpu.get_register(2), value);
    }

    #[test]
    fn ldr_should_return_data_at_specified_address_plus_offset() {
        let memory = Memory::new().unwrap();
        let cpu_memory = Arc::new(Mutex::new(memory));
        let mem = Arc::clone(&cpu_memory);
        let mut cpu = CPU::new(cpu_memory);

        let value = 0xFABCD321;
        let address: u32 = 0x200;

        let _res = mem
            .lock()
            .unwrap()
            .writeu32(address as usize + 8, value, AccessFlags::User);

        cpu.set_register(1, address);

        cpu.fetched_instruction = 0xe5912008; // ldr r2, [r1, 8]

        cpu.execute_cpu_cycle();
        cpu.execute_cpu_cycle();

        assert_eq!(cpu.get_register(2), value);
    }

    #[test]
    fn ldr_should_return_data_at_specified_address_minus_offset() {
        let memory = Memory::new().unwrap();
        let cpu_memory = Arc::new(Mutex::new(memory));
        let mem = Arc::clone(&cpu_memory);
        let mut cpu = CPU::new(cpu_memory);

        let value = 0xFABCD321;
        let address: u32 = 0x208;

        let _res = mem
            .lock()
            .unwrap()
            .writeu32(address as usize - 8, value, AccessFlags::User);

        cpu.set_register(1, address);

        cpu.fetched_instruction = 0xe5112008; // ldr r2, [r1, -8]

        cpu.execute_cpu_cycle();
        cpu.execute_cpu_cycle();

        assert_eq!(cpu.get_register(2), value);
    }

    #[test]
    fn ldr_should_return_data_at_lsl_shifted_address() {
        let memory = Memory::new().unwrap();
        let cpu_memory = Arc::new(Mutex::new(memory));
        let mem = Arc::clone(&cpu_memory);
        let mut cpu = CPU::new(cpu_memory);

        let value = 0xFABCD321;
        let address: u32 = 0x200;

        let _res = mem
            .lock()
            .unwrap()
            .writeu32(address as usize + 8, value, AccessFlags::User);

        cpu.set_register(1, address);
        cpu.set_register(3, 4);

        cpu.fetched_instruction = 0xe7912083; //  ldr r2, [r1, r3, lsl 1]

        cpu.execute_cpu_cycle();
        cpu.execute_cpu_cycle();

        assert_eq!(cpu.get_register(2), value);
    }

    #[test]
    fn ldr_should_return_a_byte_at_address() {
        let memory = Memory::new().unwrap();
        let cpu_memory = Arc::new(Mutex::new(memory));
        let mem = Arc::clone(&cpu_memory);
        let mut cpu = CPU::new(cpu_memory);

        let value = 0xFABCD321;
        let address: u32 = 0x200;

        let _res = mem
            .lock()
            .unwrap()
            .writeu32(address as usize, value, AccessFlags::User);

        cpu.set_register(1, address);

        cpu.fetched_instruction = 0xe5d12000; //  ldrb r2, [r1]

        cpu.execute_cpu_cycle();
        cpu.execute_cpu_cycle();

        assert_eq!(cpu.get_register(2), value & 0xFF);

        cpu.fetched_instruction = 0xe5d12001; //  ldrb r2, [r1, 1]

        cpu.execute_cpu_cycle();
        cpu.execute_cpu_cycle();

        assert_eq!(cpu.get_register(2), (value & 0xFF00) >> 8);
    }

    #[test]
    fn ldr_should_rotate_value_when_not_word_aligned() {
        let memory = Memory::new().unwrap();
        let cpu_memory = Arc::new(Mutex::new(memory));
        let mem = Arc::clone(&cpu_memory);
        let mut cpu = CPU::new(cpu_memory);

        let value = 0xFABCD321;
        let address: u32 = 0x202;

        let _res = mem
            .lock()
            .unwrap()
            .writeu32(0x200, value, AccessFlags::User);
        let _res = mem
            .lock()
            .unwrap()
            .writeu32(0x204, 0xABABABAB, AccessFlags::User);

        cpu.set_register(1, address);

        cpu.fetched_instruction = 0xe5912000; // ldr r2, [r1]

        cpu.execute_cpu_cycle();
        cpu.execute_cpu_cycle();

        assert_eq!(cpu.get_register(2), 0xABABFABC);

        cpu.set_register(1, 0x203);

        cpu.fetched_instruction = 0xe5912000; // ldr r2, [r1]

        cpu.execute_cpu_cycle();
        cpu.execute_cpu_cycle();
        assert_eq!(cpu.get_register(2), 0xABABABFA);
    }

    #[test]
    fn ldr_should_writeback_when_post_indexed() {
        let memory = Memory::new().unwrap();
        let cpu_memory = Arc::new(Mutex::new(memory));
        let mem = Arc::clone(&cpu_memory);
        let mut cpu = CPU::new(cpu_memory);

        let value = 0xFABCD321;
        let address: u32 = 0x200;

        let _res = mem
            .lock()
            .unwrap()
            .writeu32(0x200, value, AccessFlags::User);

        cpu.set_register(1, address);

        cpu.fetched_instruction = 0xe4912004; // ldr r2, [r1], 4

        cpu.execute_cpu_cycle();
        cpu.execute_cpu_cycle();

        assert_eq!(cpu.get_register(2), value);
        assert_eq!(cpu.get_register(1), address + 4);
    }

    #[test]
    fn str_should_store_word_at_memory_address() {
        let memory = Memory::new().unwrap();
        let cpu_memory = Arc::new(Mutex::new(memory));
        let mem = Arc::clone(&cpu_memory);
        let mut cpu = CPU::new(cpu_memory);

        let value = 0xFABCD321;
        let address: u32 = 0x200;

        cpu.set_register(1, address);
        cpu.set_register(2, value);

        cpu.fetched_instruction = 0xe5812000; // str r2, [r1]

        cpu.execute_cpu_cycle();
        cpu.execute_cpu_cycle();

        let stored_value = mem
            .lock()
            .unwrap()
            .readu32(address as usize, AccessFlags::User)
            .unwrap();

        assert_eq!(value, stored_value);
    }

    #[test]
    fn str_should_store_byte_at_address() {
        let memory = Memory::new().unwrap();
        let cpu_memory = Arc::new(Mutex::new(memory));
        let mem = Arc::clone(&cpu_memory);
        let mut cpu = CPU::new(cpu_memory);

        let value: u8 = 0x21;
        let address: u32 = 0x203;

        cpu.set_register(1, address);
        cpu.set_register(2, value.into());

        cpu.fetched_instruction = 0xe5c12000; // strb r2, [r1]

        cpu.execute_cpu_cycle();
        cpu.execute_cpu_cycle();

        let stored_value = mem
            .lock()
            .unwrap()
            .read(address as usize, AccessFlags::User)
            .unwrap();

        assert_eq!(value, stored_value);
    }

    #[test]
    fn strh_should_store_hw_at_address() {
        let memory = Memory::new().unwrap();
        let cpu_memory = Arc::new(Mutex::new(memory));
        let mem = Arc::clone(&cpu_memory);
        let mut cpu = CPU::new(cpu_memory);

        let value: u16 = 0x21;
        let address: u32 = 0x200;

        cpu.set_register(1, address);
        cpu.set_register(3, value.into());

        cpu.fetched_instruction = 0xe1c130b0; // strh r3, [r1]

        cpu.execute_cpu_cycle();
        cpu.execute_cpu_cycle();

        let stored_value = mem
            .lock()
            .unwrap()
            .readu16(address as usize, AccessFlags::User)
            .unwrap();

        assert_eq!(value as u16, stored_value);
    }

    #[test]
    fn strh_should_only_store_bottom_half_of_register() {
        let memory = Memory::new().unwrap();
        let cpu_memory = Arc::new(Mutex::new(memory));
        let mem = Arc::clone(&cpu_memory);
        let mut cpu = CPU::new(cpu_memory);

        let value: u32 = 0x1234_5678;
        let address: u32 = 0x200;

        cpu.set_register(1, address);
        cpu.set_register(3, value);

        cpu.fetched_instruction = 0xe1c130b0; // strh r3, [r1]

        cpu.execute_cpu_cycle();
        cpu.execute_cpu_cycle();

        let stored_value = mem
            .lock()
            .unwrap()
            .readu32(address as usize, AccessFlags::User)
            .unwrap();

        assert_eq!(value & 0x0000_FFFF, stored_value);
    }

    #[test]
    fn ldrh_should_only_load_bottom_half_of_register() {
        let memory = Memory::new().unwrap();
        let cpu_memory = Arc::new(Mutex::new(memory));
        let mem = Arc::clone(&cpu_memory);
        let mut cpu = CPU::new(cpu_memory);

        let value = 0xFABCD321;
        let address: u32 = 0x200;

        let _res = mem
            .lock()
            .unwrap()
            .writeu32(address as usize, value, AccessFlags::User);

        cpu.set_register(1, address);
        cpu.fetched_instruction = 0xe1d130b0; // ldrh r3, [r1]

        cpu.execute_cpu_cycle();
        cpu.execute_cpu_cycle();

        assert_eq!(cpu.get_register(3), value & 0x0000_FFFF);
    }

    #[test]
    fn ldrsh_should_return_a_signed_hw() {
        let memory = Memory::new().unwrap();
        let cpu_memory = Arc::new(Mutex::new(memory));
        let mem = Arc::clone(&cpu_memory);
        let mut cpu = CPU::new(cpu_memory);

        let value = 0x0000_FABC;
        let address: u32 = 0x200;

        let _res = mem
            .lock()
            .unwrap()
            .writeu32(address as usize, value, AccessFlags::User);

        cpu.set_register(1, address);
        cpu.fetched_instruction = 0xe1d130f0; // ldrsh r3, [r1]

        cpu.execute_cpu_cycle();
        cpu.execute_cpu_cycle();

        assert_eq!(cpu.get_register(3), value | 0xFFFF_0000);
    }

    #[test]
    fn ldrsh_should_return_a_signed_byte() {
        let memory = Memory::new().unwrap();
        let cpu_memory = Arc::new(Mutex::new(memory));
        let mem = Arc::clone(&cpu_memory);
        let mut cpu = CPU::new(cpu_memory);

        let value = 0x0000_0081;
        let address: u32 = 0x200;

        let _res = mem
            .lock()
            .unwrap()
            .writeu32(address as usize, value, AccessFlags::User);

        cpu.set_register(1, address);
        cpu.fetched_instruction = 0xe1d130d0; // ldrsb r3, [r1]

        cpu.execute_cpu_cycle();
        cpu.execute_cpu_cycle();

        assert_eq!(cpu.get_register(3), value | 0xFFFF_FF00);
    }

    #[test]
    fn ldm_should_load_multiple_registers() {
        let memory = Memory::new().unwrap();
        let cpu_memory = Arc::new(Mutex::new(memory));
        let mem = Arc::clone(&cpu_memory);
        let mut cpu = CPU::new(cpu_memory);

        let value = 0x0000_0081;
        let address: u32 = 0x200;

        cpu.set_register(5, address);

        mem.lock()
            .unwrap()
            .writeu32(address as usize, value, AccessFlags::User)
            .unwrap();

        mem.lock()
            .unwrap()
            .writeu32(address as usize + 4, 0x55, AccessFlags::User)
            .unwrap();

        cpu.fetched_instruction = 0xe8950003; // ldmia r5, {r0, r1}

        cpu.execute_cpu_cycle();
        cpu.execute_cpu_cycle();

        assert_eq!(cpu.get_register(0), value);
        assert_eq!(cpu.get_register(1), 0x55);
    }

    #[test]
    fn ldmib_should_load_multiple_registers_and_modify_base_register() {
        let memory = Memory::new().unwrap();
        let cpu_memory = Arc::new(Mutex::new(memory));
        let mem = Arc::clone(&cpu_memory);
        let mut cpu = CPU::new(cpu_memory);

        let value = 0x0000_0081;
        let address: u32 = 0x200;

        cpu.set_register(5, address);

        mem.lock()
            .unwrap()
            .writeu32(address as usize + 4, value, AccessFlags::User)
            .unwrap();

        mem.lock()
            .unwrap()
            .writeu32(address as usize + 8, 0x55, AccessFlags::User)
            .unwrap();

        cpu.fetched_instruction = 0xe9b500c0; // ldmib r5!, {r6, r7}

        cpu.execute_cpu_cycle();
        cpu.execute_cpu_cycle();

        assert_eq!(cpu.get_register(6), value);
        assert_eq!(cpu.get_register(7), 0x55);
        assert_eq!(cpu.get_register(5), address + 8);
    }

    #[test]
    fn ldmda_should_load_multiple_registers_and_modify_base_register() {
        let memory = Memory::new().unwrap();
        let cpu_memory = Arc::new(Mutex::new(memory));
        let mem = Arc::clone(&cpu_memory);
        let mut cpu = CPU::new(cpu_memory);

        let value = 0x0000_0081;
        let address: u32 = 0x200;

        cpu.set_register(5, address);

        mem.lock()
            .unwrap()
            .writeu32(address as usize, value, AccessFlags::User)
            .unwrap();

        mem.lock()
            .unwrap()
            .writeu32(address as usize - 4, 0x55, AccessFlags::User)
            .unwrap();

        cpu.fetched_instruction = 0xe83500c0; // ldmda r5!, {r6, r7}

        cpu.execute_cpu_cycle();
        cpu.execute_cpu_cycle();

        assert_eq!(cpu.get_register(6), 0x55);
        assert_eq!(cpu.get_register(7), value);
        assert_eq!(cpu.get_register(5), address - 8);
    }

    #[test]
    fn ldmdb_should_load_multiple_registers_and_modify_base_register() {
        let memory = Memory::new().unwrap();
        let cpu_memory = Arc::new(Mutex::new(memory));
        let mem = Arc::clone(&cpu_memory);
        let mut cpu = CPU::new(cpu_memory);

        let value = 0x0000_0081;
        let address: u32 = 0x200;

        cpu.set_register(5, address);

        mem.lock()
            .unwrap()
            .writeu32(address as usize - 4, value, AccessFlags::User)
            .unwrap();

        mem.lock()
            .unwrap()
            .writeu32(address as usize - 8, 0x55, AccessFlags::User)
            .unwrap();

        cpu.fetched_instruction = 0xe93500c0; // ldmdb r5!, {r6, r7}

        cpu.execute_cpu_cycle();
        cpu.execute_cpu_cycle();

        assert_eq!(cpu.get_register(6), 0x55);
        assert_eq!(cpu.get_register(7), value);
        assert_eq!(cpu.get_register(5), address - 8);
    }

    #[test]
    fn stm_should_store_multiple_registers() {
        let memory = Memory::new().unwrap();
        let cpu_memory = Arc::new(Mutex::new(memory));
        let mem = Arc::clone(&cpu_memory);
        let mut cpu = CPU::new(cpu_memory);

        let address: u32 = 0x200;

        cpu.set_register(5, address);
        cpu.set_register(6, 123);
        cpu.set_register(7, 456);

        cpu.fetched_instruction = 0xe88500c0; // stm r5, {r6, r7}

        cpu.execute_cpu_cycle();
        cpu.execute_cpu_cycle();

        assert_eq!(
            mem.lock()
                .unwrap()
                .readu32(0x200, AccessFlags::User)
                .unwrap(),
            123
        );
        assert_eq!(
            mem.lock()
                .unwrap()
                .readu32(0x204, AccessFlags::User)
                .unwrap(),
            456
        );
    }

    #[test]
    fn stmia_should_store_multiple_registers_and_writeback() {
        let memory = Memory::new().unwrap();
        let cpu_memory = Arc::new(Mutex::new(memory));
        let mem = Arc::clone(&cpu_memory);
        let mut cpu = CPU::new(cpu_memory);

        let address: u32 = 0x200;

        cpu.set_register(5, address);
        cpu.set_register(6, 123);
        cpu.set_register(7, 456);

        cpu.fetched_instruction = 0xe9a500c0; // stmib r5!, {r6, r7}

        cpu.execute_cpu_cycle();
        cpu.execute_cpu_cycle();

        assert_eq!(
            mem.lock()
                .unwrap()
                .readu32(0x204, AccessFlags::User)
                .unwrap(),
            123
        );
        assert_eq!(
            mem.lock()
                .unwrap()
                .readu32(0x208, AccessFlags::User)
                .unwrap(),
            456
        );
        assert_eq!(cpu.get_register(5), 0x208);
    }

    #[test]
    fn stmdb_should_store_multiple_registers_and_writeback() {
        let memory = Memory::new().unwrap();
        let cpu_memory = Arc::new(Mutex::new(memory));
        let mem = Arc::clone(&cpu_memory);
        let mut cpu = CPU::new(cpu_memory);

        let address: u32 = 0x200;

        cpu.set_register(5, address);
        cpu.set_register(6, 123);
        cpu.set_register(7, 456);

        cpu.fetched_instruction = 0xe92500c0; // stmdb r5!, {r6, r7}

        cpu.execute_cpu_cycle();
        cpu.execute_cpu_cycle();

        assert_eq!(
            mem.lock()
                .unwrap()
                .readu32((address - 4) as usize, AccessFlags::User)
                .unwrap(),
            456
        );
        assert_eq!(
            mem.lock()
                .unwrap()
                .readu32((address - 8) as usize, AccessFlags::User)
                .unwrap(),
            123
        );
        assert_eq!(cpu.get_register(5), address - 8);
    }

    #[test]
    fn stmda_should_store_multiple_registers_and_writeback() {
        let memory = Memory::new().unwrap();
        let cpu_memory = Arc::new(Mutex::new(memory));
        let mem = Arc::clone(&cpu_memory);
        let mut cpu = CPU::new(cpu_memory);

        let address: u32 = 0x200;

        cpu.set_register(5, address);
        cpu.set_register(6, 123);
        cpu.set_register(7, 456);

        cpu.fetched_instruction = 0xe82500c0; // stmda r5!, {r6, r7}

        cpu.execute_cpu_cycle();
        cpu.execute_cpu_cycle();

        assert_eq!(
            mem.lock()
                .unwrap()
                .readu32((address) as usize, AccessFlags::User)
                .unwrap(),
            456
        );
        assert_eq!(
            mem.lock()
                .unwrap()
                .readu32((address - 4) as usize, AccessFlags::User)
                .unwrap(),
            123
        );
        assert_eq!(cpu.get_register(5), address - 8);
    }
}
