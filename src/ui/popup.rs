use crate::ui::theme::FocusFiveTheme;
use crossterm::event::KeyCode;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::Style,
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

#[derive(PartialEq)]
pub enum EditorResult {
    Continue,
    Save,
    Cancel,
}

pub struct TextEditor {
    pub text: String,
    pub cursor_position: usize,
    pub max_length: usize,
    pub is_active: bool,
    pub title: String,
}

impl TextEditor {
    pub fn new(default_title: &str) -> Self {
        Self {
            text: String::new(),
            cursor_position: 0,
            max_length: 500,
            is_active: false,
            title: default_title.to_string(),
        }
    }

    pub fn activate(&mut self, text: &str) {
        let current_title = self.title.clone();
        let current_max = self.max_length;
        self.activate_with(&current_title, text, current_max);
    }

    pub fn activate_with(&mut self, title: &str, text: &str, max_length: usize) {
        self.title = title.to_string();
        self.text = text.to_string();
        self.cursor_position = text.len();
        self.max_length = max_length;
        self.is_active = true;
    }

    pub fn deactivate(&mut self) {
        self.is_active = false;
    }

    pub fn handle_input(&mut self, key: KeyCode) -> EditorResult {
        match key {
            KeyCode::Esc => return EditorResult::Cancel,
            KeyCode::Enter => return EditorResult::Save,
            KeyCode::Backspace => self.delete_char(),
            KeyCode::Left => self.move_cursor_left(),
            KeyCode::Right => self.move_cursor_right(),
            KeyCode::Char(c) => self.insert_char(c),
            _ => {}
        }
        EditorResult::Continue
    }

    fn insert_char(&mut self, c: char) {
        if self.text.len() < self.max_length {
            self.text.insert(self.cursor_position, c);
            self.cursor_position += 1;
        }
    }

    fn delete_char(&mut self) {
        if self.cursor_position > 0 {
            self.cursor_position -= 1;
            self.text.remove(self.cursor_position);
        }
    }

    fn move_cursor_left(&mut self) {
        if self.cursor_position > 0 {
            self.cursor_position -= 1;
        }
    }

    fn move_cursor_right(&mut self) {
        if self.cursor_position < self.text.len() {
            self.cursor_position += 1;
        }
    }

    pub fn render(&self, f: &mut Frame, theme: &FocusFiveTheme) {
        let area = centered_rect(60, 20, f.area());

        // Clear background
        f.render_widget(Clear, area);

        // Create text with cursor indicator
        let mut display_text = self.text.clone();
        if self.cursor_position <= display_text.len() {
            display_text.insert(self.cursor_position, '│');
        }

        // Character count display
        let char_count = format!("{}/{}", self.text.len(), self.max_length);
        let char_color = if self.text.len() > self.max_length - 50 {
            theme.partial
        } else {
            theme.text_secondary
        };

        // Create the popup content
        let content = vec![
            Line::from(""),
            Line::from(display_text),
            Line::from(""),
            Line::from(vec![
                Span::raw("Characters: "),
                Span::styled(char_count, Style::default().fg(char_color)),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("[Enter]", Style::default().fg(theme.header)),
                Span::raw(" Save  "),
                Span::styled("[Esc]", Style::default().fg(theme.header)),
                Span::raw(" Cancel"),
            ]),
        ];

        // Render popup
        let popup = Paragraph::new(content)
            .block(
                Block::default()
                    .title(format!(" {} ", self.title))
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(theme.header))
                    .style(Style::default().bg(theme.panel_bg)),
            )
            .style(Style::default().fg(theme.text_primary))
            .alignment(Alignment::Left)
            .wrap(ratatui::widgets::Wrap { trim: false });

        f.render_widget(popup, area);
    }
}

pub fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
