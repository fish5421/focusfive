use super::popup::centered_rect;
use super::theme::FocusFiveTheme;
use crate::models::{Indicator, IndicatorType};
use crate::widgets::{IndicatorProgress, TrendDirection};
use ratatui::{
    prelude::*,
    symbols::border,
    widgets::{Block, Borders, Clear, Gauge, Paragraph, Sparkline},
};

pub struct IndicatorDetailPopup {
    indicator: Indicator,
    progress: IndicatorProgress,
    is_updating: bool,
    update_buffer: String,
}

impl IndicatorDetailPopup {
    pub fn new(indicator: Indicator) -> Self {
        let history: Vec<f64> = indicator.history.iter().map(|entry| entry.value).collect();

        let progress =
            IndicatorProgress::new(indicator.current_value, indicator.target_value, history);

        Self {
            indicator,
            progress,
            is_updating: false,
            update_buffer: String::new(),
        }
    }

    pub fn render(&self, f: &mut Frame, theme: &FocusFiveTheme) {
        let area = centered_rect(70, 65, f.area());
        f.render_widget(Clear, area);

        let border_set = border::Set {
            top_left: "/",
            top_right: "\\",
            bottom_left: "\\",
            bottom_right: "/",
            vertical_left: "|",
            vertical_right: "|",
            horizontal_top: "-",
            horizontal_bottom: "-",
        };

        let shell = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.header))
            .border_set(border_set)
            .style(Style::default().bg(theme.panel_bg));
        f.render_widget(shell, area);

        if area.width < 6 || area.height < 10 {
            return;
        }

        let inner = Rect {
            x: area.x + 1,
            y: area.y + 1,
            width: area.width.saturating_sub(2),
            height: area.height.saturating_sub(2),
        };

        let header_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Min(6),
            ])
            .split(inner);

        let title_area = header_layout[0];
        let metrics_area = header_layout[1];
        let body_area = header_layout[2];

        let title_line = Line::from(vec![Span::styled(
            " FocusFive · Update Indicator ",
            Style::default().fg(theme.header),
        )]);

        let title = Paragraph::new(title_line)
            .alignment(Alignment::Left)
            .style(Style::default().bg(theme.panel_bg));
        f.render_widget(title, title_area);

        self.render_metrics_band(f, metrics_area, theme);

        let body_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Min(6),
                Constraint::Length(2),
                Constraint::Length(3),
                Constraint::Length(1),
            ])
            .split(body_area);

        if body_chunks.len() < 6 {
            return;
        }

        let summary_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
            .split(body_chunks[0]);

        self.render_current_summary(f, summary_chunks[0], theme);
        self.render_direction_summary(f, summary_chunks[1], theme);

        self.render_progress_row(f, body_chunks[1], theme);
        self.render_history(f, body_chunks[2], theme);

        let quick_actions_text = format!("Quick Adjust: {}", self.quick_actions_text());
        let quick_actions = Paragraph::new(quick_actions_text)
            .alignment(Alignment::Center)
            .style(Style::default().fg(theme.text_secondary).bg(theme.panel_bg));
        f.render_widget(quick_actions, body_chunks[3]);

        let input_display = if self.is_updating {
            self.update_buffer.clone()
        } else {
            self.format_value_for_input()
        };

        let input = Paragraph::new(input_display)
            .alignment(Alignment::Center)
            .style(Style::default().fg(theme.text_primary))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Input Value ")
                    .border_style(Style::default().fg(theme.border))
                    .style(Style::default().bg(theme.background)),
            );
        f.render_widget(input, body_chunks[4]);

        let footer = Paragraph::new(Line::from(vec![
            Span::styled("Enter", Style::default().fg(theme.header)),
            Span::raw(" Apply  "),
            Span::styled("Esc", Style::default().fg(theme.header)),
            Span::raw(" Close"),
        ]))
        .alignment(Alignment::Center)
        .style(Style::default().fg(theme.text_secondary).bg(theme.panel_bg));
        f.render_widget(footer, body_chunks[5]);
    }

    fn render_metrics_band(&self, f: &mut Frame, area: Rect, theme: &FocusFiveTheme) {
        if area.width < 3 {
            return;
        }

        let previous_display = self
            .indicator
            .history
            .last()
            .map(|entry| self.format_value_with_unit(entry.value))
            .unwrap_or_else(|| "—".to_string());

        let metrics_text = format!(
            " Target {} | Latest {} | Previous {} ",
            self.format_value_with_unit(self.indicator.target_value),
            self.format_value_with_unit(self.indicator.current_value),
            previous_display,
        );

        let inner_width = area.width as usize;
        let available = inner_width.saturating_sub(2);
        let clamped = self.clamp_text(&metrics_text, available);
        let padding = available.saturating_sub(clamped.chars().count());
        let left_pad = padding / 2;
        let right_pad = padding.saturating_sub(left_pad);

        let mut spans = Vec::with_capacity(5);
        spans.push(Span::styled("\\", Style::default().fg(theme.header)));
        if left_pad > 0 {
            spans.push(Span::styled(
                "-".repeat(left_pad),
                Style::default().fg(theme.border),
            ));
        }
        spans.push(Span::styled(
            clamped,
            Style::default().fg(theme.text_secondary),
        ));
        if right_pad > 0 {
            spans.push(Span::styled(
                "-".repeat(right_pad),
                Style::default().fg(theme.border),
            ));
        }
        spans.push(Span::styled("/", Style::default().fg(theme.header)));

        let paragraph =
            Paragraph::new(Line::from(spans)).style(Style::default().bg(theme.panel_bg));
        f.render_widget(paragraph, area);
    }

    fn render_current_summary(&self, f: &mut Frame, area: Rect, theme: &FocusFiveTheme) {
        if area.width < 3 || area.height < 3 {
            return;
        }

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.border))
            .style(Style::default().bg(theme.background));
        f.render_widget(block, area);

        let inner = Rect {
            x: area.x + 1,
            y: area.y + 1,
            width: area.width.saturating_sub(2),
            height: area.height.saturating_sub(2),
        };

        let mut spans = vec![
            Span::styled("Current ", Style::default().fg(theme.text_secondary)),
            Span::styled(
                self.format_value_with_unit(self.indicator.current_value),
                Style::default().fg(theme.text_primary),
            ),
        ];

        if let Some(delta) = self.format_delta_label() {
            spans.push(Span::raw(" "));
            spans.push(Span::styled(delta, Style::default().fg(theme.partial)));
        }

        let paragraph = Paragraph::new(Line::from(spans))
            .alignment(Alignment::Center)
            .style(Style::default().bg(theme.background));
        f.render_widget(paragraph, inner);
    }

    fn render_direction_summary(&self, f: &mut Frame, area: Rect, theme: &FocusFiveTheme) {
        if area.width < 3 || area.height < 3 {
            return;
        }

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.border))
            .style(Style::default().bg(theme.background));
        f.render_widget(block, area);

        let inner = Rect {
            x: area.x + 1,
            y: area.y + 1,
            width: area.width.saturating_sub(2),
            height: area.height.saturating_sub(2),
        };

        let direction_line = Line::from(vec![
            Span::styled("Direction ", Style::default().fg(theme.text_secondary)),
            Span::styled(
                self.direction_label(),
                Style::default().fg(theme.text_primary),
            ),
        ]);

        let trend_label = match self.progress.trend {
            TrendDirection::Up => "Improving",
            TrendDirection::Down => "Declining",
            TrendDirection::Stable => "Stable",
        };

        let trend_line = Line::from(vec![
            Span::styled(
                self.progress.render_trend(),
                Style::default().fg(self.trend_color(theme)),
            ),
            Span::raw(" "),
            Span::styled(trend_label, Style::default().fg(theme.text_secondary)),
        ]);

        let paragraph = Paragraph::new(vec![direction_line, trend_line])
            .alignment(Alignment::Center)
            .style(Style::default().bg(theme.background));
        f.render_widget(paragraph, inner);
    }

    fn render_progress_row(&self, f: &mut Frame, area: Rect, theme: &FocusFiveTheme) {
        if area.width < 4 {
            return;
        }

        let row_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(72), Constraint::Percentage(28)])
            .split(area);

        let percentage = self.progress.get_percentage();
        let gauge_color = match percentage {
            100.. => theme.completed,
            70..=99 => theme.partial,
            _ => theme.pending,
        };

        let gauge = Gauge::default()
            .percent(percentage)
            .label(format!("{}%", percentage))
            .gauge_style(Style::default().fg(gauge_color).bg(theme.background))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Goal Pace ")
                    .border_style(Style::default().fg(theme.border))
                    .style(Style::default().bg(theme.background)),
            );
        f.render_widget(gauge, row_chunks[0]);

        let trend = Paragraph::new(Line::from(vec![
            Span::styled(
                self.progress.render_trend(),
                Style::default().fg(self.trend_color(theme)),
            ),
            Span::raw(" "),
            Span::styled(
                match self.progress.trend {
                    TrendDirection::Up => "Improving",
                    TrendDirection::Down => "Declining",
                    TrendDirection::Stable => "Stable",
                },
                Style::default().fg(theme.text_secondary),
            ),
        ]))
        .alignment(Alignment::Center)
        .style(Style::default().bg(theme.panel_bg));
        f.render_widget(trend, row_chunks[1]);
    }

    fn render_history(&self, f: &mut Frame, area: Rect, theme: &FocusFiveTheme) {
        if area.width < 4 || area.height < 4 {
            return;
        }

        let block = Block::default()
            .borders(Borders::ALL)
            .title(" 7-Day History ")
            .border_style(Style::default().fg(theme.border))
            .style(Style::default().bg(theme.background));
        f.render_widget(block, area);

        let inner = Rect {
            x: area.x + 1,
            y: area.y + 1,
            width: area.width.saturating_sub(2),
            height: area.height.saturating_sub(2),
        };

        if inner.height == 0 {
            return;
        }

        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(1), Constraint::Length(1)])
            .split(inner);

        let mut values: Vec<f64> = self
            .indicator
            .history
            .iter()
            .map(|entry| entry.value)
            .collect();
        values.push(self.indicator.current_value);

        let window_start = values.len().saturating_sub(7);
        let window = &values[window_start..];
        let sparkline_data: Vec<u64> = window.iter().map(|value| (value * 100.0) as u64).collect();

        if !sparkline_data.is_empty() {
            let sparkline = Sparkline::default()
                .data(&sparkline_data)
                .style(Style::default().fg(theme.partial));
            f.render_widget(sparkline, layout[0]);
        } else {
            let placeholder = Paragraph::new("No history yet")
                .alignment(Alignment::Center)
                .style(
                    Style::default()
                        .fg(theme.text_secondary)
                        .bg(theme.background),
                );
            f.render_widget(placeholder, layout[0]);
        }

        let start_value = window
            .first()
            .copied()
            .unwrap_or(self.indicator.current_value);
        let end_value = window
            .last()
            .copied()
            .unwrap_or(self.indicator.current_value);
        let last_updated = self
            .indicator
            .history
            .last()
            .map(|entry| entry.timestamp.format("%Y-%m-%d").to_string())
            .unwrap_or_else(|| "—".to_string());

        let footer = format!(
            "Start {}   End {}   Last update {}",
            self.format_value_with_unit(start_value),
            self.format_value_with_unit(end_value),
            last_updated,
        );

        let footer_paragraph = Paragraph::new(footer).alignment(Alignment::Center).style(
            Style::default()
                .fg(theme.text_secondary)
                .bg(theme.background),
        );
        f.render_widget(footer_paragraph, layout[1]);
    }

    fn quick_actions_text(&self) -> String {
        match self.indicator.indicator_type {
            IndicatorType::Counter => "+/- fine   a +1   s +3   d +5   c clear".to_string(),
            IndicatorType::Duration => "+/- fine   a +0.5h   s +1h   d +2h   c reset".to_string(),
            IndicatorType::Percentage => {
                "+/- fine   a 25%   s 50%   d 75%   f 100%   c clear".to_string()
            }
            IndicatorType::Boolean => "y complete   n incomplete".to_string(),
        }
    }

    fn format_value_for_input(&self) -> String {
        match self.indicator.indicator_type {
            IndicatorType::Counter => format!("{:.0}", self.indicator.current_value),
            IndicatorType::Duration => format!("{:.2}", self.indicator.current_value),
            IndicatorType::Percentage => format!("{:.0}", self.indicator.current_value),
            IndicatorType::Boolean => {
                if self.indicator.current_value >= 1.0 {
                    "yes".to_string()
                } else {
                    "no".to_string()
                }
            }
        }
    }

    fn format_value_with_unit(&self, value: f64) -> String {
        match self.indicator.indicator_type {
            IndicatorType::Counter => format!("{:.0} {}", value, self.indicator.unit),
            IndicatorType::Duration => format!("{:.1} hrs", value),
            IndicatorType::Percentage => format!("{:.0}%", value),
            IndicatorType::Boolean => {
                if value >= 1.0 {
                    "Complete".to_string()
                } else {
                    "Incomplete".to_string()
                }
            }
        }
    }

    fn format_delta_label(&self) -> Option<String> {
        match self.indicator.indicator_type {
            IndicatorType::Boolean => None,
            IndicatorType::Counter => {
                let diff = self.indicator.current_value - self.indicator.target_value;
                if diff.abs() < f64::EPSILON {
                    Some("on target".to_string())
                } else if diff > 0.0 {
                    Some(format!("(+{} ahead)", diff.round() as i32))
                } else {
                    Some(format!("(-{} behind)", diff.abs().round() as i32))
                }
            }
            IndicatorType::Duration => {
                let diff = self.indicator.current_value - self.indicator.target_value;
                if diff.abs() < 0.05 {
                    Some("on target".to_string())
                } else if diff > 0.0 {
                    Some(format!("(+{:.1} hrs ahead)", diff))
                } else {
                    Some(format!("(-{:.1} hrs behind)", diff.abs()))
                }
            }
            IndicatorType::Percentage => {
                let diff = self.indicator.current_value - self.indicator.target_value;
                if diff.abs() < 0.5 {
                    Some("on target".to_string())
                } else if diff > 0.0 {
                    Some(format!("(+{:.0}% ahead)", diff))
                } else {
                    Some(format!("(-{:.0}% behind)", diff.abs()))
                }
            }
        }
    }

    fn trend_color(&self, theme: &FocusFiveTheme) -> Color {
        match self.progress.trend {
            TrendDirection::Up => theme.completed,
            TrendDirection::Down => theme.pending,
            TrendDirection::Stable => theme.partial,
        }
    }

    fn direction_label(&self) -> &'static str {
        "Higher is better"
    }

    fn clamp_text(&self, text: &str, width: usize) -> String {
        if width == 0 {
            return String::new();
        }

        let total_chars = text.chars().count();
        if total_chars <= width {
            return text.to_string();
        }

        if width == 1 {
            return "…".to_string();
        }

        let truncated: String = text.chars().take(width - 1).collect();
        format!("{}…", truncated)
    }

    pub fn render_update_dialog(&self, f: &mut Frame, theme: &FocusFiveTheme) {
        let area = centered_rect(55, 45, f.area());

        f.render_widget(Clear, area);

        let border_set = border::Set {
            top_left: "/",
            top_right: "\\",
            bottom_left: "\\",
            bottom_right: "/",
            vertical_left: "|",
            vertical_right: "|",
            horizontal_top: "-",
            horizontal_bottom: "-",
        };

        let dialog_block = Block::default()
            .borders(Borders::ALL)
            .border_set(border_set)
            .border_style(Style::default().fg(theme.header))
            .style(Style::default().bg(theme.panel_bg));
        f.render_widget(dialog_block, area);

        if area.width < 6 || area.height < 8 {
            return;
        }

        let inner = Rect {
            x: area.x + 1,
            y: area.y + 1,
            width: area.width.saturating_sub(2),
            height: area.height.saturating_sub(2),
        };

        let header_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Min(5),
            ])
            .split(inner);

        let title_line = Line::from(vec![Span::styled(
            format!(" Update · {} ", self.indicator.name),
            Style::default().fg(theme.header),
        )]);
        let title = Paragraph::new(title_line)
            .alignment(Alignment::Left)
            .style(Style::default().bg(theme.panel_bg));
        f.render_widget(title, header_layout[0]);

        self.render_metrics_band(f, header_layout[1], theme);

        let body_area = header_layout[2];
        let body_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(2),
                Constraint::Length(3),
                Constraint::Length(2),
                Constraint::Length(3),
                Constraint::Length(1),
            ])
            .split(body_area);

        let value_display = format!(
            "Current {} | Target {}",
            self.format_indicator_value(),
            self.format_target_value()
        );
        let values = Paragraph::new(value_display)
            .alignment(Alignment::Center)
            .style(Style::default().fg(theme.text_primary).bg(theme.panel_bg));
        f.render_widget(values, body_chunks[0]);

        let gauge_percentage = self.progress.get_percentage();
        let gauge_color = match gauge_percentage {
            100.. => theme.completed,
            70..=99 => theme.partial,
            _ => theme.pending,
        };
        let gauge = Gauge::default()
            .percent(gauge_percentage)
            .label(format!("{}%", gauge_percentage))
            .gauge_style(Style::default().fg(gauge_color).bg(theme.background))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Goal Pace ")
                    .border_style(Style::default().fg(theme.border))
                    .style(Style::default().bg(theme.background)),
            );
        f.render_widget(gauge, body_chunks[1]);

        let quick_actions = Paragraph::new(self.quick_actions_text())
            .alignment(Alignment::Center)
            .style(Style::default().fg(theme.text_secondary).bg(theme.panel_bg));
        f.render_widget(quick_actions, body_chunks[2]);

        let input_value = if self.update_buffer.is_empty() {
            self.format_value_for_input()
        } else {
            self.update_buffer.clone()
        };
        let input = Paragraph::new(input_value)
            .alignment(Alignment::Center)
            .style(Style::default().fg(theme.text_primary))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Enter value ")
                    .border_style(Style::default().fg(theme.header))
                    .style(Style::default().bg(theme.background)),
            );
        f.render_widget(input, body_chunks[3]);

        let help = Paragraph::new(self.get_help_text())
            .alignment(Alignment::Center)
            .style(Style::default().fg(theme.text_secondary).bg(theme.panel_bg));
        f.render_widget(help, body_chunks[4]);
    }

    fn format_indicator_value(&self) -> String {
        self.format_value_with_unit(self.indicator.current_value)
    }

    fn format_target_value(&self) -> String {
        self.format_value_with_unit(self.indicator.target_value)
    }

    fn get_help_text(&self) -> &str {
        match self.indicator.indicator_type {
            IndicatorType::Counter => {
                "Type amount   Enter Save   Esc Cancel   a/s/d quick add   c clear"
            }
            IndicatorType::Duration => {
                "Type hours   Enter Save   Esc Cancel   a/s/d quick add   c reset"
            }
            IndicatorType::Percentage => "Type 0-100   Enter Save   Esc Cancel   a/s/d/f shortcuts",
            IndicatorType::Boolean => "Press Y/N   Enter Save   Esc Cancel",
        }
    }

    pub fn start_update(&mut self) {
        self.is_updating = true;
        self.update_buffer = self.format_value_for_input();
    }

    pub fn cancel_update(&mut self) {
        self.is_updating = false;
        self.update_buffer.clear();
    }

    pub fn apply_update(&mut self) -> anyhow::Result<()> {
        let new_value = match self.indicator.indicator_type {
            IndicatorType::Counter | IndicatorType::Duration => {
                self.update_buffer
                    .parse::<f64>()
                    .map_err(|e| anyhow::anyhow!("Invalid number: {}", e))?
            }
            IndicatorType::Percentage => {
                let pct = self
                    .update_buffer
                    .trim_end_matches('%')
                    .parse::<f64>()
                    .map_err(|e| anyhow::anyhow!("Invalid percentage: {}", e))?;
                pct.min(100.0).max(0.0)
            }
            IndicatorType::Boolean => {
                if self.update_buffer.to_lowercase().starts_with('y')
                    || self.update_buffer == "1"
                    || self.update_buffer.to_lowercase() == "true"
                {
                    1.0
                } else {
                    0.0
                }
            }
        };

        // Store in history
        self.indicator.history.push(crate::models::IndicatorEntry {
            timestamp: chrono::Utc::now(),
            value: self.indicator.current_value,
            note: None,
        });

        // Update value
        self.indicator.current_value = new_value;

        // Update progress
        let history: Vec<f64> = self
            .indicator
            .history
            .iter()
            .map(|entry| entry.value)
            .collect();
        self.progress = IndicatorProgress::new(
            self.indicator.current_value,
            self.indicator.target_value,
            history,
        );

        self.is_updating = false;
        self.update_buffer.clear();

        Ok(())
    }

    pub fn handle_quick_action(&mut self, key: char) -> anyhow::Result<()> {
        let key = key.to_ascii_lowercase();
        let new_value = match self.indicator.indicator_type {
            IndicatorType::Counter => match key {
                '+' => self.indicator.current_value + 1.0,
                '-' => (self.indicator.current_value - 1.0).max(0.0),
                'a' => self.indicator.current_value + 1.0,
                's' => self.indicator.current_value + 3.0,
                'd' => self.indicator.current_value + 5.0,
                'c' => 0.0,
                _ => return Ok(()),
            },
            IndicatorType::Duration => match key {
                '+' => self.indicator.current_value + 0.25,
                '-' => (self.indicator.current_value - 0.25).max(0.0),
                'a' => self.indicator.current_value + 0.5,
                's' => self.indicator.current_value + 1.0,
                'd' => self.indicator.current_value + 2.0,
                'c' => 0.0,
                _ => return Ok(()),
            },
            IndicatorType::Percentage => match key {
                '+' => (self.indicator.current_value + 5.0).min(100.0),
                '-' => (self.indicator.current_value - 5.0).max(0.0),
                'a' => 25.0,
                's' => 50.0,
                'd' => 75.0,
                'f' => 100.0,
                'c' => 0.0,
                _ => return Ok(()),
            },
            IndicatorType::Boolean => match key {
                'y' => 1.0,
                'n' => 0.0,
                _ => return Ok(()),
            },
        };

        // Store in history
        self.indicator.history.push(crate::models::IndicatorEntry {
            timestamp: chrono::Utc::now(),
            value: self.indicator.current_value,
            note: None,
        });

        // Update value
        self.indicator.current_value = new_value;

        // Update progress
        let history: Vec<f64> = self
            .indicator
            .history
            .iter()
            .map(|entry| entry.value)
            .collect();
        self.progress = IndicatorProgress::new(
            self.indicator.current_value,
            self.indicator.target_value,
            history,
        );

        Ok(())
    }
}
