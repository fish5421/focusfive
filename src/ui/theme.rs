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