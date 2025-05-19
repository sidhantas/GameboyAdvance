use std::cell::Cell;

use crate::utils::bits::{sign_extend, Bits};

pub const NUM_OAM_ENTRIES: usize = 128;

#[derive(Default, Debug)]
pub struct Oam {
    data: [u16; 3],
    x: Cell<Option<i32>>,
    y: Cell<Option<i32>>,
    width: Cell<Option<i32>>,
    view_width: Cell<Option<i32>>,
    height: Cell<Option<i32>>,
    view_height: Cell<Option<i32>>,
    rotation_and_scaling_enabled: Cell<Option<bool>>,
    double_sized: Cell<Option<bool>>
}

#[derive(Debug)]
pub enum OBJMode {
    Normal,
    SemiTransparent,
    OBJWindow,
    Prohibited,
}

impl Into<OBJMode> for u16 {
    fn into(self) -> OBJMode {
        match self {
            0 => OBJMode::Normal,
            1 => OBJMode::SemiTransparent,
            2 => OBJMode::OBJWindow,
            3 => OBJMode::Prohibited,
            _ => panic!("Invalid OBJMode"),
        }
    }
}

#[derive(Debug)]
pub enum OBJShape {
    Square,
    Horizonatal,
    Vertical,
    Prohibited,
}

impl Into<OBJShape> for u16 {
    fn into(self) -> OBJShape {
        match self {
            0 => OBJShape::Square,
            1 => OBJShape::Horizonatal,
            2 => OBJShape::Vertical,
            3 => OBJShape::Prohibited,
            _ => panic!("Invalid OBJMode"),
        }
    }
}

impl Oam {
    pub fn new(data: [u16; 3]) -> Self {
        let mut oam = Self::default();

        oam.data = data;
        oam
    }
    pub fn y(&self) -> i32 {
        if let Some(y) = self.y.get() {
            return y;
        }
        let y = sign_extend((self.data[0] & 0xFF) as u32, 7) as i32;
        self.y.replace(Some(y));
        y
    }

    pub fn rotation_and_scaling_enabled(&self) -> bool {
        if let Some(rotation_and_scaling_enabled) = self.rotation_and_scaling_enabled.get() {
            return rotation_and_scaling_enabled;
        }

        let rotation_and_scaling_enabled = self.data[0].bit_is_set(8);
        self.rotation_and_scaling_enabled
            .replace(Some(rotation_and_scaling_enabled));

        rotation_and_scaling_enabled
    }

    pub fn double_sized(&self) -> bool {
        if let Some(double_sized) = self.double_sized.get() {
            return double_sized;
        }
        let double_sized = self.rotation_and_scaling_enabled() && self.data[0].bit_is_set(9);
        self.double_sized
            .replace(Some(double_sized));

        double_sized
    }

    pub fn obj_disabled(&self) -> bool {
        !self.rotation_and_scaling_enabled() && self.data[0].bit_is_set(9)
    }

    pub fn obj_mode(&self) -> OBJMode {
        ((self.data[0] >> 10) & 0x3).into()
    }

    pub fn obj_mosaic(&self) -> bool {
        self.data[0].bit_is_set(12)
    }

    pub fn color_pallete(&self) -> usize {
        (self.data[0].get_bit(13)).into()
    }

    pub fn obj_shape(&self) -> OBJShape {
        ((self.data[0] >> 14) & 0x3).into()
    }

    pub fn x(&self) -> i32 {
        if let Some(x) = self.x.get() {
            return x;
        }
        let x = sign_extend((self.data[1] & 0x1FF) as u32, 8) as i32;
        self.x.replace(Some(x));
        x
    }

    pub fn width(&self) -> i32 {
        if let Some(width) = self.width.get() {
            return width;
        }
        let width = match self.obj_shape() {
            OBJShape::Square => match self.obj_size() {
                0 => 8,
                1 => 16,
                2 => 32,
                3 => 64,
                _ => panic!("Invalid obj size"),
            },
            OBJShape::Horizonatal => match self.obj_size() {
                0 => 16,
                1 => 32,
                2 => 32,
                3 => 64,
                _ => panic!("Invalid obj size"),
            },
            OBJShape::Vertical => match self.obj_size() {
                0 => 8,
                1 => 8,
                2 => 16,
                3 => 32,
                _ => panic!("Invalid obj size"),
            },
            OBJShape::Prohibited => panic!("Invalid obj shape"),
        };
        self.width.replace(Some(width));

        width
    }

    pub fn view_width(&self) -> i32 {
        if let Some(view_width) = self.view_width.get() {
            return view_width
        }
        let view_width = if self.double_sized() {
            self.width() * 2
        } else {
            self.width()
        };

        self.view_width.replace(Some(view_width));
        
        view_width

    }

    pub fn height(&self) -> i32 {
        if let Some(height) = self.height.get() {
            return height
        }
        let height = match self.obj_shape() {
            OBJShape::Square => match self.obj_size() {
                0 => 8,
                1 => 16,
                2 => 32,
                3 => 64,
                _ => panic!("Invalid obj size"),
            },
            OBJShape::Horizonatal => match self.obj_size() {
                0 => 8,
                1 => 8,
                2 => 16,
                3 => 32,
                _ => panic!("Invalid obj size"),
            },
            OBJShape::Vertical => match self.obj_size() {
                0 => 16,
                1 => 32,
                2 => 32,
                3 => 64,
                _ => panic!("Invalid obj size"),
            },
            OBJShape::Prohibited => panic!("Invalid obj shape"),
        };

        self.height.replace(Some(height));
        height
    }

    pub fn view_height(&self) -> i32 {
        if let Some(view_height) = self.view_height.get() {
            return view_height
        }
        let view_height = if self.double_sized() {
            self.height() * 2
        } else {
            self.height()
        };

        self.view_height.replace(Some(view_height));
        view_height
    }

    pub fn rotation_scaling_parameter(&self) -> Option<usize> {
        if self.rotation_and_scaling_enabled() {
            return Some(((self.data[1] & 0x3E00) >> 9) as usize);
        }
        None
    }

    pub fn horizontal_flip(&self) -> bool {
        self.data[1].bit_is_set(12)
    }

    pub fn vertical_flip(&self) -> bool {
        self.data[1].bit_is_set(13)
    }

    pub fn obj_size(&self) -> u16 {
        (self.data[1] >> 14) & 0x3
    }

    pub fn tile_number(&self) -> usize {
        self.data[2] as usize & 0x3FF
    }

    pub fn priority(&self) -> u16 {
        (self.data[2] >> 10) & 0x3
    }

    pub fn pallete_number(&self) -> usize {
        ((self.data[2] >> 12) & 0xF).into()
    }
}
