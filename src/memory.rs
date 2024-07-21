struct MemorySegments;

impl MemorySegments {
        const BIOS: std::ops::Range<usize> = std::ops::Range {start: 0x00000000, end: 0x00004000};
        const BoardWram: std::ops::Range<usize> = std::ops::Range {start: 0x02000000, end: 0x02040000};
        const ChipWram: std::ops::Range<usize> = std::ops::Range {start: 0x03000000, end: 0x03008000};
        const IORAM: std::ops::Range<usize> = std::ops::Range {start: 0x04000000, end: 0x05000000};
        const BGRAM: std::ops::Range<usize> = std::ops::Range {start: 0x05000000, end: 0x05000400};
        const VRAM: std::ops::Range<usize> = std::ops::Range {start: 0x06000000, end: 0x06000000};
        const OAM: std::ops::Range<usize> = std::ops::Range {start: 0x07000000, end: 0x07000400};
        const FLASHROM0: std::ops::Range<usize> = std::ops::Range {start: 0x08000000, end: 0x0A000000};
        const FLASHROM1: std::ops::Range<usize> = std::ops::Range {start: 0x0A000000, end: 0x0C000000};
        const FLASHROM2: std::ops::Range<usize> = std::ops::Range {start: 0x0C000000, end: 0x0E000000};
        const SRAM: std::ops::Range<usize> = std::ops::Range {start: 0x0E000000, end: 0x0E001000};
}


pub struct Memory {
    bios: Vec<u32>,
    board_wram: Vec<u32>,
    chip_wram: Vec<u32>,
    io_ram: Vec<u32>,
    bg_ram: Vec<u32>,
    vram: Vec<u32>,
    oam: Vec<u32>,
    flash_rom_0: Vec<u32>,
    flash_rom_1: Vec<u32>,
    flash_rom_2: Vec<u32>,
    sram: Vec<u32>
}


impl Memory {
    pub fn initialize() -> Memory {
        Memory {
            bios: vec![0; MemorySegments::BIOS.len()],
            board_wram: vec![0; MemorySegments::BoardWram.len()],
            chip_wram: vec![0; MemorySegments::ChipWram.len()],
            io_ram: vec![0; MemorySegments::IORAM.len()],
            bg_ram: vec![0; MemorySegments::BGRAM.len()],
            vram: vec![0; MemorySegments::VRAM.len()],
            oam: vec![0; MemorySegments::OAM.len()],
            flash_rom_0: vec![0; MemorySegments::FLASHROM0.len()],
            flash_rom_1: vec![0; MemorySegments::FLASHROM1.len()],
            flash_rom_2: vec![0; MemorySegments::FLASHROM2.len()],
            sram: vec![0; MemorySegments::SRAM.len()]
        }
    }

    pub fn read(&self, address: usize) -> Result<u32, String> {
        match address {
            address if MemorySegments::BIOS.contains(&address) => Ok(self.bios[address]),
            _ => return Err(String::from("Not Implemeneted"))
        }
    }

    pub fn write(&mut self, address: usize, value: u32) -> Result<(), String> {
        match address {
            address if MemorySegments::BIOS.contains(&address) => self.bios[address] = value,
            _ => return Err(String::from("Not Implemeneted"))
        };

        Ok(())
    }
}

