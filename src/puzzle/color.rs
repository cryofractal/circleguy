use egui::Color32;

#[derive(Clone, Debug, Copy, PartialEq, Eq)]
pub enum Color {
    Red,
    Green,
    Blue,
    Yellow,
    Purple,
    Gray,
    Black,
    Brown,
    Cyan,
    White,
    DarkBlue,
    DarkGreen,
    DarkGray,
    DarkRed,
    LightBlue,
    LightGray,
    LightGreen,
    LightYellow,
    LightRed,
    Khaki,
    Gold,
    Magenta,
    Orange,
    None,
}

impl Color {
    pub fn to_egui(&self) -> Color32 {
        match self {
            Color::Red => Color32::RED,
            Color::Green => Color32::GREEN,
            Color::Blue => Color32::BLUE,
            Color::Yellow => Color32::YELLOW,
            Color::Purple => Color32::PURPLE,
            Color::Gray => Color32::GRAY,
            Color::Black => Color32::BLACK,
            Color::Brown => Color32::BROWN,
            Color::Cyan => Color32::CYAN,
            Color::White => Color32::WHITE,
            Color::DarkBlue => Color32::DARK_BLUE,
            Color::DarkGreen => Color32::DARK_GREEN,
            Color::DarkGray => Color32::DARK_GRAY,
            Color::DarkRed => Color32::DARK_RED,
            Color::LightBlue => Color32::LIGHT_BLUE,
            Color::LightGray => Color32::LIGHT_GRAY,
            Color::LightGreen => Color32::LIGHT_GREEN,
            Color::LightYellow => Color32::LIGHT_YELLOW,
            Color::LightRed => Color32::LIGHT_RED,
            Color::Khaki => Color32::KHAKI,
            Color::Gold => Color32::GOLD,
            Color::Magenta => Color32::MAGENTA,
            Color::Orange => Color32::ORANGE,
            Color::None => Color32::GRAY,
        }
    }
}

pub fn rainbow_float(x: f64) -> Color32 {
    let colorous::Color { r, g, b } = colorous::RAINBOW.eval_continuous((x + 2.0).fract()); // make sure it's in the range 0.0..1.0
    Color32::from_rgb(r, g, b)
}

pub fn rainbow_rational(i: usize, n: usize) -> Color32 {
    let colorous::Color { r, g, b } = colorous::RAINBOW.eval_rational(i.rem_euclid(n), n); // make sure it's in the range 0.0..1.0
    Color32::from_rgb(r, g, b)
}

pub fn golden_ratio_color(n: usize) -> Color32 {
    rainbow_float(0.61803398874989484 * n as f64)
}
