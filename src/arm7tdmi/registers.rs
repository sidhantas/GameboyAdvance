use std::ffi::c_void;

use crate::types::WORD;

struct Registers {
    register_map: [isize; 16],
    pub registers_user: [WORD; 16],
    registers_fiq: [WORD; 8],
    registers_svc: [WORD; 2],
    registers_abt: [WORD; 2],
    registers_irq: [WORD; 2],
    registers_und: [WORD; 2],
}

impl Registers {
    fn new() -> Self {
        let mut regs = Registers {
            register_map: [0; 16],
            registers_user: [0; 16],
            registers_fiq: [0; 8],
            registers_svc: [0; 2],
            registers_abt: [0; 2],
            registers_irq: [0; 2],
            registers_und: [0; 2],
        };

        for i in 0..16 {
            unsafe {
                regs.register_map[i] = (&mut regs.registers_user[i] as *mut c_void).offset_from(&mut regs as *mut c_void);
            }
        }

        regs
    }

    fn get_register(register_num: usize) {

    }
}
