enum TargetPixel {
    OBJPixel {
        priority: u16,
        pixel: u32,
        is_semi_transparent: bool
    },
    BGPixel {
        priority: u16,
        pixel: u32
    }
}

