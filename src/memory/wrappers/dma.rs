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
    pub(crate) fn handle_dma_transfer(&mut self, dma_num: usize, memory: &mut GBAMemory) -> usize {
        let mut cycles = 0;
        let dma_io_address_start = 0xB0 + 0xC * dma_num;

        let dma_cnt_h = DmaCNTH(memory.io_load(dma_io_address_start + 10));

        cycles += update_cycles_from_region(self.source, self.destination);
        let dmatransfer_type = dma_cnt_h.dma_transfer_type();
        let word_size = dmatransfer_type as usize;

        match dmatransfer_type {
            DMATransferType::Bit16 => {
                let read = memory.readu16(self.source);
                cycles += memory.writeu16(self.destination, read.data) as usize;
                self.word_count -= 1;
                adjust_source_address(
                    &mut self.source,
                    word_size,
                    dma_cnt_h.source_address_control(),
                );
                adjust_destination_address(
                    &mut self.destination,
                    word_size,
                    dma_cnt_h.destination_address_control(),
                );
                cycles += read.cycles as usize;
            }
            DMATransferType::Bit32 => {
                let read = memory.readu32(self.source);
                cycles += memory.writeu32(self.destination, read.data) as usize;
                self.word_count -= 1;
                adjust_source_address(
                    &mut self.source,
                    word_size,
                    dma_cnt_h.source_address_control(),
                );
                adjust_destination_address(
                    &mut self.destination,
                    word_size,
                    dma_cnt_h.destination_address_control(),
                );
                cycles += read.cycles as usize;
            }
        }

        if self.word_count == 0 {
            self.immediately = false;
            if dma_cnt_h.dma_repeat() {
                self.word_count = self.start_word_count;
                if matches!(
                    dma_cnt_h.destination_address_control(),
                    DestinationAddressControlMode::IncrementReload
                ) {
                    self.destination = self.start_destination;
                }
            } else {
                let disabled_dmacnt = dma_cnt_h.0 & !0x8000;
                memory.ioram.io_store(
                    DMA0CNT_H + IOBlock::dma_address_offset(dma_num),
                    disabled_dmacnt,
                );
            }
        } 

        cycles
    }
}

//pub(crate) fn handle_dma_transfer(dma_num: usize, memory: &mut GBAMemory) -> usize {
//    let mut cycles = 0;
//    let dma_io_address_start = 0xB0 + 0xC * dma_num;
//
//    let dma_cnt_h = DmaCNTH(memory.io_load(dma_io_address_start + 10));
//
//    let mut dma_controller = memory.ioram.dma_controllers[dma_num];
//    cycles += update_cycles_from_region(dma_controller.source, dma_controller.destination);
//    let dmatransfer_type = dma_cnt_h.dma_transfer_type();
//    let word_size = dmatransfer_type as usize;
//
//    match dmatransfer_type {
//        DMATransferType::Bit16 => {
//            let read = memory.readu16(dma_controller.source);
//            cycles += memory.writeu16(dma_controller.destination, read.data) as usize;
//            dma_controller.word_count -= 1;
//            adjust_source_address(
//                &mut dma_controller.source,
//                word_size,
//                dma_cnt_h.source_address_control(),
//            );
//            adjust_destination_address(
//                &mut dma_controller.destination,
//                word_size,
//                dma_cnt_h.destination_address_control(),
//            );
//            cycles += read.cycles as usize;
//        }
//        DMATransferType::Bit32 => {
//            let read = memory.readu32(dma_controller.source);
//            cycles += memory.writeu32(dma_controller.destination, read.data) as usize;
//            dma_controller.word_count -= 1;
//            adjust_source_address(
//                &mut dma_controller.source,
//                word_size,
//                dma_cnt_h.source_address_control(),
//            );
//            adjust_destination_address(
//                &mut dma_controller.destination,
//                word_size,
//                dma_cnt_h.destination_address_control(),
//            );
//            cycles += read.cycles as usize;
//        }
//    }
//
//    if dma_controller.word_count == 0 {
//        if dma_cnt_h.dma_repeat() {
//            dma_controller.word_count = dma_controller.start_word_count;
//            if matches!(
//                dma_cnt_h.destination_address_control(),
//                DestinationAddressControlMode::IncrementReload
//            ) {
//                dma_controller.destination = dma_controller.start_destination;
//            }
//        memory.ioram.cpu_events.push(CPUEvent::new(0, CPUEventType::DMA(dma_num)));
//        } else {
//            let disabled_dmacnt = dma_cnt_h.0 & !0x8000;
//            memory.ioram.io_store(DMA0CNT_H + IOBlock::dma_address_offset(dma_num), disabled_dmacnt);
//        }
//    } else {
//        memory.ioram.cpu_events.push(CPUEvent::new(0, CPUEventType::DMA(dma_num)));
//    }
//
//    memory.ioram.dma_controllers[dma_num] = dma_controller;
//
//    cycles
//}

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
