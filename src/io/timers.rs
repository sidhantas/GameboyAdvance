use crate::memory::io_handlers::IF;
use crate::memory::memory::GBAMemory;
use crate::utils::bits::Bits;

#[derive(Debug)]
pub struct Timers(pub [Timer; 4]);

impl Timers {
    pub(crate) fn new() -> Self {
        Self([Timer::default(); 4])
    }

    pub(crate) fn tick(&mut self, cpu_cycles: u32, memory: &mut GBAMemory) {
        let mut previous_timer_overflowed = false;
        for (i, timer) in self.0.iter_mut().enumerate() {
            previous_timer_overflowed = timer.increment(
                cpu_cycles.into(),
                previous_timer_overflowed,
            );

            if previous_timer_overflowed && timer.timer_irq_enable {
                let mut if_flag = memory.io_load(IF);
                if_flag.set_bit((3 + i) as u8);
                memory.ppu_io_write(IF, if_flag);
            }
        }
    }

    pub(crate) fn read_timer(&self, timer_num: usize) -> u32 {
        self.0[timer_num].counter
    }

    pub(crate) fn reload_timer(&mut self, timer_num: usize) {
        self.0[timer_num].counter = self.0[timer_num].reload_value;
    }

    pub(crate) fn set_reload_value(&mut self, timer_num: usize, reload_value: u32) {
        self.0[timer_num].reload_value = reload_value;
    }

    pub(crate) fn set_timer_enabled(&mut self, timer_num: usize, enabled: bool) {
        self.0[timer_num].timer_enabled = enabled;
    }

    pub(crate) fn set_count_up_timing(&mut self, timer_num: usize, enabled: bool) {
        self.0[timer_num].count_up_timing = enabled;
    }

    pub(crate) fn set_prescalar_value(&mut self, timer_num: usize, prescalar_value: u32) {
        self.0[timer_num].prescaler_value = prescalar_value;
    }
    
    pub(crate) fn set_timer_irq_enable(&mut self, timer_num: usize, enabled: bool) {
        self.0[timer_num].timer_irq_enable = enabled;
    }
}

#[derive(Debug, Default, Copy, Clone)]
struct Timer {
    counter: u32,
    cycles: u32,
    timer_enabled: bool,
    timer_irq_enable: bool,
    count_up_timing: bool,
    prescaler_value: u32,
    reload_value: u32
}

impl Timer {
    fn increment(
        &mut self,
        cpu_cycles: u32,
        overflow: bool,
    ) -> bool {
        if !self.timer_enabled {
            return false;
        }
        if self.count_up_timing && overflow {
            self.counter += 1;
        } else {
            self.cycles += cpu_cycles;
            let ticks = self.cycles / self.prescaler_value;
            self.cycles %= self.prescaler_value;
            self.counter += ticks as u32;
        }
        if self.counter >= u16::MAX.into() {
            self.counter -= u16::MAX as u32;
            self.counter += self.reload_value; // Reload the counter
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
            io_handlers::{IE, IF, IME, TM0CNT_H, TM0CNT_L, TM1CNT_H},
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
    fn timer_uses_reload_value_on_update() {
        let mut memory = GBAMemory::new();
        let mut timers = Timers::new();
        let mut tm0cnt_h = memory.io_load(TM0CNT_H);
        let reload_value = 0xFF;
        tm0cnt_h.set_bit(7); // enables timer
        memory.ppu_io_write(TM0CNT_L, reload_value);
        memory.ppu_io_write(TM0CNT_H, tm0cnt_h);

        timers.tick(u16::MAX as u32 + 1, &mut memory);

        assert_eq!(timers.0[0].counter, (reload_value + 1) as u32);
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
        memory.ppu_io_write(IME, 1); // Enable Timer0 IRQ
        timers.tick(u16::MAX as u32 + 1, &mut memory);

        assert_eq!(memory.io_load(IF), 1 << 3);
        assert!(matches!(
            memory.cpu_commands.get(0).unwrap(),
            CPUCallbacks::RaiseIrq
        ));
    }
}
