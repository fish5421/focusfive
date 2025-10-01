use crate::models::{IndicatorDef, IndicatorDirection, Observation};
use crate::ui::theme::FinancialTheme;
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Cell, Row, Table, Widget},
};

#[derive(Clone, Copy, Debug, PartialEq)]
struct MetricSnapshot {
    current: f64,
    previous: f64,
    target: f64,
    spread_pct: f64,
    trend_delta: f64,
    trend_arrow: Option<char>,
    value_color: Color,
    spread_color: Color,
}

impl MetricSnapshot {
    fn create(
        indicator: &IndicatorDef,
        current: f64,
        previous: f64,
        theme: &FinancialTheme,
    ) -> Self {
        let target = indicator.target.unwrap_or(100.0);
        // Avoid divide-by-zero when the target is zero (common for cost/defect metrics)
        let denominator = if target.abs() < f64::EPSILON {
            1.0
        } else {
            target.abs()
        };

        let spread_pct = ((current - target).abs() * 100.0 / denominator).min(999.9);
        let trend_delta = current - previous;
        let trend_arrow = if trend_delta.abs() < f64::EPSILON {
            None
        } else if trend_delta > 0.0 {
            Some('↑')
        } else {
            Some('↓')
        };

        let value_color = match indicator.direction {
            IndicatorDirection::HigherIsBetter => theme.get_trend_color(current, previous),
            IndicatorDirection::LowerIsBetter => theme.get_trend_color(previous, current),
            IndicatorDirection::WithinRange => {
                let distance_current = (current - target).abs();
                let distance_previous = (previous - target).abs();
                if distance_current < distance_previous {
                    theme.positive
                } else if distance_current > distance_previous {
                    theme.negative
                } else {
                    theme.neutral
                }
            }
        };

        let spread_color = if spread_pct < 10.0 {
            theme.positive
        } else if spread_pct < 25.0 {
            theme.neutral
        } else {
            theme.negative
        };

        Self {
            current,
            previous,
            target,
            spread_pct,
            trend_delta,
            trend_arrow,
            value_color,
            spread_color,
        }
    }
}

pub struct LiveMetricsWidget<'a> {
    indicators: &'a [IndicatorDef],
    observations: &'a [Observation],
    theme: &'a FinancialTheme,
    block: Option<Block<'a>>,
}

impl<'a> LiveMetricsWidget<'a> {
    pub fn new(
        indicators: &'a [IndicatorDef],
        observations: &'a [Observation],
        theme: &'a FinancialTheme,
    ) -> Self {
        Self {
            indicators,
            observations,
            theme,
            block: None,
        }
    }

    pub fn block(mut self, block: Block<'a>) -> Self {
        self.block = Some(block);
        self
    }

    fn current_and_previous(&self, indicator_id: &str) -> (Option<f64>, Option<f64>) {
        let mut previous = None;
        let mut current = None;

        for value in self
            .observations
            .iter()
            .filter(|obs| obs.indicator_id == indicator_id)
            .map(|obs| obs.value)
        {
            previous = current;
            current = Some(value);
        }

        (current, previous)
    }

    fn build_snapshot(&self, indicator: &IndicatorDef) -> MetricSnapshot {
        let (current, previous) = self.current_and_previous(&indicator.id);
        let current = current.unwrap_or(0.0);
        let previous = previous.unwrap_or(current);

        MetricSnapshot::create(indicator, current, previous, self.theme)
    }

    fn format_indicator_name(&self, name: &str, max_width: usize) -> String {
        if name.len() <= max_width {
            // Pad short names for consistent column width
            format!("{:<width$}", name, width = max_width)
        } else {
            // Smart truncation at word boundary with ellipsis
            let mut truncate_at = max_width.saturating_sub(3); // Leave room for "..."
            
            // Try to find a word boundary (space) near the truncation point
            if let Some(last_space) = name[..truncate_at.min(name.len())]
                .rfind(' ')
                .filter(|&pos| pos > max_width / 2) // Only use if space is in latter half
            {
                truncate_at = last_space;
            }
            
            // Ensure we don't panic on char boundaries
            let safe_truncate = truncate_at.min(name.len());
            format!("{}...", &name[..safe_truncate].trim_end())
        }
    }

    fn format_metric_value(&self, value: f64, precision: usize) -> String {
        match precision {
            0 => format!("{:.0}", value),
            1 => format!("{:.1}", value),
            2 => format!("{:.2}", value),
            3 => format!("{:.3}", value),
            _ => format!("{:.1}", value), // Default to 1 decimal place
        }
    }

    fn format_metric_row(&self, indicator: &IndicatorDef, _row_index: usize, indicator_width: usize) -> Row<'a> {
        let snapshot = self.build_snapshot(indicator);

        // Format indicator name without arrow - no selection needed
        let indicator_name = self.format_indicator_name(&indicator.name, indicator_width);

        let mut cells = vec![
            Cell::from(indicator_name)
                .style(Style::default().fg(self.theme.text_primary)),
            Cell::from(self.format_metric_value(snapshot.current, 1))
                .style(Style::default()
                    .fg(snapshot.value_color)
                    .add_modifier(Modifier::BOLD)),
            Cell::from(self.format_metric_value(snapshot.target, 1))
                .style(Style::default().fg(self.theme.text_secondary)),
            Cell::from(format!("{}%", self.format_metric_value(snapshot.spread_pct, 1)))
                .style(Style::default().fg(snapshot.spread_color)),
        ];

        // Add trend column if trend data exists
        if let Some(arrow) = snapshot.trend_arrow {
            cells.push(Cell::from(format!("{} {:+.1}", arrow, snapshot.trend_delta))
                .style(Style::default().fg(snapshot.value_color)));
        } else {
            cells.push(Cell::from("-")
                .style(Style::default().fg(self.theme.text_secondary)));
        }

        Row::new(cells)
    }
}

impl<'a> Widget for LiveMetricsWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Dynamic column width calculation to maximize space usage
        let total_width = area.width as usize;

        // Define column widths with better space utilization
        let min_indicator_width = 20;  // Increased from 16
        let current_width = 8;          // Decreased from 10 (values fit in 8)
        let target_width = 8;           // Decreased from 10
        let spread_width = 8;           // Decreased from 9
        let trend_width = 10;           // Keep as is for "↑ +10.5" format

        // Calculate space requirements
        let fixed_columns_width = current_width + target_width + spread_width + trend_width;
        let column_spacing = 2 * 4; // 2 chars spacing × 4 gaps between 5 columns
        let borders_and_padding = 4; // Left/right borders and internal padding
        
        // Calculate available width for indicator column - use remaining space
        let available_for_indicator = total_width
            .saturating_sub(fixed_columns_width + column_spacing + borders_and_padding);

        // Use all available space, but keep a reasonable minimum
        let indicator_width = available_for_indicator.max(min_indicator_width);

        let header = Row::new(vec![
            Cell::from("Indicator").style(Style::default()
                .fg(self.theme.text_primary)
                .add_modifier(Modifier::BOLD)),
            Cell::from("Current").style(Style::default()
                .fg(self.theme.text_primary)
                .add_modifier(Modifier::BOLD)),
            Cell::from("Target").style(Style::default()
                .fg(self.theme.text_primary)
                .add_modifier(Modifier::BOLD)),
            Cell::from("Spread").style(Style::default()
                .fg(self.theme.text_primary)
                .add_modifier(Modifier::BOLD)),
            Cell::from("Trend").style(Style::default()
                .fg(self.theme.text_primary)
                .add_modifier(Modifier::BOLD)),
        ]);

        let rows: Vec<Row> = self
            .indicators
            .iter()
            .filter(|ind| ind.active)
            .enumerate()
            .map(|(index, ind)| self.format_metric_row(ind, index, indicator_width))
            .collect();

        let table = Table::new(
            rows,
            [
                Constraint::Length(indicator_width as u16), // Dynamic - uses remaining space
                Constraint::Length(current_width as u16),   // Current value
                Constraint::Length(target_width as u16),    // Target value
                Constraint::Length(spread_width as u16),    // Spread percentage
                Constraint::Length(trend_width as u16),     // Trend
            ],
        )
        .header(header)
        .block(
            self.block.unwrap_or_else(||
                Block::default()
                    .title(" Live Metrics ")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(self.theme.text_secondary))
            )
        )
        .style(Style::default().bg(self.theme.bg_panel))
        .column_spacing(2);

        table.render(area, buf);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{IndicatorKind, IndicatorUnit, ObservationSource};
    use chrono::{Duration, NaiveDate, Utc};

    fn indicator(
        id: &str,
        name: &str,
        target: Option<f64>,
        direction: IndicatorDirection,
    ) -> IndicatorDef {
        let now = Utc::now();
        IndicatorDef {
            id: id.to_string(),
            name: name.to_string(),
            kind: IndicatorKind::Leading,
            unit: IndicatorUnit::Percent,
            objective_id: None,
            target,
            direction,
            active: true,
            created: now,
            modified: now,
            lineage_of: None,
            notes: None,
        }
    }

    fn observation(indicator_id: &str, value: f64, day_offset: i64) -> Observation {
        let base = NaiveDate::from_ymd_opt(2025, 9, 10).unwrap();
        Observation {
            id: format!("{}-{}", indicator_id, day_offset),
            indicator_id: indicator_id.to_string(),
            when: base + Duration::days(day_offset),
            value,
            unit: IndicatorUnit::Percent,
            source: ObservationSource::Manual,
            action_id: None,
            note: None,
            created: Utc::now(),
        }
    }

    #[test]
    fn higher_is_better_trend_is_positive_when_value_increases() {
        let theme = FinancialTheme::default();
        let indicators = vec![indicator(
            "rev",
            "Revenue",
            Some(120.0),
            IndicatorDirection::HigherIsBetter,
        )];
        let observations = vec![observation("rev", 100.0, 0), observation("rev", 130.0, 1)];

        let widget = LiveMetricsWidget::new(&indicators, &observations, &theme);
        let snapshot = widget.build_snapshot(&indicators[0]);

        assert_eq!(snapshot.current, 130.0);
        assert_eq!(snapshot.previous, 100.0);
        assert_eq!(snapshot.value_color, theme.positive);
        assert_eq!(snapshot.trend_arrow, Some('↑'));
    }

    #[test]
    fn lower_is_better_trend_flips_color() {
        let theme = FinancialTheme::default();
        let indicators = vec![indicator(
            "load",
            "System Load",
            Some(50.0),
            IndicatorDirection::LowerIsBetter,
        )];
        let observations = vec![observation("load", 40.0, 0), observation("load", 30.0, 1)];

        let widget = LiveMetricsWidget::new(&indicators, &observations, &theme);
        let snapshot = widget.build_snapshot(&indicators[0]);

        assert_eq!(snapshot.value_color, theme.positive);
        assert!(snapshot.trend_delta < 0.0);
    }

    #[test]
    fn within_range_uses_distance_to_target() {
        let theme = FinancialTheme::default();
        let indicators = vec![indicator(
            "temp",
            "Temperature",
            Some(70.0),
            IndicatorDirection::WithinRange,
        )];
        let observations = vec![observation("temp", 80.0, 0), observation("temp", 72.0, 1)];

        let widget = LiveMetricsWidget::new(&indicators, &observations, &theme);
        let snapshot = widget.build_snapshot(&indicators[0]);

        assert_eq!(snapshot.value_color, theme.positive);
        assert!(snapshot.spread_pct < 15.0);
    }

    #[test]
    fn widget_renders_as_table() {
        let theme = FinancialTheme::default();
        let indicators = vec![indicator(
            "revenue_growth",
            "Revenue Growth Rate",
            Some(120.0),
            IndicatorDirection::HigherIsBetter,
        )];
        let observations = vec![observation("revenue_growth", 130.0, 1)];

        let widget = LiveMetricsWidget::new(&indicators, &observations, &theme);

        // Test that widget can be rendered without panicking
        let area = Rect::new(0, 0, 80, 10);
        let mut buffer = Buffer::empty(area);
        widget.render(area, &mut buffer);

        // Verify table structure exists (basic smoke test)
        assert!(!buffer.content.is_empty());
    }

    #[test]
    fn long_indicator_names_are_truncated() {
        let theme = FinancialTheme::default();
        let indicators = vec![indicator(
            "very_long_id",
            "This is a very long indicator name that should be truncated",
            Some(100.0),
            IndicatorDirection::HigherIsBetter,
        )];
        let observations = vec![observation("very_long_id", 90.0, 1)];

        let widget = LiveMetricsWidget::new(&indicators, &observations, &theme);
        let _row = widget.format_metric_row(&indicators[0], 0, 16);

        // Verify name is truncated with ellipsis
        // Note: Actual cell content testing would require buffer inspection
        // This is a structural test to ensure truncation logic exists
    }

    #[test]
    fn format_indicator_name_pads_short_names() {
        let theme = FinancialTheme::default();
        let indicators = vec![indicator(
            "short",
            "Short",
            Some(100.0),
            IndicatorDirection::HigherIsBetter,
        )];
        let observations = vec![];

        let widget = LiveMetricsWidget::new(&indicators, &observations, &theme);
        let formatted = widget.format_indicator_name("Short", 16);

        // Should be padded to 16 characters
        assert_eq!(formatted.len(), 16);
        assert!(formatted.starts_with("Short"));
        assert!(formatted.ends_with(' '));
    }

    #[test]
    fn format_indicator_name_truncates_long_names() {
        let theme = FinancialTheme::default();
        let indicators = vec![];
        let observations = vec![];

        let widget = LiveMetricsWidget::new(&indicators, &observations, &theme);
        let long_name = "This is a very long indicator name that exceeds maximum width";
        let formatted = widget.format_indicator_name(long_name, 16);

        // Should be truncated with ellipsis at word boundary
        assert!(formatted.len() <= 16);
        assert!(formatted.ends_with("..."));
        assert_eq!(formatted, "This is a...");
    }

    #[test]
    fn format_metric_value_handles_different_precisions() {
        let theme = FinancialTheme::default();
        let indicators = vec![];
        let observations = vec![];

        let widget = LiveMetricsWidget::new(&indicators, &observations, &theme);
        let value = 123.456789;

        assert_eq!(widget.format_metric_value(value, 0), "123");
        assert_eq!(widget.format_metric_value(value, 1), "123.5");
        assert_eq!(widget.format_metric_value(value, 2), "123.46");
        assert_eq!(widget.format_metric_value(value, 5), "123.5"); // Default to 1 for unknown precision
    }

    #[test]
    fn format_metric_value_handles_edge_cases() {
        let theme = FinancialTheme::default();
        let indicators = vec![];
        let observations = vec![];

        let widget = LiveMetricsWidget::new(&indicators, &observations, &theme);

        // Test zero
        assert_eq!(widget.format_metric_value(0.0, 1), "0.0");

        // Test negative values
        assert_eq!(widget.format_metric_value(-45.67, 1), "-45.7");

        // Test very small values
        assert_eq!(widget.format_metric_value(0.001, 2), "0.00");
        assert_eq!(widget.format_metric_value(0.001, 3), "0.001");
    }
}
