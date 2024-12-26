use super::cpu::{CPUMode, CPU};

impl CPU {
    fn handle_exception(&mut self, mode: CPUMode) {
        self.set_mode(mode);
    }
}
