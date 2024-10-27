use std::{
    fs::File,
    io::{Read, Seek},
};

use crate::types::*;

const NUM_OF_ADDRESS_WIDTHS: usize = 3;
type CycleTimes = [u8; NUM_OF_ADDRESS_WIDTHS];
type AccessWidths = [bool; NUM_OF_ADDRESS_WIDTHS];

#[repr(usize)]
enum AddressWidth {
    EIGHT = 0,
    SIXTEEN = 1,
    THIRTYTWO = 2,
}

struct MemorySegment {
    range: std::ops::Range<usize>,
    wait_states: CycleTimes,
    read_access_widths: AccessWidths,
    write_access_widths: AccessWidths,
}

struct MemorySegments;

impl MemorySegments {
    const BIOS: MemorySegment = MemorySegment {
        range: std::ops::Range {
            start: 0x00000000,
            end: 0x1_0000_0000,
        },
        wait_states: [1, 1, 1],
        read_access_widths: [true, true, true],
        write_access_widths: [false, false, false],
    };
    const BOARD_WRAM: MemorySegment = MemorySegment {
        range: std::ops::Range {
            start: 0x02000000,
            end: 0x02040000,
        },
        wait_states: [3, 3, 6],
        read_access_widths: [true, true, true],
        write_access_widths: [true, true, true],
    };
    const CHIP_WRAM: MemorySegment = MemorySegment {
        range: std::ops::Range {
            start: 0x03000000,
            end: 0x03008000,
        },
        wait_states: [1, 1, 1],
        read_access_widths: [true, true, true],
        write_access_widths: [true, true, true],
    };
    const IORAM: MemorySegment = MemorySegment {
        range: std::ops::Range {
            start: 0x04000000,
            end: 0x05000000,
        },
        wait_states: [1, 1, 1],
        read_access_widths: [true, true, true],
        write_access_widths: [true, true, true],
    };
    const BGRAM: MemorySegment = MemorySegment {
        range: std::ops::Range {
            start: 0x05000000,
            end: 0x05000400,
        },
        wait_states: [1, 1, 2],
        read_access_widths: [true, true, true],
        write_access_widths: [false, true, true],
    };
    const VRAM: MemorySegment = MemorySegment {
        range: std::ops::Range {
            start: 0x06000000,
            end: 0x06000000,
        },
        wait_states: [1, 1, 2],
        read_access_widths: [true, true, true],
        write_access_widths: [false, true, true]
    };
    const OAM: MemorySegment = MemorySegment {
        range: std::ops::Range {
            start: 0x07000000,
            end: 0x07000400,
        },
        wait_states: [1, 1, 1],
        read_access_widths: [true, true, true],
        write_access_widths: [false, true, true]
    };
    const FLASHROM0: MemorySegment = MemorySegment {
        range: std::ops::Range {
            start: 0x08000000,
            end: 0x0A000000,
        },
        wait_states: [5, 5, 8],
        read_access_widths: [true, true, true],
        write_access_widths: [false, false, false]
    };
    const FLASHROM1: MemorySegment = MemorySegment {
        range: std::ops::Range {
            start: 0x0A000000,
            end: 0x0C000000,
        },
        wait_states: [5, 5, 8],
        read_access_widths: [true, true, true],
        write_access_widths: [false, false, false]
    };
    const FLASHROM2: MemorySegment = MemorySegment {
        range: std::ops::Range {
            start: 0x0C000000,
            end: 0x0E000000,
        },
        wait_states: [5, 5, 8],
        read_access_widths: [true, true, true],
        write_access_widths: [false, false, false]
    };
    const SRAM: MemorySegment = MemorySegment {
        range: std::ops::Range {
            start: 0x0E000000,
            end: 0x0E001000,
        },
        wait_states: [1, 0, 0],
        read_access_widths: [true, false, false],
        write_access_widths: [true, false, false]
    };
}

pub enum AccessFlags {
    User = (1 << 0),
    Privileged = (1 << 1),
}

#[allow(dead_code)]
pub struct Memory {
    bios: Vec<BYTE>,
    //    board_wram: Vec<BYTE>,
    //    chip_wram: Vec<BYTE>,
    //    io_ram: Vec<BYTE>,
    //    bg_ram: Vec<BYTE>,
    //    vram: Vec<BYTE>,
    //    oam: Vec<BYTE>,
    //    flash_rom_0: Vec<BYTE>,
    //    flash_rom_1: Vec<BYTE>,
    //    flash_rom_2: Vec<BYTE>,
    //    sram: Vec<BYTE>,
}

impl Memory {
    pub fn new() -> Result<Memory, std::io::Error> {
        let mem = Memory {
            bios: vec![0; MemorySegments::BIOS.range.len()],
            //board_wram: vec![0; MemorySegments::BOARD_WRAM.len()],
            //chip_wram: vec![0; MemorySegments::CHIP_WRAM.len()],
            //io_ram: vec![0; MemorySegments::IORAM.len()],
            //bg_ram: vec![0; MemorySegments::BGRAM.len()],
            //vram: vec![0; MemorySegments::VRAM.len()],
            //oam: vec![0; MemorySegments::OAM.len()],
            //flash_rom_0: vec![0; MemorySegments::FLASHROM0.len()],
            //flash_rom_1: vec![0; MemorySegments::FLASHROM1.len()],
            //flash_rom_2: vec![0; MemorySegments::FLASHROM2.len()],
            //sram: vec![0; MemorySegments::SRAM.len()],
        };

        Ok(mem)
    }

    pub fn initialize_bios(&mut self, filename: String) -> Result<(), std::io::Error> {
        let mut bios_file = File::options().read(true).open(filename)?;

        bios_file.rewind()?;
        bios_file.read(&mut self.bios)?;
        Ok(())
    }

    fn address_is_accessible(address: usize, access_flags: AccessFlags) -> bool {
        match address {
            address if MemorySegments::BIOS.range.contains(&address) => true,
            _ => false,
        }
    }

    pub fn read(&self, address: usize, access_flags: AccessFlags) -> Result<BYTE, String> {
        if Self::address_is_accessible(address, access_flags) {
            return Ok(self.bios[address]);
        }
        Err("Inaccessible Address".into())
    }

    pub fn readu16(&self, address: usize, access_flags: AccessFlags) -> Result<HWORD, String> {
        // assert!(address % 4 == 0);
        if Self::address_is_accessible(address, access_flags) {
            return Ok(u16::from_le_bytes(
                self.bios[address..address + 2].try_into().unwrap(),
            ));
        }
        Err("Inaccessible Address".into())
    }

    pub fn readu32(&self, address: usize, access_flags: AccessFlags) -> Result<WORD, String> {
        // assert!(address % 4 == 0);
        if Self::address_is_accessible(address, access_flags) {
            return Ok(u32::from_le_bytes(
                self.bios[address..address + 4].try_into().unwrap(),
            ));
        }
        Err("Inaccessible Address".into())
    }

    pub fn write(
        &mut self,
        address: usize,
        value: BYTE,
        access_flags: AccessFlags,
    ) -> Result<(), String> {
        if Self::address_is_accessible(address, access_flags) {
            self.bios[address] = value;
            return Ok(());
        }

        Ok(())
    }

    pub fn writeu16(
        &mut self,
        address: usize,
        value: HWORD,
        access_flags: AccessFlags,
    ) -> Result<(), String> {
        assert!(address % 2 == 0);
        match address {
            address if MemorySegments::BIOS.contains(&address) => {
                self.bios[address..][..2].copy_from_slice(&value.to_le_bytes())
            }
            _ => return Err(String::from("Not Implemeneted")),
        };

        Ok(())
    }

    pub fn writeu32(
        &mut self,
        address: usize,
        value: WORD,
        access_flags: AccessFlags,
    ) -> Result<(), String> {
        assert!(address % 4 == 0);
        match address {
            address if MemorySegments::BIOS.contains(&address) => {
                self.bios[address..][0..4].copy_from_slice(&value.to_le_bytes())
            }
            _ => {}
        };

        Ok(())
    }
}
