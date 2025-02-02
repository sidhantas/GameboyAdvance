use gameboy_advance::gba::GBA;

#[test]
fn test_thumb_long_branch() {
    let bios = String::from("test_files/thumb_long_branch.bin");
    let mut gba = GBA::new(bios.clone(), bios);

    {
        for _ in 0..7 {
            gba.step();
        }
        assert_eq!(gba.cpu.get_pc(), 0x9c6);
    }
}
