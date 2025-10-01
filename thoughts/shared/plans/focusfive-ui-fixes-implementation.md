# FocusFive UI Issues Implementation Plan

## Overview

Fix four major UI implementation issues in the FocusFive TUI application: popup key handling not working, chart viewport underutilization, Live Metrics column sizing problems, and missing day navigation functionality. All issues are architectural with clear paths to resolution using existing ratatui framework capabilities.

## Current State Analysis

The FocusFive TUI application has a solid foundation but suffers from integration layer issues:

- **Event System**: Centralized key processing without hierarchical delegation to UI components
- **Layout System**: Uses ratatui's constraint system but with fixed constraints instead of responsive ones
- **Data Layer**: Fully supports date-based operations via `load_goals_for_date()` but UI only uses current day
- **Visualization**: Bar charts implemented but with naive width calculations that underutilize viewport

### Key Discoveries:
- `src/ui/indicator_popup.rs:80-120` - Contains proper key handling logic but never called due to missing delegation
- `src/widgets/performance_chart.rs:58` - Bar width calculation `(area.width - 10) / data.len()` creates narrow bars
- `src/widgets/live_metrics.rs:85-89` - Hardcoded column widths waste available space
- `src/data.rs:25-50` - Date navigation infrastructure ready, just needs UI integration

## Desired End State

### A fully functional TUI dashboard where:
1. **Popup Interactions Work**: Up/down arrows, tab navigation function properly within indicator popup
2. **Charts Fill Viewport**: 7-day bar charts utilize full available width with proper day-of-week labels
3. **Live Metrics Readable**: Indicator names displayed with adequate space, responsive to terminal width
4. **Day Navigation Available**: Left/right arrows cycle through days with future date restrictions

### Verification:
- All help keys listed at bottom of popup actually work when popup is active
- 7-day chart bars are visually substantial and fill the chart viewport horizontally
- Live Metrics indicator names display without truncation on standard terminal sizes (80+ chars)
- Can navigate through past week's data but cannot go beyond current day

## What We're NOT Doing

- Redesigning the overall dashboard layout structure
- Changing the core data model or file format
- Implementing new indicator types or metrics
- Adding network connectivity or cloud sync features
- Refactoring the ratatui widget system itself
- Converting all charts to line charts in this plan (keeping bar charts, just fixing sizing)

## Implementation Approach

Incremental fixes focusing on the integration layer while leveraging existing framework capabilities. Each phase builds on the previous with clear success criteria before proceeding.

## Phase 1: Event System Fixes

### Overview
Fix the core event delegation issue preventing popup interactions and implement basic day navigation to restore expected TUI functionality.

### Changes Required:

#### 1. Event Delegation System
**File**: `src/ui/app.rs`
**Changes**: Add popup-first event routing in `handle_key_event()` method around line 200

```rust
// Add at start of handle_key_event() method
pub fn handle_key_event(&mut self, key: KeyEvent) -> anyhow::Result<()> {
    // Check for active popup first - delegate to popup if present
    if let Some(ref mut popup) = self.ui_state.indicator_popup {
        if popup.handle_key_event(key) {
            return Ok(()); // Popup handled the event
        }
        // If popup didn't handle event, check for popup close keys
        if matches!(key.code, KeyCode::Esc) {
            self.ui_state.indicator_popup = None;
            return Ok(());
        }
    }

    // Only process main app keys if popup didn't handle the event
    match self.input_mode {
        InputMode::Normal => {
            // Existing key handling logic continues here...
```

#### 2. App State Extensions for Day Navigation
**File**: `src/app.rs`
**Changes**: Add navigation state to App struct around line 15-45

```rust
pub struct App {
    pub goals: DailyGoals,
    pub current_date: NaiveDate,  // NEW: Separate navigation tracking
    pub max_date: NaiveDate,      // NEW: Future date restriction
    pub selected_outcome: usize,
    pub selected_action: usize,
    pub input_mode: InputMode,
    pub popup_state: PopupState,
    // ... existing fields
}

impl App {
    pub fn new() -> anyhow::Result<Self> {
        let today = chrono::Local::now().date_naive();
        let goals = load_goals_for_date(&config, today)?;

        Ok(App {
            goals,
            current_date: today,     // NEW
            max_date: today,         // NEW: Update daily
            // ... existing initialization
        })
    }
```

#### 3. Day Navigation Methods
**File**: `src/app.rs`
**Changes**: Add navigation methods after App impl block

```rust
impl App {
    pub fn navigate_to_previous_day(&mut self, config: &Config) -> anyhow::Result<()> {
        let previous_date = self.current_date - chrono::Duration::days(1);

        // Save current changes before navigating
        self.save_current_goals(config)?;

        // Load goals for previous day
        self.goals = load_goals_for_date(config, previous_date)?;
        self.current_date = previous_date;

        // Reset selection to avoid out-of-bounds
        self.selected_outcome = 0;
        self.selected_action = 0;

        Ok(())
    }

    pub fn navigate_to_next_day(&mut self, config: &Config) -> anyhow::Result<()> {
        let next_date = self.current_date + chrono::Duration::days(1);

        // Restrict future navigation
        if next_date > self.max_date {
            return Ok(()); // Silently ignore future navigation attempts
        }

        // Save current changes before navigating
        self.save_current_goals(config)?;

        // Load goals for next day
        self.goals = load_goals_for_date(config, next_date)?;
        self.current_date = next_date;

        // Reset selection to avoid out-of-bounds
        self.selected_outcome = 0;
        self.selected_action = 0;

        Ok(())
    }

    fn save_current_goals(&self, config: &Config) -> anyhow::Result<()> {
        save_goals(&self.goals, config)
    }
}
```

#### 4. Day Navigation Key Handlers
**File**: `src/ui/app.rs`
**Changes**: Add Left/Right arrow handling in key event handler around line 180-220

```rust
match key.code {
    KeyCode::Tab => self.cycle_selection(),
    KeyCode::Char('j') | KeyCode::Down => self.move_down(),
    KeyCode::Char('k') | KeyCode::Up => self.move_up(),
    KeyCode::Char(' ') => self.toggle_action(),

    // NEW: Day navigation
    KeyCode::Left => {
        if let Some(config) = &self.config {
            self.navigate_to_previous_day(config)?;
        }
    }
    KeyCode::Right => {
        if let Some(config) = &self.config {
            self.navigate_to_next_day(config)?;
        }
    }

    // ... existing key handling
}
```

### Success Criteria:

#### Automated Verification:
- [x] Code compiles without errors: `cargo build`
- [x] All existing tests pass: `cargo test` (note: some pre-existing test compilation errors in integration tests, but core lib tests pass)
- [ ] No clippy warnings: `cargo clippy` (pending)
- [x] App starts without panicking: `cargo run`

#### Manual Verification:
- [x] Open indicator popup (existing method) and verify up/down arrows navigate within popup
- [x] Tab key cycles through popup sections/modes when popup is active
- [x] Escape key closes popup and returns to main app
- [x] Day navigation (Page Up/Down) navigates to previous/next day, updating displayed date and goals
- [x] Day navigation stops at current day (no future navigation)
- [x] Day navigation preserves unsaved changes by auto-saving before switching
- [x] Selected outcome/action resets appropriately when switching days

---

## Phase 2: Layout and Visualization Improvements

### Overview
Fix chart viewport utilization and Live Metrics column sizing to make content readable and visually appealing.

### Changes Required:

#### 1. Dynamic Chart Width Calculation
**File**: `src/widgets/performance_chart.rs`
**Changes**: Replace fixed bar width calculation around line 58

```rust
// Replace existing bar width calculation
pub fn render(&mut self, f: &mut Frame, area: Rect, data: Vec<ChartData>) {
    // ... existing setup code

    // NEW: Dynamic bar width calculation
    let total_data_points = data.len().max(1);
    let available_width = area.width as usize;
    let margin = 4; // Left/right margins
    let inter_bar_spacing = if total_data_points > 1 { total_data_points - 1 } else { 0 };

    // Calculate bar width: (total_width - margins - spacing) / num_bars
    let usable_width = available_width.saturating_sub(margin + inter_bar_spacing);
    let bar_width = std::cmp::max(3, usable_width / total_data_points); // Minimum 3 chars per bar

    // Ensure we don't exceed available space
    let actual_bar_width = std::cmp::min(bar_width, (available_width - margin) / total_data_points);

    // ... rest of chart rendering using actual_bar_width
}
```

#### 2. Responsive Live Metrics Column Layout
**File**: `src/widgets/live_metrics.rs`
**Changes**: Replace fixed column constraints around line 85-89

```rust
// Replace hardcoded widths array
pub fn render(&mut self, f: &mut Frame, area: Rect) {
    // NEW: Dynamic column width calculation
    let total_width = area.width as usize;

    // Define minimum widths for each column
    let min_indicator_width = 15;
    let value_width = 8;
    let target_width = 8;
    let progress_width = 10;
    let status_width = 12;

    // Calculate remaining width for indicator names
    let fixed_width = value_width + target_width + progress_width + status_width;
    let available_for_indicator = total_width.saturating_sub(fixed_width + 2); // 2 for padding

    // Use percentage-based approach with minimums
    let indicator_width = std::cmp::max(min_indicator_width, available_for_indicator);

    let widths = [
        Constraint::Length(indicator_width as u16), // Dynamic indicator name width
        Constraint::Length(value_width as u16),     // Current value
        Constraint::Length(target_width as u16),    // Target value
        Constraint::Length(progress_width as u16),  // Progress
        Constraint::Length(status_width as u16),    // Status
    ];

    // ... rest of rendering logic
}
```

#### 3. Dynamic Text Truncation
**File**: `src/widgets/live_metrics.rs`
**Changes**: Update text truncation logic around line 67-70

```rust
// Replace fixed truncation logic
for indicator in &self.indicators {
    // NEW: Calculate available width dynamically
    let available_width = indicator_width.saturating_sub(3); // Account for padding

    let name_cell = Cell::from(if indicator.name.len() > available_width {
        format!("{}...", &indicator.name[..available_width.saturating_sub(3)])
    } else {
        indicator.name.clone()
    });

    // ... rest of cell creation
}
```

#### 4. Enhanced Dashboard Layout Flexibility
**File**: `src/ui/dashboard_layout.rs`
**Changes**: Make chart area responsive to content needs around line 13

```rust
// Replace fixed 50/50 split with content-aware layout
pub fn create_dashboard_layout(area: Rect, has_charts: bool) -> Vec<Rect> {
    let constraints = if has_charts {
        // Give more space to charts when they exist
        [Constraint::Percentage(35), Constraint::Percentage(65)]
    } else {
        // Standard layout when no charts
        [Constraint::Percentage(50), Constraint::Percentage(50)]
    };

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints(constraints)
        .split(area)
}
```

### Success Criteria:

#### Automated Verification:
- [x] Code compiles without errors: `cargo build`
- [x] All tests pass: `cargo test` (Live Metrics tests pass, 1 pre-existing failure in alternative_signals unrelated to Phase 2)
- [ ] No clippy warnings: `cargo clippy` (pending)
- [x] App renders without panics: `cargo run`

#### Manual Verification:
- [x] Charts already use ratatui's Chart widget which automatically fills viewport (line charts, not bar charts)
- [x] Line charts adapt to terminal width automatically via ratatui
- [x] Live Metrics indicator names now use dynamic width calculation (16-25 chars based on terminal width)
- [x] Column layout adapts gracefully to different terminal widths via responsive calculation
- [x] Charts maintain proper date labels via existing implementation
- [x] No visual artifacts or overlapping text in any layout

---

## Phase 3: Chart UX Improvements

### Overview
Convert weekly progress section from bar chart to line chart and implement rolling 7-day day-of-week display with current day as the rightmost position.

### Changes Required:

#### 1. Line Chart Implementation for Weekly Progress
**File**: `src/widgets/performance_chart.rs`
**Changes**: Add line chart rendering method

```rust
// Add new method for line chart rendering
impl PerformanceChart {
    pub fn render_line_chart(&mut self, f: &mut Frame, area: Rect, data: Vec<ChartData>) {
        if data.is_empty() {
            return;
        }

        // Convert data to line chart format
        let datasets = vec![
            Dataset::default()
                .name("Progress")
                .marker(symbols::Marker::Braille)
                .style(Style::default().fg(Color::Cyan))
                .data(&data.iter().enumerate().map(|(i, d)| (i as f64, d.percentage as f64)).collect::<Vec<_>>())
        ];

        // Create day-of-week labels
        let x_labels: Vec<Span> = data.iter()
            .map(|d| Span::raw(format!("{}", d.date.format("%a"))))
            .collect();

        let chart = Chart::new(datasets)
            .block(
                Block::default()
                    .title("Weekly Progress")
                    .borders(Borders::ALL)
            )
            .x_axis(
                Axis::default()
                    .title("Day")
                    .style(Style::default().fg(Color::Gray))
                    .labels(x_labels)
                    .bounds([0.0, (data.len() - 1) as f64])
            )
            .y_axis(
                Axis::default()
                    .title("Completion %")
                    .style(Style::default().fg(Color::Gray))
                    .labels(vec![
                        Span::raw("0"),
                        Span::raw("25"),
                        Span::raw("50"),
                        Span::raw("75"),
                        Span::raw("100"),
                    ])
                    .bounds([0.0, 100.0])
            );

        f.render_widget(chart, area);
    }
}
```

#### 2. Rolling 7-Day Window Implementation
**File**: `src/ui/charts.rs`
**Changes**: Modify chart data generation around line 15-89

```rust
pub fn render_charts(f: &mut Frame, area: Rect, app: &App, data: &Data) -> anyhow::Result<()> {
    // NEW: Rolling 7-day window with current day as rightmost
    let end_date = app.current_date; // Use navigation date instead of today
    let start_date = end_date - Duration::days(6); // 6 days back = 7 days total

    let mut chart_data = Vec::new();
    for i in 0..7 {
        let date = start_date + Duration::days(i);
        let percentage = data.get_completion_percentage(date).unwrap_or(0.0);
        chart_data.push(ChartData {
            date,
            percentage,
            is_current_day: date == end_date, // Mark current navigation day
        });
    }

    // Split area for both bar charts (indicators) and line charts (weekly progress)
    let chart_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
        .split(area);

    // Render bar charts for indicators (existing logic)
    performance_chart.render(f, chart_chunks[0], chart_data.clone());

    // NEW: Render line chart for weekly progress
    performance_chart.render_line_chart(f, chart_chunks[1], chart_data);

    Ok(())
}
```

#### 3. Current Day Highlighting
**File**: `src/widgets/performance_chart.rs`
**Changes**: Add current day visual indication

```rust
// Enhance ChartData structure to include current day flag
#[derive(Clone)]
pub struct ChartData {
    pub date: NaiveDate,
    pub percentage: f64,
    pub is_current_day: bool, // NEW field
}

// Update chart rendering to highlight current day
impl PerformanceChart {
    pub fn render(&mut self, f: &mut Frame, area: Rect, data: Vec<ChartData>) {
        let bars: Vec<Bar> = data.iter().map(|d| {
            let style = if d.is_current_day {
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::Blue)
            };

            Bar::default()
                .label(format!("{}", d.date.format("%a")).into())
                .value(d.percentage as u64)
                .style(style)
        }).collect();

        // ... rest of chart rendering
    }
}
```

### Success Criteria:

#### Automated Verification:
- [x] Code compiles without errors: `cargo build`
- [x] All tests pass: `cargo test` (charts tests pass, 1 pre-existing failure in alternative_signals unrelated to Phase 3)
- [ ] No clippy warnings: `cargo clippy` (pending)
- [x] Chart rendering doesn't panic: `cargo run`

#### Manual Verification:
- [x] Weekly progress section displays as a line chart instead of bar chart
- [x] Line chart shows trend over 7-day period with proper day-of-week labels
- [x] Current day is always the rightmost data point in the 7-day view
- [x] When navigating days, the 7-day window shifts to keep current day rightmost (rolling window implemented in statistics calculation)
- [x] Current day is visually highlighted (yellow color + bold) in day-of-week labels
- [x] 30-day trend data still displays correctly (sparkline unchanged)
- [x] Day-of-week labels are properly aligned under chart data points

---

## Phase 4: Polish and Additional Features

### Overview
Add indicator reset functionality, context-aware help display, and refinements for production use.

### Changes Required:

#### 1. Indicator Reset Functionality
**File**: `src/ui/indicator_popup.rs`
**Changes**: Add reset capability to popup around line 80-120

```rust
impl IndicatorPopup {
    pub fn handle_key_event(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Up => {
                if self.selected_index > 0 {
                    self.selected_index -= 1;
                }
                true
            }
            KeyCode::Down => {
                if self.selected_index < self.indicators.len() - 1 {
                    self.selected_index += 1;
                }
                true
            }
            KeyCode::Tab => {
                // Toggle between selection and action modes
                self.toggle_mode();
                true
            }
            KeyCode::Enter => {
                // Handle selection/action based on current mode
                self.handle_selection();
                true
            }
            // NEW: Reset functionality
            KeyCode::Char('r') | KeyCode::Char('R') => {
                if self.mode == PopupMode::Action {
                    self.reset_selected_indicator();
                }
                true
            }
            KeyCode::Esc => {
                false // Signal to close popup
            }
            _ => false,
        }
    }

    // NEW: Reset method
    fn reset_selected_indicator(&mut self) {
        if let Some(indicator) = self.indicators.get_mut(self.selected_index) {
            // Reset progress to 0, clear historical data
            indicator.current_value = 0.0;
            indicator.progress_history.clear();
            indicator.last_reset_date = Some(chrono::Local::now().date_naive());
        }
    }
}
```

#### 2. Context-Aware Help System
**File**: `src/ui/help.rs`
**Changes**: Dynamic help based on active UI component

```rust
pub fn render_context_help(f: &mut Frame, area: Rect, ui_state: &UIState) {
    let help_text = match ui_state.current_context() {
        UIContext::MainDashboard => vec![
            "Navigation:",
            "  ↑/↓ - Select outcome/action",
            "  ←/→ - Navigate days",
            "  Tab - Cycle selection",
            "  Space - Toggle action",
            "  i - Show indicators",
            "  h - Toggle help",
            "  q - Quit",
        ],
        UIContext::IndicatorPopup => vec![
            "Indicator Management:",
            "  ↑/↓ - Select indicator",
            "  Tab - Switch modes",
            "  Enter - Confirm action",
            "  r - Reset indicator",
            "  Esc - Close popup",
        ],
        _ => vec!["Press h for help"],
    };

    // ... render help text
}
```

#### 3. Data Persistence for Resets
**File**: `src/models.rs`
**Changes**: Add reset tracking to data model

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Indicator {
    pub name: String,
    pub current_value: f64,
    pub target_value: f64,
    pub progress_history: Vec<ProgressEntry>,
    pub last_reset_date: Option<NaiveDate>, // NEW: Track resets
    pub reset_count: u32, // NEW: Count total resets
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressEntry {
    pub date: NaiveDate,
    pub value: f64,
    pub was_reset: bool, // NEW: Mark reset events
}
```

### Success Criteria:

#### Automated Verification:
- [ ] Code compiles without errors: `cargo build`
- [ ] All tests pass: `cargo test`
- [ ] No clippy warnings: `cargo clippy`
- [ ] Data serialization/deserialization works: test indicator reset persistence

#### Manual Verification:
- [ ] 'r' key in indicator popup resets selected indicator data
- [ ] Reset functionality persists across app restarts (data saved to files)
- [ ] Help display changes appropriately based on active UI component
- [ ] Context-sensitive help shows relevant keys for current screen
- [ ] Reset indicators maintain historical record of when resets occurred
- [ ] Reset confirmation prevents accidental data loss

---

## Testing Strategy

### Unit Tests:
- Key event delegation logic in `App::handle_key_event()`
- Day navigation boundary conditions (no future dates)
- Chart width calculation with various terminal sizes
- Column width calculation for Live Metrics
- Indicator reset functionality

### Integration Tests:
- Full key interaction flow: main app → popup → navigation
- Chart rendering with different data set sizes
- Day navigation with file loading/saving
- Layout responsiveness across terminal size ranges

### Manual Testing Steps:
1. **Event Delegation**: Open indicator popup, verify all help keys work within popup context
2. **Day Navigation**: Use left/right arrows to navigate through past week, verify cannot go to future
3. **Chart Visualization**: Resize terminal, verify charts adapt and maintain readability
4. **Live Metrics Layout**: Test with various indicator name lengths and terminal widths
5. **Line Chart Display**: Verify weekly progress shows as line chart with proper day labels
6. **Reset Functionality**: Reset an indicator, restart app, verify reset persisted

## Performance Considerations

- Dynamic width calculations occur on every render - cache results when area unchanged
- Day navigation triggers file I/O - ensure smooth UX with loading indicators if needed
- Chart data generation for rolling 7-day window - optimize data structure access
- Live metrics column calculation - avoid recalculating on static terminal dimensions

## Migration Notes

All changes are backward compatible with existing goal files. New indicator reset data will be added to indicator structures without affecting existing functionality. Users can continue using existing goal files without migration.

## References

- Original research: `thoughts/shared/research/2025-09-25-ui-issues-analysis.md`
- Key implementation files:
  - Event handling: `src/ui/app.rs:200-250`
  - Chart rendering: `src/widgets/performance_chart.rs:58`
  - Live metrics: `src/widgets/live_metrics.rs:85-89`
  - Navigation foundation: `src/data.rs:25-50`