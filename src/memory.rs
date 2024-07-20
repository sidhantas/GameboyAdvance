enum MemorySegments {
    BIOS,
    BoardWram,
    ChipWram,
    IORAM,
    BGRAM,
    VRAM,
    OAM,
    FLASHROM0,
    FLASHROM1,
    FLASHROM2,
    SRAM
}


impl MemorySegments {
    pub fn memory_segment_range(&self) ->  Segment{
        match self {
            MemorySegments::BIOS => Segment {start: 0x00000000, end: 0x00003FFF},
            MemorySegments::BoardWram => Segment {start: 0x02000000, end: 0x0203FFFF},
            MemorySegments::ChipWram => Segment {start: 0x03000000, end: 0x03007FFF},
            MemorySegments::IORAM => Segment {start: 0x04000000, end: 0x04FFFFFF},
            MemorySegments::BGRAM => Segment {start: 0x05000000, end: 0x050003FF},
            MemorySegments::VRAM => Segment {start: 0x06000000, end: 0x05FFFFFF},
            MemorySegments::OAM => Segment {start: 0x07000000, end: 0x070003FF},
            MemorySegments::FLASHROM0 => Segment {start: 0x08000000, end: 0x09FFFFFF},
            MemorySegments::FLASHROM1 => Segment {start: 0x0A000000, end: 0x0BFFFFFF},
            MemorySegments::FLASHROM2 => Segment {start: 0x0C000000, end: 0x0DFFFFFF},
            MemorySegments::SRAM => Segment {start: 0x0E000000, end: 0x0E000FFF},
        }
    }
}

struct Segment {
    start: usize,
    end: usize,
}

impl Segment {
    fn size(&self) -> usize {
        assert!(self.end > self.start);
        return self.end - self.start + 1;
    }
}

struct _Memory {
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


pub struct Memory {
    memory: _Memory
}

impl Memory {
    pub fn initialize() -> Memory {
        let memory = _Memory {
            bios: vec![0; MemorySegments::BIOS.memory_segment_range().size()],
            board_wram: vec![0; MemorySegments::BoardWram.memory_segment_range().size()],
            chip_wram: vec![0; MemorySegments::ChipWram.memory_segment_range().size()],
            io_ram: vec![0; MemorySegments::IORAM.memory_segment_range().size()],
            bg_ram: vec![0; MemorySegments::BGRAM.memory_segment_range().size()],
            vram: vec![0; MemorySegments::VRAM.memory_segment_range().size()],
            oam: vec![0; MemorySegments::OAM.memory_segment_range().size()],
            flash_rom_0: vec![0; MemorySegments::FLASHROM0.memory_segment_range().size()],
            flash_rom_1: vec![0; MemorySegments::FLASHROM1.memory_segment_range().size()],
            flash_rom_2: vec![0; MemorySegments::FLASHROM2.memory_segment_range().size()],
            sram: vec![0; MemorySegments::SRAM.memory_segment_range().size()]
        };

        Memory { memory }
    }

    pub fn read(&self, address: u32) -> Result<u16, ()>{
        Ok(0x00)
    }

    pub fn write(&self, address: u32, value: u16) -> Result<(), ()> {
        Ok(())
    }
}

