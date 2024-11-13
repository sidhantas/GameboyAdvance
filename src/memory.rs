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

pub enum AccessFlags {
    User = (1 << 0),
    Privileged = (1 << 1),
}

pub struct MemoryFetch<T> {
    pub cycles: CYCLES,
    pub data: T,
}

impl Into<MemoryFetch<WORD>> for MemoryFetch<BYTE> {
    fn into(self) -> MemoryFetch<WORD> {
        MemoryFetch {
            data: self.data.into(),
            cycles: self.cycles
        }
    }
}

impl Into<MemoryFetch<WORD>> for MemoryFetch<HWORD> {
    fn into(self) -> MemoryFetch<WORD> {
        MemoryFetch {
            data: self.data.into(),
            cycles: self.cycles
        }
    }
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

#[derive(PartialEq)]
struct MemorySegment {
    range: std::ops::Range<usize>,
    wait_states: CycleTimes,
    read_access_widths: AccessWidths,
    write_access_widths: AccessWidths,
}

impl Memory {
    const BIOS: MemorySegment = MemorySegment {
        range: std::ops::Range {
            start: 0x00000000,
            end: 0x00003FFF,
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
            end: 0x06017FFF,
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
        Memory::BIOS,
        Memory::BOARD_WRAM,
        Memory::CHIP_WRAM,
        Memory::IORAM,
        Memory::BGRAM,
        Memory::VRAM,
        Memory::OAM,
        Memory::FLASHROM0,
        Memory::FLASHROM1,
        Memory::FLASHROM2,
        Memory::SRAM,
    ];

    pub fn new() -> Result<Memory, std::io::Error> {
        let mem = Memory {
            bios: vec![0; Memory::BIOS.range.len()],
            board_wram: vec![0; Memory::BOARD_WRAM.range.len()],
            chip_wram: vec![0; Memory::CHIP_WRAM.range.len()],
            io_ram: vec![0; Memory::IORAM.range.len()],
            bg_ram: vec![0; Memory::BGRAM.range.len()],
            vram: vec![0; Memory::VRAM.range.len()],
            oam: vec![0; Memory::OAM.range.len()],
            flash_rom_0: vec![0; Memory::FLASHROM0.range.len()],
            flash_rom_1: vec![0; Memory::FLASHROM1.range.len()],
            flash_rom_2: vec![0; Memory::FLASHROM2.range.len()],
            sram: vec![0; Memory::SRAM.range.len()],
        };

        Ok(mem)
    }

    pub fn initialize_bios(&mut self, filename: String) -> Result<(), std::io::Error> {
        let mut bios_file = File::options().read(true).open(filename)?;

        bios_file.rewind()?;
        bios_file.read(&mut self.bios)?;
        Ok(())
    }

    fn can_read(segment: &MemorySegment, access_flags: &AccessFlags, width: AddressWidth) -> bool {
        (*segment).read_access_widths[width as usize]
    }

    fn memory_segment_to_region(&self, segment: &MemorySegment) -> &Vec<BYTE> {
        match *segment {
            Memory::BIOS => &self.bios,
            Memory::BOARD_WRAM => &self.board_wram,
            Memory::CHIP_WRAM => &self.chip_wram,
            Memory::IORAM => &self.io_ram,
            Memory::BGRAM => &self.bg_ram,
            Memory::VRAM => &self.vram,
            Memory::OAM => &self.oam,
            Memory::FLASHROM0 => &self.flash_rom_0,
            Memory::FLASHROM1 => &self.flash_rom_1,
            Memory::FLASHROM2 => &self.flash_rom_2,
            Memory::SRAM => &self.sram,
            _ => panic!("Invalid Region")
        }
    }

    pub fn read(&self, address: usize, access_flags: AccessFlags) -> MemoryFetch<BYTE> {
        for segment in Memory::SEGMENTS {
            if segment.range.contains(&address)
                && Self::can_read(&segment, &access_flags, AddressWidth::EIGHT)
            {
                let region = self.memory_segment_to_region(&segment);
                return MemoryFetch {
                    data: region[address - segment.range.start],
                    cycles: segment.wait_states[AddressWidth::EIGHT as usize],
                };
            }
        }

        MemoryFetch {
            cycles: 1,
            data: 0,
        }
    }

    pub fn readu16(&self, address: usize, access_flags: AccessFlags) -> MemoryFetch<HWORD> {
        // assert!(address % 4 == 0);
        for segment in Memory::SEGMENTS {
            if segment.range.contains(&address)
                && Self::can_read(&segment, &access_flags, AddressWidth::SIXTEEN)
            {
                let region = self.memory_segment_to_region(&segment);
                let address = address - segment.range.start;
                return MemoryFetch {
                    data: u16::from_le_bytes(
                    region[address..address + 2].try_into().unwrap(),
                ),
                    cycles: segment.wait_states[AddressWidth::SIXTEEN as usize],
                };
            }
        }

        MemoryFetch {
            cycles: 1,
            data: 0,
        }
    }

    pub fn readu32(&self, address: usize, access_flags: AccessFlags) -> MemoryFetch<WORD> {
        // assert!(address % 4 == 0);
        for segment in Memory::SEGMENTS {
            if segment.range.contains(&address)
                && Self::can_read(&segment, &access_flags, AddressWidth::THIRTYTWO)
            {
                let region = self.memory_segment_to_region(&segment);
                let address = address - segment.range.start;
                return MemoryFetch {
                    data: u32::from_le_bytes(
                    region[address..address + 4].try_into().unwrap(),
                ),
                    cycles: segment.wait_states[AddressWidth::THIRTYTWO as usize],
                };
            }
        }

        MemoryFetch {
            cycles: 1,
            data: 0,
        }
    }

    fn can_write(segment: &MemorySegment, _access_flags: &AccessFlags, width: AddressWidth) -> bool {
        (*segment).write_access_widths[width as usize]
    }

    fn mut_memory_segment_to_region(&mut self, segment: &MemorySegment) -> &mut Vec<BYTE> {
        match *segment {
            Memory::BIOS => &mut self.bios,
            Memory::BOARD_WRAM => &mut self.board_wram,
            Memory::CHIP_WRAM => &mut self.chip_wram,
            Memory::IORAM => &mut self.io_ram,
            Memory::BGRAM => &mut self.bg_ram,
            Memory::VRAM => &mut self.vram,
            Memory::OAM => &mut self.oam,
            Memory::FLASHROM0 => &mut self.flash_rom_0,
            Memory::FLASHROM1 => &mut self.flash_rom_1,
            Memory::FLASHROM2 => &mut self.flash_rom_2,
            Memory::SRAM => &mut self.sram,
            _ => panic!(),
        }
    }

    pub fn write(
        &mut self,
        address: usize,
        value: BYTE,
        access_flags: AccessFlags,
    ) -> CYCLES {
        for segment in Memory::SEGMENTS {
            if segment.range.contains(&address)
                && Self::can_write(&segment, &access_flags, AddressWidth::EIGHT)
            {
                let region = self.mut_memory_segment_to_region(&segment);
                let address = address - segment.range.start;
                region[address] = value;
                return segment.wait_states[AddressWidth::EIGHT as usize]
            }
        }

        1
    }

    pub fn writeu16(
        &mut self,
        address: usize,
        value: HWORD,
        access_flags: AccessFlags,
    ) -> CYCLES {
        assert!(address % 2 == 0);
        for segment in Memory::SEGMENTS {
            if segment.range.contains(&address)
                && Self::can_write(&segment, &access_flags, AddressWidth::SIXTEEN)
            {
                let region = self.mut_memory_segment_to_region(&segment);
                let address = address - segment.range.start;
                region[address..][..2].copy_from_slice(&value.to_le_bytes());
                return segment.wait_states[AddressWidth::SIXTEEN as usize]
            }
        }

        1
    }

    pub fn writeu32(
        &mut self,
        address: usize,
        value: WORD,
        access_flags: AccessFlags,
    ) -> CYCLES {
        assert!(address % 4 == 0);
        for segment in Memory::SEGMENTS {
            if segment.range.contains(&address)
                && Self::can_write(&segment, &access_flags, AddressWidth::THIRTYTWO)
            {
                let region = self.mut_memory_segment_to_region(&segment);
                let address = address - segment.range.start;
                region[address..][..4].copy_from_slice(&value.to_le_bytes());
                return segment.wait_states[AddressWidth::THIRTYTWO as usize]
            }
        }

        1
    }
}
