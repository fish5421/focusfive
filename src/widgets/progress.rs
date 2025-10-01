use ratatui::{
    prelude::*,
    widgets::{Gauge, Paragraph},
};

pub struct IndicatorProgress {
    pub current: f64,
    pub target: f64,
    pub history: Vec<f64>,
    pub trend: TrendDirection,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TrendDirection {
    Up,
    Down,
    Stable,
}

impl IndicatorProgress {
    /// Create a new IndicatorProgress
    pub fn new(current: f64, target: f64, history: Vec<f64>) -> Self {
        let trend = Self::calculate_trend(&history);
        Self {
            current,
            target,
            history,
            trend,
        }
    }

    /// Calculate trend from history
    fn calculate_trend(history: &[f64]) -> TrendDirection {
        if history.len() < 2 {
            return TrendDirection::Stable;
        }

        let recent = &history[history.len().saturating_sub(5)..];
        if recent.len() < 2 {
            return TrendDirection::Stable;
        }

        let first_half_avg: f64 =
            recent[..recent.len() / 2].iter().sum::<f64>() / (recent.len() / 2) as f64;
        let second_half_avg: f64 = recent[recent.len() / 2..].iter().sum::<f64>()
            / (recent.len() - recent.len() / 2) as f64;

        let diff = second_half_avg - first_half_avg;
        let threshold = first_half_avg * 0.05; // 5% change threshold

        if diff > threshold {
            TrendDirection::Up
        } else if diff < -threshold {
            TrendDirection::Down
        } else {
            TrendDirection::Stable
        }
    }

    /// Render a progress bar as a string
    pub fn render_bar(&self) -> String {
        let percentage = (self.current / self.target * 100.0).min(100.0).max(0.0);
        let filled = (percentage / 10.0) as usize;
        let empty = 10_usize.saturating_sub(filled);

        format!("{}{}", "█".repeat(filled), "░".repeat(empty))
    }

    /// Create a styled paragraph with the progress bar
    pub fn render_bar_widget(&self) -> Paragraph<'static> {
        let percentage = (self.current / self.target * 100.0).min(100.0).max(0.0);
        let bar = self.render_bar();

        let style = match percentage as u32 {
            100.. => Style::default().fg(Color::Green),
            70..=99 => Style::default().fg(Color::Yellow),
            _ => Style::default().fg(Color::Red),
        };

        Paragraph::new(bar).style(style)
    }

    /// Get sparkline data (for use with Sparkline widget)
    pub fn get_sparkline_data(&self) -> Vec<u64> {
        self.history.iter().map(|&v| (v * 100.0) as u64).collect()
    }

    /// Get the trend arrow symbol
    pub fn render_trend(&self) -> &str {
        match self.trend {
            TrendDirection::Up => "↗",
            TrendDirection::Down => "↘",
            TrendDirection::Stable => "→",
        }
    }

    /// Get progress percentage
    pub fn get_percentage(&self) -> u16 {
        ((self.current / self.target * 100.0).min(100.0).max(0.0)) as u16
    }

    /// Create a gauge widget for the progress
    pub fn render_gauge(&self) -> Gauge<'static> {
        let percentage = self.get_percentage();
        let style = match percentage {
            100.. => Style::default().fg(Color::Green),
            70..=99 => Style::default().fg(Color::Yellow),
            _ => Style::default().fg(Color::Red),
        };

        Gauge::default()
            .percent(percentage)
            .style(style)
            .gauge_style(style)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_progress_bar_rendering() {
        let progress = IndicatorProgress::new(50.0, 100.0, vec![]);
        assert_eq!(progress.render_bar(), "█████░░░░░");

        let progress = IndicatorProgress::new(100.0, 100.0, vec![]);
        assert_eq!(progress.render_bar(), "██████████");

        let progress = IndicatorProgress::new(0.0, 100.0, vec![]);
        assert_eq!(progress.render_bar(), "░░░░░░░░░░");
    }

    #[test]
    fn test_percentage_calculation() {
        let progress = IndicatorProgress::new(50.0, 100.0, vec![]);
        assert_eq!(progress.get_percentage(), 50);

        let progress = IndicatorProgress::new(150.0, 100.0, vec![]);
        assert_eq!(progress.get_percentage(), 100); // Capped at 100

        let progress = IndicatorProgress::new(0.0, 100.0, vec![]);
        assert_eq!(progress.get_percentage(), 0);
    }

    #[test]
    fn test_trend_calculation() {
        // Upward trend
        let progress = IndicatorProgress::new(50.0, 100.0, vec![10.0, 20.0, 30.0, 40.0, 50.0]);
        assert_eq!(progress.trend, TrendDirection::Up);

        // Downward trend
        let progress = IndicatorProgress::new(50.0, 100.0, vec![50.0, 40.0, 30.0, 20.0, 10.0]);
        assert_eq!(progress.trend, TrendDirection::Down);

        // Stable trend
        let progress = IndicatorProgress::new(50.0, 100.0, vec![30.0, 31.0, 30.0, 31.0, 30.0]);
        assert_eq!(progress.trend, TrendDirection::Stable);

        // Empty history
        let progress = IndicatorProgress::new(50.0, 100.0, vec![]);
        assert_eq!(progress.trend, TrendDirection::Stable);
    }

    #[test]
    fn test_trend_symbols() {
        let mut progress = IndicatorProgress::new(50.0, 100.0, vec![]);
        progress.trend = TrendDirection::Up;
        assert_eq!(progress.render_trend(), "↗");

        progress.trend = TrendDirection::Down;
        assert_eq!(progress.render_trend(), "↘");

        progress.trend = TrendDirection::Stable;
        assert_eq!(progress.render_trend(), "→");
    }
}
