use std::mem::size_of;

use crate::{
    arm7tdmi::cpu::{CPU, LINK_REGISTER, PC_REGISTER},
    types::{CYCLES, REGISTER, WORD},
    utils::bits::Bits,
};

impl CPU {
    pub fn ldr_pc_relative(&mut self, instruction: u32) -> CYCLES {
        let mut cycles = 0;
        let rd = (instruction & 0x0700) >> 8;
        let offset = (instruction & 0x00FF) * 4;
        let address = (self.get_pc() & !2) + offset;
        let memory_fetch = self.memory.readu32(address as usize);

        cycles += memory_fetch.cycles;
        let data = memory_fetch.data;
        cycles += self.advance_pipeline();

        self.set_register(rd, data);

        cycles
    }

    pub fn sdt_register_offset(&mut self, instruction: u32) -> CYCLES {
        let mut cycles = 1;
        let ro = (instruction & 0x01C0) >> 6;
        let rb = (instruction & 0x0038) >> 3;
        let rd = instruction & 0x0007;
        let opcode = (instruction & 0x0C00) >> 10;

        let operation = match opcode {
            0b00 => CPU::str_instruction_execution,
            0b01 => CPU::str_instruction_execution,
            0b10 => CPU::ldr_instruction_execution,
            0b11 => CPU::ldr_instruction_execution,
            _ => panic!(),
        };

        let address = self.get_register(rb) + self.get_register(ro);
        let is_byte_transfer = opcode.bit_is_set(0);

        cycles += operation(self, rd, address, is_byte_transfer);

        cycles
    }

    pub fn sdt_sign_extend_byte_or_halfword(&mut self, instruction: u32) -> CYCLES {
        let opcode = (instruction & 0x0C00) >> 10;
        let ro = (instruction & 0x01C0) >> 6;
        let rb = (instruction & 0x0038) >> 3;
        let rd = instruction & 0x0007;

        let operation = match opcode {
            0b00 => CPU::strh_execution,
            0b01 => CPU::ldrsb_execution,
            0b10 => CPU::ldrh_execution,
            0b11 => CPU::ldrsh_execution,
            _ => panic!(),
        };
        let address = self.get_register(rb) + self.get_register(ro);

        let cycles = operation(self, rd, address);

        cycles
    }

    pub fn sdt_imm_offset(&mut self, instruction: u32) -> CYCLES {
        let mut cycles = 1;
        let opcode = (instruction & 0x1800) >> 11;
        let imm = (instruction & 0x07C0) >> 6;
        let rb = (instruction & 0x0038) >> 3;
        let rd = instruction & 0x0007;

        let base_address = self.get_register(rb);
        let operation = match opcode {
            0b00 => CPU::str_instruction_execution,
            0b01 => CPU::ldr_instruction_execution,
            0b10 => CPU::str_instruction_execution,
            0b11 => CPU::ldr_instruction_execution,
            _ => panic!(),
        };

        let is_byte_transfer = opcode.bit_is_set(1);

        let address = if is_byte_transfer {
            base_address + imm
        } else {
            base_address + imm * 4
        };

        cycles += operation(self, rd, address, is_byte_transfer);

        cycles
    }

    pub fn sdt_halfword_imm(&mut self, instruction: u32) -> CYCLES {
        let mut cycles = 1;
        let opcode = instruction.get_bit(11);
        let imm = (instruction & 0x07C0) >> 5;
        let rb = (instruction & 0x0038) >> 3;
        let rd = instruction & 0x0007;

        let operation = match opcode {
            0b0 => Self::strh_execution,
            0b1 => Self::ldrh_execution,
            _ => panic!(),
        };

        let address = self.get_register(rb) + imm;

        cycles += operation(self, rd, address);
        cycles
    }

    pub fn thumb_sdt_sp_imm(&mut self, instruction: u32) -> CYCLES {
        let mut cycles = 1;
        let opcode = instruction.get_bit(11);
        let rd = (instruction & 0x0700) >> 8;
        let imm = instruction & 0x00FF;
        let operation = match opcode {
            0b0 => CPU::str_instruction_execution,
            0b1 => CPU::ldr_instruction_execution,
            _ => panic!(),
        };

        let address = self.get_sp() + imm * 4;

        cycles += operation(self, rd, address, false);

        cycles
    }

    pub fn thumb_push_pop(&mut self, instruction: u32) -> CYCLES {
        let mut cycles = 1;
        let opcode = instruction.get_bit(11);

        let mut register_list: Vec<REGISTER> = Vec::new();

        for i in 0..8 {
            if instruction.bit_is_set(i) {
                register_list.push(i as REGISTER);
            }
        }

        let base_address;

        cycles += match opcode {
            0b0 => {
                // STMDB (PUSH)
                if instruction.bit_is_set(8) {
                    register_list.push(LINK_REGISTER);
                }
                base_address =
                    self.get_sp() - register_list.len() as u32 * size_of::<WORD>() as u32;
                self.set_executed_instruction(format!("PUSH {:?}", register_list));
                self.stm_execution(base_address as usize, true, &register_list)
            }
            0b1 => {
                // LDMIA (POP)
                if instruction.bit_is_set(8) {
                    register_list.push(PC_REGISTER as u32);
                }
                base_address = self.get_sp();
                self.set_executed_instruction(format!("POP {:?}", register_list));
                self.ldm_execution(base_address as usize, true, &register_list)
            }
            _ => panic!(),
        };

        self.set_sp(base_address);

        cycles
    }

    pub fn thumb_multiple_load_or_store(&mut self, instruction: u32) -> CYCLES {
        let mut cycles = 0;
        let opcode = instruction.get_bit(11);
        let rb = (instruction & 0x0700) >> 8;

        let mut register_list: Vec<REGISTER> = Vec::new();

        for i in 0..7 {
            if instruction.bit_is_set(i) {
                register_list.push(i as REGISTER);
            }
        }

        let operation = match opcode {
            0b0 => {
                if instruction.bit_is_set(8) {
                    register_list.push(LINK_REGISTER);
                }
                CPU::stm_execution
            }
            0b1 => {
                if instruction.bit_is_set(8) {
                    register_list.push(PC_REGISTER as u32);
                }
                CPU::ldm_execution
            }
            _ => panic!(),
        };

        let base_address = self.get_register(rb);

        cycles += operation(self, base_address as usize, false, &register_list);

        // Always write back
        self.set_register(
            rb,
            base_address + register_list.len() as u32 * size_of::<WORD>() as u32,
        );
        cycles
    }
}

#[cfg(test)]
mod thumb_ldr_str_tests {
    

    use crate::{
        arm7tdmi::cpu::{InstructionMode, CPU},
        memory::memory::GBAMemory,
    };

    #[test]
    fn should_load_data_relative_to_pc() {
        let memory = GBAMemory::new();
        
        let mut cpu = CPU::new(memory);
        cpu.set_instruction_mode(InstructionMode::THUMB);
        cpu.memory.writeu32(0x3000024, 0x55);

        cpu.set_pc(0x3000016);
        cpu.prefetch[0] = Some(0x4d03); // ldr r5, [pc, 12]
        cpu.execute_cpu_cycle();
        cpu.execute_cpu_cycle();

        assert_eq!(cpu.get_register(5), 0x55);
    }
}
