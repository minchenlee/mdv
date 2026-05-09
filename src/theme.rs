use iced::Color;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThemeMode { Light, Dark, System }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThemePreset {
    OneDark,
    OneLight,
    GitHubDark,
    GitHubLight,
    Solarized,
    SolarizedLight,
}

impl ThemePreset {
    pub const ALL: [ThemePreset; 6] = [
        ThemePreset::OneLight,
        ThemePreset::OneDark,
        ThemePreset::GitHubLight,
        ThemePreset::GitHubDark,
        ThemePreset::SolarizedLight,
        ThemePreset::Solarized,
    ];

    pub fn label(self) -> &'static str {
        match self {
            ThemePreset::OneDark => "One Dark",
            ThemePreset::OneLight => "One Light",
            ThemePreset::GitHubDark => "GitHub Dark",
            ThemePreset::GitHubLight => "GitHub Light",
            ThemePreset::Solarized => "Solarized Dark",
            ThemePreset::SolarizedLight => "Solarized Light",
        }
    }

    pub fn is_dark(self) -> bool {
        matches!(
            self,
            ThemePreset::OneDark | ThemePreset::GitHubDark | ThemePreset::Solarized
        )
    }

    pub fn next(self) -> ThemePreset {
        let idx = ThemePreset::ALL.iter().position(|t| *t == self).unwrap_or(0);
        ThemePreset::ALL[(idx + 1) % ThemePreset::ALL.len()]
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Palette {
    pub bg: Color,
    pub surface: Color,
    pub surface_alt: Color,
    pub sidebar: Color,
    pub fg: Color,
    pub muted: Color,
    pub subtle: Color,
    pub accent: Color,
    pub accent_fg: Color,
    pub code_bg: Color,
    pub code_border: Color,
    pub rule: Color,
    pub selection: Color,
    pub match_bg: Color,
    pub match_current_bg: Color,
    pub scroller: Color,
    pub scroller_hover: Color,
    pub indent_guide: Color,
    pub tree_selected_bg: Color,
    pub tree_selected_border: Color,
}

const fn rgb(r: u8, g: u8, b: u8) -> Color {
    Color::from_rgb(r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0)
}
const fn rgba(r: u8, g: u8, b: u8, a: f32) -> Color {
    Color::from_rgba(r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0, a)
}

impl Palette {
    // Atom / Zed One Dark
    pub const ONE_DARK: Palette = Palette {
        bg: rgb(40, 44, 52),
        surface: rgb(33, 37, 43),
        surface_alt: rgb(47, 52, 61),
        sidebar: rgb(33, 37, 43),
        fg: rgb(220, 223, 228),
        muted: rgb(150, 156, 167),
        subtle: rgb(100, 106, 117),
        accent: rgb(229, 160, 107),
        accent_fg: rgb(26, 18, 12),
        code_bg: rgb(36, 40, 47),
        code_border: rgba(255, 255, 255, 0.06),
        rule: rgba(255, 255, 255, 0.07),
        selection: rgba(229, 160, 107, 0.25),
        match_bg: rgba(229, 192, 123, 0.45),
        match_current_bg: rgba(229, 130, 50, 0.85),
        scroller: rgba(255, 255, 255, 0.0),
        scroller_hover: rgba(255, 255, 255, 0.22),
        indent_guide: rgba(255, 255, 255, 0.06),
        tree_selected_bg: rgba(229, 160, 107, 0.12),
        tree_selected_border: rgb(229, 160, 107),
    };

    // Atom / Zed One Light
    pub const ONE_LIGHT: Palette = Palette {
        bg: rgb(250, 250, 250),
        surface: rgb(255, 255, 255),
        surface_alt: rgb(240, 240, 241),
        sidebar: rgb(245, 245, 246),
        fg: rgb(56, 58, 66),
        muted: rgb(112, 116, 124),
        subtle: rgb(160, 164, 172),
        accent: rgb(217, 119, 87),
        accent_fg: rgb(255, 255, 255),
        code_bg: rgb(244, 244, 244),
        code_border: rgba(0, 0, 0, 0.08),
        rule: rgba(0, 0, 0, 0.08),
        selection: rgba(217, 119, 87, 0.22),
        match_bg: rgba(252, 207, 80, 0.65),
        match_current_bg: rgba(252, 130, 30, 0.90),
        scroller: rgba(0, 0, 0, 0.0),
        scroller_hover: rgba(0, 0, 0, 0.30),
        indent_guide: rgba(0, 0, 0, 0.08),
        tree_selected_bg: rgba(217, 119, 87, 0.10),
        tree_selected_border: rgb(217, 119, 87),
    };

    // GitHub Dark
    pub const GITHUB_DARK: Palette = Palette {
        bg: rgb(13, 17, 23),
        surface: rgb(22, 27, 34),
        surface_alt: rgb(33, 38, 45),
        sidebar: rgb(13, 17, 23),
        fg: rgb(201, 209, 217),
        muted: rgb(139, 148, 158),
        subtle: rgb(110, 118, 129),
        accent: rgb(253, 140, 115),
        accent_fg: rgb(13, 17, 23),
        code_bg: rgb(22, 27, 34),
        code_border: rgb(48, 54, 61),
        rule: rgb(48, 54, 61),
        selection: rgba(253, 140, 115, 0.25),
        match_bg: rgba(187, 128, 9, 0.45),
        match_current_bg: rgba(255, 140, 30, 0.85),
        scroller: rgba(255, 255, 255, 0.0),
        scroller_hover: rgba(255, 255, 255, 0.22),
        indent_guide: rgb(33, 38, 45),
        tree_selected_bg: rgba(253, 140, 115, 0.12),
        tree_selected_border: rgb(253, 140, 115),
    };

    // GitHub Light
    pub const GITHUB_LIGHT: Palette = Palette {
        bg: rgb(255, 255, 255),
        surface: rgb(246, 248, 250),
        surface_alt: rgb(234, 238, 242),
        sidebar: rgb(246, 248, 250),
        fg: rgb(36, 41, 47),
        muted: rgb(101, 109, 118),
        subtle: rgb(140, 149, 159),
        accent: rgb(188, 76, 0),
        accent_fg: rgb(255, 255, 255),
        code_bg: rgb(246, 248, 250),
        code_border: rgb(208, 215, 222),
        rule: rgb(208, 215, 222),
        selection: rgba(188, 76, 0, 0.18),
        match_bg: rgba(252, 207, 80, 0.65),
        match_current_bg: rgba(252, 130, 30, 0.90),
        scroller: rgba(0, 0, 0, 0.0),
        scroller_hover: rgba(0, 0, 0, 0.30),
        indent_guide: rgb(208, 215, 222),
        tree_selected_bg: rgba(188, 76, 0, 0.08),
        tree_selected_border: rgb(188, 76, 0),
    };

    // Solarized Dark
    pub const SOLARIZED_DARK: Palette = Palette {
        bg: rgb(0, 43, 54),
        surface: rgb(7, 54, 66),
        surface_alt: rgb(20, 67, 79),
        sidebar: rgb(0, 38, 48),
        fg: rgb(147, 161, 161),
        muted: rgb(101, 123, 131),
        subtle: rgb(88, 110, 117),
        accent: rgb(203, 75, 22),
        accent_fg: rgb(253, 246, 227),
        code_bg: rgb(7, 54, 66),
        code_border: rgba(255, 255, 255, 0.07),
        rule: rgba(255, 255, 255, 0.07),
        selection: rgba(203, 75, 22, 0.25),
        match_bg: rgba(181, 137, 0, 0.55),
        match_current_bg: rgba(203, 75, 22, 0.90),
        scroller: rgba(255, 255, 255, 0.0),
        scroller_hover: rgba(255, 255, 255, 0.22),
        indent_guide: rgba(255, 255, 255, 0.07),
        tree_selected_bg: rgba(203, 75, 22, 0.12),
        tree_selected_border: rgb(203, 75, 22),
    };

    // Solarized Light
    pub const SOLARIZED_LIGHT: Palette = Palette {
        bg: rgb(253, 246, 227),
        surface: rgb(238, 232, 213),
        surface_alt: rgb(228, 222, 203),
        sidebar: rgb(245, 238, 219),
        fg: rgb(101, 123, 131),
        muted: rgb(131, 148, 150),
        subtle: rgb(147, 161, 161),
        accent: rgb(203, 75, 22),
        accent_fg: rgb(253, 246, 227),
        code_bg: rgb(238, 232, 213),
        code_border: rgba(0, 0, 0, 0.10),
        rule: rgba(0, 0, 0, 0.10),
        selection: rgba(203, 75, 22, 0.18),
        match_bg: rgba(181, 137, 0, 0.45),
        match_current_bg: rgba(203, 75, 22, 0.85),
        scroller: rgba(0, 0, 0, 0.0),
        scroller_hover: rgba(0, 0, 0, 0.30),
        indent_guide: rgba(0, 0, 0, 0.08),
        tree_selected_bg: rgba(203, 75, 22, 0.10),
        tree_selected_border: rgb(203, 75, 22),
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
        body_size: 15.5, line_height: 1.65, measure_ch: 74,
        h1_size: 30.0, h2_size: 24.0, h3_size: 20.0,
        h4_size: 17.0, h5_size: 15.5, h6_size: 14.5,
        code_size: 13.5,
    };
}

pub fn palette_for(preset: ThemePreset) -> Palette {
    match preset {
        ThemePreset::OneDark => Palette::ONE_DARK,
        ThemePreset::OneLight => Palette::ONE_LIGHT,
        ThemePreset::GitHubDark => Palette::GITHUB_DARK,
        ThemePreset::GitHubLight => Palette::GITHUB_LIGHT,
        ThemePreset::Solarized => Palette::SOLARIZED_DARK,
        ThemePreset::SolarizedLight => Palette::SOLARIZED_LIGHT,
    }
}

pub fn resolve_mode(mode: ThemeMode) -> ThemePreset {
    match mode {
        ThemeMode::Light => ThemePreset::OneLight,
        ThemeMode::Dark => ThemePreset::OneDark,
        ThemeMode::System => match dark_light::detect() {
            dark_light::Mode::Dark => ThemePreset::OneDark,
            _ => ThemePreset::OneLight,
        },
    }
}

pub fn resolve(mode: ThemeMode) -> Palette {
    palette_for(resolve_mode(mode))
}
