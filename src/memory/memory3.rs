#![allow(unused)]
use std::{
    fs::File,
    io::{Read, Seek},
};

use crate::types::*;

use super::memory::MemoryFetch;

const NUM_OF_ADDRESS_WIDTHS: usize = 3;
pub(super) type CycleTimes = [u8; NUM_OF_ADDRESS_WIDTHS];
pub(super) type AccessWidths = [bool; NUM_OF_ADDRESS_WIDTHS];

#[repr(usize)]
enum AddressWidth {
    EIGHT = 0,
    SIXTEEN = 1,
    THIRTYTWO = 2,
}

pub struct GBAMemory {
    pub(super) bios: Vec<BYTE>,
    pub(super) board_wram: Vec<BYTE>,
    pub(super) chip_wram: Vec<BYTE>,
    pub(super) io_ram: Vec<BYTE>,
    pub(super) bg_ram: Vec<BYTE>,
    pub(super) vram: Vec<BYTE>,
    pub(super) oam: Vec<BYTE>,
    pub(super) flash_rom_0: Vec<BYTE>,
    pub(super) flash_rom_1: Vec<BYTE>,
    pub(super) flash_rom_2: Vec<BYTE>,
    pub(super) sram: Vec<BYTE>,
}

#[derive(PartialEq)]
struct MemorySegment {
     range: std::ops::Range<usize>,
     wait_states: CycleTimes,
     read_access_widths: AccessWidths,
     write_access_widths: AccessWidths,
}

impl GBAMemory {
    const BIOS: MemorySegment = MemorySegment {
        range: std::ops::Range {
            start: 0x00000000,
            end: 0x00004000,
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
            end: 0x040003FF,
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
            end: 0x06018000,
        },
        wait_states: [1, 1, 2],
        read_access_widths: [true, true, true],
        write_access_widths: [false, true, true],
    };
    const OAM: MemorySegment = MemorySegment {
        range: std::ops::Range {
            start: 0x07000000,
            end: 0x07000400,
        },
        wait_states: [1, 1, 1],
        read_access_widths: [true, true, true],
        write_access_widths: [false, true, true],
    };
    const FLASHROM0: MemorySegment = MemorySegment {
        range: std::ops::Range {
            start: 0x08000000,
            end: 0x0A000000,
        },
        wait_states: [5, 5, 8],
        read_access_widths: [true, true, true],
        write_access_widths: [false, false, false],
    };
    const FLASHROM1: MemorySegment = MemorySegment {
        range: std::ops::Range {
            start: 0x0A000000,
            end: 0x0C000000,
        },
        wait_states: [5, 5, 8],
        read_access_widths: [true, true, true],
        write_access_widths: [false, false, false],
    };
    const FLASHROM2: MemorySegment = MemorySegment {
        range: std::ops::Range {
            start: 0x0C000000,
            end: 0x0E000000,
        },
        wait_states: [5, 5, 8],
        read_access_widths: [true, true, true],
        write_access_widths: [false, false, false],
    };
    const SRAM: MemorySegment = MemorySegment {
        range: std::ops::Range {
            start: 0x0E000000,
            end: 0x0E001000,
        },
        wait_states: [1, 0, 0],
        read_access_widths: [true, false, false],
        write_access_widths: [true, false, false],
    };
    const SEGMENTS: [MemorySegment; 11] = [
        GBAMemory::BIOS,
        GBAMemory::BOARD_WRAM,
        GBAMemory::CHIP_WRAM,
        GBAMemory::IORAM,
        GBAMemory::BGRAM,
        GBAMemory::VRAM,
        GBAMemory::OAM,
        GBAMemory::FLASHROM0,
        GBAMemory::FLASHROM1,
        GBAMemory::FLASHROM2,
        GBAMemory::SRAM,
    ];

    pub fn new() -> Result<GBAMemory, std::io::Error> {
        let mem = GBAMemory {
            bios: vec![0; GBAMemory::BIOS.range.len()],
            board_wram: vec![0; GBAMemory::BOARD_WRAM.range.len()],
            chip_wram: vec![0; GBAMemory::CHIP_WRAM.range.len()],
            io_ram: vec![0; GBAMemory::IORAM.range.len()],
            bg_ram: vec![0; GBAMemory::BGRAM.range.len()],
            vram: vec![0; GBAMemory::VRAM.range.len()],
            oam: vec![0; GBAMemory::OAM.range.len()],
            flash_rom_0: vec![0; GBAMemory::FLASHROM0.range.len()],
            flash_rom_1: vec![0; GBAMemory::FLASHROM1.range.len()],
            flash_rom_2: vec![0; GBAMemory::FLASHROM2.range.len()],
            sram: vec![0; GBAMemory::SRAM.range.len()],
        };

        Ok(mem)
    }

    pub fn initialize_bios(&mut self, filename: String) -> Result<(), std::io::Error> {
        let mut bios_file = File::options().read(true).open(filename)?;

        bios_file.rewind()?;
        bios_file.read(&mut self.bios)?;
        Ok(())
    }

    fn can_read(segment: &MemorySegment, width: AddressWidth) -> bool {
        (*segment).read_access_widths[width as usize]
    }

    fn memory_segment_to_region(&self, segment: &MemorySegment) -> &Vec<BYTE> {
        match *segment {
            GBAMemory::BIOS => &self.bios,
            GBAMemory::BOARD_WRAM => &self.board_wram,
            GBAMemory::CHIP_WRAM => &self.chip_wram,
            GBAMemory::IORAM => &self.io_ram,
            GBAMemory::BGRAM => &self.bg_ram,
            GBAMemory::VRAM => &self.vram,
            GBAMemory::OAM => &self.oam,
            GBAMemory::FLASHROM0 => &self.flash_rom_0,
            GBAMemory::FLASHROM1 => &self.flash_rom_1,
            GBAMemory::FLASHROM2 => &self.flash_rom_2,
            GBAMemory::SRAM => &self.sram,
            _ => panic!("Invalid Region"),
        }
    }

    pub fn read(&self, address: usize) -> MemoryFetch<BYTE> {
        for segment in GBAMemory::SEGMENTS {
            if segment.range.contains(&address) && Self::can_read(&segment, AddressWidth::EIGHT) {
                let region = self.memory_segment_to_region(&segment);
                return MemoryFetch {
                    data: region[address - segment.range.start],
                    cycles: segment.wait_states[AddressWidth::EIGHT as usize],
                };
            }
        }

        MemoryFetch { cycles: 1, data: 0 }
    }

    pub fn readu16(&self, address: usize) -> MemoryFetch<HWORD> {
        // assert!(address % 4 == 0);
        for segment in GBAMemory::SEGMENTS {
            if segment.range.contains(&address) && Self::can_read(&segment, AddressWidth::SIXTEEN) {
                let region = self.memory_segment_to_region(&segment);
                let address = address - segment.range.start;
                return MemoryFetch {
                    data: u16::from_le_bytes(region[address..address + 2].try_into().unwrap()),
                    cycles: segment.wait_states[AddressWidth::SIXTEEN as usize],
                };
            }
        }

        MemoryFetch { cycles: 1, data: 0 }
    }

    pub fn readu32(&self, address: usize) -> MemoryFetch<WORD> {
        // assert!(address % 4 == 0);
        for segment in GBAMemory::SEGMENTS {
            if segment.range.contains(&address) && Self::can_read(&segment, AddressWidth::THIRTYTWO)
            {
                let region = self.memory_segment_to_region(&segment);
                let address = address - segment.range.start;
                return MemoryFetch {
                    data: u32::from_le_bytes(region[address..address + 4].try_into().unwrap()),
                    cycles: segment.wait_states[AddressWidth::THIRTYTWO as usize],
                };
            }
        }

        MemoryFetch { cycles: 1, data: 0 }
    }

    fn can_write(segment: &MemorySegment, width: AddressWidth) -> bool {
        (*segment).write_access_widths[width as usize]
    }

    fn mut_memory_segment_to_region(&mut self, segment: &MemorySegment) -> &mut Vec<BYTE> {
        match *segment {
            GBAMemory::BIOS => &mut self.bios,
            GBAMemory::BOARD_WRAM => &mut self.board_wram,
            GBAMemory::CHIP_WRAM => &mut self.chip_wram,
            GBAMemory::IORAM => &mut self.io_ram,
            GBAMemory::BGRAM => &mut self.bg_ram,
            GBAMemory::VRAM => &mut self.vram,
            GBAMemory::OAM => &mut self.oam,
            GBAMemory::FLASHROM0 => &mut self.flash_rom_0,
            GBAMemory::FLASHROM1 => &mut self.flash_rom_1,
            GBAMemory::FLASHROM2 => &mut self.flash_rom_2,
            GBAMemory::SRAM => &mut self.sram,
            _ => panic!(),
        }
    }

    pub fn write(&mut self, address: usize, value: BYTE) -> CYCLES {
        for segment in GBAMemory::SEGMENTS {
            if segment.range.contains(&address) && Self::can_write(&segment, AddressWidth::EIGHT) {
                let address = address - segment.range.start;
                if segment == GBAMemory::IORAM {
                    self.io_writeu8(address, value);
                } else {
                    let region = self.mut_memory_segment_to_region(&segment);
                    region[address] = value;
                }
                return segment.wait_states[AddressWidth::EIGHT as usize];
            }
        }

        1
    }

    pub fn writeu16(&mut self, address: usize, value: HWORD) -> CYCLES {
        assert!(address % 2 == 0);
        for segment in GBAMemory::SEGMENTS {
            if segment.range.contains(&address) && Self::can_write(&segment, AddressWidth::SIXTEEN)
            {
                let address = address - segment.range.start;
                if segment == GBAMemory::IORAM {
                    self.io_writeu16(address, value);
                } else {
                    let region = self.mut_memory_segment_to_region(&segment);
                    region[address..][..2].copy_from_slice(&value.to_le_bytes());
                }
                return segment.wait_states[AddressWidth::SIXTEEN as usize];
            }
        }

        1
    }

    pub fn writeu32(&mut self, address: usize, value: WORD) -> CYCLES {
        assert!(address % 4 == 0);
        for segment in GBAMemory::SEGMENTS {
            if segment.range.contains(&address)
                && Self::can_write(&segment, AddressWidth::THIRTYTWO)
            {
                let address = address - segment.range.start;
                if segment == GBAMemory::IORAM {
                    self.io_writeu32(address, value);
                } else {
                    let region = self.mut_memory_segment_to_region(&segment);
                    region[address..][..4].copy_from_slice(&value.to_le_bytes());
                }
                return segment.wait_states[AddressWidth::THIRTYTWO as usize];
            }
        }

        1
    }
}
