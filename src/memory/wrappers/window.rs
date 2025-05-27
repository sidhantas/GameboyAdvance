use crate::utils::bits::Bits;

pub struct WININ(pub u16);

impl WININ {
    pub fn window_0_bg0_enabled(&self) -> bool {
        self.0.bit_is_set(0)
    }
    pub fn window_0_bg1_enabled(&self) -> bool {
        self.0.bit_is_set(1)
    }
    pub fn window_0_bg2_enabled(&self) -> bool {
        self.0.bit_is_set(2)
    }
    pub fn window_0_bg3_enabled(&self) -> bool {
        self.0.bit_is_set(3)
    }
    pub fn window_0_obj_enabled(&self) -> bool {
        self.0.bit_is_set(4)
    }
    pub fn window_0_color_special_effects_enabled(&self) -> bool {
        self.0.bit_is_set(5)
    }

    pub fn window_1_bg0_enabled(&self) -> bool {
        self.0.bit_is_set(8)
    }
    pub fn window_1_bg1_enabled(&self) -> bool {
        self.0.bit_is_set(9)
    }
    pub fn window_1_bg2_enabled(&self) -> bool {
        self.0.bit_is_set(10)
    }
    pub fn window_1_bg3_enabled(&self) -> bool {
        self.0.bit_is_set(11)
    }
    pub fn window_1_obj_enabled(&self) -> bool {
        self.0.bit_is_set(12)
    }
    pub fn window_1_color_special_effects_enabled(&self) -> bool {
        self.0.bit_is_set(13)
    }
}

pub struct WinOut(pub u16);

impl WinOut {
    pub fn bg0_enabled(&self) -> bool {
        self.0.bit_is_set(0)
    }
    pub fn bg1_enabled(&self) -> bool {
        self.0.bit_is_set(1)
    }
    pub fn bg2_enabled(&self) -> bool {
        self.0.bit_is_set(2)
    }
    pub fn bg3_enabled(&self) -> bool {
        self.0.bit_is_set(3)
    }
    pub fn obj_enabled(&self) -> bool {
        self.0.bit_is_set(4)
    }
    pub fn color_special_effects_enabled(&self) -> bool {
        self.0.bit_is_set(5)
    }
    pub fn obj_window_bg0_enabled(&self) -> bool {
        self.0.bit_is_set(8)
    }
    pub fn obj_window_bg1_enabled(&self) -> bool {
        self.0.bit_is_set(9)
    }
    pub fn obj_window_bg2_enabled(&self) -> bool {
        self.0.bit_is_set(10)
    }
    pub fn obj_window_bg3_enabled(&self) -> bool {
        self.0.bit_is_set(11)
    }
    pub fn obj_window_obj_enabled(&self) -> bool {
        self.0.bit_is_set(12)
    }
    pub fn obj_window_color_special_effects_enabled(&self) -> bool {
        self.0.bit_is_set(13)
    }

}
