use std::cmp::{max, min};
use std::collections::HashSet;

use num_traits::clamp;

use crate::debugger::terminal_commands::PPUToDisplayCommands::{Render, RenderWithBorders};
use crate::graphics::ppu::{PPUModes, HBLANK, HBLANK_FLAG, HDRAW, PPU, VBLANK_FLAG, VDRAW};
use crate::memory::memory::GBAMemory;
use crate::memory::oam::{Oam, NUM_OAM_ENTRIES};

impl PPU {
    pub(crate) fn hblank(
        &mut self,
        mut dots: u32,
        memory: &mut GBAMemory,
        disp_stat: &mut u16,
    ) -> u32 {
        while dots > 0 {
            if self.x >= HDRAW + HBLANK {
                self.y += 1;
                self.x = 0;
                if self.y >= VDRAW {
                    *disp_stat &= !HBLANK_FLAG;
                    *disp_stat |= VBLANK_FLAG;
                    if self.show_borders {
                        self.ppu_to_display_sender
                            .try_send(RenderWithBorders(memory.get_oam_borders()))
                            .unwrap()
                    } else {
                        self.ppu_to_display_sender.try_send(Render).unwrap();
                    }
                    self.current_mode = PPUModes::VBLANK;
                } else {
                    self.obj_selection(memory);
                    self.current_mode = PPUModes::HDRAW;
                }
                return dots;
            }
            self.x += 1;
            dots -= 1;
        }
        return 0;
    }

    fn obj_selection(&mut self, memory: &mut GBAMemory) {
        self.current_line_objects.clear();
        for i in 0..NUM_OAM_ENTRIES {
            let oam = memory.oam.oam_read(i);
            self.current_line_objects.try_push(oam, self.y);
        }
    }
}

#[derive(Debug)]
pub struct OAMQueue {
    queue: Vec<Oam>,
    active_objects: HashSet<usize>,
    intervals: [Vec<Position>; HDRAW as usize],
    i: usize,
}

#[derive(Clone, Copy, Debug)]
enum Position {
    Start(usize),
    Stop(usize),
}

impl OAMQueue {
    pub fn new() -> Self {
        Self {
            queue: Vec::new(),
            active_objects: HashSet::new(),
            intervals: [(); HDRAW as usize].map(|_| Vec::<Position>::new()),
            i: 0,
        }
    }

    pub fn clear(&mut self) {
        self.queue.clear();
        self.active_objects.clear();
        for i in &mut self.intervals {
            i.clear();
        }
        self.i = 0;
    }

    pub fn try_push(&mut self, oam: Oam, curr_y: i32) {
        if (oam.y() < curr_y && curr_y < oam.y() + oam.view_height()) && !oam.obj_disabled() {
            let position = self.queue.len();
            let start = Position::Start(position);
            let stop = Position::Stop(position);
            let x_start = clamp(oam.x(), 0, 239) as usize;
            let x_end = clamp(oam.x() + oam.view_width() + 1, 0, 239) as usize;
            self.intervals[x_start].push(start);
            self.intervals[x_end].push(stop);
            self.queue.push(oam);
        }
    }

    pub fn update_active_oams(&mut self) {
        for interval in self.intervals[min(self.i, 239)].iter() {
            match interval {
                Position::Start(obj) => {
                    self.active_objects.insert(*obj);
                }
                Position::Stop(obj) => {
                    self.active_objects.remove(obj);
                }
            }
        }

        self.i += 1;
    }

    pub fn current_x_objects(&self) -> &HashSet<usize> {
        &self.active_objects
    }

    pub fn get_oam(&self, obj: usize) -> &Oam {
        &self.queue[obj]
    }
}
