use std::{
    fs::File,
    io::{Read, Seek},
};

use crate::types::*;
struct MemorySegments;

impl MemorySegments {
    const BIOS: std::ops::Range<usize> = std::ops::Range {
        start: 0x00000000,
        end: 0x00004000,
    };
    const BOARD_WRAM: std::ops::Range<usize> = std::ops::Range {
        start: 0x02000000,
        end: 0x02040000,
    };
    const CHIP_WRAM: std::ops::Range<usize> = std::ops::Range {
        start: 0x03000000,
        end: 0x03008000,
    };
    const IORAM: std::ops::Range<usize> = std::ops::Range {
        start: 0x04000000,
        end: 0x05000000,
    };
    const BGRAM: std::ops::Range<usize> = std::ops::Range {
        start: 0x05000000,
        end: 0x05000400,
    };
    const VRAM: std::ops::Range<usize> = std::ops::Range {
        start: 0x06000000,
        end: 0x06000000,
    };
    const OAM: std::ops::Range<usize> = std::ops::Range {
        start: 0x07000000,
        end: 0x07000400,
    };
    const FLASHROM0: std::ops::Range<usize> = std::ops::Range {
        start: 0x08000000,
        end: 0x0A000000,
    };
    const FLASHROM1: std::ops::Range<usize> = std::ops::Range {
        start: 0x0A000000,
        end: 0x0C000000,
    };
    const FLASHROM2: std::ops::Range<usize> = std::ops::Range {
        start: 0x0C000000,
        end: 0x0E000000,
    };
    const SRAM: std::ops::Range<usize> = std::ops::Range {
        start: 0x0E000000,
        end: 0x0E001000,
    };
}

pub enum AccessFlags {
    User = (1 << 0),
    Privileged = (1 << 1)
}

#[allow(dead_code)]
pub struct Memory {
    bios: Vec<BYTE>,
    board_wram: Vec<BYTE>,
    chip_wram: Vec<BYTE>,
    io_ram: Vec<BYTE>,
    bg_ram: Vec<BYTE>,
    vram: Vec<BYTE>,
    oam: Vec<BYTE>,
    flash_rom_0: Vec<BYTE>,
    flash_rom_1: Vec<BYTE>,
    flash_rom_2: Vec<BYTE>,
    sram: Vec<BYTE>,
}

impl Memory {
    pub fn new() -> Result<Memory, std::io::Error> {
        let mem = Memory {
            bios: vec![0; MemorySegments::BIOS.len()],
            board_wram: vec![0; MemorySegments::BOARD_WRAM.len()],
            chip_wram: vec![0; MemorySegments::CHIP_WRAM.len()],
            io_ram: vec![0; MemorySegments::IORAM.len()],
            bg_ram: vec![0; MemorySegments::BGRAM.len()],
            vram: vec![0; MemorySegments::VRAM.len()],
            oam: vec![0; MemorySegments::OAM.len()],
            flash_rom_0: vec![0; MemorySegments::FLASHROM0.len()],
            flash_rom_1: vec![0; MemorySegments::FLASHROM1.len()],
            flash_rom_2: vec![0; MemorySegments::FLASHROM2.len()],
            sram: vec![0; MemorySegments::SRAM.len()],
        };

        Ok(mem)
    }

    pub fn initialize_bios(&mut self, filename: String) -> Result<(), std::io::Error> {
        let mut bios_file = File::options().read(true).open(filename)?;

        bios_file.rewind()?;
        bios_file.read(&mut self.bios)?;
        Ok(())
    }

    pub fn read(&self, address: usize, access_flags: AccessFlags) -> Result<BYTE, String> {
        match address {
            address if MemorySegments::BIOS.contains(&address) => Ok(self.bios[address]),
            _ => return Err(String::from("Not Implemeneted")),
        }
    }

    pub fn readu16(&self, address: usize, access_flags: AccessFlags) -> Result<HWORD, String> {
        // assert!(address % 4 == 0);
        match address {
            address if MemorySegments::BIOS.contains(&address) => Ok(u16::from_le_bytes(
                self.bios[address..address + 2].try_into().unwrap(),
            )),
            _ => return Err(String::from("Not Implemeneted")),
        }
    }

    pub fn readu32(&self, address: usize, access_flags: AccessFlags) -> Result<WORD, String> {
        // assert!(address % 4 == 0);
        match address {
            address if MemorySegments::BIOS.contains(&address) => Ok(u32::from_le_bytes(
                self.bios[address..address + 4].try_into().unwrap(),
            )),
            _ => return Err(String::from("Not Implemeneted")),
        }
    }

    pub fn write(&mut self, address: usize, value: BYTE, access_flags: AccessFlags) -> Result<(), String> {
        match address {
            address if MemorySegments::BIOS.contains(&address) => self.bios[address] = value,
            _ => return Err(String::from("Not Implemeneted")),
        };

        Ok(())
    }

    pub fn writeu16(&mut self, address: usize, value: HWORD, access_flags: AccessFlags) -> Result<(), String> {
        assert!(address % 2 == 0);
        match address {
            address if MemorySegments::BIOS.contains(&address) => {
                self.bios[address..][..2].copy_from_slice(&value.to_le_bytes())
            },
            _ => return Err(String::from("Not Implemeneted")),
        };

        Ok(())
    }

    pub fn writeu32(&mut self, address: usize, value: WORD, access_flags: AccessFlags) -> Result<(), String> {
        assert!(address % 4 == 0);
        match address {
            address if MemorySegments::BIOS.contains(&address) => {
                self.bios[address..][..4].copy_from_slice(&value.to_le_bytes())
            },
            _ => return Err(String::from("Not Implemeneted")),
        };

        Ok(())

    }
}
