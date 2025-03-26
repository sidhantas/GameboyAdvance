use crate::{
    debugger::breakpoints::{Breakpoint, TriggeredWatchpoints}, io::timers::Timers, types::{BYTE, CYCLES, HWORD, WORD}
};
use std::{
    cell::RefCell, fmt::Display, fs::File, io::{Read, Seek}, rc::Rc, sync::Arc, usize
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

const EX_WRAM_MIRROR_MASK: usize = 0x3FFFF;
const IWRAM_MIRROR_MASK: usize = 0x7FFF;
const BGRAM_MIRROR_MASK: usize = 0x3FF;
const OAM_MIRROR_MASK: usize = 0x3FF;

pub struct GBAMemory {
    bios: Vec<u8>,
    exwram: Vec<u8>,
    iwram: Vec<u8>,
    pub(super) ioram: Vec<u16>,
    pub pallete_ram: Vec<u8>,
    pub vram: Vec<u8>,
    pub oam: Vec<u8>,
    rom: Vec<u8>,
    sram: Vec<u8>,
    wait_cycles_u16: [u8; 15],
    wait_cycles_u32: [u8; 15],
    pub cpu_commands: Vec<CPUCallbacks>,
    pub timers: Option<Timers>,
    pub(crate) breakpoint_checker: Option<Box<dyn Fn(&GBAMemory, usize) -> ()>>,
    pub triggered_breakpoints: Rc<RefCell<Vec<TriggeredWatchpoints>>>,
    pub breakpoints: Option<Vec<Breakpoint>>
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
            bios: vec![0; BIOS_SIZE],
            exwram: vec![0; EXWRAM_SIZE],
            iwram: vec![0; IWRAM_SIZE],
            ioram: vec![0; IORAM_SIZE >> 1],
            pallete_ram: vec![0; BGRAM_SIZE],
            vram: vec![0; VRAM_SIZE],
            oam: vec![0; OAM_SIZE],
            rom: vec![0; ROM_SIZE],
            sram: vec![0; SRAM_SIZE],
            wait_cycles_u16,
            wait_cycles_u32,
            cpu_commands: Vec::new(),
            timers: Some(Timers::new()),
            breakpoint_checker: None,
            triggered_breakpoints: Rc::new(RefCell::new(Vec::new())),
            breakpoints: None,
        };

        memory.io_store(0x088, 0x200);
        memory.io_store(KEYINPUT, 0x03FF);
        memory
    }

    pub fn initialize_bios(&mut self, filename: String) -> Result<(), std::io::Error> {
        let mut index = 0;
        let mut bios_file = File::options().read(true).open(filename)?;
        let mut buffer = [0; 1];
        bios_file.rewind()?;
        while let Ok(read_bytes) = bios_file.read(&mut buffer[..]) {
            if read_bytes == 0 {
                break;
            }
            self.bios[index] = buffer.clone()[0];
            index += 1;
        }
        Ok(())
    }

    pub fn initialize_rom(&mut self, filename: String) -> Result<(), std::io::Error> {
        let mut index = 0;
        let mut rom_file = File::options().read(true).open(filename).unwrap();
        let mut buffer = [0; 1];
        rom_file.rewind()?;
        while let Ok(read_bytes) = rom_file.read(&mut buffer[..]) {
            if read_bytes == 0 {
                break;
            }
            self.rom[index] = buffer.clone()[0];
            index += 1;
        }

        Ok(())
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
            EXWRAM_REGION => (address & EX_WRAM_MIRROR_MASK, self.exwram.as_mut()),
            IWRAM_REGION => (address & IWRAM_MIRROR_MASK, self.iwram.as_mut()),
            BGRAM_REGION => (address & BGRAM_MIRROR_MASK, self.pallete_ram.as_mut()),
            VRAM_REGION => (address & 0xFFFFFF, self.vram.as_mut()),
            OAM_REGION => (address & OAM_MIRROR_MASK, self.oam.as_mut()),
            ROM0A_REGION..=ROM2B_REGION => return None,
            SRAM_REGION => (address & SRAM_REGION, self.sram.as_mut()),
            _ => return None,
        };

        if mirror_masked_address + SIZE > slice.len() {
            return None;
        }

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
            EXWRAM_REGION => (address & EX_WRAM_MIRROR_MASK, self.exwram.as_ref()),
            IWRAM_REGION => (address & IWRAM_MIRROR_MASK, self.iwram.as_ref()),
            BGRAM_REGION => (address & BGRAM_MIRROR_MASK, self.pallete_ram.as_ref()),
            VRAM_REGION => (address & 0xFFFFFF, self.vram.as_ref()),
            OAM_REGION => (address & OAM_MIRROR_MASK, self.oam.as_ref()),
            ROM0A_REGION..=ROM2B_REGION => (address & 0xFFFFFF, self.rom.as_ref()),
            SRAM_REGION => (address & SRAM_REGION, self.sram.as_ref()),
            _ => return None,
        };
        if mirror_masked_address + SIZE > slice.len() {
            return None;
        }

        Some(
            slice[mirror_masked_address..][..SIZE]
                .as_ref()
                .try_into()
                .unwrap(),
        )
    }

    pub fn write(&mut self, address: usize, value: u8) -> CYCLES {
        let region = address >> 24;

        if region == IORAM_REGION {
            self.io_writeu8(address, value);
        }
        if let Some(memory_reference) =
            self.get_memory_slice_mut::<{ std::mem::size_of::<u8>() }>(region, address)
        {
            memory_reference[0] = value;
        }
        if let Some(breakpoint_checker) = &self.breakpoint_checker {
            breakpoint_checker(self, address);
        }

        self.wait_cycles_u16[region]
    }

    pub fn writeu16(&mut self, address: usize, value: u16) -> CYCLES {
        let region = address >> 24;
        if region == IORAM_REGION {
            self.io_writeu16(address, value);
        }
        if let Some(memory_reference) =
            self.get_memory_slice_mut::<{ std::mem::size_of::<u16>() }>(region, address)
        {
            memory_reference.copy_from_slice(&value.to_le_bytes());
        }
        if let Some(breakpoint_checker) = &self.breakpoint_checker {
            breakpoint_checker(self, address);
        }

        self.wait_cycles_u16[region]
    }

    pub fn writeu32(&mut self, address: usize, value: u32) -> CYCLES {
        let region = address >> 24;
        if region == IORAM_REGION {
            self.io_writeu32(address, value);
        }
        if let Some(memory_reference) =
            self.get_memory_slice_mut::<{ std::mem::size_of::<u32>() }>(region, address)
        {
            memory_reference.copy_from_slice(&value.to_le_bytes());
        }
        if let Some(breakpoint_checker) = &self.breakpoint_checker {
            breakpoint_checker(self, address);
        }

        self.wait_cycles_u32[region]
    }

    pub fn read(&self, address: usize) -> MemoryFetch<u8> {
        let region = address >> 24;
        let read = if region == IORAM_REGION {
            self.io_readu8(address).unwrap()
        } else {
            self.get_memory_slice::<{ std::mem::size_of::<u8>() }>(region, address)
                .map_or(0, |slice| slice[0])
        };

        if let Some(breakpoint_checker) = &self.breakpoint_checker {
            breakpoint_checker(self, address);
        }

        MemoryFetch {
            cycles: self.wait_cycles_u16[region],
            data: read,
        }
    }

    pub fn read_raw(&self, address: usize) -> u8 {
        let region = address >> 24;
        if region == IORAM_REGION {
            self.io_readu8(address).unwrap()
        } else {
            self.get_memory_slice::<{ std::mem::size_of::<u8>() }>(region, address)
                .map_or(0, |slice| slice[0])
        }
    }

    pub fn readu16(&self, address: usize) -> MemoryFetch<u16> {
        let region = address >> 24;
        let read = if region == IORAM_REGION {
            self.io_readu16(address).unwrap()
        } else {
            self.get_memory_slice::<{ std::mem::size_of::<u16>() }>(region, address)
                .map_or(0, |slice| u16::from_le_bytes(*slice))
        };
        if let Some(breakpoint_checker) = &self.breakpoint_checker {
            breakpoint_checker(self, address);
        }
        MemoryFetch {
            cycles: self.wait_cycles_u16[region],
            data: read,
        }
    }

    pub fn readu32(&self, address: usize) -> MemoryFetch<u32> {
        let region = address >> 24;
        let data = if region == IORAM_REGION {
            self.io_readu32(address).unwrap()
        } else {
            self.get_memory_slice::<{ std::mem::size_of::<u32>() }>(region, address)
                .map_or(0, |slice| u32::from_le_bytes(*slice))
        };
        if let Some(breakpoint_checker) = &self.breakpoint_checker {
            breakpoint_checker(self, address);
        }
        MemoryFetch {
            cycles: self.wait_cycles_u32[region],
            data: data.rotate_right(8 * (address as u32 & 0b11)),
        }
    }
}

#[cfg(test)]
mod memory2_tests {
    use super::GBAMemory;

    #[test]
    fn can_writeu32() {
        let mut memory = GBAMemory::new();
        let address = 0x2000004;
        let value = 0x12345678;
        memory.writeu32(address, value);

        let readu32 = memory.readu32(address).data;
        assert_eq!(readu32, value);

        let readu16 = memory.readu16(address).data;
        assert_eq!(readu16, 0x5678);
        let readu16 = memory.readu16(address + 2).data;
        assert_eq!(readu16, 0x1234);

        let read = memory.read(address).data;
        assert_eq!(read, 0x78);
        let read = memory.read(address + 2).data;
        assert_eq!(read, 0x34);

        assert_eq!(memory.exwram[0x4], 0x78);
        assert_eq!(memory.exwram[0x5], 0x56);
        assert_eq!(memory.exwram[0x6], 0x34);
        assert_eq!(memory.exwram[0x7], 0x12);
    }

    #[test]
    fn can_writeu16() {
        let mut memory = GBAMemory::new();
        let address = 0x2000004;
        let value = 0x1234;
        memory.writeu16(address, value);
        let readu32 = memory.readu32(address).data;
        assert_eq!(readu32, 0x1234);

        let readu16 = memory.readu16(address).data;
        assert_eq!(readu16, 0x1234);

        let read = memory.read(address).data;
        assert_eq!(read, 0x34);
        let read = memory.read(address + 1).data;
        assert_eq!(read, 0x12);

        assert_eq!(memory.exwram[0x4], 0x34);
        assert_eq!(memory.exwram[0x5], 0x12);
        assert_eq!(memory.exwram[0x6], 0x00);
        assert_eq!(memory.exwram[0x7], 0x00);
    }

    #[test]
    fn can_writeu8() {
        let mut memory = GBAMemory::new();
        let address = 0x2000004;
        memory.write(address, 0x12);
        memory.write(address + 1, 0x34);
        memory.write(address + 2, 0x56);
        memory.write(address + 3, 0x78);

        let readu32 = memory.readu32(address).data;
        assert_eq!(readu32, 0x78563412);

        let readu16 = memory.readu16(address).data;
        assert_eq!(readu16, 0x3412);

        let read = memory.read(address).data;
        assert_eq!(read, 0x12);
        let read = memory.read(address + 1).data;
        assert_eq!(read, 0x34);

        assert_eq!(memory.exwram[0x4], 0x12);
        assert_eq!(memory.exwram[0x5], 0x34);
        assert_eq!(memory.exwram[0x6], 0x56);
        assert_eq!(memory.exwram[0x7], 0x78);
    }
}

//#[cfg(test)]
//mod tests {
//    use super::GBAMemory;
//
//    #[test]
//    fn can_read_byte_from_bios() {
//        let mut memory = GBAMemory::new();
//        let address = 0x4;
//        let value = 0x12345678;
//        memory.bios[address >> 2] = value;
//
//        // Test that reads happen in little endian byte order
//        for i in 0..4 {
//            let mem_fetch = memory.read(address + i);
//            assert_eq!(mem_fetch.data, ((value >> (i * 8)) & 0xFF) as u8);
//            assert_eq!(mem_fetch.cycles, 1);
//        }
//    }
//
//    #[test]
//    fn can_read_byte_from_exwram() {
//        let mut memory = GBAMemory::new();
//        let address = 0x02000034;
//        let value = 0x12345678;
//        memory.exwram[(address & !0x2000000) >> 2] = value;
//
//        // Test that reads happen in little endian byte order
//        for i in 0..4 {
//            let mem_fetch = memory.read(address + i);
//            assert_eq!(mem_fetch.data, ((value >> (i * 8)) & 0xFF) as u8);
//            assert_eq!(mem_fetch.cycles, 3);
//        }
//    }
//
//    #[test]
//    fn can_read_byte_from_iwram() {
//        let mut memory = GBAMemory::new();
//        let address = 0x03000034;
//        let value = 0x12345678;
//        memory.iwram[(address & !0x3000000) >> 2] = value;
//
//        // Test that reads happen in little endian byte order
//        for i in 0..4 {
//            let mem_fetch = memory.read(address + i);
//            assert_eq!(mem_fetch.data, ((value >> (i * 8)) & 0xFF) as u8);
//            assert_eq!(mem_fetch.cycles, 1);
//        }
//    }
//
//    #[test]
//    fn can_read_byte_from_bgram() {
//        let mut memory = GBAMemory::new();
//        let address = 0x05000034;
//        let value = 0x12345678;
//        memory.bgram[(address & !0x5000000) >> 2] = value;
//
//        // Test that reads happen in little endian byte order
//        for i in 0..4 {
//            let mem_fetch = memory.read(address + i);
//            assert_eq!(mem_fetch.data, ((value >> (i * 8)) & 0xFF) as u8);
//            assert_eq!(mem_fetch.cycles, 1);
//        }
//    }
//
//    // Test that reads happen in little endian byte order
//    #[test]
//    fn can_read_byte_from_vram() {
//        let mut memory = GBAMemory::new();
//        let address = 0x06000034;
//        let value = 0x12345678;
//        memory.vram[(address & !0x6000000) >> 2] = value;
//
//        // Test that reads happen in little endian byte order
//        for i in 0..4 {
//            let mem_fetch = memory.read(address + i);
//            assert_eq!(mem_fetch.data, ((value >> (i * 8)) & 0xFF) as u8);
//            assert_eq!(mem_fetch.cycles, 1);
//        }
//    }
//
//    #[test]
//    fn can_read_byte_from_oam() {
//        let mut memory = GBAMemory::new();
//        let address = 0x07000034;
//        let value = 0x12345678;
//        memory.oam[(address & !0x7000000) >> 2] = value;
//
//        // Test that reads happen in little endian byte order
//        for i in 0..4 {
//            let mem_fetch = memory.read(address + i);
//            assert_eq!(mem_fetch.data, ((value >> (i * 8)) & 0xFF) as u8);
//            assert_eq!(mem_fetch.cycles, 1);
//        }
//    }
//
//    #[test]
//    fn can_write_byte_to_oam() {
//        let mut memory = GBAMemory::new();
//        let address = 0x07000034;
//        let value = 0x12345678;
//        for i in 0..4 {
//            memory.write(address + i, (value >> (i * 8)) as u8);
//            let mem_fetch = memory.read(address + i);
//            assert_eq!(mem_fetch.data, ((value >> (i * 8)) & 0xFF) as u8);
//            assert_eq!(mem_fetch.cycles, 1);
//        }
//    }
//
//    #[test]
//    fn can_read_byte_from_rom() {
//        let mut memory = GBAMemory::new();
//        let address = 0x08000034;
//        let value = 0x12345678;
//        memory.rom[(address & 0xFFFFFF) >> 2] = value;
//
//        // Test that reads happen in little endian byte order
//        for i in 0..4 {
//            let mem_fetch = memory.read(address + i);
//            assert_eq!(mem_fetch.data, ((value >> (i * 8)) & 0xFF) as u8);
//            assert_eq!(mem_fetch.cycles, 5);
//        }
//    }
//
//    #[test]
//    fn can_read_byte_from_sram() {
//        let mut memory = GBAMemory::new();
//        let address = 0x0E000034;
//        let value = 0x12345678;
//        memory.sram[(address & 0xFFFFFF) >> 2] = value;
//
//        // Test that reads happen in little endian byte order
//        for i in 0..4 {
//            let mem_fetch = memory.read(address + i);
//            assert_eq!(mem_fetch.data, ((value >> (i * 8)) & 0xFF) as u8);
//            assert_eq!(mem_fetch.cycles, 5);
//        }
//    }
//
//    #[test]
//    fn can_read_hword_from_bios() {
//        let mut memory = GBAMemory::new();
//        let address = 0x4;
//        let value = 0x12345678;
//        memory.bios[address >> 2] = value;
//
//        let fetch = memory.readu16(address);
//        assert_eq!(fetch.data, 0x5678);
//        let fetch = memory.readu16(address + 2);
//
//        assert_eq!(fetch.data, 0x1234);
//        assert_eq!(fetch.cycles, 1);
//    }
//
//    #[test]
//    fn can_read_hword_from_exwram() {
//        let mut memory = GBAMemory::new();
//        let address = 0x2000034;
//        let value = 0x12345678;
//        memory.exwram[(address & 0xFFFFFF) >> 2] = value;
//
//        let fetch = memory.readu16(address);
//
//        assert_eq!(fetch.data, 0x5678);
//        assert_eq!(fetch.cycles, 3);
//    }
//
//    #[test]
//    fn can_write_bytes_to_exwram() {
//        let mut memory = GBAMemory::new();
//        let address = 0x2000034;
//        let value = 0x12345678;
//        memory.exwram[(address & 0xFFFFFF) >> 2] = value;
//
//        let cycles = memory.write(address, 0x55);
//        memory.write(address + 1, 0x32);
//        memory.write(address + 2, 0xde);
//        memory.write(address + 3, 0xab);
//        let fetch = memory.readu32(address);
//
//        assert_eq!(cycles, 3);
//        assert_eq!(fetch.data, 0xabde3255);
//    }
//
//    #[test]
//    fn can_write_hword_to_exwram() {
//        let mut memory = GBAMemory::new();
//        let address = 0x2000034;
//        let value = 0x12345678;
//        memory.exwram[(address & 0xFFFFFF) >> 2] = value;
//
//        let cycles = memory.writeu16(address, 0x1255);
//        let cycles = memory.writeu16(address + 2, 0x1255);
//        let fetch = memory.readu32(address);
//
//        assert_eq!(cycles, 3);
//        assert_eq!(fetch.data, 0x12551255);
//    }
//
//    #[test]
//    fn can_write_word_to_exwram() {
//        let mut memory = GBAMemory::new();
//        let address = 0x2000034;
//        let value = 0x12345678;
//        memory.exwram[(address & 0xFFFFFF) >> 2] = value;
//
//        let cycles = memory.writeu32(address, 0xabcdef12);
//        let fetch = memory.readu32(address);
//
//        assert_eq!(cycles, 6);
//        assert_eq!(fetch.data, 0xabcdef12);
//    }
//}
