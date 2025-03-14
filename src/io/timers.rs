use crate::memory::io_handlers::IF;
use crate::memory::memory::GBAMemory;
use crate::memory::wrappers::tmcnt::TMCntH;
use crate::utils::bits::Bits;

#[derive(Debug)]
pub struct Timers([Timer; 4]);

impl Timers {
    pub(crate) fn new() -> Self {
        Self([Timer::default(); 4])
    }

    pub(crate) fn tick(&mut self, cpu_cycles: u32, memory: &mut GBAMemory) {
        let mut previous_timer_overflowed = false;
        let mut if_flag = memory.io_load(IF);
        for (i, timer) in self.0.iter_mut().enumerate() {
            let tmcnt = TMCntH(memory.io_load(0x102 + 0x4 * i));
            previous_timer_overflowed =
                timer.increment(cpu_cycles.into(), &tmcnt, previous_timer_overflowed);

            if previous_timer_overflowed && tmcnt.timer_irq_enable() {
                if_flag.set_bit((3 + i) as u8);
            }
        }

        memory.ppu_io_write(IF, if_flag);
    }

    pub(crate) fn read_timer(&self, timer_num: usize) -> u32 {
        self.0[timer_num].counter
    }
}

#[derive(Debug, Default, Copy, Clone)]
struct Timer {
    counter: u32,
    cycles: u32,
}

impl Timer {
    fn increment(&mut self, cpu_cycles: u32, tmcnt: &TMCntH, overflow: bool) -> bool {
        if !tmcnt.timer_enabled() {
            return false;
        }
        if tmcnt.count_up_timing() && overflow {
            self.counter += 1;
        } else {
            self.cycles += cpu_cycles;
            let ticks = self.cycles / tmcnt.prescaler_value();
            self.cycles %= tmcnt.prescaler_value();
            self.counter += ticks as u32;
        }
        if self.counter >= u16::MAX.into() {
            self.counter -= u16::MAX as u32;
            return true;
        }
        return false;
    }
}

#[cfg(test)]
mod timer_tests {
    use std::u16;

    use crate::{
        io::timers::Timers,
        memory::{
            io_handlers::{IE, IF, IME, TM0CNT_H, TM1CNT_H},
            memory::{CPUCallbacks, GBAMemory},
        },
        utils::bits::Bits,
    };

    #[test]
    fn test_timer_increments() {
        let mut memory = GBAMemory::new();
        let mut timers = Timers::new();
        let mut tmcnt = memory.io_load(TM0CNT_H);
        tmcnt.set_bit(7); // enables timer
        memory.ppu_io_write(TM0CNT_H, tmcnt);

        timers.tick(1, &mut memory);

        assert_eq!(timers.0[0].counter, 1);
    }

    #[test]
    fn timer_applies_prescalar_value() {
        let mut memory = GBAMemory::new();
        let mut timers = Timers::new();
        let mut tmcnt = memory.io_load(TM0CNT_H);
        tmcnt.set_bit(7); // enables timer
        tmcnt.set_bit(1); // set prescalar to 256 clocks
        memory.ppu_io_write(TM0CNT_H, tmcnt);

        timers.tick(256, &mut memory);

        assert_eq!(timers.0[0].counter, 1);
    }

    #[test]
    fn timer_doesnt_tick_prematurely() {
        let mut memory = GBAMemory::new();
        let mut timers = Timers::new();
        let mut tmcnt = memory.io_load(TM0CNT_H);
        tmcnt.set_bit(7); // enables timer
        tmcnt.set_bit(1); // set prescalar to 256 clocks
        memory.ppu_io_write(TM0CNT_H, tmcnt);

        timers.tick(255, &mut memory);

        assert_eq!(timers.0[0].counter, 0);
    }

    #[test]
    fn overflow_timer_works() {
        let mut memory = GBAMemory::new();
        let mut timers = Timers::new();
        let mut tmcnt0 = memory.io_load(TM0CNT_H);
        tmcnt0.set_bit(7); // enables timer
        memory.ppu_io_write(TM0CNT_H, tmcnt0);

        let mut tmcnt1 = memory.io_load(TM1CNT_H);
        tmcnt1.set_bit(7); // enables timer
        tmcnt1.set_bit(2); // enable count up timing
        memory.ppu_io_write(TM1CNT_H, tmcnt1);

        timers.tick(u16::MAX as u32 + 1, &mut memory);

        assert_eq!(timers.0[0].counter, 1);
        assert_eq!(timers.0[1].counter, 1);
    }

    #[test]
    fn check_irq_gets_triggerd_from_overflow() {
        let mut memory = GBAMemory::new();
        let mut timers = Timers::new();
        let mut tmcnt0 = memory.io_load(TM0CNT_H);
        tmcnt0.set_bit(7); // enables timer
        tmcnt0.set_bit(6); // enables IRQ
        memory.ppu_io_write(TM0CNT_H, tmcnt0);
        memory.ppu_io_write(IE, 1 << 3); // Enable Timer0 IRQ
        memory.ppu_io_write(IME, 1 ); // Enable Timer0 IRQ
        timers.tick(u16::MAX as u32 + 1, &mut memory);

        assert_eq!(memory.io_load(IF), 1 << 3);
        assert!(matches!(
            memory.cpu_commands.get(0).unwrap(),
            CPUCallbacks::RaiseIrq
        ));
    }
}
