use crate::{
    types::{CYCLES, WORD},
    utils::bits::Bits,
};

use super::cpu::CPU;

impl CPU {
    pub fn single_data_swap(&mut self, instruction: WORD) -> CYCLES {
        let mut cycles = 1; // 1 I cycle
        let is_byte_swap = instruction.bit_is_set(22);
        let rn = (instruction & 0x000F_0000) >> 16;
        let rd = (instruction & 0x0000_F000) >> 12;
        let rm = instruction & 0x0000_000F;
        let address = self.get_register(rn) as usize;

        let memory_data = if is_byte_swap {
            let mut memory = self.memory.lock().unwrap();
            let memory_fetch = memory.read(address, self.get_access_mode());
            cycles += memory_fetch.cycles;
            cycles += memory.write(address, self.get_register(rm) as u8, self.get_access_mode());

            memory_fetch.data as u32
        } else {
            let mut memory = self.memory.lock().unwrap();
            let memory_fetch = memory.readu32(address, self.get_access_mode());

            cycles += memory_fetch.cycles;
            cycles += memory.writeu32(address, self.get_register(rm), self.get_access_mode());

            memory_fetch.data
        };

        self.set_executed_instruction(format!("SWP {} {} [{:#x}]", rd, rm, address));
        self.set_register(rd, memory_data);

        cycles
    }
}

#[cfg(test)]
mod single_data_swap_test {
    use std::sync::{Arc, Mutex};

    use crate::{
        arm7tdmi::cpu::CPU,
        memory::memory::{AccessFlags, Memory},
    };

    #[test]
    fn swap_instruction_should_store_and_load_at_the_same_time() {
        let memory = Memory::new().unwrap();
        let cpu_memory = Arc::new(Mutex::new(memory));
        let mem = Arc::clone(&cpu_memory);
        let mut cpu = CPU::new(cpu_memory);

        cpu.set_register(1, 0x3000200);
        cpu.set_register(3, 10);
        mem.lock()
            .unwrap()
            .writeu32(0x3000200, 5, AccessFlags::User);

        cpu.prefetch[0] = Some(0xe1014093); // swp r4, r3, [r1]

        cpu.execute_cpu_cycle();
        cpu.execute_cpu_cycle();

        assert_eq!(cpu.get_register(4), 5);
        assert_eq!(
            mem.lock()
                .unwrap()
                .readu32(0x3000200, AccessFlags::User)
                .data,
            10
        );
    }

    #[test]
    fn swap_instruction_should_work_with_equal_rn_and_rm() {
        let memory = Memory::new().unwrap();
        let cpu_memory = Arc::new(Mutex::new(memory));
        let mem = Arc::clone(&cpu_memory);
        let mut cpu = CPU::new(cpu_memory);

        let address = 0x3000200;

        cpu.set_register(1, address);
        mem.lock()
            .unwrap()
            .writeu32(address as usize, 5, AccessFlags::User);

        cpu.prefetch[0] = Some(0xe1014091); // swp r4, r1, [r1]

        cpu.execute_cpu_cycle();
        cpu.execute_cpu_cycle();

        assert_eq!(cpu.get_register(4), 5);
        assert_eq!(
            mem.lock()
                .unwrap()
                .readu32(0x3000200, AccessFlags::User)
                .data,
            0x3000200
        );
    }

    #[test]
    fn swap_should_work_with_equal_rm_and_rd() {
        let memory = Memory::new().unwrap();
        let cpu_memory = Arc::new(Mutex::new(memory));
        let mem = Arc::clone(&cpu_memory);
        let mut cpu = CPU::new(cpu_memory);

        let address = 0x3000200;

        cpu.set_register(4, 15);

        cpu.set_register(1, address);
        mem.lock()
            .unwrap()
            .writeu32(address as usize, 5, AccessFlags::User);

        cpu.prefetch[0] = Some(0xe1014094); // swp r4, r4, [r1]

        cpu.execute_cpu_cycle();
        cpu.execute_cpu_cycle();

        assert_eq!(cpu.get_register(4), 5);
        assert_eq!(
            mem.lock()
                .unwrap()
                .readu32(0x3000200, AccessFlags::User)
                .data,
            15
        );
    }

    #[test]
    fn swpb_should_only_store_and_load_a_byte_and_clear_upper_rd() {
        let memory = Memory::new().unwrap();
        let cpu_memory = Arc::new(Mutex::new(memory));
        let mem = Arc::clone(&cpu_memory);
        let mut cpu = CPU::new(cpu_memory);

        let address = 0x3000200;

        cpu.set_register(3, 0x1234_FABC);
        cpu.set_register(4, 0xFFFF_FFFF);

        cpu.set_register(1, address);
        mem.lock()
            .unwrap()
            .writeu32(address as usize, 0x7890_DD12, AccessFlags::User);

        cpu.prefetch[0] = Some(0xe1414093); // swpb r4, r3, [r1]

        cpu.execute_cpu_cycle();
        cpu.execute_cpu_cycle();

        assert_eq!(cpu.get_register(4), 0x12);
        assert_eq!(
            mem.lock().unwrap().read(0x3000200, AccessFlags::User).data,
            0xBC
        );
    }
}
