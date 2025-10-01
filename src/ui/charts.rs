use crate::ui::{stats::Statistics, theme::FocusFiveTheme};
use chrono::{Datelike, Duration, NaiveDate};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    symbols,
    text::{Line, Span},
    widgets::{Axis, BarChart, Block, Borders, Chart, Dataset, Gauge, GraphType, Paragraph, Sparkline},
};

/// Data structure for weekly line chart that owns its data
pub struct WeeklyLineChart<'a> {
    data: Vec<(f64, f64)>,
    current_date: NaiveDate,
    theme: &'a FocusFiveTheme,
    line_color: Color,
}

impl<'a> WeeklyLineChart<'a> {
    pub fn new(stats: &Statistics, current_date: NaiveDate, theme: &'a FocusFiveTheme) -> Self {
        // Convert weekly trend data to line chart format
        let data: Vec<(f64, f64)> = stats
            .weekly_trend
            .iter()
            .enumerate()
            .map(|(i, &percentage)| (i as f64, percentage))
            .collect();

        // Determine line color based on average completion
        let avg_completion = if !stats.weekly_trend.is_empty() {
            stats.weekly_trend.iter().sum::<f64>() / stats.weekly_trend.len() as f64
        } else {
            0.0
        };

        let line_color = if avg_completion >= 80.0 {
            theme.completed
        } else if avg_completion >= 40.0 {
            theme.partial
        } else {
            theme.pending
        };

        Self {
            data,
            current_date,
            theme,
            line_color,
        }
    }

    pub fn render(&self, f: &mut ratatui::Frame, area: Rect) {
        use ratatui::widgets::{Chart, Dataset, Axis, Paragraph, GraphType};
        use ratatui::layout::{Layout, Constraint, Direction};
        use ratatui::symbols;
        
        // Split area into chart area and label area
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(5),      // Chart area (needs more height for line chart)
                Constraint::Length(1),   // Labels area
            ])
            .split(area);
            
        // Prepare data for the chart (x values from 0-6, y values as percentages)
        let chart_data: Vec<(f64, f64)> = self.data.clone();
        
        // Create the dataset
        let dataset = Dataset::default()
            .marker(symbols::Marker::Braille)
            .graph_type(GraphType::Line)
            .style(Style::default().fg(self.line_color))
            .data(&chart_data);
            
        // Create x-axis with day labels
        let x_labels: Vec<String> = (0..7).map(|i| {
            let date = self.current_date - Duration::days((6 - i) as i64);
            match date.weekday() {
                chrono::Weekday::Mon => "M",
                chrono::Weekday::Tue => "T",
                chrono::Weekday::Wed => "W",
                chrono::Weekday::Thu => "T",
                chrono::Weekday::Fri => "F",
                chrono::Weekday::Sat => "S",
                chrono::Weekday::Sun => "S",
            }.to_string()
        }).collect();
        
        let x_axis = Axis::default()
            .style(Style::default().fg(self.theme.text_secondary))
            .bounds([0.0, 6.0]);
            
        let y_axis = Axis::default()
            .style(Style::default().fg(self.theme.text_secondary))
            .bounds([0.0, 100.0]);
        
        // Create the chart
        let chart = Chart::new(vec![dataset])
            .block(
                Block::default()
                    .title(" WEEKLY PROGRESS (7-DAY) ")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(self.theme.border))
                    .style(Style::default().bg(self.theme.panel_bg)),
            )
            .x_axis(x_axis)
            .y_axis(y_axis);
            
        f.render_widget(chart, chunks[0]);
        
        // Calculate label spacing to match chart x-axis
        let inner_width = chunks[1].width.saturating_sub(2) as usize;
        let spacing_per_label = inner_width / 7;
        let total_used = spacing_per_label * 7;
        let left_padding = (inner_width - total_used) / 2;
        
        // Create full day labels with highlighting for today
        let mut label_spans = Vec::new();
        
        // Add initial padding
        if left_padding > 0 {
            label_spans.push(Span::raw(" ".repeat(left_padding)));
        }
        
        // Add each day label
        for i in 0..7 {
            let date = self.current_date - Duration::days((6 - i) as i64);
            let day_name = match date.weekday() {
                chrono::Weekday::Mon => "Mon",
                chrono::Weekday::Tue => "Tue",
                chrono::Weekday::Wed => "Wed",
                chrono::Weekday::Thu => "Thu",
                chrono::Weekday::Fri => "Fri",
                chrono::Weekday::Sat => "Sat",
                chrono::Weekday::Sun => "Sun",
            };
            
            // Calculate padding to center the 3-char day name within its space
            let label_padding = (spacing_per_label.saturating_sub(3)) / 2;
            let right_padding = spacing_per_label.saturating_sub(3 + label_padding);
            
            // Add left padding for this label
            if label_padding > 0 && i > 0 {
                label_spans.push(Span::raw(" ".repeat(label_padding)));
            } else if i == 0 && label_padding > 0 {
                label_spans.push(Span::raw(" ".repeat(label_padding)));
            }
            
            // Highlight today (last day in the 7-day window)
            if i == 6 {
                label_spans.push(Span::styled(
                    day_name,
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ));
            } else {
                label_spans.push(Span::styled(
                    day_name,
                    Style::default().fg(self.theme.text_secondary),
                ));
            }
            
            // Add right padding for this label (except for the last one)
            if i < 6 && right_padding > 0 {
                label_spans.push(Span::raw(" ".repeat(right_padding)));
            }
        }
        
        let labels = Paragraph::new(Line::from(label_spans))
            .style(Style::default().bg(self.theme.panel_bg));
            
        f.render_widget(labels, chunks[1]);
    }
}

/// Legacy bar chart function kept for compatibility (deprecated)
#[deprecated(note = "Use create_weekly_line_chart instead")]
pub fn create_weekly_chart<'a>(stats: &Statistics, theme: &FocusFiveTheme) -> BarChart<'a> {
    // Create bar data from the weekly trend
    let data: Vec<(&str, u64)> = vec![
        (
            "D1",
            stats.weekly_trend.get(0).unwrap_or(&0.0).round() as u64,
        ),
        (
            "D2",
            stats.weekly_trend.get(1).unwrap_or(&0.0).round() as u64,
        ),
        (
            "D3",
            stats.weekly_trend.get(2).unwrap_or(&0.0).round() as u64,
        ),
        (
            "D4",
            stats.weekly_trend.get(3).unwrap_or(&0.0).round() as u64,
        ),
        (
            "D5",
            stats.weekly_trend.get(4).unwrap_or(&0.0).round() as u64,
        ),
        (
            "D6",
            stats.weekly_trend.get(5).unwrap_or(&0.0).round() as u64,
        ),
        (
            "D7",
            stats.weekly_trend.get(6).unwrap_or(&0.0).round() as u64,
        ),
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
                .style(Style::default().bg(theme.panel_bg)),
        )
        .data(&data)
        .bar_width(3)
        .bar_gap(1)
        .value_style(Style::default().fg(theme.text_secondary))
        .label_style(Style::default().fg(theme.text_secondary))
        .style(Style::default().fg(bar_color))
}

pub fn render_trend_sparkline(
    data: &[f64],
    title: &str,
    theme: &FocusFiveTheme,
    f: &mut ratatui::Frame,
    area: ratatui::layout::Rect,
) {
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
                .style(Style::default().bg(theme.panel_bg)),
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
                .style(Style::default().bg(theme.panel_bg)),
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
    theme: &FocusFiveTheme,
) -> (Gauge<'a>, Gauge<'a>, Gauge<'a>) {
    let (work_pct, health_pct, family_pct) = stats.outcome_percentages;

    let work_gauge = Gauge::default()
        .block(
            Block::default()
                .title(" WORK ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme.work_color))
                .style(Style::default().bg(theme.panel_bg)),
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
                .style(Style::default().bg(theme.panel_bg)),
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
                .style(Style::default().bg(theme.panel_bg)),
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
        let current_date = chrono::Local::now().date_naive();

        // These should not panic
        #[allow(deprecated)]
        let _weekly_bar = create_weekly_chart(&stats, &theme);
        let _weekly_line = WeeklyLineChart::new(&stats, current_date, &theme);
        let _gauge = create_daily_gauge(stats.daily_completion, "TODAY", &theme);
        let _outcome_gauges = create_outcome_gauges(&stats, &theme);

        // Test sparkline rendering without actually rendering (just make sure the function doesn't panic)
        // We can't test actual rendering without a frame, but we can test the logic
        let values: Vec<u64> = stats
            .monthly_trend
            .iter()
            .map(|&v| v.round() as u64)
            .collect();
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
        let values: Vec<u64> = stats
            .monthly_trend
            .iter()
            .map(|&v| v.round() as u64)
            .collect();
        assert_eq!(values.len(), 0);
    }
}
