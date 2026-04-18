use crate::utils::bits::Bits;

pub(crate) struct WININ(pub(crate) u16);

impl WININ {
    pub(crate) fn window_0_bg0_enabled(&self) -> bool {
        self.0.bit_is_set(0)
    }
    pub(crate) fn window_0_bg1_enabled(&self) -> bool {
        self.0.bit_is_set(1)
    }
    pub(crate) fn window_0_bg2_enabled(&self) -> bool {
        self.0.bit_is_set(2)
    }
    pub(crate) fn window_0_bg3_enabled(&self) -> bool {
        self.0.bit_is_set(3)
    }
    pub(crate) fn window_0_obj_enabled(&self) -> bool {
        self.0.bit_is_set(4)
    }
    pub(crate) fn window_0_color_special_effects_enabled(&self) -> bool {
        self.0.bit_is_set(5)
    }

    pub(crate) fn window_1_bg0_enabled(&self) -> bool {
        self.0.bit_is_set(8)
    }
    pub(crate) fn window_1_bg1_enabled(&self) -> bool {
        self.0.bit_is_set(9)
    }
    pub(crate) fn window_1_bg2_enabled(&self) -> bool {
        self.0.bit_is_set(10)
    }
    pub(crate) fn window_1_bg3_enabled(&self) -> bool {
        self.0.bit_is_set(11)
    }
    pub(crate) fn window_1_obj_enabled(&self) -> bool {
        self.0.bit_is_set(12)
    }
    pub(crate) fn window_1_color_special_effects_enabled(&self) -> bool {
        self.0.bit_is_set(13)
    }
}

pub(crate) struct WinOut(pub(crate) u16);

impl WinOut {
    pub(crate) fn bg0_enabled(&self) -> bool {
        self.0.bit_is_set(0)
    }
    pub(crate) fn bg1_enabled(&self) -> bool {
        self.0.bit_is_set(1)
    }
    pub(crate) fn bg2_enabled(&self) -> bool {
        self.0.bit_is_set(2)
    }
    pub(crate) fn bg3_enabled(&self) -> bool {
        self.0.bit_is_set(3)
    }
    pub(crate) fn obj_enabled(&self) -> bool {
        self.0.bit_is_set(4)
    }
    pub(crate) fn color_special_effects_enabled(&self) -> bool {
        self.0.bit_is_set(5)
    }
    pub(crate) fn obj_window_bg0_enabled(&self) -> bool {
        self.0.bit_is_set(8)
    }
    pub(crate) fn obj_window_bg1_enabled(&self) -> bool {
        self.0.bit_is_set(9)
    }
    pub(crate) fn obj_window_bg2_enabled(&self) -> bool {
        self.0.bit_is_set(10)
    }
    pub(crate) fn obj_window_bg3_enabled(&self) -> bool {
        self.0.bit_is_set(11)
    }
    pub(crate) fn obj_window_obj_enabled(&self) -> bool {
        self.0.bit_is_set(12)
    }
    pub(crate) fn obj_window_color_special_effects_enabled(&self) -> bool {
        self.0.bit_is_set(13)
    }

}
