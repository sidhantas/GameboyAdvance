use crate::{memory::memory::GBAMemory, utils::bits::Bits};

#[derive(Default)]
pub struct DMAControl {
    source: usize,
    destination: usize,
    word_count: usize
}

pub struct DmaCNTH(u16);

enum DestinationAddressControlMode {
    Increment = 0,
    Decrement = 1,
    Fixed = 2,
    IncrementReload = 3,
}

enum SourceAddressControlMode {
    Increment = 0,
    Decrement = 1,
    Fixed = 2,
    Prohibited = 3,
}

enum DMATransferType {
    Bit32,
    Bit16,
}

impl DmaCNTH {
    pub fn destination_address_control(&self) -> DestinationAddressControlMode {
        match (self.0 >> 5) & 0b11 {
            0 => DestinationAddressControlMode::Increment,
            1 => DestinationAddressControlMode::Decrement,
            2 => DestinationAddressControlMode::Fixed,
            3 => DestinationAddressControlMode::IncrementReload,
            _ => unreachable!(),
        }
    }

    pub fn source_address_control(&self) -> DestinationAddressControlMode {
        match (self.0 >> 5) & 0b11 {
            0 => DestinationAddressControlMode::Increment,
            1 => DestinationAddressControlMode::Decrement,
            2 => DestinationAddressControlMode::Fixed,
            3 => DestinationAddressControlMode::IncrementReload,
            _ => unreachable!(),
        }
    }

    pub fn dma_repeat(&self) -> bool {
        self.0.bit_is_set(9)
    }

    pub fn dma_transfer_type(&self) -> DMATransferType {
        match self.0.get_bit(10) {
            0 => DMATransferType::Bit16,
            1 => DMATransferType::Bit32,
            _ => unreachable!(),
        }
    }

    pub fn gamepak_drq(&self) -> bool {
        self.0.bit_is_set(11)
    }

    pub fn irq_at_end(&self) -> bool {
        self.0.bit_is_set(14)
    }

    pub fn dma_enabled(&self) -> bool {
        self.0.bit_is_set(15)
    }
}

pub fn handle_dma_transfer(dma_num: usize, memory: &mut GBAMemory) -> usize {
    let mut cycles = 0;
    let dma_io_address_start = 0xB0 + 0xC * dma_num;
    let (dma_start_address, dma_destination_address) =
        get_start_addresses(dma_num, memory, dma_io_address_start);
    cycles += update_cycles_from_region(dma_start_address, dma_destination_address);

    let word_count = memory.io_load(dma_io_address_start + 8) as usize;
    let dma_cnt_h = DmaCNTH(memory.io_load(dma_io_address_start + 10));

    for i in 0..word_count {
        let read = memory.readu16(dma_start_address + 2 * i);
        cycles += read.cycles as usize;
        cycles += memory.writeu16(dma_destination_address + 2 * i, read.data) as usize;
    }

    cycles
}

fn update_cycles_from_region(dma_start_address: usize, dma_destination_address: usize) -> usize {
    let mut cycles = 0;
    let start_region = dma_start_address >> 24;
    if start_region > 0x8 && start_region != 0xE {
        cycles += 2;
    } else {
        cycles += 1;
    }

    let destination_region = dma_destination_address >> 24;
    if destination_region > 0x8 && destination_region != 0xE {
        cycles += 2;
    } else {
        cycles += 1;
    }
    cycles
}

fn get_start_addresses(
    dma_num: usize,
    memory: &mut GBAMemory,
    dma_io_address_start: usize,
) -> (usize, usize) {
    let mut dma_source_address = (memory.io_load(dma_io_address_start + 2) as usize) << 16
        & (memory.io_load(dma_io_address_start) as usize);

    if dma_num == 0 {
        dma_source_address &= 0x07FFFFFF;
    } else {
        dma_source_address &= 0x0FFFFFFF;
    }

    let mut dma_destination_address = (memory.io_load(dma_io_address_start + 6) as usize) << 16
        & (memory.io_load(dma_io_address_start + 4) as usize);

    if dma_num == 3 {
        dma_destination_address &= 0x0FFFFFFF;
    } else {
        dma_destination_address &= 0x07FFFFFF;
    }
    (dma_source_address, dma_destination_address)
}
