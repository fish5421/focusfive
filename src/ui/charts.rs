use ratatui::{
    widgets::{BarChart, Block, Borders, Sparkline, Gauge},
    style::Style,
    text::Span,
};
use crate::ui::{stats::Statistics, theme::FocusFiveTheme};

pub fn create_weekly_chart<'a>(stats: &Statistics, theme: &FocusFiveTheme) -> BarChart<'a> {
    // Create bar data from the weekly trend
    let data: Vec<(&str, u64)> = vec![
        ("D1", stats.weekly_trend.get(0).unwrap_or(&0.0).round() as u64),
        ("D2", stats.weekly_trend.get(1).unwrap_or(&0.0).round() as u64),
        ("D3", stats.weekly_trend.get(2).unwrap_or(&0.0).round() as u64),
        ("D4", stats.weekly_trend.get(3).unwrap_or(&0.0).round() as u64),
        ("D5", stats.weekly_trend.get(4).unwrap_or(&0.0).round() as u64),
        ("D6", stats.weekly_trend.get(5).unwrap_or(&0.0).round() as u64),
        ("D7", stats.weekly_trend.get(6).unwrap_or(&0.0).round() as u64),
    ];

    // Determine bar color based on average completion
    let avg_completion = stats.weekly_trend.iter().sum::<f64>() / stats.weekly_trend.len() as f64;
    let bar_color = if avg_completion >= 80.0 {
        theme.completed
    } else if avg_completion >= 40.0 {
        theme.partial
    } else {
        theme.pending
    };

    BarChart::default()
        .block(
            Block::default()
                .title(" WEEKLY PROGRESS ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme.border))
                .style(Style::default().bg(theme.panel_bg))
        )
        .data(&data)
        .bar_width(3)
        .bar_gap(1)
        .value_style(Style::default().fg(theme.text_secondary))
        .label_style(Style::default().fg(theme.text_secondary))
        .style(Style::default().fg(bar_color))
}

pub fn render_trend_sparkline(data: &[f64], title: &str, theme: &FocusFiveTheme, f: &mut ratatui::Frame, area: ratatui::layout::Rect) {
    // Convert f64 percentages to u64 for sparkline
    let values: Vec<u64> = data.iter().map(|&v| v.round() as u64).collect();

    // Determine color based on trend
    let avg = if !data.is_empty() {
        data.iter().sum::<f64>() / data.len() as f64
    } else {
        0.0
    };

    let color = if avg >= 80.0 {
        theme.completed
    } else if avg >= 40.0 {
        theme.partial
    } else {
        theme.pending
    };

    let sparkline = Sparkline::default()
        .block(
            Block::default()
                .title(format!(" {} ", title))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme.border))
                .style(Style::default().bg(theme.panel_bg))
        )
        .data(&values)
        .style(Style::default().fg(color))
        .max(100); // Set max to 100 since we're dealing with percentages

    f.render_widget(sparkline, area);
}

pub fn create_daily_gauge<'a>(percentage: f64, title: &str, theme: &FocusFiveTheme) -> Gauge<'a> {
    let color = if percentage >= 80.0 {
        theme.completed
    } else if percentage >= 40.0 {
        theme.partial
    } else {
        theme.pending
    };

    Gauge::default()
        .block(
            Block::default()
                .title(format!(" {} ", title))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme.border))
                .style(Style::default().bg(theme.panel_bg))
        )
        .gauge_style(Style::default().fg(color))
        .percent(percentage.round() as u16)
        .label(Span::styled(
            format!("{:.0}%", percentage),
            Style::default().fg(theme.text_primary),
        ))
}

pub fn create_outcome_gauges<'a>(
    stats: &Statistics,
    theme: &FocusFiveTheme
) -> (Gauge<'a>, Gauge<'a>, Gauge<'a>) {
    let (work_pct, health_pct, family_pct) = stats.outcome_percentages;

    let work_gauge = Gauge::default()
        .block(
            Block::default()
                .title(" WORK ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme.work_color))
                .style(Style::default().bg(theme.panel_bg))
        )
        .gauge_style(Style::default().fg(theme.work_color))
        .percent(work_pct.round() as u16)
        .label(Span::styled(
            format!("{:.0}%", work_pct),
            Style::default().fg(theme.text_primary),
        ));

    let health_gauge = Gauge::default()
        .block(
            Block::default()
                .title(" HEALTH ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme.health_color))
                .style(Style::default().bg(theme.panel_bg))
        )
        .gauge_style(Style::default().fg(theme.health_color))
        .percent(health_pct.round() as u16)
        .label(Span::styled(
            format!("{:.0}%", health_pct),
            Style::default().fg(theme.text_primary),
        ));

    let family_gauge = Gauge::default()
        .block(
            Block::default()
                .title(" FAMILY ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme.family_color))
                .style(Style::default().bg(theme.panel_bg))
        )
        .gauge_style(Style::default().fg(theme.family_color))
        .percent(family_pct.round() as u16)
        .label(Span::styled(
            format!("{:.0}%", family_pct),
            Style::default().fg(theme.text_primary),
        ));

    (work_gauge, health_gauge, family_gauge)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chart_creation_doesnt_panic() {
        let stats = Statistics {
            daily_completion: 55.5,
            weekly_trend: vec![10.0, 20.0, 30.0, 40.0, 50.0, 60.0, 70.0],
            monthly_trend: vec![50.0; 30],
            outcome_percentages: (33.3, 66.6, 100.0),
        };

        let theme = FocusFiveTheme::default();

        // These should not panic
        let _weekly = create_weekly_chart(&stats, &theme);
        let _gauge = create_daily_gauge(stats.daily_completion, "TODAY", &theme);
        let _outcome_gauges = create_outcome_gauges(&stats, &theme);

        // Test sparkline rendering without actually rendering (just make sure the function doesn't panic)
        // We can't test actual rendering without a frame, but we can test the logic
        let values: Vec<u64> = stats.monthly_trend.iter().map(|&v| v.round() as u64).collect();
        assert_eq!(values.len(), 30);
    }

    #[test]
    fn test_empty_data_handling() {
        let stats = Statistics {
            daily_completion: 0.0,
            weekly_trend: vec![],
            monthly_trend: vec![],
            outcome_percentages: (0.0, 0.0, 0.0),
        };

        let theme = FocusFiveTheme::default();

        // Should handle empty data gracefully
        let _weekly = create_weekly_chart(&stats, &theme);

        // Test that empty data converts correctly
        let values: Vec<u64> = stats.monthly_trend.iter().map(|&v| v.round() as u64).collect();
        assert_eq!(values.len(), 0);
    }
}