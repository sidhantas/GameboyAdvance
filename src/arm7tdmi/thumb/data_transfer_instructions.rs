use crate::{
    arm7tdmi::cpu::{FlagsRegister, InstructionMode, CPU, PC_REGISTER},
    types::{CYCLES, REGISTER},
    utils::bits::{sign_extend, Bits},
};


impl CPU {
    pub fn ldr_pc_relative(&mut self, instruction: u32) -> CYCLES{
        let mut cycles = 1;
        let rd = (instruction & 0x0700) >> 8;
        let offset = (instruction & 0x00FF) * 4;
        let address = (self.get_pc() & !2) + offset;
        let data = {
            let memory = self.memory.lock().unwrap();
            memory.readu32(address as usize, self.get_access_mode()).unwrap().into()
        };

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
            _ => panic!()
        };

        let address = self.get_register(rb) + self.get_register(ro);
        let is_byte_transfer = opcode.bit_is_set(0);

        cycles += operation(self, rd, address, is_byte_transfer, self.get_access_mode());
        
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
            _ => panic!()

        };
        let address = self.get_register(rb) + self.get_register(ro);

        operation(self, rd, address);

        1
    }
    
}


#[cfg(test)]
mod thumb_ldr_str_tests {
    use std::sync::{Arc, Mutex};

    use crate::{
        arm7tdmi::cpu::{InstructionMode, CPU},
        memory::{AccessFlags, Memory},
    };

    #[test]
    fn should_load_data_relative_to_pc() {
        let memory = Memory::new().unwrap();
        let memory = Arc::new(Mutex::new(memory));
        let mem = memory.clone();
        let mut cpu = CPU::new(memory);
        cpu.inst_mode = InstructionMode::THUMB;
        mem.lock()
            .unwrap()
            .writeu32(0x24, 0x55, AccessFlags::User)
            .unwrap();

        cpu.set_pc(0x16);
        cpu.fetched_instruction = 0x4d03; // ldr r5, [pc, 12]
        cpu.execute_cpu_cycle();
        cpu.execute_cpu_cycle();

        
        assert_eq!(cpu.get_register(5), 0x55);
    }
}
