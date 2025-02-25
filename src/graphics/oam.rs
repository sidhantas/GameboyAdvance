use crate::utils::bits::Bits;

pub const OAM_START: usize = 0x7000000;
pub const NUM_OAM_ENTRIES: usize = 128;

#[derive(Debug)]
pub struct OAM<'a>(pub &'a [u16; 3]);

#[derive(Debug)]
enum OBJMode {
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

enum OBJShape {
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

impl<'a> OAM<'a> {
    fn y(&self) -> u16 {
        self.0[0] & 0xFF
    }

    fn rotation_and_scaling_enabled(&self) -> bool {
        self.0[0].bit_is_set(8)
    }

    fn double_sized(&self) -> bool {
        self.rotation_and_scaling_enabled() && self.0[0] & 0x200 > 0
    }

    fn obj_disabled(&self) -> bool {
        !self.rotation_and_scaling_enabled() && self.0[0] & 0x200 > 0
    }

    fn obj_mode(&self) -> OBJMode {
        ((self.0[0] >> 10) & 0x3).into()
    }

    fn obj_mosaic(&self) -> bool {
        self.0[0].bit_is_set(12)
    }

    fn color_pallete(&self) -> u16 {
        self.0[0].get_bit(13)
    }

    fn obj_shape(&self) -> OBJShape {
        ((self.0[0] >> 14) & 0x3).into()
    }

    fn x(&self) -> u16 {
        self.0[1] & 0x1FF
    }

    fn height(&self) -> u16 {
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

    fn rotation_scaling_parameter(&self) -> u16 {
        if self.rotation_and_scaling_enabled() {
            return self.0[1] & 0x3E00;
        }
        0
    }

    fn horizontal_flip(&self) -> bool {
        self.0[1].bit_is_set(12)
    }

    fn vertical_flip(&self) -> bool {
        self.0[1].bit_is_set(13)
    }

    fn obj_size(&self) -> u16 {
        (self.0[1] >> 14) & 0x3
    }

    fn tile_number(&self) -> u16 {
        self.0[2] & 0x3FF
    }

    fn priority(&self) -> u16 {
        (self.0[2] >> 10) & 0x3
    }

    fn pallete_number(&self) -> u16 {
        (self.0[2] >> 12) & 0xF
    }
}

#[cfg(test)]
mod oam_tests {
    use crate::graphics::oam::OBJMode;

    use super::OAM;

    #[test]
    fn can_get_y_from_bits() {
        let oam = OAM(&[0x2620, 0xc2ad, 0x0a40]);
        assert_eq!(oam.y(), 0x20);
    }

    #[test]
    fn can_check_if_rotation_scaling_enabled() {
        let oam = OAM(&[0x2620, 0xc2ad, 0x0a40]);
        assert_eq!(oam.rotation_and_scaling_enabled(), false);
        let oam = OAM(&[0x2720, 0xc2ad, 0x0a40]);
        assert_eq!(oam.rotation_and_scaling_enabled(), true);
    }

    #[test]
    fn can_check_if_double_sized() {
        let oam = OAM(&[0x2620, 0xc2ad, 0x0a40]);
        assert_eq!(oam.double_sized(), false);
        let oam = OAM(&[0x2720, 0xc2ad, 0x0a40]);
        assert_eq!(oam.double_sized(), true);
        let oam = OAM(&[0x2520, 0xc2ad, 0x0a40]);
        assert_eq!(oam.double_sized(), false);
    }

    #[test]
    fn can_check_obj_disabled() {
        let oam = OAM(&[0x2620, 0xc2ad, 0x0a40]);
        assert_eq!(oam.obj_disabled(), true);
        let oam = OAM(&[0x2720, 0xc2ad, 0x0a40]);
        assert_eq!(oam.obj_disabled(), false);
        let oam = OAM(&[0x2320, 0xc2ad, 0x0a40]);
        assert_eq!(oam.obj_disabled(), false);
    }

    #[test]
    fn can_get_obj_mode() {
        let oam = OAM(&[0x2620, 0xc2ad, 0x0a40]);
        assert!(matches!(oam.obj_mode(), OBJMode::SemiTransparent));
        let oam = OAM(&[0x2E20, 0xc2ad, 0x0a40]);
        assert!(matches!(oam.obj_mode(), OBJMode::Prohibited));
        let oam = OAM(&[0x2A20, 0xc2ad, 0x0a40]);
        assert!(matches!(oam.obj_mode(), OBJMode::OBJWindow));
        let oam = OAM(&[0x2220, 0xc2ad, 0x0a40]);
        assert!(matches!(oam.obj_mode(), OBJMode::Normal));
    }

    #[test]
    fn can_get_obj_mosaic() {
        let oam = OAM(&[0x2620, 0xc2ad, 0x0a40]);
        assert_eq!(oam.obj_mosaic(), false);

        let oam = OAM(&[0x3620, 0xc2ad, 0x0a40]);
        assert_eq!(oam.obj_mosaic(), true);
    }

    #[test]
    fn can_get_x() {
        let oam = OAM(&[0x2620, 0xc2ad, 0x0a40]);
        assert_eq!(oam.x(), 0xad);

        let oam = OAM(&[0x2620, 0xc3fd, 0x0a40]);
        assert_eq!(oam.x(), 0x1fd);
    }
}
