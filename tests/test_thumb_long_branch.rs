use std::sync::{Arc, Mutex};

use gameboy_advance::{gba::GBA, graphics::display::CANVAS_AREA};

#[test]
fn test_thumb_long_branch() {
    let bios = String::from("test_files/thumb_long_branch.bin");
    let mut gba = GBA::new(
        bios.clone(),
        bios.clone(),
        Arc::new(Mutex::new([0; CANVAS_AREA])),
    );

    {
        for _ in 0..7 {
            gba.step();
        }
        assert_eq!(gba.cpu.get_pc(), 0x9c6);
    }
}
