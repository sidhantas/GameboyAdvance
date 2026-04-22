use std::convert::identity;
use std::mem::swap;
use std::sync::mpsc::Sender;
use std::sync::Arc;

use sdl2::cpuinfo::cpu_count;

use crate::arm7tdmi::cpu;
use crate::arm7tdmi::instruction_table::instruction_to_string;
use crate::debugger::terminal_commands::PPUToDisplayCommands;
use crate::graphics::display::DisplayBuffer;
use crate::graphics::ppu::PPU;
use crate::memory::io_handlers::IF;
use crate::memory::memory::{CPUEvent, CPUEventType};
use crate::memory::wrappers::dma::handle_dma_transfer;
use crate::utils::bits::Bits;
use crate::utils::instruction_to_string::print_register;
use crate::utils::utils::KillSignal;
use crate::{arm7tdmi::cpu::CPU, memory::memory::GBAMemory};

pub(crate) static KILL_SIGNAL: KillSignal = KillSignal::new();

pub(crate) struct GBA {
    pub(crate) cpu: CPU,
    pub(crate) memory: GBAMemory,
    pub(crate) ppu: PPU,
    display_buffer: Arc<DisplayBuffer>,
    cpu_events: Vec<CPUEvent>
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
            cpu_events: Vec::new()
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
            cpu_events: Vec::new()
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
        let cpu_cycles = self.cpu.execute_cpu_cycle(&mut self.memory);
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
        self.memory.handle_io_events();

        if self.memory.ioram.cpu_events.is_empty() {
            return;
        }
        swap(&mut self.cpu_events, &mut self.memory.ioram.cpu_events);

        for mut command in self.cpu_events.drain(..) {
            command.delay -= cpu_cycles as i32;
            if command.delay <= 0 {
                match command.event {
                    CPUEventType::Halt => self.cpu.halt(),
                    CPUEventType::RaiseIrq => {
                        self.cpu.interrupt_triggered = true;
                    }
                    CPUEventType::DMA(dma_num) => {
                        handle_dma_transfer(
                            dma_num,
                            &mut self.memory,
                        );
                    }
                    _ => panic!("{:#?}", command),
                }
            } else {
                self.memory.ioram.cpu_events.push(command);
            }
        }
    }
}

