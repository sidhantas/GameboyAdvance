use std::sync::{Arc, Mutex};

use gameboy_advance::{arm7tdmi::cpu::CPU, memory::Memory};

#[test]
fn test_thumb_long_branch() {
    let bios = String::from("test_files/thumb_long_branch.bin");
    let mut memory = Memory::new().expect("Unable to initialize memory for CPU");
    memory
        .initialize_bios(bios)
        .expect("Unable to initialize bios for CPU");

    let memory = Arc::new(Mutex::new(memory));

    let cpu = Arc::new(Mutex::new(CPU::new(memory)));

    {
        let mut cpu = cpu.lock().unwrap();
        for _ in 0..9 {
            cpu.execute_cpu_cycle();
        }
        assert_eq!(cpu.get_pc(), 0x9c6);
    }
}
