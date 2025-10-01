use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Gauge, Paragraph, Widget},
};

use crate::models::{Action, ActionStatus, OutcomeType};
use crate::ui::theme::FinancialTheme;

#[derive(Debug, Default, Clone)]
struct SentimentBreakdown {
    done: usize,
    in_progress: usize,
    planned: usize,
    skipped: usize,
    blocked: usize,
}

impl SentimentBreakdown {
    fn from_actions(actions: &[Action]) -> Self {
        let mut breakdown = Self::default();

        for action in actions {
            match action.status {
                ActionStatus::Done => breakdown.done += 1,
                ActionStatus::InProgress => breakdown.in_progress += 1,
                ActionStatus::Planned => breakdown.planned += 1,
                ActionStatus::Skipped => breakdown.skipped += 1,
                ActionStatus::Blocked => breakdown.blocked += 1,
            }
        }

        breakdown
    }

    fn total(&self) -> usize {
        self.done + self.in_progress + self.planned + self.skipped + self.blocked
    }

    fn positive_count(&self) -> usize {
        self.done
    }

    fn active_count(&self) -> usize {
        self.in_progress + self.planned
    }

    fn risk_count(&self) -> usize {
        self.skipped + self.blocked
    }

    fn percentage(&self, count: usize) -> f64 {
        let total = self.total();
        if total == 0 {
            0.0
        } else {
            (count as f64 / total as f64) * 100.0
        }
    }

    fn positive_pct(&self) -> f64 {
        self.percentage(self.positive_count())
    }

    fn active_pct(&self) -> f64 {
        self.percentage(self.active_count())
    }

    fn risk_pct(&self) -> f64 {
        self.percentage(self.risk_count())
    }

    fn done_pct(&self) -> f64 {
        self.percentage(self.done)
    }

    fn in_progress_pct(&self) -> f64 {
        self.percentage(self.in_progress)
    }

    fn planned_pct(&self) -> f64 {
        self.percentage(self.planned)
    }

    fn skipped_pct(&self) -> f64 {
        self.percentage(self.skipped)
    }

    fn blocked_pct(&self) -> f64 {
        self.percentage(self.blocked)
    }

    fn momentum_score(&self) -> u16 {
        let total = self.total();
        if total == 0 {
            return 0;
        }

        let weighted = (self.done as f64 * 1.0)
            + (self.in_progress as f64 * 0.7)
            + (self.planned as f64 * 0.4)
            + (self.skipped as f64 * 0.15);

        ((weighted / total as f64) * 100.0)
            .round()
            .clamp(0.0, 100.0) as u16
    }
}

pub struct SentimentWidget<'a> {
    outcome: OutcomeType,
    actions: &'a [Action],
    theme: &'a FinancialTheme,
    title_color: Option<Color>,
}

impl<'a> SentimentWidget<'a> {
    pub fn new(outcome: OutcomeType, actions: &'a [Action], theme: &'a FinancialTheme) -> Self {
        Self {
            outcome,
            actions,
            theme,
            title_color: None,
        }
    }

    pub fn title_color(mut self, color: Color) -> Self {
        self.title_color = Some(color);
        self
    }

    fn bar_width(&self, available_width: u16) -> usize {
        if available_width <= 24 {
            return 0;
        }
        let dynamic = available_width.saturating_sub(24) as usize;
        dynamic.min(30)
    }

    fn render_bar(&self, percentage: f64, width: usize) -> String {
        if width == 0 {
            return String::new();
        }

        let clamped = percentage.clamp(0.0, 100.0);
        let filled = ((clamped / 100.0) * width as f64).round() as usize;
        let filled = filled.min(width);

        format!(
            "{}{}",
            "█".repeat(filled),
            "░".repeat(width.saturating_sub(filled))
        )
    }

    fn category_line(
        &self,
        label: &str,
        count: usize,
        percentage: f64,
        color: Color,
        bar_width: usize,
    ) -> Line<'static> {
        let mut spans = vec![
            Span::styled(
                format!("{:<10}", label),
                Style::default().fg(color).add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!("{:>3}", count),
                Style::default().fg(self.theme.text_primary),
            ),
            Span::raw("  "),
            Span::styled(format!("{:>5.1}%", percentage), Style::default().fg(color)),
        ];

        if bar_width > 0 {
            spans.push(Span::raw("  "));
            spans.push(Span::styled(
                self.render_bar(percentage, bar_width),
                Style::default().fg(color),
            ));
        }

        Line::from(spans)
    }

    fn detail_line(&self, breakdown: &SentimentBreakdown) -> Line<'static> {
        let statuses = [
            (
                "Done",
                breakdown.done,
                self.theme.positive,
                breakdown.done_pct(),
            ),
            (
                "InProg",
                breakdown.in_progress,
                self.theme.neutral,
                breakdown.in_progress_pct(),
            ),
            (
                "Plan",
                breakdown.planned,
                self.theme.neutral,
                breakdown.planned_pct(),
            ),
            (
                "Block",
                breakdown.blocked,
                self.theme.negative,
                breakdown.blocked_pct(),
            ),
            (
                "Skip",
                breakdown.skipped,
                self.theme.negative,
                breakdown.skipped_pct(),
            ),
        ];

        let mut spans = Vec::with_capacity(statuses.len() * 3);

        for (idx, (label, count, color, pct)) in statuses.into_iter().enumerate() {
            if idx > 0 {
                spans.push(Span::raw("   "));
            }

            spans.push(Span::styled(
                format!("{}", label),
                Style::default().fg(color).add_modifier(Modifier::BOLD),
            ));
            spans.push(Span::raw(" "));
            spans.push(Span::styled(
                format!("{} ({:.0}%)", count, pct.round()),
                Style::default().fg(self.theme.text_secondary),
            ));
        }

        Line::from(spans)
    }

    fn summary_lines(
        &self,
        breakdown: &SentimentBreakdown,
        bar_width: usize,
    ) -> Vec<Line<'static>> {
        let mut lines = Vec::new();

        lines.push(Line::from(vec![Span::styled(
            format!("Total Actions: {}", breakdown.total()),
            Style::default()
                .fg(self.theme.info)
                .add_modifier(Modifier::BOLD),
        )]));
        lines.push(Line::raw(""));

        lines.push(self.category_line(
            "Positive",
            breakdown.positive_count(),
            breakdown.positive_pct(),
            self.theme.positive,
            bar_width,
        ));
        lines.push(self.category_line(
            "Active",
            breakdown.active_count(),
            breakdown.active_pct(),
            self.theme.neutral,
            bar_width,
        ));
        lines.push(self.category_line(
            "At Risk",
            breakdown.risk_count(),
            breakdown.risk_pct(),
            self.theme.negative,
            bar_width,
        ));
        lines.push(Line::raw(""));
        lines.push(self.detail_line(breakdown));

        lines
    }

    fn render_momentum(&self, breakdown: &SentimentBreakdown, area: Rect, buf: &mut Buffer) {
        if area.area() == 0 {
            return;
        }

        let block = Block::default()
            .title(" MOMENTUM ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(self.theme.text_dim))
            .style(Style::default().bg(self.theme.bg_panel));
        let inner = block.inner(area);
        block.render(area, buf);

        if inner.area() == 0 {
            return;
        }

        let score = breakdown.momentum_score();
        let gauge_color = self.theme.get_status_color(score as f64);

        Gauge::default()
            .percent(score)
            .label(format!("{}%", score))
            .gauge_style(Style::default().fg(gauge_color).bg(self.theme.bg_panel))
            .style(Style::default().bg(self.theme.bg_panel).fg(gauge_color))
            .render(inner, buf);
    }
}

impl<'a> Widget for SentimentWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let title_color = self.title_color.unwrap_or(self.theme.text_dim);
        let block = Block::default()
            .title(format!(
                " {} SENTIMENT ",
                self.outcome.as_str().to_uppercase()
            ))
            .title_style(
                Style::default()
                    .fg(title_color)
                    .add_modifier(Modifier::BOLD),
            )
            .borders(Borders::ALL)
            .border_style(Style::default().fg(self.theme.text_dim))
            .style(Style::default().bg(self.theme.bg_panel));

        let inner = block.inner(area);
        block.render(area, buf);

        if inner.area() == 0 {
            return;
        }

        let breakdown = SentimentBreakdown::from_actions(self.actions);

        if breakdown.total() == 0 {
            Paragraph::new("No actions recorded for this outcome")
                .style(
                    Style::default()
                        .fg(self.theme.text_secondary)
                        .bg(self.theme.bg_panel),
                )
                .alignment(Alignment::Center)
                .render(inner, buf);
            return;
        }

        let bar_width = self.bar_width(inner.width);
        let lines = self.summary_lines(&breakdown, bar_width);

        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(5), Constraint::Length(3)])
            .split(inner);

        Paragraph::new(lines)
            .style(
                Style::default()
                    .fg(self.theme.text_primary)
                    .bg(self.theme.bg_panel),
            )
            .render(layout[0], buf);

        self.render_momentum(&breakdown, layout[1], buf);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn action_with_status(status: ActionStatus) -> Action {
        let mut action = Action::new("test".to_string());
        action.set_status(status);
        action
    }

    #[test]
    fn breakdown_counts_statuses_correctly() {
        let actions = vec![
            action_with_status(ActionStatus::Done),
            action_with_status(ActionStatus::InProgress),
            action_with_status(ActionStatus::InProgress),
            action_with_status(ActionStatus::Planned),
            action_with_status(ActionStatus::Skipped),
            action_with_status(ActionStatus::Blocked),
        ];

        let breakdown = SentimentBreakdown::from_actions(&actions);

        assert_eq!(breakdown.total(), 6);
        assert_eq!(breakdown.positive_count(), 1);
        assert_eq!(breakdown.active_count(), 3);
        assert_eq!(breakdown.risk_count(), 2);
        assert!((breakdown.positive_pct() - 16.6667).abs() < 0.01);
        assert!((breakdown.active_pct() - 50.0).abs() < 0.01);
        assert!((breakdown.risk_pct() - 33.3333).abs() < 0.01);
    }

    #[test]
    fn momentum_score_weights_statuses() {
        let actions = vec![
            action_with_status(ActionStatus::Done),
            action_with_status(ActionStatus::Done),
            action_with_status(ActionStatus::InProgress),
            action_with_status(ActionStatus::Planned),
            action_with_status(ActionStatus::Blocked),
        ];

        let breakdown = SentimentBreakdown::from_actions(&actions);

        // (2*1.0 + 1*0.7 + 1*0.4 + 0*0.15) / 5 * 100 = 62
        assert_eq!(breakdown.momentum_score(), 62);
    }

    #[test]
    fn momentum_score_handles_empty_actions() {
        let breakdown = SentimentBreakdown::from_actions(&[]);
        assert_eq!(breakdown.momentum_score(), 0);
    }
}
