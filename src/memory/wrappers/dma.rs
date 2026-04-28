use std::fmt::Display;

use crate::{
    memory::{
        io_handlers::{IOBlock, DMA0CNT_H},
        memory::{CPUEvent, CPUEventType, GBAMemory},
    },
    utils::bits::Bits,
};

#[derive(Default, Debug, Clone, Copy, PartialEq)]
pub(crate) struct DMAControl {
    pub(crate) immediately: bool,
    pub(crate) source: usize,
    pub(crate) destination: usize,
    pub(crate) word_count: usize,
    pub(crate) start_destination: usize,
    pub(crate) start_word_count: usize,
}

pub(crate) fn print_dma(dma_num: usize, control: &DMAControl, memory: &GBAMemory) {
    let dmacnt = DmaCNTH(memory.io_load(DMA0CNT_H + IOBlock::dma_address_offset(dma_num)));
    print!("{} ", dmacnt.start_timing());
    println!(
        "0x{:08x} => 0x{:08x} Remaining: {:08x}",
        control.source, control.destination, control.word_count
    );
}

pub(crate) struct DmaCNTH(pub(crate) u16);

pub(crate) enum DestinationAddressControlMode {
    Increment = 0,
    Decrement = 1,
    Fixed = 2,
    IncrementReload = 3,
}

pub(crate) enum SourceAddressControlMode {
    Increment = 0,
    Decrement = 1,
    Fixed = 2,
    Prohibited = 3,
}

#[repr(usize)]
#[derive(Copy, Clone)]
pub(crate) enum DMATransferType {
    Bit16 = 2,
    Bit32 = 4,
}

#[derive(PartialEq, Debug)]
pub(crate) enum StartTiming {
    Immediately,
    VBlank,
    HBlank,
    Special,
}

impl Display for StartTiming {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                StartTiming::Immediately => "Immediately",
                StartTiming::VBlank => "VBlank",
                StartTiming::HBlank => "HBlank",
                StartTiming::Special => "Special",
            }
        )
    }
}

impl DmaCNTH {
    pub(crate) fn destination_address_control(&self) -> DestinationAddressControlMode {
        match (self.0 >> 5) & 0b11 {
            0 => DestinationAddressControlMode::Increment,
            1 => DestinationAddressControlMode::Decrement,
            2 => DestinationAddressControlMode::Fixed,
            3 => DestinationAddressControlMode::IncrementReload,
            _ => unreachable!(),
        }
    }

    pub(crate) fn source_address_control(&self) -> SourceAddressControlMode {
        match (self.0 >> 7) & 0b11 {
            0 => SourceAddressControlMode::Increment,
            1 => SourceAddressControlMode::Decrement,
            2 => SourceAddressControlMode::Fixed,
            3 => SourceAddressControlMode::Prohibited,
            _ => unreachable!(),
        }
    }

    pub(crate) fn dma_repeat(&self) -> bool {
        self.0.bit_is_set(9)
    }

    pub(crate) fn dma_transfer_type(&self) -> DMATransferType {
        match self.0.get_bit(10) {
            0 => DMATransferType::Bit16,
            1 => DMATransferType::Bit32,
            _ => unreachable!(),
        }
    }

    pub(crate) fn gamepak_drq(&self) -> bool {
        self.0.bit_is_set(11)
    }

    pub(crate) fn start_timing(&self) -> StartTiming {
        match (self.0 >> 12) & 0b11 {
            0 => StartTiming::Immediately,
            1 => StartTiming::VBlank,
            2 => StartTiming::HBlank,
            3 => StartTiming::Special,
            _ => unreachable!(),
        }
    }

    pub(crate) fn irq_at_end(&self) -> bool {
        self.0.bit_is_set(14)
    }

    pub(crate) fn dma_enabled(&self) -> bool {
        self.0.bit_is_set(15)
    }
}

impl DMAControl {
    pub(crate) fn handle_dma_transfer(&mut self, dma_num: usize, memory: &mut GBAMemory) -> u8 {
        if self.word_count <= 0 {
            return 0;
        }
        let mut cycles = 0;
        let dma_io_address_start = 0xB0 + 0xC * dma_num;

        let dmacnt = DmaCNTH(memory.io_load(dma_io_address_start + 10));

        cycles += update_cycles_from_region(self.source, self.destination);
        let dmatransfer_type = dmacnt.dma_transfer_type();
        let word_size = dmatransfer_type as usize;

        self.word_count -= 1;
        match dmatransfer_type {
            DMATransferType::Bit16 => {
                let read = memory.readu16(self.source);
                cycles += memory.writeu16(self.destination, read.data);
                adjust_source_address(&mut self.source, word_size, dmacnt.source_address_control());
                adjust_destination_address(
                    &mut self.destination,
                    word_size,
                    dmacnt.destination_address_control(),
                );

                cycles += read.cycles;
            }
            DMATransferType::Bit32 => {
                let read = memory.readu32(self.source);
                cycles += memory.writeu32(self.destination, read.data);
                adjust_source_address(&mut self.source, word_size, dmacnt.source_address_control());
                adjust_destination_address(
                    &mut self.destination,
                    word_size,
                    dmacnt.destination_address_control(),
                );
                cycles += read.cycles;
            }
        }

        if self.word_count <= 0 {
            self.immediately = false;
            if dmacnt.irq_at_end() {
                memory.add_event(CPUEvent::now(CPUEventType::DMAIrq(dma_num)));
            }
            if dmacnt.dma_repeat() {
                self.word_count = self.start_word_count;
                if matches!(
                    dmacnt.destination_address_control(),
                    DestinationAddressControlMode::IncrementReload
                ) {
                    self.destination = self.start_destination;
                }
            } else {
                let disabled_dmacnt = dmacnt.0 & !0x8000;
                memory.ioram.io_store(
                    DMA0CNT_H + IOBlock::dma_address_offset(dma_num),
                    disabled_dmacnt,
                );
            }
            if dmacnt.irq_at_end() {
                memory.add_event(CPUEvent::irq());
            }
        }

        cycles
    }
}

fn adjust_source_address(
    source_address: &mut usize,
    word_size: usize,
    control_mode: SourceAddressControlMode,
) {
    match control_mode {
        SourceAddressControlMode::Increment => *source_address += word_size,
        SourceAddressControlMode::Decrement => *source_address -= word_size,
        SourceAddressControlMode::Fixed => {}
        SourceAddressControlMode::Prohibited => panic!(),
    }
}

fn adjust_destination_address(
    destination_address: &mut usize,
    word_size: usize,
    control_mode: DestinationAddressControlMode,
) {
    match control_mode {
        DestinationAddressControlMode::Increment => *destination_address += word_size,
        DestinationAddressControlMode::Decrement => *destination_address -= word_size,
        DestinationAddressControlMode::Fixed => {}
        DestinationAddressControlMode::IncrementReload => *destination_address += word_size,
    }
}

fn update_cycles_from_region(dma_start_address: usize, dma_destination_address: usize) -> u8 {
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
