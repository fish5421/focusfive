use crate::models::Observation;
use crate::ui::theme::FinancialTheme;
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    symbols,
    text::Span,
    widgets::{Axis, Block, Borders, Chart, Dataset, GraphType, Widget},
};

pub struct PerformanceChart<'a> {
    observations: &'a [Observation],
    indicator_id: &'a str,
    theme: &'a FinancialTheme,
    title: &'a str,
    title_color: Option<Color>,
}

impl<'a> PerformanceChart<'a> {
    pub fn new(
        observations: &'a [Observation],
        indicator_id: &'a str,
        theme: &'a FinancialTheme,
        title: &'a str,
    ) -> Self {
        Self {
            observations,
            indicator_id,
            theme,
            title,
            title_color: None,
        }
    }

    pub fn title_color(mut self, color: Color) -> Self {
        self.title_color = Some(color);
        self
    }

    fn filtered_observations(&self) -> Vec<&'a Observation> {
        let mut filtered: Vec<&'a Observation> = self
            .observations
            .iter()
            .filter(|obs| obs.indicator_id == self.indicator_id)
            .collect();
        filtered.sort_by_key(|obs| obs.when);
        filtered
    }

    fn prepare_dataset(&self, filtered: &[&'a Observation]) -> Vec<(f64, f64)> {
        if filtered.is_empty() {
            return vec![(0.0, 0.0), (1.0, 0.0)];
        }

        let mut points: Vec<(f64, f64)> = filtered
            .iter()
            .enumerate()
            .map(|(idx, obs)| (idx as f64, obs.value))
            .collect();

        if points.len() == 1 {
            points.push((1.0, points[0].1));
        }

        points
    }

    fn compute_y_bounds(&self, data: &[(f64, f64)]) -> (f64, f64) {
        if data.is_empty() {
            return (0.0, 1.0);
        }

        let mut y_min = f64::INFINITY;
        let mut y_max = f64::NEG_INFINITY;
        for (_, value) in data {
            if *value < y_min {
                y_min = *value;
            }
            if *value > y_max {
                y_max = *value;
            }
        }

        if !y_min.is_finite() || !y_max.is_finite() {
            return (0.0, 1.0);
        }

        if (y_max - y_min).abs() < f64::EPSILON {
            let spread = (y_max.abs() * 0.05).max(1.0);
            (y_min - spread, y_max + spread)
        } else {
            let padding = (y_max - y_min) * 0.05;
            (y_min - padding, y_max + padding)
        }
    }

    fn compute_x_bounds(&self, len: usize) -> [f64; 2] {
        if len <= 1 {
            [0.0, 1.0]
        } else {
            [0.0, (len - 1) as f64]
        }
    }

    fn x_axis_labels(&self, filtered: &[&'a Observation]) -> Vec<Span<'static>> {
        if filtered.is_empty() {
            return vec![Span::raw("NO DATA")];
        }

        let format_date = |obs: &&Observation| obs.when.format("%b %d").to_string();

        if filtered.len() == 1 {
            let label = format_date(&filtered[0]);
            return vec![Span::raw(label.clone()), Span::raw(label)];
        }

        let first = format_date(&filtered[0]);
        let mid = format_date(&filtered[filtered.len() / 2]);
        let last = format_date(&filtered[filtered.len() - 1]);

        let mut labels = vec![first, mid, last];
        labels.dedup();
        if labels.len() == 1 {
            labels.push(labels[0].clone());
        }

        labels.into_iter().map(Span::raw).collect()
    }

    fn y_axis_labels(&self, min: f64, max: f64) -> Vec<Span<'static>> {
        let mid = (min + max) / 2.0;
        vec![min, mid, max]
            .into_iter()
            .map(|value| Span::raw(format!("{value:.1}")))
            .collect()
    }

    fn trend_color(&self, filtered: &[&'a Observation]) -> Color {
        if filtered.len() < 2 {
            return self.theme.neutral;
        }

        let first = filtered.first().unwrap().value;
        let last = filtered.last().unwrap().value;
        self.theme.get_trend_color(last, first)
    }
}

impl<'a> Widget for PerformanceChart<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let filtered = self.filtered_observations();
        let data = self.prepare_dataset(&filtered);
        let bounds = self.compute_y_bounds(&data);
        let x_bounds = self.compute_x_bounds(data.len());
        let trend_color = self.trend_color(&filtered);
        let x_labels = self.x_axis_labels(&filtered);
        let y_labels = self.y_axis_labels(bounds.0, bounds.1);

        let dataset = Dataset::default()
            .name(self.title)
            .marker(symbols::Marker::Braille)
            .graph_type(GraphType::Line)
            .style(Style::default().fg(trend_color))
            .data(&data);

        let title_color = self.title_color.unwrap_or(self.theme.text_dim);
        let chart = Chart::new(vec![dataset])
            .block(
                Block::default()
                    .title(format!(
                        " {} PERFORMANCE (7-DAY) ",
                        self.title.to_uppercase()
                    ))
                    .title_style(
                        Style::default()
                            .fg(title_color)
                            .add_modifier(ratatui::style::Modifier::BOLD),
                    )
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(self.theme.text_dim))
                    .style(Style::default().bg(self.theme.bg_panel)),
            )
            .x_axis(
                Axis::default()
                    .style(Style::default().fg(self.theme.text_dim))
                    .bounds(x_bounds)
                    .labels(x_labels),
            )
            .y_axis(
                Axis::default()
                    .style(Style::default().fg(self.theme.text_dim))
                    .bounds([bounds.0, bounds.1])
                    .labels(y_labels),
            );

        chart.render(area, buf);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{IndicatorUnit, ObservationSource};
    use chrono::{Duration, NaiveDate, Utc};

    fn observation(id: &str, offset: i64, value: f64) -> Observation {
        Observation {
            id: format!("{id}-{offset}"),
            indicator_id: id.to_string(),
            when: NaiveDate::from_ymd_opt(2025, 9, 1).unwrap() + Duration::days(offset),
            value,
            unit: IndicatorUnit::Percent,
            source: ObservationSource::Manual,
            action_id: None,
            note: None,
            created: Utc::now(),
        }
    }

    #[test]
    fn dataset_is_sorted_and_extends_single_point() {
        let theme = FinancialTheme::default();
        let observations = vec![
            observation("ind", 2, 40.0),
            observation("ind", 0, 20.0),
            observation("ind", 1, 30.0),
        ];
        let chart = PerformanceChart::new(&observations, "ind", &theme, "Indicator");

        let filtered = chart.filtered_observations();
        assert_eq!(filtered.len(), 3);
        let data = chart.prepare_dataset(&filtered);

        assert_eq!(data.len(), 3);
        assert_eq!(data[0], (0.0, 20.0));
        assert_eq!(data[1], (1.0, 30.0));
        assert_eq!(data[2], (2.0, 40.0));
    }

    #[test]
    fn dataset_handles_empty_and_single_point() {
        let theme = FinancialTheme::default();
        let observations: Vec<Observation> = Vec::new();
        let chart = PerformanceChart::new(&observations, "ind", &theme, "Indicator");

        let filtered = chart.filtered_observations();
        assert!(filtered.is_empty());
        let data = chart.prepare_dataset(&filtered);
        assert_eq!(data, vec![(0.0, 0.0), (1.0, 0.0)]);

        let single = vec![observation("ind", 0, 55.0)];
        let chart = PerformanceChart::new(&single, "ind", &theme, "Indicator");
        let filtered = chart.filtered_observations();
        assert_eq!(filtered.len(), 1);
        let data = chart.prepare_dataset(&filtered);
        assert_eq!(data, vec![(0.0, 55.0), (1.0, 55.0)]);
    }

    #[test]
    fn y_bounds_expand_for_flat_series() {
        let theme = FinancialTheme::default();
        let observations = vec![observation("ind", 0, 75.0), observation("ind", 1, 75.0)];
        let chart = PerformanceChart::new(&observations, "ind", &theme, "Indicator");
        let filtered = chart.filtered_observations();
        let data = chart.prepare_dataset(&filtered);
        let (min, max) = chart.compute_y_bounds(&data);

        assert!(max > min);
        assert!(min < 75.0);
        assert!(max > 75.0);
    }

    #[test]
    fn trend_color_reflects_direction() {
        let theme = FinancialTheme::default();
        let observations = vec![observation("ind", 0, 10.0), observation("ind", 1, 15.0)];
        let chart = PerformanceChart::new(&observations, "ind", &theme, "Indicator");
        let filtered = chart.filtered_observations();
        assert_eq!(chart.trend_color(&filtered), theme.positive);

        let declining = vec![observation("ind", 0, 15.0), observation("ind", 1, 10.0)];
        let chart = PerformanceChart::new(&declining, "ind", &theme, "Indicator");
        let filtered = chart.filtered_observations();
        assert_eq!(chart.trend_color(&filtered), theme.negative);
    }
}
