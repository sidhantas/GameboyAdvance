use crate::{
    memory::AccessFlags,
    types::{CYCLES, REGISTER, WORD},
    utils::bits::Bits,
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

        cycles += match instruction.bit_is_set(20) {
            true => {
                self.ldr_instruction_execution(rd, access_address, is_byte_transfer, access_mode)
            }
            false => {
                self.str_instruction_execution(rd, access_address, is_byte_transfer, access_mode)
            }
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
}

#[cfg(test)]
mod sdt_tests {
    use std::sync::{Arc, Mutex};

    use crate::{arm7tdmi::cpu::CPU, memory::{AccessFlags, Memory}};

    #[test]
    fn ldr_should_return_data_at_specified_address() {
        let memory = Memory::new().unwrap();
        let cpu_memory = Arc::new(Mutex::new(memory));
        let mem = Arc::clone(&cpu_memory);
        let mut cpu = CPU::new(cpu_memory);

        let value = 0xFABCD321;
        let address: u32 = 0x200;

        let _res = mem.lock().unwrap().writeu32(address as usize, value, AccessFlags::User);

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

        let _res = mem.lock().unwrap().writeu32(address as usize + 8, value, AccessFlags::User);

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

        let _res = mem.lock().unwrap().writeu32(address as usize - 8, value, AccessFlags::User);

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

        let _res = mem.lock().unwrap().writeu32(address as usize + 8, value, AccessFlags::User);

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

        let _res = mem.lock().unwrap().writeu32(address as usize, value, AccessFlags::User);

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

        let _res = mem.lock().unwrap().writeu32(0x200, value, AccessFlags::User);
        let _res = mem.lock().unwrap().writeu32(0x204, 0xABABABAB, AccessFlags::User);

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

        let _res = mem.lock().unwrap().writeu32(0x200, value, AccessFlags::User);

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

        let stored_value = mem.lock().unwrap().readu32(address as usize, AccessFlags::User).unwrap();

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

        let stored_value = mem.lock().unwrap().read(address as usize, AccessFlags::User).unwrap();

        assert_eq!(value, stored_value);
    }
}
