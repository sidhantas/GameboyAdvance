use std::fmt::Display;

use num_traits::Signed;

use crate::{
    arm7tdmi::{
        arm::data_transfer_instructions::{
            BlockDTOpcodes, LoadOpcodes, RegisterList, SdtOpcode, SignedAndHwDtInstruction, SignedAndHwDtLoadOpcodes, SignedAndHwDtOpcodes, StoreOpcodes
        },
        cpu::{CPU, LINK_REGISTER, PC_REGISTER, STACK_POINTER},
        instruction_table::{DecodeThumbInstructionToString, Execute, Operand},
    },
    memory::memory::GBAMemory,
    types::{CYCLES, REGISTER, WORD},
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
        self.opcode().execute(
            cpu,
            memory,
            self.rd(),
            (cpu.get_register(self.rb()) + self.imm()) as usize,
        )
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

pub struct ThumbSdtSpImm(pub u32);

impl ThumbSdtSpImm {
    fn rd(&self) -> REGISTER {
        (self.0 & 0x0700) >> 8
    }

    fn imm(&self) -> u32 {
        self.0 & 0x00FF
    }

    fn opcode(&self) -> SdtOpcode {
        match self.0.get_bit(11) {
            0b0 => SdtOpcode::Store(StoreOpcodes::STR),
            0b1 => SdtOpcode::Load(LoadOpcodes::LDR),
            _ => unreachable!(),
        }
    }
}

impl Execute for ThumbSdtSpImm {
    fn execute(self, cpu: &mut CPU, memory: &mut GBAMemory) -> CYCLES {
        self.opcode().execute(
            cpu,
            memory,
            self.rd(),
            (cpu.get_sp() + self.imm() * 4) as usize,
        )
    }
}

impl DecodeThumbInstructionToString for ThumbSdtSpImm {
    fn instruction_to_string(&self) -> String {
        format!(
            "{} {}, [sp, {}]",
            self.opcode(),
            self.rd(),
            Operand::Immediate(self.imm())
        )
    }
}

pub struct ThumbPushPop(pub u32);

impl ThumbPushPop {
    fn opcode(&self) -> ThumbPushPopOpcodes {
        use ThumbPushPopOpcodes::*;
        match self.0.get_bit(11) {
            0b0 => PUSH,
            0b1 => POP,
            _ => unreachable!()
        }
    }

    fn register_list(&self) -> impl Iterator<Item = REGISTER> {
        let mut rlist = RegisterList {
            list: self.0 & 0xFF,
            i: 0
        };

        if self.0.bit_is_set(8) {
            match self.opcode() {
                ThumbPushPopOpcodes::PUSH => rlist.list.set_bit(14),
                ThumbPushPopOpcodes::POP => rlist.list.set_bit(15),
            }
        }

        rlist
    }
}

enum ThumbPushPopOpcodes {
    PUSH,
    POP
}

impl Display for ThumbPushPopOpcodes {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            ThumbPushPopOpcodes::PUSH => "push",
            ThumbPushPopOpcodes::POP => "pop",
        })
    }
}

impl Execute for ThumbPushPop {
    fn execute(self, cpu: &mut CPU, memory: &mut GBAMemory) -> CYCLES {
        let mut cycles = 0;
        cycles += cpu.advance_pipeline(memory);
        match self.opcode() {
            ThumbPushPopOpcodes::PUSH => {
                let base_address = cpu.get_sp() as usize - self.register_list().count() * size_of::<WORD>();
                let mut curr_address = base_address;
                for register in self.register_list() {
                    let data = cpu.get_register(register);
                    cycles += memory.writeu32(curr_address, data);
                    curr_address += size_of::<WORD>();
                }
                cpu.set_register(STACK_POINTER, base_address as u32);
            },
            ThumbPushPopOpcodes::POP => {
                cycles += 1;
                let mut curr_address = cpu.get_sp() as usize;
                for register in self.register_list() {
                    let memory_fetch = memory.readu32(curr_address);
                    cycles += memory_fetch.cycles;
                    let data = memory_fetch.data;
                    cpu.set_register(register, data);
                    curr_address += size_of::<WORD>();
                    if register == PC_REGISTER as u32 {
                        cpu.flush_pipeline(memory);
                    }
                }
                cpu.set_register(STACK_POINTER, curr_address as u32);
            },
        };

        cycles
    }
}

impl DecodeThumbInstructionToString for ThumbPushPop {
    fn instruction_to_string(&self) -> String {
        let mut rlist = Vec::new();

        for register in self.register_list() {
            rlist.push(print_register(&register));
        }

        let rlist = format!("{{{}}}", rlist.join(","));

        format!("{} {rlist}", self.opcode())
    }
}

pub struct ThumbBlockDT(pub u32);

impl ThumbBlockDT {
    fn opcode(&self) -> BlockDTOpcodes {
        use BlockDTOpcodes::*;
        match self.0.get_bit(11) {
            0b0 => STM,
            0b1 => LDM,
            _ => unreachable!()
        }
    }

    fn register_list(&self) -> impl Iterator<Item = REGISTER> {
        RegisterList {
            list: self.0 & 0xFF,
            i: 0
        }
    }

    fn rb(&self) -> REGISTER {
        (self.0 & 0x0700) >> 8
    }
}

impl Execute for ThumbBlockDT {
    fn execute(self, cpu: &mut CPU, memory: &mut GBAMemory) -> CYCLES {
        let mut cycles = 0;
        match self.opcode() {
            BlockDTOpcodes::STM => {
                let mut curr_address = cpu.get_register(self.rb()) as usize;
                for register in self.register_list() {
                    let data = cpu.get_register(register);
                    cycles += memory.writeu32(curr_address, data);
                    curr_address += size_of::<WORD>();
                }
                cpu.set_register(self.rb(), curr_address as u32);
            },
            BlockDTOpcodes::LDM => {
                cycles += 1;
                let mut curr_address = cpu.get_register(self.rb()) as usize;
                for register in self.register_list() {
                    let memory_fetch = memory.readu32(curr_address);
                    cycles += memory_fetch.cycles;
                    let data = memory_fetch.data;
                    cpu.set_register(register, data);
                    curr_address += size_of::<WORD>();
                }
                cpu.set_register(self.rb(), curr_address as u32);
            },
        }
        cycles
    }
}

impl DecodeThumbInstructionToString for ThumbBlockDT {
    fn instruction_to_string(&self) -> String {
        let mut rlist = Vec::new();

        for register in self.register_list() {
            rlist.push(print_register(&register));
        }

        let rlist = format!("{{{}}}", rlist.join(","));

        format!("{}, {}!, {}", match self.opcode() {
            BlockDTOpcodes::STM => "stmia",
            BlockDTOpcodes::LDM => "ldmia",
        }, print_register(&self.rb()), rlist)
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
