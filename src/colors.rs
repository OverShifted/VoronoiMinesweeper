use sdl2::pixels::Color;

pub const CELL_COVERED: Color = Color::WHITE;
pub const CELL_FLAGGED: Color = Color::RGB(90, 190, 255);

pub const CELL_UNCOVERED: Color = Color::RGB(220, 220, 220);
pub const CELL_BOMBED: Color = Color::RGB(255, 0, 80);

pub const CELL_OUTLINE: Color = Color::BLACK;
pub const NUMBERS: Color = Color::RGB(10, 70, 255);
