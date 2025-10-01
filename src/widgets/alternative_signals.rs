use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Widget},
};

use crate::models::{IndicatorDef, IndicatorDirection, IndicatorKind, IndicatorUnit};
use crate::ui::theme::FinancialTheme;

#[derive(Debug, Clone)]
pub struct AlternativeSignal<'a> {
    pub indicator: &'a IndicatorDef,
    pub latest_value: f64,
    pub previous_value: Option<f64>,
    pub weight: f64,
}

pub struct AlternativeSignalsWidget<'a> {
    signals: Vec<AlternativeSignal<'a>>,
    theme: &'a FinancialTheme,
    selected: Option<usize>,
    title_color: Option<ratatui::style::Color>,
}

impl<'a> AlternativeSignalsWidget<'a> {
    pub fn new(
        signals: Vec<AlternativeSignal<'a>>,
        theme: &'a FinancialTheme,
        selected: Option<usize>,
    ) -> Self {
        Self {
            signals,
            theme,
            selected,
            title_color: None,
        }
    }

    pub fn title_color(mut self, color: ratatui::style::Color) -> Self {
        self.title_color = Some(color);
        self
    }

    fn format_signal_line(theme: &FinancialTheme, signal: &AlternativeSignal<'a>) -> ListItem<'a> {
        let indicator = signal.indicator;
        let strength = Self::compute_signal_strength(indicator, signal.latest_value);
        let color = Self::signal_color(theme, strength);
        let weight_text = format!("Wt {:>5.1}%", signal.weight.max(0.0));
        let value_text = Self::format_value(indicator, signal.latest_value);
        let target_text = indicator
            .target
            .map(|target| Self::format_value(indicator, target))
            .unwrap_or_else(|| "--".to_string());
        let delta_text = Self::directional_delta(indicator, signal.latest_value)
            .map(|delta| format!("Δ{:+.1}", delta))
            .unwrap_or_else(|| "Δ--".to_string());
        let label = match indicator.kind {
            IndicatorKind::Leading => "LEADING",
            IndicatorKind::Lagging => "LAGGING",
        };

        let mut spans = Vec::with_capacity(12);
        spans.push(Span::styled(
            format!("{:<20}", indicator.name),
            Style::default().fg(theme.text_primary),
        ));
        spans.push(Span::raw("  "));
        spans.push(Span::styled(
            weight_text,
            Style::default().fg(theme.text_secondary),
        ));
        spans.push(Span::raw("  "));
        spans.push(Span::styled(
            label,
            Style::default()
                .fg(match indicator.kind {
                    IndicatorKind::Leading => theme.info,
                    IndicatorKind::Lagging => theme.text_secondary,
                })
                .add_modifier(Modifier::ITALIC),
        ));
        spans.push(Span::raw("  "));
        spans.push(Span::styled(
            format!("Val {}", value_text),
            Style::default().fg(theme.text_primary),
        ));
        spans.push(Span::raw("  "));
        spans.push(Span::styled(
            format!("Target {}", target_text),
            Style::default().fg(theme.text_secondary),
        ));
        spans.push(Span::raw("  "));
        spans.push(Span::styled(
            delta_text,
            Style::default().fg(color).add_modifier(Modifier::BOLD),
        ));

        if let Some(previous) = signal.previous_value {
            let delta = signal.latest_value - previous;
            let arrow = if delta > 0.05 {
                '\u{2191}'
            } else if delta < -0.05 {
                '\u{2193}'
            } else {
                '\u{2192}'
            };
            let trend_color = theme.get_trend_color(signal.latest_value, previous);
            spans.push(Span::raw("  "));
            spans.push(Span::styled(
                format!("{} {:+.1}", arrow, delta),
                Style::default().fg(trend_color),
            ));
        }

        spans.push(Span::raw("  "));
        let bar = Self::create_signal_bar(strength);
        spans.push(Span::styled(bar, Style::default().fg(color)));
        spans.push(Span::raw(" "));
        spans.push(Span::styled(
            format!("{:>5.1}%", strength),
            Style::default().fg(color),
        ));

        ListItem::new(Line::from(spans))
    }

    fn compute_signal_strength(indicator: &IndicatorDef, value: f64) -> f64 {
        let target = indicator.target.unwrap_or(100.0);

        match indicator.direction {
            IndicatorDirection::HigherIsBetter => {
                if target.abs() <= f64::EPSILON {
                    if value >= 0.0 {
                        100.0
                    } else {
                        0.0
                    }
                } else {
                    (value / target * 100.0).clamp(0.0, 100.0)
                }
            }
            IndicatorDirection::LowerIsBetter => {
                if target.abs() <= f64::EPSILON {
                    if value.abs() <= f64::EPSILON {
                        100.0
                    } else {
                        0.0
                    }
                } else {
                    (target / value.max(f64::EPSILON) * 100.0).clamp(0.0, 100.0)
                }
            }
            IndicatorDirection::WithinRange => {
                let tolerance = (target.abs() * 0.2).max(1.0);
                let diff = (value - target).abs();
                let ratio = (diff / tolerance).min(1.0);
                (100.0 - ratio * 100.0).clamp(0.0, 100.0)
            }
        }
    }

    fn directional_delta(indicator: &IndicatorDef, value: f64) -> Option<f64> {
        indicator.target.map(|target| match indicator.direction {
            IndicatorDirection::HigherIsBetter => value - target,
            IndicatorDirection::LowerIsBetter => target - value,
            IndicatorDirection::WithinRange => target - value,
        })
    }

    fn signal_color(theme: &FinancialTheme, strength: f64) -> ratatui::style::Color {
        if strength >= 80.0 {
            theme.positive
        } else if strength >= 50.0 {
            theme.neutral
        } else {
            theme.negative
        }
    }

    fn create_signal_bar(percentage: f64) -> String {
        let clamped = percentage.clamp(0.0, 100.0);
        let filled = ((clamped / 10.0).round() as usize).min(10);
        let empty = 10 - filled;
        format!("{}{}", "\u{2588}".repeat(filled), "\u{2591}".repeat(empty))
    }

    fn format_value(indicator: &IndicatorDef, value: f64) -> String {
        match &indicator.unit {
            IndicatorUnit::Count => format!("{:.0}", value),
            IndicatorUnit::Minutes => format!("{:.0}m", value),
            IndicatorUnit::Dollars => format!("${:.0}", value),
            IndicatorUnit::Percent => format!("{:.1}%", value),
            IndicatorUnit::Custom(label) => format!("{:.1} {}", value, label),
        }
    }
}

impl<'a> Widget for AlternativeSignalsWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let AlternativeSignalsWidget {
            signals,
            theme,
            selected,
            title_color,
        } = self;

        let items: Vec<ListItem<'a>> = signals
            .iter()
            .enumerate()
            .map(|(idx, signal)| {
                let mut item = Self::format_signal_line(theme, signal);
                if Some(idx) == selected {
                    item = item.style(
                        Style::default()
                            .bg(theme.text_dim)
                            .fg(theme.text_primary)
                            .add_modifier(Modifier::BOLD),
                    );
                }
                item
            })
            .collect();

        let title_color = title_color.unwrap_or(theme.text_dim);
        let list = List::new(items)
            .block(
                Block::default()
                    .title(" ALTERNATIVE DATA SIGNALS ")
                    .title_style(
                        Style::default()
                            .fg(title_color)
                            .add_modifier(Modifier::BOLD),
                    )
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(theme.text_dim))
                    .style(Style::default().bg(theme.bg_panel)),
            )
            .style(Style::default().bg(theme.bg_panel));

        list.render(area, buf);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn build_indicator(
        direction: IndicatorDirection,
        target: Option<f64>,
        unit: IndicatorUnit,
    ) -> IndicatorDef {
        let mut indicator = IndicatorDef::new("Test".to_string(), IndicatorKind::Leading, unit);
        indicator.direction = direction;
        indicator.target = target;
        indicator
    }

    #[test]
    fn strength_caps_above_target_for_higher_is_better() {
        let indicator = build_indicator(
            IndicatorDirection::HigherIsBetter,
            Some(100.0),
            IndicatorUnit::Percent,
        );
        let strength = AlternativeSignalsWidget::compute_signal_strength(&indicator, 150.0);
        assert!((strength - 100.0).abs() < f64::EPSILON);
    }

    #[test]
    fn strength_improves_when_lower_value_is_better() {
        let indicator = build_indicator(
            IndicatorDirection::LowerIsBetter,
            Some(80.0),
            IndicatorUnit::Minutes,
        );
        let strength = AlternativeSignalsWidget::compute_signal_strength(&indicator, 40.0);
        assert!(strength > 80.0);
    }

    #[test]
    fn zero_target_lower_direction_handles_small_values() {
        let indicator = build_indicator(
            IndicatorDirection::LowerIsBetter,
            Some(0.0),
            IndicatorUnit::Percent,
        );

        let perfect = AlternativeSignalsWidget::compute_signal_strength(&indicator, 0.0);
        assert!((perfect - 100.0).abs() < f64::EPSILON);

        let miss = AlternativeSignalsWidget::compute_signal_strength(&indicator, 1.0);
        assert!((miss - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn signal_bar_respects_bounds() {
        let bar = AlternativeSignalsWidget::create_signal_bar(55.0);
        assert_eq!(
            bar,
            "\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2591}\u{2591}\u{2591}\u{2591}\u{2591}"
        );
    }
}
