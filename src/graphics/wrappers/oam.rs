use crate::utils::bits::Bits;

pub const NUM_OAM_ENTRIES: usize = 128;

#[derive(Debug)]
pub struct Oam<'a>(pub &'a [u16; 3]);

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

impl<'a> Oam<'a> {
    pub fn y(&self) -> u32 {
        (self.0[0] & 0xFF).into()
    }

    pub fn rotation_and_scaling_enabled(&self) -> bool {
        self.0[0].bit_is_set(8)
    }

    pub fn double_sized(&self) -> bool {
        self.rotation_and_scaling_enabled() && self.0[0].bit_is_set(9)
    }

    pub fn obj_disabled(&self) -> bool {
        !self.rotation_and_scaling_enabled() && self.0[0].bit_is_set(9)
    }

    pub fn obj_mode(&self) -> OBJMode {
        ((self.0[0] >> 10) & 0x3).into()
    }

    pub fn obj_mosaic(&self) -> bool {
        self.0[0].bit_is_set(12)
    }

    pub fn color_pallete(&self) -> usize {
        (self.0[0].get_bit(13)).into()
    }

    pub fn obj_shape(&self) -> OBJShape {
        ((self.0[0] >> 14) & 0x3).into()
    }

    pub fn x(&self) -> u32 {
        (self.0[1] & 0x1FF).into()
    }

    pub fn width(&self) -> u32 {
        match self.obj_shape() {
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
        }
    }

    pub fn height(&self) -> u32 {
        match self.obj_shape() {
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
        }
    }

    pub fn rotation_scaling_parameter(&self) -> u16 {
        if self.rotation_and_scaling_enabled() {
            return self.0[1] & 0x3E00;
        }
        0
    }

    pub fn horizontal_flip(&self) -> bool {
        self.0[1].bit_is_set(12)
    }

    pub fn vertical_flip(&self) -> bool {
        self.0[1].bit_is_set(13)
    }

    pub fn obj_size(&self) -> u16 {
        (self.0[1] >> 14) & 0x3
    }

    pub fn tile_number(&self) -> usize {
        self.0[2] as usize & 0x3FF
    }

    pub fn priority(&self) -> u16 {
        (self.0[2] >> 10) & 0x3
    }

    pub fn pallete_number(&self) -> usize {
        ((self.0[2] >> 12) & 0xF).into()
    }
}

#[cfg(test)]
mod oam_tests {
    use super::{Oam, OBJMode};

    #[test]
    fn can_get_y_from_bits() {
        let oam = Oam(&[0x2620, 0xc2ad, 0x0a40]);
        assert_eq!(oam.y(), 0x20);
    }

    #[test]
    fn can_check_if_rotation_scaling_enabled() {
        let oam = Oam(&[0x2620, 0xc2ad, 0x0a40]);
        assert_eq!(oam.rotation_and_scaling_enabled(), false);
        let oam = Oam(&[0x2720, 0xc2ad, 0x0a40]);
        assert_eq!(oam.rotation_and_scaling_enabled(), true);
    }

    #[test]
    fn can_check_if_double_sized() {
        let oam = Oam(&[0x2620, 0xc2ad, 0x0a40]);
        assert_eq!(oam.double_sized(), false);
        let oam = Oam(&[0x2720, 0xc2ad, 0x0a40]);
        assert_eq!(oam.double_sized(), true);
        let oam = Oam(&[0x2520, 0xc2ad, 0x0a40]);
        assert_eq!(oam.double_sized(), false);
    }

    #[test]
    fn can_check_obj_disabled() {
        let oam = Oam(&[0x2620, 0xc2ad, 0x0a40]);
        assert_eq!(oam.obj_disabled(), true);
        let oam = Oam(&[0x2720, 0xc2ad, 0x0a40]);
        assert_eq!(oam.obj_disabled(), false);
        let oam = Oam(&[0x2320, 0xc2ad, 0x0a40]);
        assert_eq!(oam.obj_disabled(), false);
    }

    #[test]
    fn can_get_obj_mode() {
        let oam = Oam(&[0x2620, 0xc2ad, 0x0a40]);
        assert!(matches!(oam.obj_mode(), OBJMode::SemiTransparent));
        let oam = Oam(&[0x2E20, 0xc2ad, 0x0a40]);
        assert!(matches!(oam.obj_mode(), OBJMode::Prohibited));
        let oam = Oam(&[0x2A20, 0xc2ad, 0x0a40]);
        assert!(matches!(oam.obj_mode(), OBJMode::OBJWindow));
        let oam = Oam(&[0x2220, 0xc2ad, 0x0a40]);
        assert!(matches!(oam.obj_mode(), OBJMode::Normal));
    }

    #[test]
    fn can_get_obj_mosaic() {
        let oam = Oam(&[0x2620, 0xc2ad, 0x0a40]);
        assert_eq!(oam.obj_mosaic(), false);

        let oam = Oam(&[0x3620, 0xc2ad, 0x0a40]);
        assert_eq!(oam.obj_mosaic(), true);
    }

    #[test]
    fn can_get_x() {
        let oam = Oam(&[0x2620, 0xc2ad, 0x0a40]);
        assert_eq!(oam.x(), 0xad);

        let oam = Oam(&[0x2620, 0xc3fd, 0x0a40]);
        assert_eq!(oam.x(), 0x1fd);
    }
}
