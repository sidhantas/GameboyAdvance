use crate::types::WORD;

use super::cpu::CPUMode;

#[derive(Debug)]
pub struct Registers {
    pub active_registers: [WORD; 16],
    registers_user: [WORD; 16],
    registers_svc: [WORD; 16],
    registers_abt: [WORD; 16],
    registers_irq: [WORD; 16],
    registers_und: [WORD; 16],
    current_mode: CPUMode,
}

impl Registers {
    pub fn new() -> Self {
        Registers {
            active_registers: [0; 16],
            registers_user: [0; 16],
            registers_svc: [0; 16],
            registers_abt: [0; 16],
            registers_irq: [0; 16],
            registers_und: [0; 16],
            current_mode: CPUMode::SVC,
        }
    }

    pub fn get_register(&self, register_num: usize) -> WORD {
        self.active_registers[register_num]
    }

    pub fn set_register(&mut self, register_num: usize, value: WORD) {
        self.active_registers[register_num] = value;
    }

    pub fn update_registers(&mut self, cpu_mode: CPUMode) {
        // store current registers
        for i in 0..13 {
            self.registers_user[i] = self.active_registers[i]
        }
        self.registers_user[15] = self.active_registers[15];
        match self.current_mode {
            CPUMode::USER | CPUMode::SYS => {
                for i in 13..15 {
                    self.registers_user[i] = self.active_registers[i]
                }
            }
            CPUMode::FIQ => {
                unreachable!()
            }
            CPUMode::IRQ => {
                for i in 13..15 {
                    self.registers_irq[i] = self.active_registers[i]
                }
            }
            CPUMode::SVC => {
                for i in 13..15 {
                    self.registers_svc[i] = self.active_registers[i]
                }
            }
            CPUMode::ABT => {
                for i in 13..15 {
                    self.registers_abt[i] = self.active_registers[i]
                }
            }
            CPUMode::UND => {
                for i in 13..15 {
                    self.registers_und[i] = self.active_registers[i]
                }
            }
            CPUMode::INVALID(_) => todo!(),
        }

        self.current_mode = cpu_mode;

        for i in 0..13 {
            self.active_registers[i] = self.registers_user[i];
        }
        self.active_registers[15] = self.registers_user[15];

        match self.current_mode {
            CPUMode::USER | CPUMode::SYS => {
                for i in 13..15 {
                    self.active_registers[i] = self.registers_user[i];
                }
            }
            CPUMode::FIQ => unreachable!(),
            CPUMode::IRQ => {
                for i in 13..15 {
                    self.active_registers[i] = self.registers_irq[i]
                }
            }
            CPUMode::SVC => {
                for i in 13..15 {
                    self.active_registers[i] = self.registers_svc[i]
                }
            }
            CPUMode::ABT => {
                for i in 13..15 {
                    self.active_registers[i] = self.registers_svc[i]
                }
            }
            CPUMode::UND => {
                for i in 13..15 {
                    self.active_registers[i] = self.registers_und[i]
                }
            }
            CPUMode::INVALID(_) => todo!(),
        }
    }
}
