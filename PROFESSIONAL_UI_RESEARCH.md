# Research: Professional TUI Implementation for FocusFive

**Date**: 2025-09-16T14:09:35-0400
**Topic**: Incorporating professional financial terminal UI styling into FocusFive

## Research Question
How to make the FocusFive UI look more professional by analyzing and incorporating styling from a financial terminal TUI application

## Summary
The FocusFive project currently has no UI implementation (Phase 1 complete, data layer only). Based on analysis of the provided financial terminal screenshot and research into ratatui capabilities, I've developed a comprehensive implementation plan to create a professional TUI with dark theme, multi-panel layout, color-coded data, and sophisticated typography that matches the aesthetic of professional trading terminals.

## Reference Image Analysis

The provided screenshot shows a sophisticated financial terminal with these key design elements:

### Visual Design Patterns
1. **Color Scheme**
   - Dark background (#0F0F0F - very dark gray/black)
   - Panel backgrounds (#191919 - slightly lighter)
   - Border colors (#3C3C3C - medium gray)
   - Accent colors: Yellow/gold for headers, green for positive, orange for neutral, red for negative

2. **Layout Structure**
   - Multiple distinct panels with clear borders
   - Section headers in uppercase yellow/gold
   - Two-column data presentation (labels left, values right)
   - Horizontal bar charts for visual data representation
   - Line graphs for trend visualization

3. **Typography**
   - Monospace font throughout
   - Mixed case for data, uppercase for headers
   - Right-aligned numerical values
   - Color coding for different data types
   - Clear visual hierarchy

4. **Information Architecture**
   - Header with title and metadata (date, time, volume, status)
   - Main data sections with clear groupings
   - Visual indicators (bars, percentages)
   - Performance metrics at bottom

## Current Implementation Status

### Phase 1 Complete (Data Layer Only)
- `src/main.rs:1-19` - Simple CLI that creates sample data and exits
- `src/models.rs` - Core data structures (DailyGoals, Outcome, Action)
- `src/data.rs` - File I/O and markdown parsing
- **No UI code exists** - Greenfield implementation opportunity

### Dependencies Needed
Current (`Cargo.toml:9-14`):
```toml
anyhow = "1.0"
chrono = "0.4"
regex = "1.0"
serde = { version = "1.0", features = ["derive"] }
```

Required additions for TUI:
```toml
ratatui = "0.28"
crossterm = "0.28"
```

## Implementation Plan

### 1. Professional Color Theme System

```rust
// src/ui/theme.rs
pub struct FocusFiveTheme {
    // Dark background colors
    pub background: Color,      // #0F0F0F
    pub panel_bg: Color,        // #191919
    pub border: Color,          // #3C3C3C

    // Text colors
    pub text_primary: Color,    // #DCDCDC (light gray)
    pub text_secondary: Color,  // #A0A0A0 (medium gray)
    pub header: Color,          // #FFC800 (golden yellow)

    // Status colors
    pub completed: Color,       // #00FF80 (bright green)
    pub pending: Color,         // #FF5050 (bright red)
    pub partial: Color,         // #FFA500 (orange)

    // Outcome-specific colors
    pub work_color: Color,      // #64B5F6 (light blue)
    pub health_color: Color,    // #81C784 (light green)
    pub family_color: Color,    // #FFB74D (light orange)
}
```

### 2. Professional Layout Structure

```rust
// src/ui/layout.rs
pub fn create_professional_layout(area: Rect) -> AppLayout {
    AppLayout {
        header: Layout::default()
            .constraints([Constraint::Length(3)])
            .split(area)[0],

        main: Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(30),  // Outcomes panel
                Constraint::Percentage(40),  // Actions panel
                Constraint::Percentage(30),  // Stats panel
            ])
            .split(area)[1..4],

        footer: Layout::default()
            .constraints([Constraint::Length(2)])
            .split(area)[4],
    }
}
```

### 3. Professional Widget Styling

```rust
// src/ui/widgets.rs
pub fn create_panel_block(title: &str, theme: &FocusFiveTheme) -> Block {
    Block::default()
        .title(format!(" {} ", title.to_uppercase()))
        .title_style(Style::default()
            .fg(theme.header)
            .add_modifier(Modifier::BOLD))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(theme.border))
        .style(Style::default().bg(theme.panel_bg))
}

pub fn create_outcome_item(outcome: &Outcome, theme: &FocusFiveTheme) -> ListItem {
    let completed = outcome.actions.iter().filter(|a| a.completed).count();
    let percentage = (completed as f32 / 3.0 * 100.0) as u8;

    let color = match percentage {
        100 => theme.completed,
        0 => theme.pending,
        _ => theme.partial,
    };

    let content = Line::from(vec![
        Span::styled(
            format!("{:<10}", outcome.outcome_type),
            Style::default().fg(theme.text_primary)
        ),
        Span::styled(
            format!("{:>3}%", percentage),
            Style::default().fg(color).add_modifier(Modifier::BOLD)
        ),
        Span::raw(" "),
        Span::styled(
            create_progress_bar(percentage, 10),
            Style::default().fg(color)
        ),
    ]);

    ListItem::new(content)
}
```

### 4. Data Visualization Components

```rust
// src/ui/charts.rs
pub fn create_completion_chart(goals_history: &[DailyGoals], theme: &FocusFiveTheme) -> BarChart {
    let data: Vec<Bar> = goals_history.iter()
        .map(|g| {
            let completed = calculate_completion_percentage(g);
            Bar::default()
                .value(completed)
                .label(g.date.format("%m/%d").to_string().into())
                .style(Style::default().fg(
                    if completed >= 80 { theme.completed }
                    else if completed >= 40 { theme.partial }
                    else { theme.pending }
                ))
        })
        .collect();

    BarChart::default()
        .block(create_panel_block("WEEKLY PROGRESS", theme))
        .data(&data)
        .bar_width(3)
        .bar_gap(1)
        .value_style(Style::default().fg(theme.text_secondary))
}
```

### 5. Status Bar Implementation

```rust
// src/ui/status.rs
pub fn render_status_bar(f: &mut Frame, area: Rect, app: &App, theme: &FocusFiveTheme) {
    let status_items = vec![
        Span::styled(
            format!(" {} ", app.current_date.format("%B %d, %Y")),
            Style::default().fg(theme.text_primary)
        ),
        Span::raw(" • "),
        Span::styled(
            format!("Day {}", app.day_number.unwrap_or(1)),
            Style::default().fg(theme.header)
        ),
        Span::raw(" • "),
        Span::styled(
            format!("{}% Complete", calculate_daily_progress(app)),
            Style::default().fg(theme.completed)
        ),
        Span::raw(" • "),
        Span::styled(
            if app.has_unsaved_changes { "UNSAVED" } else { "SAVED" },
            Style::default().fg(
                if app.has_unsaved_changes { theme.pending } else { theme.completed }
            )
        ),
    ];

    let status_line = Paragraph::new(Line::from(status_items))
        .style(Style::default().bg(theme.panel_bg))
        .alignment(Alignment::Center);

    f.render_widget(status_line, area);
}
```

### 6. Main Application Structure

```rust
// src/ui/app.rs
pub struct App {
    pub goals: DailyGoals,
    pub current_date: NaiveDate,
    pub selected_outcome: OutcomeType,
    pub selected_action: usize,
    pub theme: FocusFiveTheme,
    pub has_unsaved_changes: bool,
    pub mode: AppMode,
}

pub fn render_app(f: &mut Frame, app: &App) {
    let layout = create_professional_layout(f.size());

    // Header with title and date
    render_header(f, layout.header, app);

    // Outcomes panel (left)
    render_outcomes_panel(f, layout.main[0], app);

    // Actions panel (center)
    render_actions_panel(f, layout.main[1], app);

    // Statistics panel (right)
    render_stats_panel(f, layout.main[2], app);

    // Status bar (bottom)
    render_status_bar(f, layout.footer, app);
}
```

## File Structure for UI Implementation

```
src/
├── main.rs         # Modified to initialize TUI
├── models.rs       # Existing (unchanged)
├── data.rs         # Existing (unchanged)
├── lib.rs          # Existing (unchanged)
└── ui/             # New directory
    ├── mod.rs      # UI module exports
    ├── app.rs      # Application state and rendering
    ├── theme.rs    # Color theme and styling
    ├── layout.rs   # Layout management
    ├── widgets.rs  # Custom widget implementations
    ├── charts.rs   # Data visualization components
    ├── status.rs   # Status bar and indicators
    └── input.rs    # Keyboard input handling
```

## Key Implementation Steps

1. **Add Dependencies**
   ```bash
   cargo add ratatui crossterm
   ```

2. **Create UI Module Structure**
   - Implement theme system matching financial terminal aesthetic
   - Build layout engine for multi-panel interface
   - Create custom widgets with professional styling

3. **Implement Core UI Loop**
   ```rust
   // src/main.rs (modified)
   fn main() -> Result<()> {
       let mut terminal = init_terminal()?;
       let app = App::new()?;

       let result = run_app(&mut terminal, app);

       restore_terminal(&mut terminal)?;
       result
   }
   ```

4. **Add Keyboard Navigation**
   - Tab: Switch between outcomes
   - j/k: Navigate actions
   - Space: Toggle completion
   - q: Quit
   - s: Force save

5. **Implement Visual Feedback**
   - Color-coded completion status
   - Progress bars for each outcome
   - Unsaved changes indicator
   - Current selection highlighting

## Architecture Insights

The implementation leverages:
- **Ratatui's styling system** for professional color schemes
- **Block widgets** with borders for panel separation
- **List widgets** for outcome/action display
- **BarChart/Sparkline** for progress visualization
- **Paragraph widgets** for status information
- **Crossterm** for cross-platform terminal manipulation

## Performance Considerations

- Render only on state changes (not continuous)
- Cache layout calculations
- Minimize file I/O (batch saves)
- Use immediate mode rendering (ratatui default)

## Validation Approach

1. Visual testing with different terminal sizes
2. Color contrast verification
3. Keyboard navigation testing
4. File save/load integrity checks
5. Cross-platform compatibility testing

## Next Steps

1. Create `src/ui/` directory structure
2. Implement theme system first
3. Build layout engine
4. Add basic rendering
5. Implement keyboard handling
6. Add data visualization
7. Polish with animations and transitions

This implementation will transform FocusFive from a simple CLI tool into a professional-looking terminal application that rivals commercial financial terminals in visual sophistication while maintaining the simplicity of the 3x3 goal structure.