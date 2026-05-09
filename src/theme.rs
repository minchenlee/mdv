use iced::Color;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThemeMode { Light, Dark, System }

#[derive(Debug, Clone, Copy)]
pub struct Palette {
    pub bg: Color,
    pub fg: Color,
    pub muted: Color,
    pub accent: Color,
    pub code_bg: Color,
    pub code_border: Color,
    pub rule: Color,
    pub selection: Color,
}

impl Palette {
    pub const LIGHT: Palette = Palette {
        bg: Color::from_rgb(0.98, 0.98, 0.97),
        fg: Color::from_rgb(0.10, 0.10, 0.10),
        muted: Color::from_rgb(0.45, 0.45, 0.45),
        accent: Color::from_rgb(0.20, 0.40, 0.85),
        code_bg: Color::from_rgb(0.95, 0.95, 0.93),
        code_border: Color::from_rgb(0.88, 0.88, 0.85),
        rule: Color::from_rgb(0.85, 0.85, 0.82),
        selection: Color::from_rgba(0.20, 0.40, 0.85, 0.25),
    };
    pub const DARK: Palette = Palette {
        bg: Color::from_rgb(0.086, 0.094, 0.106),
        fg: Color::from_rgb(0.91, 0.90, 0.88),
        muted: Color::from_rgb(0.60, 0.60, 0.58),
        accent: Color::from_rgb(0.55, 0.75, 1.0),
        code_bg: Color::from_rgb(0.12, 0.13, 0.15),
        code_border: Color::from_rgb(0.20, 0.21, 0.23),
        rule: Color::from_rgb(0.22, 0.22, 0.24),
        selection: Color::from_rgba(0.55, 0.75, 1.0, 0.25),
    };
}

#[derive(Debug, Clone, Copy)]
pub struct Typography {
    pub body_size: f32,
    pub line_height: f32,
    pub measure_ch: u32,
    pub h1_size: f32,
    pub h2_size: f32,
    pub h3_size: f32,
    pub h4_size: f32,
    pub h5_size: f32,
    pub h6_size: f32,
    pub code_size: f32,
}

impl Typography {
    pub const DEFAULT: Typography = Typography {
        body_size: 16.0, line_height: 1.6, measure_ch: 70,
        h1_size: 32.0, h2_size: 26.0, h3_size: 21.0,
        h4_size: 18.0, h5_size: 16.0, h6_size: 15.0,
        code_size: 14.0,
    };
}

pub fn resolve(mode: ThemeMode) -> Palette {
    match mode {
        ThemeMode::Light => Palette::LIGHT,
        ThemeMode::Dark => Palette::DARK,
        ThemeMode::System => match dark_light::detect() {
            dark_light::Mode::Dark => Palette::DARK,
            _ => Palette::LIGHT,
        },
    }
}
