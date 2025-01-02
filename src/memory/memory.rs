use std::{fs::File, io::{Read, Seek}};
use crate::types::{BYTE, CYCLES, HWORD, WORD};

pub struct MemoryFetch<T> {
    pub cycles: CYCLES,
    pub data: T,
}

impl Into<MemoryFetch<WORD>> for MemoryFetch<BYTE> {
    fn into(self) -> MemoryFetch<WORD> {
        MemoryFetch {
            data: self.data.into(),
            cycles: self.cycles,
        }
    }
}

impl Into<MemoryFetch<WORD>> for MemoryFetch<HWORD> {
    fn into(self) -> MemoryFetch<WORD> {
        MemoryFetch {
            data: self.data.into(),
            cycles: self.cycles,
        }
    }
}

const BIOS_REGION: usize = 0x0;
const EXWRAM_REGION: usize = 0x2;
const IWRAM_REGION: usize = 0x3;
const IORAM_REGION: usize = 0x4;
const BGRAM_REGION: usize = 0x5;
const VRAM_REGION: usize = 0x6;
const OAM_REGION: usize = 0x7;
const ROM0A_REGION: usize = 0x8;
const ROM0B_REGION: usize = 0x9;
const ROM1A_REGION: usize = 0xA;
const ROM1B_REGION: usize = 0xB;
const ROM2A_REGION: usize = 0xC;
const ROM2B_REGION: usize = 0xD;
const SRAM_REGION: usize = 0xE;

const BIOS_SIZE: usize = 0x4000;
const EXWRAM_SIZE: usize = 0x40000;
const IWRAM_SIZE: usize = 0x8000;
const IORAM_SIZE: usize = 0x3FF;
const BGRAM_SIZE: usize = 0x400;
const VRAM_SIZE: usize = 0x18000;
const OAM_SIZE: usize = 0x400;
const ROM_SIZE: usize = 0x1000000;
const SRAM_SIZE: usize = 0x10000;

pub struct GBAMemory {
    bios: Vec<u32>,
    exwram: Vec<u32>,
    iwram: Vec<u32>,
    ioram: Vec<u32>,
    bgram: Vec<u32>,
    vram: Vec<u32>,
    oam: Vec<u32>,
    rom: Vec<u32>,
    sram: Vec<u32>,
    wait_cycles_u16: [u8; 15],
    wait_cycles_u32: [u8; 15],
}

#[inline(always)]
fn memory_load(region: &Vec<u32>, address: usize) -> u32 {
    region[address >> 2]
}

#[inline(always)]
fn memory_store(region: &mut Vec<u32>, address: usize, value: u32) {
    region[address >> 2] = value;
}

impl GBAMemory {
    pub fn new() -> Self {
        let mut wait_cycles_u16 = [0; 15];
        wait_cycles_u16[BIOS_REGION] = 1;
        wait_cycles_u16[IWRAM_REGION] = 1;
        wait_cycles_u16[EXWRAM_REGION] = 3;
        wait_cycles_u16[IORAM_REGION] = 1;
        wait_cycles_u16[OAM_REGION] = 1;
        wait_cycles_u16[BGRAM_REGION] = 1;
        wait_cycles_u16[VRAM_REGION] = 1;
        wait_cycles_u16[ROM0A_REGION] = 5;
        wait_cycles_u16[ROM0B_REGION] = 5;
        wait_cycles_u16[ROM1A_REGION] = 5;
        wait_cycles_u16[ROM1B_REGION] = 5;
        wait_cycles_u16[ROM2A_REGION] = 5;
        wait_cycles_u16[ROM2B_REGION] = 5;
        wait_cycles_u16[SRAM_REGION] = 5;

        let mut wait_cycles_u32 = [0; 15];
        wait_cycles_u32[BIOS_REGION] = 1;
        wait_cycles_u32[IWRAM_REGION] = 1;
        wait_cycles_u32[EXWRAM_REGION] = 6;
        wait_cycles_u32[IORAM_REGION] = 1;
        wait_cycles_u32[OAM_REGION] = 1;
        wait_cycles_u32[BGRAM_REGION] = 2;
        wait_cycles_u32[VRAM_REGION] = 2;
        wait_cycles_u32[ROM0A_REGION] = 8;
        wait_cycles_u32[ROM0B_REGION] = 8;
        wait_cycles_u32[ROM1A_REGION] = 8;
        wait_cycles_u32[ROM1B_REGION] = 8;
        wait_cycles_u32[ROM2A_REGION] = 8;
        wait_cycles_u32[ROM2B_REGION] = 8;

        Self {
            bios: vec![0; BIOS_SIZE >> 2],
            exwram: vec![0; EXWRAM_SIZE >> 2],
            iwram: vec![0; IWRAM_SIZE >> 2],
            ioram: vec![0; IORAM_SIZE >> 2],
            bgram: vec![0; BGRAM_SIZE >> 2],
            vram: vec![0; VRAM_SIZE >> 2],
            oam: vec![0; OAM_SIZE >> 2],
            rom: vec![0; ROM_SIZE >> 2],
            sram: vec![0; SRAM_SIZE >> 2],
            wait_cycles_u16,
            wait_cycles_u32,
        }
    }

    pub fn initialize_bios(&mut self, filename: String) -> Result<(), std::io::Error> {
        let mut index = 0;
        let mut bios_file = File::options().read(true).open(filename)?;
        let mut buffer = [0; 4];
        bios_file.rewind()?;
        while let Ok(read_bytes) = bios_file.read(&mut buffer[..]) {
            if read_bytes == 0 {
                break;
            }
            self.bios[index] = u32::from_le_bytes(buffer.clone());
            index += 1;
        };
        Ok(())
    }

    pub fn read(&self, address: usize) -> MemoryFetch<u8> {
        let region = address >> 24;
        let data = match region {
            BIOS_REGION => memory_load(&self.bios, address).to_le_bytes()[address & 0b11],
            EXWRAM_REGION => {
                memory_load(&self.exwram, (address & 0xFFFFFF)).to_le_bytes()[address & 0b11]
            }
            IWRAM_REGION => {
                memory_load(&self.iwram, (address & 0xFFFFFF)).to_le_bytes()[address & 0b11]
            }
            IORAM_REGION => {0},
            BGRAM_REGION => {
                memory_load(&self.bgram, (address & 0xFFFFFF)).to_le_bytes()[address & 0b11]
            }
            VRAM_REGION => {
                memory_load(&self.vram, (address & 0xFFFFFF)).to_le_bytes()[address & 0b11]
            }
            OAM_REGION => {
                memory_load(&self.oam, (address & 0xFFFFFF)).to_le_bytes()[address & 0b11]
            }
            ROM0A_REGION..=ROM2B_REGION => {
                memory_load(&self.rom, (address & 0xFFFFFF)).to_le_bytes()[address & 0b11]
            }
            SRAM_REGION => {
                memory_load(&self.sram, (address & 0xFFFFFF)).to_le_bytes()[address & 0b11]
            }
            _ => panic!(),
        };

        MemoryFetch {
            data,
            cycles: self.wait_cycles_u16[region],
        }
    }

    pub fn readu16(&self, address: usize) -> MemoryFetch<u16> {
        let region = address >> 24;
        let data = match region {
            BIOS_REGION => memory_load(&self.bios, address),
            EXWRAM_REGION => memory_load(&self.exwram, address & 0xFFFFFF),
            IWRAM_REGION => memory_load(&self.iwram, address & 0xFFFFFF),
            IORAM_REGION => {0},
            BGRAM_REGION => memory_load(&self.bgram, address & 0xFFFFFF),
            VRAM_REGION => memory_load(&self.vram, address & 0xFFFFFF),
            OAM_REGION => memory_load(&self.oam, address & 0xFFFFFF),
            ROM0A_REGION..=ROM2B_REGION => memory_load(&self.rom, address & 0xFFFFFF),
            SRAM_REGION => memory_load(&self.sram, address & 0xFFFFFF),
            _ => panic!(),
        };

        let shift_amount = 16 * ((address >> 1) & 0x1);
        let data = data >> shift_amount;

        MemoryFetch {
            data: data as u16,
            cycles: self.wait_cycles_u16[region],
        }
    }

    pub fn readu32(&self, address: usize) -> MemoryFetch<u32> {
        let region = address >> 24;
        let data = match region {
            BIOS_REGION => memory_load(&self.bios, address),
            EXWRAM_REGION => memory_load(&self.exwram, (address & 0xFFFFFF)),
            IWRAM_REGION => memory_load(&self.iwram, (address & 0xFFFFFF)),
            IORAM_REGION => {0},
            BGRAM_REGION => memory_load(&self.bgram, (address & 0xFFFFFF)),
            VRAM_REGION => memory_load(&self.vram, (address & 0xFFFFFF)),
            OAM_REGION => memory_load(&self.oam, (address & 0xFFFFFF)),
            ROM0A_REGION..=ROM2B_REGION => memory_load(&self.rom, (address & 0xFFFFFF)),
            SRAM_REGION => memory_load(&self.sram, (address & 0xFFFFFF)),
            _ => panic!("address: {address}"),
        };

        MemoryFetch {
            data: data.rotate_right(8 * (address as u32 & 0b11)),
            cycles: self.wait_cycles_u32[region],
        }
    }

    pub fn write(&mut self, address: usize, value: u8) -> CYCLES {
        let region = address >> 24;
        match region {
            BIOS_REGION => {}
            EXWRAM_REGION => {
                let mut current_value = memory_load(&self.exwram, address & 0xFFFFFF);
                current_value &= !(0xFF << 8 * (address & 0b11));
                let value = current_value | ((value as u32) << 8 * (address & 0b11));
                memory_store(&mut self.exwram, address & 0xFFFFFF, value);
            }
            IWRAM_REGION => {
                let mut current_value = memory_load(&self.iwram, address & 0xFFFFFF);
                current_value &= !(0xFF << 8 * (address & 0b11));
                let value = current_value | ((value as u32) << 8 * (address & 0b11));
                memory_store(&mut self.iwram, address & 0xFFFFFF, value);
            }
            IORAM_REGION => {},
            BGRAM_REGION => {
                let mut current_value = memory_load(&self.bgram, address & 0xFFFFFF);
                current_value &= !(0xFF << 8 * (address & 0b11));
                let value = current_value | ((value as u32) << 8 * (address & 0b11));
                memory_store(&mut self.bgram, address & 0xFFFFFF, value);
            }
            VRAM_REGION => {
                let mut current_value = memory_load(&self.vram, address & 0xFFFFFF);
                current_value &= !(0xFF << 8 * (address & 0b11));
                let value = current_value | ((value as u32) << 8 * (address & 0b11));
                memory_store(&mut self.vram, address & 0xFFFFFF, value);
            }
            OAM_REGION => {
                let mut current_value = memory_load(&self.oam, address & 0xFFFFFF);
                current_value &= !(0xFF << 8 * (address & 0b11));
                let value = current_value | ((value as u32) << 8 * (address & 0b11));
                memory_store(&mut self.oam, address & 0xFFFFFF, value);
            }
            ROM0A_REGION..=ROM2B_REGION => {}
            SRAM_REGION => {
                let mut current_value = memory_load(&self.sram, address & 0xFFFFFF);
                current_value &= !(0xFF << 8 * (address & 0b11));
                let value = current_value | ((value as u32) << 8 * (address & 0b11));
                memory_store(&mut self.sram, address & 0xFFFFFF, value);
            }
            _ => panic!(),
        };

        self.wait_cycles_u16[region]
    }

    pub fn writeu16(&mut self, address: usize, value: u16) -> CYCLES {
        let region = address >> 24;
        match region {
            BIOS_REGION => {}
            EXWRAM_REGION => {
                let mut current_value = memory_load(&self.exwram, address & 0xFFFFFE);
                current_value &= !(0xFFFF << 16 * (address & 0b1));
                let value = current_value | ((value as u32) << 16 * (address >> 1 & 0b1));
                memory_store(&mut self.exwram, address & 0xFFFFFF, value);
            }
            IWRAM_REGION => {
                let mut current_value = memory_load(&self.iwram, address & 0xFFFFFE);
                current_value &= !(0xFFFF << 16 * (address & 0b11));
                let value = current_value | ((value as u32) << 16 * (address >> 1 & 0b1));
                memory_store(&mut self.iwram, address & 0xFFFFFF, value);
            }
            IORAM_REGION => {},
            BGRAM_REGION => {
                let mut current_value = memory_load(&self.bgram, address & 0xFFFFFE);
                current_value &= !(0xFFFF << 16 * (address & 0b11));
                let value = current_value | ((value as u32) << 16 * (address >> 1 & 0b1));
                memory_store(&mut self.bgram, address & 0xFFFFFF, value);
            }
            VRAM_REGION => {
                let mut current_value = memory_load(&self.vram, address & 0xFFFFFE);
                current_value &= !(0xFFFF << 16 * (address & 0b11));
                let value = current_value | ((value as u32) << 16 * (address >> 1 & 0b1));
                memory_store(&mut self.vram, address & 0xFFFFFF, value);
            }
            OAM_REGION => {
                let mut current_value = memory_load(&self.oam, address & 0xFFFFFE);
                current_value &= !(0xFFFF << 16 * (address & 0b11));
                let value = current_value | ((value as u32) << 16 * (address >> 1 & 0b1));
                memory_store(&mut self.oam, address & 0xFFFFFF, value);
            }
            ROM0A_REGION..=ROM2B_REGION => {}
            SRAM_REGION => {
                let mut current_value = memory_load(&self.sram, address & 0xFFFFFE);
                current_value &= !(0xFFFF << 16 * (address & 0b11));
                let value = current_value | ((value as u32) << 16 * (address >> 1 & 0b1));
                memory_store(&mut self.sram, address & 0xFFFFFF, value);
            }
            _ => panic!(),
        };

        self.wait_cycles_u16[region]
    }

    pub fn writeu32(&mut self, address: usize, value: u32) -> CYCLES {
        let region = address >> 24;
        match region {
            BIOS_REGION => {}
            EXWRAM_REGION => {
                memory_store(&mut self.exwram, address & 0xFFFFFF, value);
            }
            IWRAM_REGION => {
                memory_store(&mut self.iwram, address & 0xFFFFFF, value);
            }
            IORAM_REGION => {},
            BGRAM_REGION => {
                memory_store(&mut self.bgram, address & 0xFFFFFF, value);
            }
            VRAM_REGION => {
                memory_store(&mut self.vram, address & 0xFFFFFF, value);
            }
            OAM_REGION => {
                memory_store(&mut self.oam, address & 0xFFFFFF, value);
            }
            ROM0A_REGION..=ROM2B_REGION => {}
            SRAM_REGION => {
                memory_store(&mut self.sram, address & 0xFFFFFF, value);
            }
            _ => panic!(),
        };

        self.wait_cycles_u32[region]
    }
}

#[cfg(test)]
mod tests {
    use super::GBAMemory;

    #[test]
    fn can_read_byte_from_bios() {
        let mut memory = GBAMemory::new();
        let address = 0x4;
        let value = 0x12345678;
        memory.bios[address >> 2] = value;

        // Test that reads happen in little endian byte order
        for i in 0..4 {
            let mem_fetch = memory.read(address + i);
            assert_eq!(mem_fetch.data, ((value >> (i * 8)) & 0xFF) as u8);
            assert_eq!(mem_fetch.cycles, 1);
        }
    }

    #[test]
    fn can_read_byte_from_exwram() {
        let mut memory = GBAMemory::new();
        let address = 0x02000034;
        let value = 0x12345678;
        memory.exwram[(address & !0x2000000) >> 2] = value;

        // Test that reads happen in little endian byte order
        for i in 0..4 {
            let mem_fetch = memory.read(address + i);
            assert_eq!(mem_fetch.data, ((value >> (i * 8)) & 0xFF) as u8);
            assert_eq!(mem_fetch.cycles, 3);
        }
    }

    #[test]
    fn can_read_byte_from_iwram() {
        let mut memory = GBAMemory::new();
        let address = 0x03000034;
        let value = 0x12345678;
        memory.iwram[(address & !0x3000000) >> 2] = value;

        // Test that reads happen in little endian byte order
        for i in 0..4 {
            let mem_fetch = memory.read(address + i);
            assert_eq!(mem_fetch.data, ((value >> (i * 8)) & 0xFF) as u8);
            assert_eq!(mem_fetch.cycles, 1);
        }
    }

    #[test]
    fn can_read_byte_from_bgram() {
        let mut memory = GBAMemory::new();
        let address = 0x05000034;
        let value = 0x12345678;
        memory.bgram[(address & !0x5000000) >> 2] = value;

        // Test that reads happen in little endian byte order
        for i in 0..4 {
            let mem_fetch = memory.read(address + i);
            assert_eq!(mem_fetch.data, ((value >> (i * 8)) & 0xFF) as u8);
            assert_eq!(mem_fetch.cycles, 1);
        }
    }

    // Test that reads happen in little endian byte order
    #[test]
    fn can_read_byte_from_vram() {
        let mut memory = GBAMemory::new();
        let address = 0x06000034;
        let value = 0x12345678;
        memory.vram[(address & !0x6000000) >> 2] = value;

        // Test that reads happen in little endian byte order
        for i in 0..4 {
            let mem_fetch = memory.read(address + i);
            assert_eq!(mem_fetch.data, ((value >> (i * 8)) & 0xFF) as u8);
            assert_eq!(mem_fetch.cycles, 1);
        }
    }

    #[test]
    fn can_read_byte_from_oam() {
        let mut memory = GBAMemory::new();
        let address = 0x07000034;
        let value = 0x12345678;
        memory.oam[(address & !0x7000000) >> 2] = value;

        // Test that reads happen in little endian byte order
        for i in 0..4 {
            let mem_fetch = memory.read(address + i);
            assert_eq!(mem_fetch.data, ((value >> (i * 8)) & 0xFF) as u8);
            assert_eq!(mem_fetch.cycles, 1);
        }
    }

    #[test]
    fn can_read_byte_from_rom() {
        let mut memory = GBAMemory::new();
        let address = 0x08000034;
        let value = 0x12345678;
        memory.rom[(address & 0xFFFFFF) >> 2] = value;

        // Test that reads happen in little endian byte order
        for i in 0..4 {
            let mem_fetch = memory.read(address + i);
            assert_eq!(mem_fetch.data, ((value >> (i * 8)) & 0xFF) as u8);
            assert_eq!(mem_fetch.cycles, 5);
        }
    }

    #[test]
    fn can_read_byte_from_sram() {
        let mut memory = GBAMemory::new();
        let address = 0x0E000034;
        let value = 0x12345678;
        memory.sram[(address & 0xFFFFFF) >> 2] = value;

        // Test that reads happen in little endian byte order
        for i in 0..4 {
            let mem_fetch = memory.read(address + i);
            assert_eq!(mem_fetch.data, ((value >> (i * 8)) & 0xFF) as u8);
            assert_eq!(mem_fetch.cycles, 5);
        }
    }

    #[test]
    fn can_read_hword_from_bios() {
        let mut memory = GBAMemory::new();
        let address = 0x4;
        let value = 0x12345678;
        memory.bios[address >> 2] = value;

        let fetch = memory.readu16(address);
        assert_eq!(fetch.data, 0x5678);
        let fetch = memory.readu16(address + 2);

        assert_eq!(fetch.data, 0x1234);
        assert_eq!(fetch.cycles, 1);
    }

    #[test]
    fn can_read_hword_from_exwram() {
        let mut memory = GBAMemory::new();
        let address = 0x2000034;
        let value = 0x12345678;
        memory.exwram[(address & 0xFFFFFF) >> 2] = value;

        let fetch = memory.readu16(address);

        assert_eq!(fetch.data, 0x5678);
        assert_eq!(fetch.cycles, 3);
    }

    #[test]
    fn can_write_bytes_to_exwram() {
        let mut memory = GBAMemory::new();
        let address = 0x2000034;
        let value = 0x12345678;
        memory.exwram[(address & 0xFFFFFF) >> 2] = value;

        let cycles = memory.write(address, 0x55);
        memory.write(address + 1, 0x32);
        memory.write(address + 2, 0xde);
        memory.write(address + 3, 0xab);
        let fetch = memory.readu32(address);

        assert_eq!(cycles, 3);
        assert_eq!(fetch.data, 0xabde3255);
    }

    #[test]
    fn can_write_hword_to_exwram() {
        let mut memory = GBAMemory::new();
        let address = 0x2000034;
        let value = 0x12345678;
        memory.exwram[(address & 0xFFFFFF) >> 2] = value;

        let cycles = memory.writeu16(address, 0x1255);
        let fetch = memory.readu32(address);

        assert_eq!(cycles, 3);
        assert_eq!(fetch.data, 0x12341255);
    }

    #[test]
    fn can_write_word_to_exwram() {
        let mut memory = GBAMemory::new();
        let address = 0x2000034;
        let value = 0x12345678;
        memory.exwram[(address & 0xFFFFFF) >> 2] = value;

        let cycles = memory.writeu32(address, 0xabcdef12);
        let fetch = memory.readu32(address);

        assert_eq!(cycles, 6);
        assert_eq!(fetch.data, 0xabcdef12);
    }
}
