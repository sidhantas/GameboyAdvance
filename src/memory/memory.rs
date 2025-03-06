use num_traits::{ToBytes, Unsigned};

use crate::{
    graphics::oam::OAM,
    types::{BYTE, CYCLES, HWORD, WORD},
};
use std::{
    collections::HashMap,
    fmt::Display,
    fs::File,
    io::{Read, Seek},
    usize,
};

use super::io_handlers::{DISPSTAT, IF, KEYINPUT};

pub struct MemoryFetch<T> {
    pub cycles: CYCLES,
    pub data: T,
}

impl<T> MemoryFetch<T> {
    pub fn new(data: T, cycles: CYCLES) -> Self {
        Self { cycles, data }
    }
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

#[derive(Clone, Copy, Debug)]
pub enum CPUCallbacks {
    Halt,
    Stop,
    RaiseIrq,
}

pub struct GBAMemory {
    bios: Vec<u32>,
    exwram: Vec<u32>,
    iwram: Vec<u32>,
    pub(super) ioram: Vec<u16>,
    bgram: Vec<u32>,
    vram: Vec<u32>,
    pub oam: Vec<u32>,
    rom: Vec<u32>,
    sram: Vec<u32>,
    wait_cycles_u16: [u8; 15],
    wait_cycles_u32: [u8; 15],
    pub cpu_commands: Vec<CPUCallbacks>,
}

#[inline(always)]
fn memory_load(region: &Vec<u32>, address: usize) -> u32 {
    *region.get(address >> 2).unwrap_or(&0)
}

#[inline(always)]
fn memory_store(region: &mut Vec<u32>, address: usize, value: u32) {
    let store_address = address >> 2;
    if store_address < region.len() {
        region[store_address] = value;
    }
}

#[inline]
fn memory_store2(region: &mut Vec<u8>, address: usize, value: u8) {
    if address < region.len() {
        region[address] = value;
    }
}

#[derive(Debug)]
pub enum MemoryError {
    NoIODefinition(usize),
    ReadError(usize),
    WriteError(usize, u32),
}

impl Display for MemoryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MemoryError::ReadError(address) => write!(f, "Read Error: {:#X}", address),
            MemoryError::WriteError(address, value) => {
                write!(f, "Write Error: {:#X} <- {:#X}", address, value)
            }
            MemoryError::NoIODefinition(address) => {
                write!(f, "No IO Definition Provided: {:#X}", address)
            }
        }
    }
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

        let mut memory = Self {
            bios: vec![0; BIOS_SIZE >> 2],
            exwram: vec![0; EXWRAM_SIZE >> 2],
            iwram: vec![0; IWRAM_SIZE >> 2],
            ioram: vec![0; IORAM_SIZE >> 1],
            bgram: vec![0; BGRAM_SIZE >> 2],
            vram: vec![0; VRAM_SIZE >> 2],
            oam: vec![0; OAM_SIZE >> 2],
            rom: vec![0; ROM_SIZE >> 2],
            sram: vec![0; SRAM_SIZE >> 2],
            wait_cycles_u16,
            wait_cycles_u32,
            cpu_commands: Vec::new(),
        };
        memory.io_store(0x088, 0x200);
        memory.io_store(KEYINPUT, 0x03FF);
        memory
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
        }
        Ok(())
    }

    pub fn initialize_rom(&mut self, filename: String) -> Result<(), std::io::Error> {
        let mut index = 0;
        let mut rom_file = File::options().read(true).open(filename).unwrap();
        let mut buffer = [0; 4];
        rom_file.rewind()?;
        while let Ok(read_bytes) = rom_file.read(&mut buffer[..]) {
            if read_bytes == 0 {
                break;
            }
            self.rom[index] = u32::from_le_bytes(buffer.clone());
            index += 1;
        }

        Ok(())
    }
}

const EX_WRAM_MIRROR_MASK: usize = 0x3FFFF;
const IW_WRAM_MIRROR_MASK: usize = 0x7FFF;
const BGRAM_MIRROR_MASK: usize = 0x3FF;
const OAM_MIRROR_MASK: usize = 0x3FF;

impl GBAMemory {
    fn try_read(&self, address: usize) -> Result<MemoryFetch<u8>, MemoryError> {
        let region = address >> 24;
        let data = match region {
            BIOS_REGION => memory_load(&self.bios, address).to_le_bytes()[address & 0b11],
            EXWRAM_REGION => memory_load(&self.exwram, address & EX_WRAM_MIRROR_MASK).to_le_bytes()
                [address & 0b11],
            IWRAM_REGION => memory_load(&self.iwram, address & IW_WRAM_MIRROR_MASK).to_le_bytes()
                [address & 0b11],
            IORAM_REGION => self.io_readu8(address)?,
            BGRAM_REGION => {
                memory_load(&self.bgram, address & BGRAM_MIRROR_MASK).to_le_bytes()[address & 0b11]
            }
            VRAM_REGION => {
                memory_load(&self.vram, address & 0xFFFFFF).to_le_bytes()[address & 0b11]
            }
            OAM_REGION => {
                memory_load(&self.oam, address & OAM_MIRROR_MASK).to_le_bytes()[address & 0b11]
            }
            ROM0A_REGION..=ROM2B_REGION => {
                memory_load(&self.rom, address & 0xFFFFFF).to_le_bytes()[address & 0b11]
            }
            SRAM_REGION => {
                memory_load(&self.sram, address & 0xFFFFFF).to_le_bytes()[address & 0b11]
            }
            _ => return Err(MemoryError::ReadError(address)),
        };

        Ok(MemoryFetch::new(data, self.wait_cycles_u16[region]))
    }

    fn try_readu16(&self, address: usize) -> Result<MemoryFetch<u16>, MemoryError> {
        let region = address >> 24;
        let data = match region {
            BIOS_REGION => memory_load(&self.bios, address),
            EXWRAM_REGION => memory_load(&self.exwram, address & EX_WRAM_MIRROR_MASK),
            IWRAM_REGION => memory_load(&self.iwram, address & IW_WRAM_MIRROR_MASK),
            IORAM_REGION => {
                return Ok(MemoryFetch {
                    data: self.io_readu16(address)?,
                    cycles: self.wait_cycles_u16[region],
                })
            }
            BGRAM_REGION => memory_load(&self.bgram, address & BGRAM_MIRROR_MASK),
            VRAM_REGION => memory_load(&self.vram, address & 0xFFFFFF),
            OAM_REGION => memory_load(&self.oam, address & OAM_MIRROR_MASK),
            ROM0A_REGION..=ROM2B_REGION => memory_load(&self.rom, address & 0xFFFFFF),
            SRAM_REGION => memory_load(&self.sram, address & 0xFFFFFF),
            _ => return Err(MemoryError::ReadError(address)),
        };

        let shift_amount = 16 * ((address >> 1) & 0x1);
        let data = data >> shift_amount;

        Ok(MemoryFetch::new(data as u16, self.wait_cycles_u16[region]))
    }

    fn try_readu32(&self, address: usize) -> Result<MemoryFetch<u32>, MemoryError> {
        let region = address >> 24;
        let data = match region {
            BIOS_REGION => memory_load(&self.bios, address),
            EXWRAM_REGION => memory_load(&self.exwram, address & EX_WRAM_MIRROR_MASK),
            IWRAM_REGION => memory_load(&self.iwram, address & IW_WRAM_MIRROR_MASK),
            IORAM_REGION => self.io_readu32(address)?,
            BGRAM_REGION => memory_load(&self.bgram, address & BGRAM_MIRROR_MASK),
            VRAM_REGION => memory_load(&self.vram, address & 0xFFFFFF),
            OAM_REGION => memory_load(&self.oam, address & OAM_MIRROR_MASK),
            ROM0A_REGION..=ROM2B_REGION => memory_load(&self.rom, address & 0xFFFFFF),
            SRAM_REGION => memory_load(&self.sram, address & 0xFFFFFF),
            _ => return Err(MemoryError::ReadError(address)),
        };

        Ok(MemoryFetch::new(
            data.rotate_right(8 * (address as u32 & 0b11)),
            self.wait_cycles_u32[region],
        ))
    }

    fn try_write(&mut self, address: usize, value: u8) -> Result<CYCLES, MemoryError> {
        let region = address >> 24;
        match region {
            BIOS_REGION => {}
            EXWRAM_REGION => {
                let mirror_masked_address = address & EX_WRAM_MIRROR_MASK;
                let mut current_value = memory_load(&self.exwram, mirror_masked_address);
                current_value &= !(0xFF << 8 * (address & 0b11));
                let value = current_value | ((value as u32) << (8 * (address & 0b11)));
                memory_store(&mut self.exwram, mirror_masked_address, value);
            }
            IWRAM_REGION => {
                let mirror_masked_address = address & IW_WRAM_MIRROR_MASK;
                let mut current_value = memory_load(&self.iwram, mirror_masked_address);
                current_value &= !(0xFF << 8 * (mirror_masked_address & 0b11));
                let value =
                    current_value | ((value as u32) << (8 * (mirror_masked_address & 0b11)));
                memory_store(&mut self.iwram, mirror_masked_address, value);
            }
            IORAM_REGION => self.io_writeu8(address, value)?,
            BGRAM_REGION => {
                let mirror_masked_address = address & BGRAM_MIRROR_MASK;
                let mut current_value = memory_load(&self.bgram, mirror_masked_address);
                current_value &= !(0xFF << 8 * (mirror_masked_address & 0b11));
                let value =
                    current_value | ((value as u32) << (8 * (mirror_masked_address & 0b11)));
                memory_store(&mut self.bgram, mirror_masked_address, value);
            }
            VRAM_REGION => {
                let mut current_value = memory_load(&self.vram, address & 0xFFFFFF);
                current_value &= !(0xFF << 8 * (address & 0b11));
                let value = current_value | ((value as u32) << (8 * (address & 0b11)));
                memory_store(&mut self.vram, address & 0xFFFFFF, value);
            }
            OAM_REGION => {
                let mirror_masked_address = address & OAM_MIRROR_MASK;
                let mut current_value = memory_load(&self.oam, mirror_masked_address);
                current_value &= !(0xFF << 8 * (mirror_masked_address & 0b11));
                let value =
                    current_value | ((value as u32) << (8 * (mirror_masked_address & 0b11)));
                memory_store(&mut self.oam, mirror_masked_address, value);
            }
            ROM0A_REGION..=ROM2B_REGION => {}
            SRAM_REGION => {
                let mut current_value = memory_load(&self.sram, address & 0xFFFFFF);
                current_value &= !(0xFF << 8 * (address & 0b11));
                let value = current_value | ((value as u32) << (8 * (address & 0b11)));
                memory_store(&mut self.sram, address & 0xFFFFFF, value);
            }
            _ => return Err(MemoryError::WriteError(address, value as u32)),
        };

        Ok(self.wait_cycles_u16[region])
    }

    fn try_writeu16(&mut self, address: usize, value: u16) -> Result<CYCLES, MemoryError> {
        let region = address >> 24;
        match region {
            BIOS_REGION => {}
            EXWRAM_REGION => {
                let mirror_masked_address = address & EX_WRAM_MIRROR_MASK;
                let mut current_value = memory_load(&self.exwram, mirror_masked_address & 0xFFFFFE);
                current_value &= !(0xFFFFu32 << (16 * ((mirror_masked_address >> 1) & 0b1)));
                let value =
                    current_value | ((value as u32) << (16 * ((mirror_masked_address >> 1) & 0b1)));
                memory_store(&mut self.exwram, mirror_masked_address & 0xFFFFFF, value);
            }
            IWRAM_REGION => {
                let mirror_masked_address = address & IW_WRAM_MIRROR_MASK;
                let mut current_value = memory_load(&self.iwram, mirror_masked_address & 0xFFFFFE);
                current_value &= !(0xFFFFu32 << (16 * ((mirror_masked_address >> 1) & 0b1)));
                let value =
                    current_value | ((value as u32) << (16 * ((mirror_masked_address >> 1) & 0b1)));
                memory_store(&mut self.iwram, mirror_masked_address & 0xFFFFFF, value);
            }
            IORAM_REGION => self.io_writeu16(address, value)?,
            BGRAM_REGION => {
                let mirror_masked_address = address & BGRAM_MIRROR_MASK;
                let mut current_value = memory_load(&self.bgram, mirror_masked_address & 0xFFFFFE);
                current_value &= !(0xFFFFu32 << (16 * ((mirror_masked_address >> 1) & 0b1)));
                let value =
                    current_value | ((value as u32) << (16 * ((mirror_masked_address >> 1) & 0b1)));
                memory_store(&mut self.bgram, mirror_masked_address & 0xFFFFFF, value);
            }
            VRAM_REGION => {
                let mut current_value = memory_load(&self.vram, address & 0xFFFFFE);
                current_value &= !(0xFFFFu32 << (16 * ((address >> 1) & 0b1)));
                let value = current_value | ((value as u32) << (16 * ((address >> 1) & 0b1)));
                memory_store(&mut self.vram, address & 0xFFFFFF, value);
            }
            OAM_REGION => {
                let mirror_masked_address = address & OAM_MIRROR_MASK;
                let mut current_value = memory_load(&self.oam, mirror_masked_address & 0xFFFFFE);
                current_value &= !(0xFFFFu32 << (16 * ((mirror_masked_address >> 1) & 0b1)));
                let value =
                    current_value | ((value as u32) << (16 * ((mirror_masked_address >> 1) & 0b1)));
                memory_store(&mut self.oam, mirror_masked_address & 0xFFFFFF, value);
            }
            ROM0A_REGION..=ROM2B_REGION => {}
            SRAM_REGION => {
                let mut current_value = memory_load(&self.sram, address & 0xFFFFFE);
                current_value &= !(0xFFFFu32 << (16 * ((address >> 1) & 0b1)));
                let value = current_value | ((value as u32) << (16 * ((address >> 1) & 0b1)));
                memory_store(&mut self.sram, address & 0xFFFFFF, value);
            }
            _ => return Err(MemoryError::WriteError(address, value as u32)),
        };

        Ok(self.wait_cycles_u16[region])
    }

    fn try_writeu32(&mut self, address: usize, value: u32) -> Result<CYCLES, MemoryError> {
        let region = address >> 24;
        match region {
            BIOS_REGION => {}
            EXWRAM_REGION => {
                let mirror_masked_address = address & EX_WRAM_MIRROR_MASK;
                memory_store(&mut self.exwram, mirror_masked_address, value);
            }
            IWRAM_REGION => {
                let mirror_masked_address = address & IW_WRAM_MIRROR_MASK;
                memory_store(&mut self.iwram, mirror_masked_address & 0xFFFFFF, value);
            }
            IORAM_REGION => self.io_writeu32(address, value)?,
            BGRAM_REGION => {
                let mirror_masked_address = address & OAM_MIRROR_MASK;
                memory_store(&mut self.bgram, mirror_masked_address & 0xFFFFFF, value);
            }
            VRAM_REGION => {
                memory_store(&mut self.vram, address & 0xFFFFFF, value);
            }
            OAM_REGION => {
                let mirror_masked_address = address & OAM_MIRROR_MASK;
                memory_store(&mut self.oam, mirror_masked_address & 0xFFFFFF, value);
            }
            ROM0A_REGION..=ROM2B_REGION => {}
            SRAM_REGION => {
                memory_store(&mut self.sram, address & 0xFFFFFF, value);
            }
            _ => return Err(MemoryError::WriteError(address, value as u32)),
        };

        Ok(self.wait_cycles_u32[region])
    }
}

impl GBAMemory {
    pub fn read(&self, address: usize) -> MemoryFetch<u8> {
        self.try_read(address).unwrap()
    }

    pub fn readu16(&self, address: usize) -> MemoryFetch<u16> {
        self.try_readu16(address).unwrap()
    }

    pub fn readu32(&self, address: usize) -> MemoryFetch<u32> {
        self.try_readu32(address).unwrap()
    }

    pub fn write(&mut self, address: usize, value: u8) -> CYCLES {
        self.try_write(address, value).unwrap()
    }

    pub fn writeu16(&mut self, address: usize, value: u16) -> CYCLES {
        self.try_writeu16(address, value).unwrap()
    }

    pub fn writeu32(&mut self, address: usize, value: u32) -> CYCLES {
        self.try_writeu32(address, value).unwrap()
    }
}

pub struct GBAMemory2 {
    bios: Vec<u8>,
    exwram: Vec<u8>,
    iwram: Vec<u8>,
    pub(super) ioram: Vec<u16>,
    bgram: Vec<u8>,
    vram: Vec<u8>,
    pub oam: Vec<u8>,
    rom: Vec<u8>,
    sram: Vec<u8>,
    wait_cycles_u16: [u8; 15],
    wait_cycles_u32: [u8; 15],
    pub cpu_commands: Vec<CPUCallbacks>,
}

impl GBAMemory2 {
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

        let memory = Self {
            bios: vec![0; BIOS_SIZE],
            exwram: vec![0; EXWRAM_SIZE],
            iwram: vec![0; IWRAM_SIZE],
            ioram: vec![0; IORAM_SIZE >> 1],
            bgram: vec![0; BGRAM_SIZE],
            vram: vec![0; VRAM_SIZE],
            oam: vec![0; OAM_SIZE],
            rom: vec![0; ROM_SIZE],
            sram: vec![0; SRAM_SIZE],
            wait_cycles_u16,
            wait_cycles_u32,
            cpu_commands: Vec::new(),
        };

        memory
    }

    const fn get_slice_alignment(size: usize) -> usize {
        match size {
            1 => !0x0,
            2 => !0x1,
            4 => !0x3,
            _ => unreachable!(),
        }
    }

    #[inline]
    fn get_memory_slice_mut<const SIZE: usize>(
        &mut self,
        region: usize,
        address: usize,
    ) -> Option<&mut [u8; SIZE]> {
        let address = address & Self::get_slice_alignment(SIZE);

        let (mirror_masked_address, slice): (usize, &mut Vec<u8>) = match region {
            BIOS_REGION => return None,
            EXWRAM_REGION => {
                let mirror_masked_address = address & EX_WRAM_MIRROR_MASK;
                (mirror_masked_address, self.exwram.as_mut())
            }
            _ => return None,
        };

        Some(
            slice[mirror_masked_address..][..SIZE]
                .as_mut()
                .try_into()
                .unwrap(),
        )
    }

    #[inline]
    fn get_memory_slice<const SIZE: usize>(
        &self,
        region: usize,
        address: usize,
    ) -> Option<&[u8; SIZE]> {
        let address = address & Self::get_slice_alignment(SIZE);

        let (mirror_masked_address, slice): (usize, &Vec<u8>) = match region {
            BIOS_REGION => (address, self.bios.as_ref()),
            EXWRAM_REGION => {
                let mirror_masked_address = address & EX_WRAM_MIRROR_MASK;
                (mirror_masked_address, self.exwram.as_ref())
            }
            _ => return None,
        };

        Some(
            slice[mirror_masked_address..][..SIZE]
                .as_ref()
                .try_into()
                .unwrap(),
        )
    }

    pub fn writeu8(&mut self, address: usize, value: u8) -> CYCLES {
        let region = address >> 24;
        let memory_reference = self
            .get_memory_slice_mut::<{ std::mem::size_of::<u8>() }>(region, address)
            .unwrap();

        memory_reference[0] = value;
        self.wait_cycles_u16[region]
    }

    pub fn writeu16(&mut self, address: usize, value: u16) -> CYCLES {
        let region = address >> 24;
        let memory_reference = self
            .get_memory_slice_mut::<{ std::mem::size_of::<u16>() }>(region, address)
            .unwrap();

        memory_reference.copy_from_slice(&value.to_le_bytes());
        self.wait_cycles_u16[region]
    }

    pub fn writeu32(&mut self, address: usize, value: u32) -> CYCLES {
        let region = address >> 24;
        let memory_reference = self
            .get_memory_slice_mut::<{ std::mem::size_of::<u32>() }>(region, address)
            .unwrap();

        memory_reference.copy_from_slice(&value.to_le_bytes());
        self.wait_cycles_u32[region]
    }

    pub fn readu8(&self, address: usize) -> MemoryFetch<u8> {
        let region = address >> 24;
        let memory_reference = self
            .get_memory_slice::<{ std::mem::size_of::<u8>() }>(region, address)
            .unwrap();

        MemoryFetch {
            cycles: self.wait_cycles_u16[region],
            data: memory_reference[0],
        }
    }

    pub fn readu16(&self, address: usize) -> MemoryFetch<u16> {
        let region = address >> 24;
        let memory_reference = self
            .get_memory_slice::<{ std::mem::size_of::<u16>() }>(region, address)
            .unwrap();

        MemoryFetch {
            cycles: self.wait_cycles_u16[region],
            data: u16::from_le_bytes(*memory_reference),
        }
    }

    pub fn readu32(&self, address: usize) -> MemoryFetch<u32> {
        let region = address >> 24;
        let memory_reference = self
            .get_memory_slice::<{ std::mem::size_of::<u32>() }>(region, address)
            .unwrap();
        let data = u32::from_le_bytes(*memory_reference);
        MemoryFetch {
            cycles: self.wait_cycles_u32[region],
            data: data.rotate_right(8 * (address as u32 & 0b11)),
        }
    }
}

#[cfg(test)]
mod memory2_tests {
    use super::GBAMemory2;

    #[test]
    fn can_writeu32() {
        let mut memory = GBAMemory2::new();
        let address = 0x2000004;
        let value = 0x12345678;
        memory.writeu32(address, value);

        let readu32 = memory.readu32(address).data;
        assert_eq!(readu32, value);

        let readu16 = memory.readu16(address).data;
        assert_eq!(readu16, 0x5678);
        let readu16 = memory.readu16(address + 2).data;
        assert_eq!(readu16, 0x1234);

        let read = memory.readu8(address).data;
        assert_eq!(read, 0x78);
        let read = memory.readu8(address + 2).data;
        assert_eq!(read, 0x34);

        assert_eq!(memory.exwram[0x4], 0x78);
        assert_eq!(memory.exwram[0x5], 0x56);
        assert_eq!(memory.exwram[0x6], 0x34);
        assert_eq!(memory.exwram[0x7], 0x12);
    }

    #[test]
    fn can_writeu16() {
        let mut memory = GBAMemory2::new();
        let address = 0x2000004;
        let value = 0x1234;
        memory.writeu16(address, value);
        let readu32 = memory.readu32(address).data;
        assert_eq!(readu32, 0x1234);

        let readu16 = memory.readu16(address).data;
        assert_eq!(readu16, 0x1234);

        let read = memory.readu8(address).data;
        assert_eq!(read, 0x34);
        let read = memory.readu8(address + 1).data;
        assert_eq!(read, 0x12);

        assert_eq!(memory.exwram[0x4], 0x34);
        assert_eq!(memory.exwram[0x5], 0x12);
        assert_eq!(memory.exwram[0x6], 0x00);
        assert_eq!(memory.exwram[0x7], 0x00);
    }

    #[test]
    fn can_writeu8() {
        let mut memory = GBAMemory2::new();
        let address = 0x2000004;
        memory.writeu8(address, 0x12);
        memory.writeu8(address + 1, 0x34);
        memory.writeu8(address + 2, 0x56);
        memory.writeu8(address + 3, 0x78);

        let readu32 = memory.readu32(address).data;
        assert_eq!(readu32, 0x78563412);

        let readu16 = memory.readu16(address).data;
        assert_eq!(readu16, 0x3412);

        let read = memory.readu8(address).data;
        assert_eq!(read, 0x12);
        let read = memory.readu8(address + 1).data;
        assert_eq!(read, 0x34);

        assert_eq!(memory.exwram[0x4], 0x12);
        assert_eq!(memory.exwram[0x5], 0x34);
        assert_eq!(memory.exwram[0x6], 0x56);
        assert_eq!(memory.exwram[0x7], 0x78);
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
    fn can_write_byte_to_oam() {
        let mut memory = GBAMemory::new();
        let address = 0x07000034;
        let value = 0x12345678;
        for i in 0..4 {
            memory.write(address + i, (value >> (i * 8)) as u8);
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
        let cycles = memory.writeu16(address + 2, 0x1255);
        let fetch = memory.readu32(address);

        assert_eq!(cycles, 3);
        assert_eq!(fetch.data, 0x12551255);
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
