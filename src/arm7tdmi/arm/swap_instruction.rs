use crate::{
    arm7tdmi::{
        cpu::CPU,
        instruction_table::{DecodeARMInstructionToString, Execute},
    },
    memory::memory::GBAMemory,
    types::{CYCLES, REGISTER, WORD},
    utils::{bits::Bits, instruction_to_string::print_register},
};

pub struct SwapInstruction(pub u32);

impl SwapInstruction {
    fn swap_byte(&self) -> bool {
        self.0.bit_is_set(22)
    }

    fn rn(&self) -> REGISTER {
        (self.0 & 0x000F_0000) >> 16
    }

    fn rd(&self) -> REGISTER {
        (self.0 & 0x0000_F000) >> 12
    }

    fn rm(&self) -> REGISTER {
        self.0 & 0x0000_000F
    }
}

impl Execute for SwapInstruction {
    fn execute(self, cpu: &mut CPU, memory: &mut GBAMemory) -> CYCLES {
        let mut cycles = 1; // 1 I cycle
        let address = cpu.get_register(self.rn()) as usize;

        let memory_data = if self.swap_byte() {
            let memory_fetch = memory.read(address);
            cycles += memory_fetch.cycles;
            cycles += memory.write(address, cpu.get_register(self.rm()) as u8);

            memory_fetch.data as u32
        } else {
            let memory_fetch = memory.readu32(address);
            cycles += memory_fetch.cycles;
            cycles += memory.writeu32(address, cpu.get_register(self.rm()));

            memory_fetch.data
        };

        cpu.set_register(self.rd(), memory_data);

        cycles
    }
}

impl DecodeARMInstructionToString for SwapInstruction {
    fn instruction_to_string(&self, condition_code: &str) -> String {
        let byte_swap = if self.swap_byte() { "b" } else { "" };
        format!(
            "swp{byte_swap}{condition_code} {}, {}, [{}]",
            print_register(&self.rd()),
            print_register(&self.rm()),
            print_register(&self.rn())
        )
    }
}

#[cfg(test)]
mod single_data_swap_test {
    use crate::gba::GBA;

    #[test]
    fn swap_instruction_should_store_and_load_at_the_same_time() {
        let mut gba = GBA::new_no_bios();

        gba.cpu.set_register(1, 0x3000200);
        gba.cpu.set_register(3, 10);
        gba.memory.writeu32(0x3000200, 5);

        gba.cpu.prefetch[0] = Some(0xe1014093); // swp r4, r3, [r1]

        gba.step();
        gba.step();

        assert_eq!(gba.cpu.get_register(4), 5);
        assert_eq!(gba.memory.readu32(0x3000200).data, 10);
    }

    #[test]
    fn swap_instruction_should_work_with_equal_rn_and_rm() {
        let mut gba = GBA::new_no_bios();

        let address = 0x3000200;

        gba.cpu.set_register(1, address);
        gba.memory.writeu32(address as usize, 5);

        gba.cpu.prefetch[0] = Some(0xe1014091); // swp r4, r1, [r1]

        gba.step();
        gba.step();

        assert_eq!(gba.cpu.get_register(4), 5);
        assert_eq!(gba.memory.readu32(0x3000200).data, 0x3000200);
    }

    #[test]
    fn swap_should_work_with_equal_rm_and_rd() {
        let mut gba = GBA::new_no_bios();

        let address = 0x3000200;

        gba.cpu.set_register(4, 15);

        gba.cpu.set_register(1, address);
        gba.memory.writeu32(address as usize, 5);

        gba.cpu.prefetch[0] = Some(0xe1014094); // swp r4, r4, [r1]

        gba.step();
        gba.step();

        assert_eq!(gba.cpu.get_register(4), 5);
        assert_eq!(gba.memory.readu32(0x3000200).data, 15);
    }

    #[test]
    fn swpb_should_only_store_and_load_a_byte_and_clear_upper_rd() {
        let mut gba = GBA::new_no_bios();

        let address = 0x3000200;

        gba.cpu.set_register(3, 0x1234_FABC);
        gba.cpu.set_register(4, 0xFFFF_FFFF);

        gba.cpu.set_register(1, address);
        gba.memory.writeu32(address as usize, 0x7890_DD12);

        gba.cpu.prefetch[0] = Some(0xe1414093); // swpb r4, r3, [r1]

        gba.step();
        gba.step();

        assert_eq!(gba.cpu.get_register(4), 0x12);
        assert_eq!(gba.memory.read(0x3000200).data, 0xBC);
    }
}
