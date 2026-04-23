use std::collections::BinaryHeap;
use std::convert::identity;
use std::mem::swap;
use std::sync::mpsc::Sender;
use std::sync::Arc;

use crate::arm7tdmi::instruction_table::instruction_to_string;
use crate::debugger::terminal_commands::PPUToDisplayCommands;
use crate::graphics::display::DisplayBuffer;
use crate::graphics::ppu::{PPUModes, PPU};
use crate::memory::io_handlers::{IOBlock, DMA0CNT_H, IF};
use crate::memory::memory::{CPUEvent, CPUEventType};
use crate::memory::wrappers::dma::{DMAControl, DmaCNTH, StartTiming};
use crate::utils::bits::Bits;
use crate::utils::instruction_to_string::print_register;
use crate::utils::utils::KillSignal;
use crate::{arm7tdmi::cpu::CPU, memory::memory::GBAMemory};

pub(crate) static KILL_SIGNAL: KillSignal = KillSignal::new();

pub(crate) struct GBA {
    pub(crate) cpu: CPU,
    pub(crate) memory: GBAMemory,
    pub(crate) ppu: PPU,
    pub(crate) dma_controllers: [DMAControl; 4],
    display_buffer: Arc<DisplayBuffer>,
    cpu_events: Vec<CPUEvent>,
    active_dma: Option<usize>,
}

impl GBA {
    #[cfg(test)]
    pub(crate) fn new_no_bios() -> Self {
        use std::sync::mpsc::channel;

        Self {
            memory: GBAMemory::new(),
            cpu: CPU::new(),
            ppu: PPU::new(channel().0),
            display_buffer: Arc::new(DisplayBuffer::new()),
            cpu_events: Vec::new(),
            dma_controllers: [DMAControl::default(); 4],
            active_dma: None,
        }
    }

    pub(crate) fn new(
        bios: String,
        rom: String,
        display_buffer: Arc<DisplayBuffer>,
        ppu_to_display_sender: Sender<PPUToDisplayCommands>,
    ) -> Self {
        let mut memory = GBAMemory::new();
        memory.initialize_bios(bios).unwrap();
        memory.initialize_rom(rom).unwrap();
        let mut gba = Self {
            memory,
            cpu: CPU::new(),
            ppu: PPU::new(ppu_to_display_sender),
            display_buffer,
            cpu_events: Vec::new(),
            dma_controllers: [DMAControl::default(); 4],
            active_dma: None,
        };
        gba.cpu.flush_pipeline(&mut gba.memory);
        gba
    }

    pub(crate) fn reset(&mut self) {
        self.memory.clear_ram();
        self.cpu.reset();
        self.ppu.reset();
    }

    pub(crate) fn get_status(&self) {
        for i in 0..4 {
            for j in 0..4 {
                let register = i * 4 + j;
                print!(
                    "{}: 0x{:<8x} ",
                    print_register(&register),
                    self.cpu.get_register(register)
                );
            }
            println!();
        }

        println!("CPSR: {}", self.cpu.get_cpsr());
        println!("CYCLES: {}", self.cpu.cycles);
        for i in 0..4 {
            println!("DMA {i}: {}", self.dma_controllers[i]);
        }
        println!("Active DMA: {:#?}", self.active_dma);
        println!(
            "Last Instruction: {} {:#x}",
            instruction_to_string(
                self.cpu.executed_instruction_hex >> 28,
                self.cpu
                    .decode_instruction(self.cpu.executed_instruction_hex),
            ),
            self.cpu.executed_instruction_hex
        );

        println!(
            "Next Instruction: {}",
            instruction_to_string(
                self.cpu.prefetch[1] >> 28,
                self.cpu.decode_instruction(self.cpu.prefetch[1])
            )
        );
    }

    pub(crate) fn step(&mut self) {
        let cpu_cycles = if let Some(dma) = self.active_dma {
            let cycles = self.dma_controllers[dma].handle_dma_transfer(dma, &mut self.memory);
            println!("Running DMA");
            if self.dma_controllers[dma].word_count == 0 {
                self.active_dma =
                    Self::preempt_dma(None, &mut self.memory, StartTiming::Immediately)
            }
            cycles
        } else {
            self.cpu.execute_cpu_cycle(&mut self.memory)
        };
        self.ppu
            .advance_ppu(cpu_cycles, &mut self.memory, &self.display_buffer);
        let triggered_irqs = self.memory.ioram.timers.tick(cpu_cycles.into());
        if triggered_irqs.into_iter().any(identity) {
            let mut if_flag = self.memory.io_load(IF);
            for i in 0..triggered_irqs.len() {
                if triggered_irqs[i] == true {
                    if_flag.set_bit((3 + i) as u8);
                }
            }
            self.memory.privileged_io_write(IF, if_flag);
        }
        while !self.memory.ioram.cpu_events.is_empty() {
            swap(&mut self.cpu_events, &mut self.memory.ioram.cpu_events);
            for mut command in self.cpu_events.drain(..) {
                command.delay -= cpu_cycles as i32;
                if command.delay <= 0 {
                    match command.event {
                        CPUEventType::Halt => self.cpu.halt(),
                        CPUEventType::RaiseIrq => {
                            self.cpu.interrupt_triggered = true;
                        }
                        CPUEventType::DMA(dma_num, dma_controller) => {
                            self.dma_controllers[dma_num] = dma_controller;
                            self.active_dma = Self::preempt_dma(
                                self.active_dma,
                                &mut self.memory,
                                StartTiming::Immediately,
                            );
                        }
                        CPUEventType::HDraw => {
                            self.active_dma = Self::preempt_dma(
                                self.active_dma,
                                &mut self.memory,
                                StartTiming::Immediately,
                            );
                            self.memory.ioram.handle_hdraw();
                        }
                        CPUEventType::VBlank => {
                            self.active_dma = Self::preempt_dma(
                                self.active_dma,
                                &mut self.memory,
                                StartTiming::VBlank,
                            );
                            self.memory.ioram.handle_vblank();
                        }
                        CPUEventType::HBlank => {
                            self.active_dma = None;
                            self.active_dma = Self::preempt_dma(
                                self.active_dma,
                                &mut self.memory,
                                StartTiming::HBlank,
                            );
                            self.memory.ioram.handle_hblank()
                        }
                        CPUEventType::VCount(value) => self.memory.ioram.handle_vcount(value),
                        _ => panic!("{:#?}", command),
                    }
                } else {
                    self.memory.ioram.cpu_events.push(command);
                }
            }
        }
    }

    fn preempt_dma(
        current_dma: Option<usize>,
        memory: &mut GBAMemory,
        condition: StartTiming,
    ) -> Option<usize> {
        let old_dma = match current_dma {
            Some(0) => return current_dma, // 0 is highest priority and cannot be preempted
            Some(i) => i,
            None => 4,
        };
        for i in 0..old_dma {
            let dmacnt = DmaCNTH(
                memory
                    .ioram
                    .io_load(DMA0CNT_H + IOBlock::dma_address_offset(i)),
            );

            if dmacnt.dma_enabled()
                && (dmacnt.start_timing() == condition
                    || dmacnt.start_timing() == StartTiming::Immediately)
            {
                return Some(i);
            }
        }

        current_dma
    }
}
