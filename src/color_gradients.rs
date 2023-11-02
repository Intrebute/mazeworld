use tiny_skia::Color;



pub fn glacier_colors() -> [Color; 3] {
    [
        Color::from_rgba8(149, 197, 215, u8::MAX),
        Color::from_rgba8(65, 133, 165, u8::MAX),
        Color::from_rgba8(44, 184, 218, u8::MAX),
    ]
}

pub fn fire_colors() -> [Color; 4] {
    [
        Color::from_rgba8(51, 0,0,u8::MAX),
        Color::from_rgba8(143, 0,0,u8::MAX),
        Color::from_rgba8(205, 113,0,u8::MAX),
        Color::from_rgba8(255, 255,0,u8::MAX),
    ]
}

pub fn trans_colors() -> [Color; 5] {
    [
        Color::from_rgba8(91, 206, 250, u8::MAX),
        Color::from_rgba8(245, 169, 184, u8::MAX),
        Color::from_rgba8(u8::MAX, u8::MAX, u8::MAX, u8::MAX),
        Color::from_rgba8(245, 169, 184, u8::MAX),
        Color::from_rgba8(91, 206, 250, u8::MAX),
    ]
}