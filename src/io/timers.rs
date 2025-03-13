use crate::memory::io_handlers::{IF};
use crate::memory::memory::GBAMemory;
use crate::utils::bits::Bits;

#[derive(Debug)]
pub struct Timers([Timer; 4]);

impl Timers {
    pub(crate) fn new() -> Self {
        Self([Timer::default(); 4])
    }

    pub(crate) fn tick(&mut self, cpu_cycles: u32, io: &mut GBAMemory) {
        let mut previous_timer_overflowed = false;
        let mut if_flag = io.io_load(IF);
        for (i, timer) in self.0.iter_mut().enumerate() {
            let tmcnt = TMCntH(io.io_load(0x102 + 0x2 * i));
            previous_timer_overflowed =
                timer.increment(cpu_cycles.into(), &tmcnt, previous_timer_overflowed);

            if previous_timer_overflowed && tmcnt.timer_irq_enable() {
                if_flag.set_bit(1 << (2 + i));
            }
        }

        io.ppu_io_write(IF, if_flag);
    }
}

#[derive(Debug, Default, Copy, Clone)]
struct Timer {
    counter: u32,
    cycles: u32,
}

impl Timer {
    fn increment(&mut self, cpu_cycles: u32, tmcnt: &TMCntH, overflow: bool) -> bool {
        if tmcnt.timer_enabled() {
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
    use crate::{
        memory::io::timers::Timers,
        memory::io_handlers::{definitions::TM0CNT_H, io_handlers::IOBus},
        utils::bits::Bits,
    };

    #[test]
    fn test_timer_increments() {
        let mut io = IOBus::new();
        let mut timers = Timers::new();
        let mut tmcnt = io.io_load(TM0CNT_H);
        tmcnt.set_bit(6); // enables timer
        io.ppu_io_write(TM0CNT_H, tmcnt);

        timers.tick(1, &mut io);

        assert_eq!(timers.0[0].counter, 1);
    }

    #[test]
    fn timer_applies_prescalar_value() {
        let mut io = IOBus::new();
        let mut timers = Timers::new();
        let mut tmcnt = io.io_load(TM0CNT_H);
        tmcnt.set_bit(6); // enables timer
        tmcnt.set_bit(1); // set prescalar to 256 clocks
        io.ppu_io_write(TM0CNT_H, tmcnt);

        timers.tick(256, &mut io);

        assert_eq!(timers.0[0].counter, 1);
    }

    #[test]
    fn timer_doesnt_tick_prematurely() {
        let mut io = IOBus::new();
        let mut timers = Timers::new();
        let mut tmcnt = io.io_load(TM0CNT_H);
        tmcnt.set_bit(6); // enables timer
        tmcnt.set_bit(1); // set prescalar to 256 clocks
        io.ppu_io_write(TM0CNT_H, tmcnt);

        timers.tick(255, &mut io);

        assert_eq!(timers.0[0].counter, 0);
    }
}
