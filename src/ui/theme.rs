use ratatui::style::Color;

pub struct FocusFiveTheme {
    // Dark backgrounds
    pub background: Color,
    pub panel_bg: Color,
    pub border: Color,

    // Text colors
    pub text_primary: Color,
    pub text_secondary: Color,
    pub header: Color,

    // Status colors
    pub completed: Color,
    pub pending: Color,
    pub partial: Color,

    // Outcome colors
    pub work_color: Color,
    pub health_color: Color,
    pub family_color: Color,
}

impl Default for FocusFiveTheme {
    fn default() -> Self {
        Self {
            background: Color::Rgb(15, 15, 15),
            panel_bg: Color::Rgb(25, 25, 25),
            border: Color::Rgb(60, 60, 60),
            text_primary: Color::Rgb(220, 220, 220),
            text_secondary: Color::Rgb(160, 160, 160),
            header: Color::Rgb(255, 200, 0),
            completed: Color::Rgb(0, 255, 128),
            pending: Color::Rgb(255, 80, 80),
            partial: Color::Rgb(255, 165, 0),
            work_color: Color::Rgb(100, 181, 246),
            health_color: Color::Rgb(129, 199, 132),
            family_color: Color::Rgb(255, 183, 77),
        }
    }
}

pub struct FinancialTheme {
    // Dark backgrounds
    pub bg_primary: Color,   // #0A0A0A - Almost black
    pub bg_secondary: Color, // #151515 - Dark gray
    pub bg_panel: Color,     // #1A1A1A - Panel background

    // Status colors
    pub positive: Color, // #00FF41 - Bright green (gains)
    pub negative: Color, // #FF0040 - Bright red (losses)
    pub neutral: Color,  // #FFB000 - Amber (neutral)
    pub info: Color,     // #00BFFF - Cyan (information)

    // Text colors
    pub text_primary: Color,   // #E0E0E0 - Primary text
    pub text_secondary: Color, // #808080 - Secondary text
    pub text_dim: Color,       // #404040 - Dimmed text

    // Accent colors
    pub accent_blue: Color,   // #0080FF - Blue accent
    pub accent_purple: Color, // #B000FF - Purple accent
    pub accent_yellow: Color, // #FFD700 - Gold accent
}

impl Default for FinancialTheme {
    fn default() -> Self {
        Self {
            bg_primary: Color::Rgb(10, 10, 10),
            bg_secondary: Color::Rgb(21, 21, 21),
            bg_panel: Color::Rgb(26, 26, 26),
            positive: Color::Rgb(0, 255, 65),
            negative: Color::Rgb(255, 0, 64),
            neutral: Color::Rgb(255, 176, 0),
            info: Color::Rgb(0, 191, 255),
            text_primary: Color::Rgb(224, 224, 224),
            text_secondary: Color::Rgb(128, 128, 128),
            text_dim: Color::Rgb(64, 64, 64),
            accent_blue: Color::Rgb(0, 128, 255),
            accent_purple: Color::Rgb(176, 0, 255),
            accent_yellow: Color::Rgb(255, 215, 0),
        }
    }
}

impl FinancialTheme {
    pub fn get_trend_color(&self, value: f64, previous: f64) -> Color {
        if value > previous {
            self.positive
        } else if value < previous {
            self.negative
        } else {
            self.neutral
        }
    }

    pub fn get_status_color(&self, percentage: f64) -> Color {
        if percentage >= 80.0 {
            self.positive
        } else if percentage >= 50.0 {
            self.neutral
        } else {
            self.negative
        }
    }
}
