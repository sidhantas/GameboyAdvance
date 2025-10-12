use crate::{
    arm7tdmi::{
        cpu::FlagsRegister,
        instruction_table::{DecodeARMInstructionToString, Execute},
    },
    types::REGISTER,
    utils::{bits::Bits, instruction_to_string::print_register},
};

pub struct MultiplyInstruction(pub u32);

enum MultiplyOpcodes {
    MUL,
    MLA,
    UMULL,
    UMLAL,
    SMULL,
    SMLAL,
}

impl MultiplyInstruction {
    fn opcode(&self) -> MultiplyOpcodes {
        use MultiplyOpcodes::*;
        match self.0 & 0xF << 20 {
            0b0000 => MUL,
            0b0001 => MLA,
            0b0100 => UMULL,
            0b0101 => UMLAL,
            0b0110 => SMULL,
            0b0111 => SMLAL,
            _ => panic!(),
        }
    }
    fn rd(&self) -> REGISTER {
        (self.0 & 0x000F_0000) >> 16
    }

    fn rs(&self) -> REGISTER {
        (self.0 & 0x0000_0F00) >> 8
    }

    fn rn(&self) -> REGISTER {
        (self.0 & 0x0000_F000) >> 12
    }

    fn rm(&self) -> REGISTER {
        self.0 & 0x0000_000F
    }

    fn set_flags(&self) -> bool {
        self.0.bit_is_set(20)
    }
}

impl Execute for MultiplyInstruction {
    fn execute(
        self,
        cpu: &mut crate::arm7tdmi::cpu::CPU,
        _memory: &mut crate::memory::memory::GBAMemory,
    ) -> crate::types::CYCLES {
        match self.opcode() {
            MultiplyOpcodes::MUL => {
                let operand1 = cpu.get_register(self.rm()) as u64;
                let operand2 = cpu.get_register(self.rs()) as u64;
                let result = (operand1 * operand2) as u32;
                cpu.set_register(self.rd(), result);
                cpu.set_flag_from_bit(FlagsRegister::N, result.get_bit(31) as u8);
                if self.set_flags() {
                    if result == 0 {
                        cpu.reset_flag(FlagsRegister::Z);
                    } else {
                        cpu.set_flag(FlagsRegister::Z);
                    }
                    cpu.reset_flag(FlagsRegister::C);
                }
                return if operand2 & 0xFFFF_FF00 == 0 || operand2 & 0xFFFF_FF00 == 0xFFFF_FF00 {
                    1
                } else if operand2 & 0xFFFF_0000 == 0 || operand2 & 0xFFFF_0000 == 0xFFFF_0000 {
                    2
                } else if operand2 & 0xFF00_0000 == 0 || operand2 & 0xFF00_0000 == 0xFFFF_0000 {
                    3
                } else {
                    4
                };
            }
            MultiplyOpcodes::MLA => {
                let operand1 = cpu.get_register(self.rm()) as u64;
                let operand2 = cpu.get_register(self.rs()) as u64;
                let acc = cpu.get_register(self.rn()) as u64;

                let result = (operand1 * operand2 + acc) as u32;
                if self.set_flags() {
                    cpu.set_flag_from_bit(FlagsRegister::N, result.get_bit(31) as u8);
                    if result == 0 {
                        cpu.reset_flag(FlagsRegister::Z);
                    } else {
                        cpu.set_flag(FlagsRegister::Z);
                    }
                    cpu.reset_flag(FlagsRegister::C);
                }
                if operand2 & 0xFFFF_FF00 == 0 || operand2 & 0xFFFF_FF00 == 0xFFFF_FF00 {
                    2
                } else if operand2 & 0xFFFF_0000 == 0 || operand2 & 0xFFFF_0000 == 0xFFFF_0000 {
                    3
                } else if operand2 & 0xFF00_0000 == 0 || operand2 & 0xFF00_0000 == 0xFFFF_0000 {
                    4
                } else {
                    5
                }
            }
            MultiplyOpcodes::UMULL => {
                let operand1 = cpu.get_register(self.rm()) as u64;
                let operand2 = cpu.get_register(self.rs()) as u64;
                let result = operand1 * operand2;

                cpu.set_register(self.rd(), (result >> 32) as u32);
                cpu.set_register(self.rn(), (result & 0xFFFF_FFFF) as u32);
                todo!()
            }
            MultiplyOpcodes::UMLAL => todo!(),
            MultiplyOpcodes::SMULL => todo!(),
            MultiplyOpcodes::SMLAL => todo!(),
        }
    }
}

impl DecodeARMInstructionToString for MultiplyInstruction {
    fn instruction_to_string(&self, condition_code: &str) -> String {
        let set_flags = if self.set_flags() { "s" } else { "" };

        match self.opcode() {
            MultiplyOpcodes::MUL => {
                format!(
                    "mul{set_flags}{condition_code} {}, {}, {}",
                    print_register(&self.rd()),
                    print_register(&self.rm()),
                    print_register(&self.rs())
                )
            }
            MultiplyOpcodes::MLA => format!(
                "mla{set_flags}{condition_code} {}, {}, {}, {}",
                print_register(&self.rd()),
                print_register(&self.rm()),
                print_register(&self.rs()),
                print_register(&self.rn()),
            ),
            MultiplyOpcodes::UMULL => todo!(),
            MultiplyOpcodes::UMLAL => todo!(),
            MultiplyOpcodes::SMULL => todo!(),
            MultiplyOpcodes::SMLAL => todo!(),
        }
    }
}
