use num_traits::Signed;

use crate::{
    arm7tdmi::{
        arm::data_transfer_instructions::{
            LoadOpcodes, SdtOpcode, SignedAndHwDtInstruction, SignedAndHwDtLoadOpcodes,
            SignedAndHwDtOpcodes, StoreOpcodes,
        },
        cpu::{CPU, LINK_REGISTER, PC_REGISTER, STACK_POINTER},
        instruction_table::{DecodeThumbInstructionToString, Execute, Operand},
    },
    memory::memory::GBAMemory,
    types::{CYCLES, REGISTER},
    utils::{bits::Bits, instruction_to_string::print_register},
};

pub struct LdrPCRelative(pub u32);

impl LdrPCRelative {
    fn rd(&self) -> REGISTER {
        (self.0 & 0x0700) >> 8
    }

    fn offset(&self) -> u32 {
        (self.0 & 0x00FF) * 4
    }
}

impl Execute for LdrPCRelative {
    fn execute(self, cpu: &mut CPU, memory: &mut GBAMemory) -> CYCLES {
        let mut cycles = 1;
        let address = (cpu.get_pc() & !2) + self.offset();
        let memory_fetch = memory.readu32(address as usize);

        cycles += memory_fetch.cycles;
        let data = memory_fetch.data;
        cycles += cpu.advance_pipeline(memory);

        cpu.set_register(self.rd(), data);
        cycles
    }
}

impl DecodeThumbInstructionToString for LdrPCRelative {
    fn instruction_to_string(&self) -> String {
        format!(
            "ldr {}, [pc, {}]",
            print_register(&self.rd()),
            Operand::Immediate(self.offset())
        )
    }
}

pub struct ThumbSdtRegisterOffset(pub u32);

enum ThumbSdtRegisterOffsetOpcodes {
    SdtOpcode(SdtOpcode),
    SignedHwOpcode(SignedAndHwDtOpcodes),
}

impl ThumbSdtRegisterOffset {
    fn opcode(&self) -> ThumbSdtRegisterOffsetOpcodes {
        use LoadOpcodes::*;
        use SdtOpcode::*;
        use StoreOpcodes::*;

        match (self.0 >> 9) & 0b111 {
            0b000 => ThumbSdtRegisterOffsetOpcodes::SdtOpcode(Store(STR)),
            0b010 => ThumbSdtRegisterOffsetOpcodes::SdtOpcode(Store(STRB)),
            0b100 => ThumbSdtRegisterOffsetOpcodes::SdtOpcode(Load(LDR)),
            0b110 => ThumbSdtRegisterOffsetOpcodes::SdtOpcode(Load(LDRB)),
            0b001 => ThumbSdtRegisterOffsetOpcodes::SignedHwOpcode(SignedAndHwDtOpcodes::STRH),
            0b011 => ThumbSdtRegisterOffsetOpcodes::SignedHwOpcode(SignedAndHwDtOpcodes::Load(
                SignedAndHwDtLoadOpcodes::LDRSB,
            )),
            0b101 => ThumbSdtRegisterOffsetOpcodes::SignedHwOpcode(SignedAndHwDtOpcodes::Load(
                SignedAndHwDtLoadOpcodes::LDRH,
            )),
            0b111 => ThumbSdtRegisterOffsetOpcodes::SignedHwOpcode(SignedAndHwDtOpcodes::Load(
                SignedAndHwDtLoadOpcodes::LDRSH,
            )),
            _ => unreachable!(),
        }
    }

    fn ro(&self) -> REGISTER {
        (self.0 & 0x01C0) >> 6
    }

    fn rb(&self) -> REGISTER {
        (self.0 & 0x0038) >> 3
    }

    fn rd(&self) -> REGISTER {
        self.0 & 0x0007
    }
}

impl Execute for ThumbSdtRegisterOffset {
    fn execute(self, cpu: &mut CPU, memory: &mut GBAMemory) -> CYCLES {
        let access_address = cpu.get_register(self.rb()) + cpu.get_register(self.ro());
        match self.opcode() {
            ThumbSdtRegisterOffsetOpcodes::SdtOpcode(opcode) => {
                opcode.execute(cpu, memory, self.rd(), access_address as usize)
            }
            ThumbSdtRegisterOffsetOpcodes::SignedHwOpcode(opcode) => {
                opcode.execute(cpu, memory, self.rd(), access_address as usize)
            }
        }
    }
}

impl DecodeThumbInstructionToString for ThumbSdtRegisterOffset {
    fn instruction_to_string(&self) -> String {
        let opcode = match self.opcode() {
            ThumbSdtRegisterOffsetOpcodes::SdtOpcode(SdtOpcode::Load(LoadOpcodes::LDR)) => "ldr",
            ThumbSdtRegisterOffsetOpcodes::SdtOpcode(SdtOpcode::Load(LoadOpcodes::LDRB)) => "ldrb",
            ThumbSdtRegisterOffsetOpcodes::SdtOpcode(SdtOpcode::Store(StoreOpcodes::STR)) => "str",
            ThumbSdtRegisterOffsetOpcodes::SdtOpcode(SdtOpcode::Store(StoreOpcodes::STRB)) => {
                "strb"
            }
            ThumbSdtRegisterOffsetOpcodes::SignedHwOpcode(SignedAndHwDtOpcodes::STRH) => "strh",
            ThumbSdtRegisterOffsetOpcodes::SignedHwOpcode(SignedAndHwDtOpcodes::Load(
                SignedAndHwDtLoadOpcodes::LDRSH,
            )) => "ldrsh",
            ThumbSdtRegisterOffsetOpcodes::SignedHwOpcode(SignedAndHwDtOpcodes::Load(
                SignedAndHwDtLoadOpcodes::LDRH,
            )) => "ldrh",
            ThumbSdtRegisterOffsetOpcodes::SignedHwOpcode(SignedAndHwDtOpcodes::Load(
                SignedAndHwDtLoadOpcodes::LDRSB,
            )) => "ldrsb",
        };

        format!(
            "{opcode} {}, [{}, {}]",
            print_register(&self.rd()),
            print_register(&self.rb()),
            print_register(&self.ro())
        )
    }
}

pub struct ThumbSdtImmOffset(pub u32);

impl ThumbSdtImmOffset {
    fn opcode(&self) -> SdtOpcode {
        match (self.0 & 0x1800) >> 11 {
            0b00 => SdtOpcode::Store(StoreOpcodes::STR),
            0b01 => SdtOpcode::Load(LoadOpcodes::LDR),
            0b10 => SdtOpcode::Store(StoreOpcodes::STRB),
            0b11 => SdtOpcode::Load(LoadOpcodes::LDRB),
            _ => unreachable!(),
        }
    }

    fn imm(&self) -> u32 {
        (self.0 & 0x07C0) >> 6
    }

    fn rb(&self) -> REGISTER {
        (self.0 & 0x0038) >> 3
    }

    fn rd(&self) -> REGISTER {
        self.0 & 0x0007
    }
}

impl Execute for ThumbSdtImmOffset {
    fn execute(self, cpu: &mut CPU, memory: &mut GBAMemory) -> CYCLES {
        let access_address = cpu.get_register(self.rb())
            + self.imm()
                * match self.opcode() {
                    SdtOpcode::Load(LoadOpcodes::LDR) => 4,
                    SdtOpcode::Load(LoadOpcodes::LDRB) => 1,
                    SdtOpcode::Store(StoreOpcodes::STR) => 4,
                    SdtOpcode::Store(StoreOpcodes::STRB) => 1,
                };

        self.opcode()
            .execute(cpu, memory, self.rd(), access_address as usize)
    }
}

impl DecodeThumbInstructionToString for ThumbSdtImmOffset {
    fn instruction_to_string(&self) -> String {
        format!(
            "{} {}, [{}, {}]",
            self.opcode(),
            print_register(&self.rd()),
            print_register(&self.rb()),
            Operand::Immediate(self.imm())
        )
    }
}

pub struct ThumbSdtHwImmOffset(pub u32);

enum ThumbSdtHwImmOffsetOpcodes {
    STRH,
    LDRH,
}

impl ThumbSdtHwImmOffset {
    fn opcode(&self) -> SignedAndHwDtOpcodes {
        match self.0.get_bit(11) {
            0b0 => SignedAndHwDtOpcodes::STRH,
            0b1 => SignedAndHwDtOpcodes::Load(SignedAndHwDtLoadOpcodes::LDRH),
            _ => unreachable!(),
        }
    }
    fn imm(&self) -> u32 {
        (self.0 & 0x07C0) >> 5
    }

    fn rb(&self) -> REGISTER {
        (self.0 & 0x0038) >> 3
    }

    fn rd(&self) -> REGISTER {
        self.0 & 0x0007
    }
}

impl Execute for ThumbSdtHwImmOffset {
    fn execute(self, cpu: &mut CPU, memory: &mut GBAMemory) -> CYCLES {
        self.opcode().execute(cpu, memory, self.rd(), (cpu.get_register(self.rb()) + self.imm()) as usize)
    }
}

impl DecodeThumbInstructionToString for ThumbSdtHwImmOffset {
    fn instruction_to_string(&self) -> String {
        format!(
            "{} {}, [{}, {}]",
            self.opcode(),
            print_register(&self.rd()),
            print_register(&self.rb()),
            Operand::Immediate(self.imm())
        )
    }
}

impl CPU {
    pub fn thumb_sdt_sp_imm(&mut self, instruction: u32, memory: &mut GBAMemory) -> CYCLES {
        let opcode = instruction.get_bit(11);
        let rd = (instruction & 0x0700) >> 8;
        let imm = instruction & 0x00FF;
        let operation = match opcode {
            0b0 => CPU::str_instruction_execution,
            0b1 => CPU::ldr_instruction_execution,
            _ => panic!(),
        };

        let address = self.get_sp() + imm * 4;

        operation(self, rd, address, false, memory)
    }

    pub fn thumb_push_pop(&mut self, instruction: u32, memory: &mut GBAMemory) -> CYCLES {
        let mut cycles = 0;
        let opcode = instruction.get_bit(11);

        let mut register_list: Vec<REGISTER> = Vec::new();

        for i in 0..8 {
            if instruction.bit_is_set(i) {
                register_list.push(i as REGISTER);
            }
        }
        cycles += self.advance_pipeline(memory);

        match opcode {
            0b0 => {
                // STMDB (PUSH)
                if instruction.bit_is_set(8) {
                    register_list.push(LINK_REGISTER);
                }
                cycles += self.stmdb_execution(
                    self.get_sp() as usize,
                    &register_list,
                    Some(STACK_POINTER),
                    memory,
                )
            }
            0b1 => {
                // LDMIA (POP)
                if instruction.bit_is_set(8) {
                    register_list.push(PC_REGISTER as u32);
                }
                cycles += self.ldmia_execution(
                    self.get_sp() as usize,
                    &register_list,
                    Some(STACK_POINTER),
                    memory,
                );
                if instruction.bit_is_set(8) {
                    cycles += self.flush_pipeline(memory);
                }
            }
            _ => panic!(),
        };
        cycles
    }

    pub fn thumb_multiple_load_or_store(
        &mut self,
        instruction: u32,
        memory: &mut GBAMemory,
    ) -> CYCLES {
        let opcode = instruction.get_bit(11);
        let rb = (instruction & 0x0700) >> 8;

        let mut register_list: Vec<REGISTER> = Vec::new();

        for i in 0..8 {
            if instruction.bit_is_set(i) {
                register_list.push(i as REGISTER);
            }
        }

        let base_address = self.get_register(rb) as usize;

        match opcode {
            0b0 => self.stmia_execution(base_address, &register_list, Some(rb), memory),
            0b1 => self.ldmia_execution(base_address, &register_list, Some(rb), memory),
            _ => panic!(),
        }
    }
}

#[cfg(test)]
mod thumb_ldr_str_tests {

    use crate::{arm7tdmi::cpu::InstructionMode, gba::GBA};

    #[test]
    fn should_load_data_relative_to_pc() {
        let mut gba = GBA::new_no_bios();
        gba.cpu.set_instruction_mode(InstructionMode::THUMB);
        gba.memory.writeu32(0x3000024, 0x55);

        gba.cpu.set_pc(0x3000016);
        gba.cpu.prefetch[0] = Some(0x4d03); // ldr r5, [pc, 12]
        gba.step();
        gba.step();

        assert_eq!(gba.cpu.get_register(5), 0x55);
    }
}
