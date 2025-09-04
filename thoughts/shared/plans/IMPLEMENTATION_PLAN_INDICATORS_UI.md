# Indicator UI Enhancement Implementation Plan

## Overview

Enhance the FocusFive TUI to provide expandable action items that reveal linked objectives and their associated indicators. This creates a three-level hierarchy (Action → Objective → Indicators) that clearly shows how daily tasks contribute to medium-term goals measured by specific indicators.

## Current State Analysis

The current FocusFive system has:
- **Simple Action Display**: Actions shown as `[✓] Action text` with no metadata
- **Fixed 3x3 Structure**: 3 outcomes with exactly 3 actions each  
- **Disconnected Models**: Actions, Objectives, and Indicators exist but aren't linked
- **Basic UI**: Three-pane layout with 25% width for actions pane
- **No Expandable Elements**: Static list display without drill-down capability

### Key Discoveries:
- Actions are stored in fixed arrays at `src/models.rs:45-50`
- UI renders actions at `src/ui.rs:190-221` 
- Objectives/Indicators exist at `src/models.rs:75-105` but aren't connected
- Ratatui supports custom rendering but not native expand/collapse

## Desired End State

Users will be able to:
1. **Expand actions** to see the linked objective and ALL its indicators
2. **View objective context** showing what medium-term goal the action serves
3. **See progress visualizations** with bars and current/target values
4. **Manually update indicators** through a unified update dialog
5. **Track overall objective progress** calculated from all its indicators

### UI Example:
```
▼ [ ] Review 3 SaaS listings
  └─ 📎 Objective: Make my first LOI offer
      ├─ Businesses Reviewed [████████░░] 18/25 week
      ├─ Market Research     [██████░░░░] 6.5/10 hrs
      ├─ Due Diligence       [███░░░░░░░] 40%
      └─ LOI Template Ready  [██████████] ✓ Complete
      Overall Progress: 65%
```

### Verification:
- Actions display with expand/collapse symbols (`▶`/`▼`)
- Objectives appear with `📎` icon when expanded
- All indicators for the objective are shown (not just action-specific)
- Universal update dialog handles different indicator types
- Overall objective progress calculated and displayed

## What We're NOT Doing

- **NOT** implementing automatic indicator updates (all manual)
- **NOT** adding time bounds or categories to objectives (keeping it simple)
- **NOT** changing the core 3x3 structure constraint
- **NOT** implementing complex graphing (keeping it terminal-friendly)
- **NOT** adding cloud sync or external dependencies
- **NOT** breaking existing markdown file format
- **NOT** adding mouse support (keyboard-only navigation)

## Implementation Approach

We'll implement this in 5 phases:
1. Extend data models with Action→Objective→Indicators hierarchy
2. Build expandable UI state management for three-level display
3. Implement progress visualization widgets
4. Add universal indicator update dialog with type-aware inputs  
5. Calculate and display overall objective progress

---

## Phase 1: Data Model Extensions

### Overview
Extend the Action, Objective, and Indicator structs to support the Action→Objective→Indicators hierarchy.

### Changes Required:

#### 1. Extend Action Struct  
**File**: `src/models.rs`
**Changes**: Add UUID and link to ONE objective

```rust
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Action {
    pub id: Uuid,                              // NEW: Unique identifier
    pub text: String,
    pub completed: bool,
    pub objective_id: Option<Uuid>,            // NEW: Link to ONE objective
    pub created_at: DateTime<Utc>,            // NEW: Timestamp
    pub completed_at: Option<DateTime<Utc>>,  // NEW: When completed
}
```

#### 2. Enhance Objective Struct
**File**: `src/models.rs`
**Changes**: Ensure objective owns multiple indicators

```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Objective {
    pub id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub outcome_type: OutcomeType,
    pub indicators: Vec<Uuid>,                 // Has MANY indicators
    pub created_at: DateTime<Utc>,
}
```

#### 3. Define Indicator Types
**File**: `src/models.rs`
**Changes**: Support different indicator types for flexible updates

```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Indicator {
    pub id: Uuid,
    pub name: String,
    pub indicator_type: IndicatorType,
    pub current_value: f64,
    pub target_value: f64,
    pub unit: String,                         // "count", "hours", "percentage", "boolean"
    pub history: Vec<IndicatorEntry>,         // Track changes over time
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum IndicatorType {
    Counter,     // Incremental counting (businesses reviewed)
    Duration,    // Time-based (hours of research)
    Percentage,  // 0-100% (completion percentage)
    Boolean,     // Complete/Incomplete (template ready)
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IndicatorEntry {
    pub timestamp: DateTime<Utc>,
    pub value: f64,
    pub note: Option<String>,
}
```

#### 4. Create UI State Management
**File**: `src/ui_state.rs` (new file)
**Changes**: Add expandable list state tracking

```rust
use std::collections::HashSet;
use uuid::Uuid;

pub struct ExpandableActionState {
    pub expanded_actions: HashSet<Uuid>,      // Which actions are expanded
    pub selected_action_index: usize,         // Current selection
    pub selected_indicator_index: Option<usize>, // Sub-selection within expanded
    pub indicator_update_mode: bool,          // Whether we're in update mode
    pub update_buffer: String,                // Input buffer for updates
}

impl ExpandableActionState {
    pub fn toggle_expansion(&mut self, action_id: Uuid) {
        if self.expanded_actions.contains(&action_id) {
            self.expanded_actions.remove(&action_id);
        } else {
            self.expanded_actions.insert(action_id);
        }
    }
    
    pub fn is_expanded(&self, action_id: &Uuid) -> bool {
        self.expanded_actions.contains(action_id)
    }
}
```

#### 5. Update Markdown Parser
**File**: `src/data.rs`
**Changes**: Parse objective link from markdown

```rust
// Enhanced markdown format:
// - [ ] Review 3 SaaS listings
//   objective: make-first-loi-offer
//   
impl DailyGoals {
    pub fn parse_enhanced(content: &str) -> Result<Self> {
        // Existing parsing logic...
        
        // NEW: Parse objective metadata  
        let objective_regex = Regex::new(r"^\s+objective:\s*(.+)$")?;
        
        // Parse each action with metadata
        for line in lines {
            if let Some(caps) = action_regex.captures(line) {
                let mut action = parse_action(caps)?;
                
                // Look for objective line following the action
                if let Some(obj_line) = lines.peek() {
                    if let Some(obj_caps) = objective_regex.captures(obj_line) {
                        let objective_ref = &obj_caps[1];
                        action.objective_id = find_objective_by_ref(objective_ref)?;
                    }
                }
                // ... continue parsing
            }
        }
    }
}
```

### Success Criteria:

#### Automated Verification:
- [ ] Code compiles: `cargo build`
- [ ] Unit tests pass: `cargo test models`
- [ ] Type checking passes: `cargo check`
- [ ] Linting passes: `cargo clippy`

#### Manual Verification:
- [ ] Actions can store indicator relationships
- [ ] UUID generation works correctly
- [ ] Markdown parsing handles new format
- [ ] Backward compatibility maintained

---

## Phase 2: Expandable UI Implementation

### Overview
Implement state-driven expandable list items showing Action→Objective→Indicators hierarchy.

### Changes Required:

#### 1. Update UI Rendering
**File**: `src/ui.rs`
**Changes**: Add three-level expandable rendering

```rust
fn render_actions_pane(f: &mut Frame, area: Rect, app: &TerminalApp) {
    let current_outcome = get_current_outcome(&app.goals, app.selected_outcome);
    
    // Build display items with expansion
    let mut display_items = Vec::new();
    let mut selectable_indices = Vec::new();
    
    for (idx, action) in current_outcome.actions.iter().enumerate() {
        // Add expansion symbol
        let symbol = if app.ui_state.is_expanded(&action.id) { "▼ " } else { "▶ " };
        let checkbox = if action.completed { "[✓]" } else { "[ ]" };
        
        // Main action line
        let action_line = format!("{}{} {}", symbol, checkbox, action.text);
        display_items.push(ListItem::new(action_line));
        selectable_indices.push((idx, None, None)); // Action level
        
        // Add objective and indicators if expanded
        if app.ui_state.is_expanded(&action.id) {
            if let Some(objective_id) = action.objective_id {
                let objective = app.get_objective(objective_id);
                
                // Objective line with icon
                let obj_line = format!("  └─ 📎 Objective: {}", objective.title);
                display_items.push(ListItem::new(obj_line));
                selectable_indices.push((idx, Some(0), None)); // Objective level
                
                // Add ALL indicators for this objective
                for (ind_idx, indicator_id) in objective.indicators.iter().enumerate() {
                    let indicator = app.get_indicator(*indicator_id);
                    let progress_bar = render_mini_progress(indicator);
                    let value_display = format_indicator_value(indicator);
                    
                    let indicator_line = format!("      {} {} [{}] {}", 
                        if ind_idx == objective.indicators.len() - 1 { "└─" } else { "├─" },
                        indicator.name,
                        progress_bar,
                        value_display
                    );
                    display_items.push(ListItem::new(indicator_line));
                    selectable_indices.push((idx, Some(0), Some(ind_idx))); // Indicator level
                }
                
                // Add overall progress line
                let overall_progress = calculate_objective_progress(&objective, &app.indicators);
                let overall_line = format!("      Overall Progress: {:.0}%", overall_progress);
                display_items.push(ListItem::new(overall_line));
            } else {
                // Action has no objective
                let no_obj_line = "  └─ (No objective linked)";
                display_items.push(ListItem::new(no_obj_line));
            }
        }
    }
    
    let actions_list = List::new(display_items)
        .block(Block::bordered().title(title))
        .highlight_style(Style::default().bg(Color::DarkGray))
        .highlight_symbol("→ ");
    
    f.render_stateful_widget(actions_list, area, &mut app.list_state);
}

fn render_mini_progress(indicator: &Indicator) -> String {
    let progress = indicator.current_value / indicator.target_value;
    let filled = (progress.min(1.0) * 10.0) as usize;
    let empty = 10 - filled;
    
    format!("{}{}",
        "█".repeat(filled),
        "░".repeat(empty)
    )
}

fn format_indicator_value(indicator: &Indicator) -> String {
    match indicator.indicator_type {
        IndicatorType::Counter => format!("{}/{}", 
            indicator.current_value as i32, 
            indicator.target_value as i32
        ),
        IndicatorType::Duration => format!("{:.1}/{:.1} hrs", 
            indicator.current_value, 
            indicator.target_value
        ),
        IndicatorType::Percentage => format!("{:.0}%", 
            indicator.current_value
        ),
        IndicatorType::Boolean => {
            if indicator.current_value >= 1.0 { "✓" } else { "✗" }.to_string()
        }
    }
}

fn calculate_objective_progress(objective: &Objective, indicators: &HashMap<Uuid, Indicator>) -> f64 {
    if objective.indicators.is_empty() {
        return 0.0;
    }
    
    let mut total_progress = 0.0;
    for indicator_id in &objective.indicators {
        if let Some(indicator) = indicators.get(indicator_id) {
            let progress = (indicator.current_value / indicator.target_value).min(1.0);
            total_progress += progress;
        }
    }
    
    (total_progress / objective.indicators.len() as f64) * 100.0
}
```

#### 2. Handle Expansion Navigation
**File**: `src/ui.rs`
**Changes**: Add keyboard handlers for expansion

```rust
fn handle_key_event(key: KeyEvent, app: &mut TerminalApp) -> io::Result<()> {
    match app.input_mode {
        InputMode::Normal => match key.code {
            KeyCode::Enter | KeyCode::Char('e') => {
                // Toggle expansion of current action
                if let Some(action) = get_selected_action(app) {
                    app.ui_state.toggle_expansion(action.id);
                }
            }
            KeyCode::Char('j') | KeyCode::Down => {
                // Navigate through expanded items
                app.navigate_down();
            }
            KeyCode::Char('k') | KeyCode::Up => {
                // Navigate through expanded items  
                app.navigate_up();
            }
            KeyCode::Char('+') | KeyCode::Char('=') => {
                // Quick increment indicator
                if let Some(indicator) = get_selected_indicator(app) {
                    app.increment_indicator(indicator.id);
                }
            }
            KeyCode::Char('-') | KeyCode::Char('_') => {
                // Quick decrement indicator
                if let Some(indicator) = get_selected_indicator(app) {
                    app.decrement_indicator(indicator.id);
                }
            }
            // ... existing handlers
        }
    }
}
```

### Success Criteria:

#### Automated Verification:
- [ ] UI compiles with new rendering: `cargo build`
- [ ] Expansion state persists correctly: `cargo test ui_state`
- [ ] Navigation works through expanded items: `cargo test navigation`

#### Manual Verification:
- [ ] Actions show expand/collapse symbols
- [ ] Enter/e key toggles expansion
- [ ] Indicators appear indented under actions
- [ ] Navigation works through nested items

---

## Phase 3: Progress Visualization

### Overview
Implement progress bars, sparklines, and trend indicators for visualizing indicator progress.

### Changes Required:

#### 1. Create Progress Widgets
**File**: `src/widgets/progress.rs` (new file)
**Changes**: Custom progress visualization components

```rust
use ratatui::{
    prelude::*,
    widgets::{Sparkline, Gauge, Paragraph},
};

pub struct IndicatorProgress {
    pub current: f64,
    pub target: f64,
    pub history: Vec<f64>,
    pub trend: TrendDirection,
}

pub enum TrendDirection {
    Up,
    Down, 
    Stable,
}

impl IndicatorProgress {
    pub fn render_bar(&self) -> Paragraph {
        let percentage = (self.current / self.target * 100.0).min(100.0);
        let filled = (percentage / 10.0) as usize;
        let empty = 10 - filled;
        
        let bar = format!("{}{}",
            "█".repeat(filled),
            "░".repeat(empty)
        );
        
        let style = match percentage {
            p if p >= 100.0 => Style::default().fg(Color::Green),
            p if p >= 70.0 => Style::default().fg(Color::Yellow),
            _ => Style::default().fg(Color::Red),
        };
        
        Paragraph::new(bar).style(style)
    }
    
    pub fn render_sparkline(&self) -> Sparkline {
        Sparkline::default()
            .data(&self.history)
            .style(Style::default().fg(Color::Cyan))
    }
    
    pub fn render_trend(&self) -> &str {
        match self.trend {
            TrendDirection::Up => "↗",
            TrendDirection::Down => "↘",
            TrendDirection::Stable => "→",
        }
    }
}
```

#### 2. Indicator Detail Popup
**File**: `src/ui.rs`
**Changes**: Add detail view for selected indicators

```rust
fn render_indicator_detail(f: &mut Frame, area: Rect, indicator: &Indicator, history: &[f64]) {
    let chunks = Layout::vertical([
        Constraint::Length(3),   // Header
        Constraint::Length(4),   // Current value
        Constraint::Length(3),   // Progress bar
        Constraint::Min(5),      // Sparkline
        Constraint::Length(3),   // Actions
    ]).split(area);
    
    // Header
    let header = Paragraph::new(vec![
        Line::from(vec![
            Span::styled(&indicator.name, Style::default().bold()),
            Span::raw(" - "),
            Span::raw(&indicator.description.as_deref().unwrap_or("")),
        ])
    ])
    .block(Block::bordered().title("Indicator Detail"));
    f.render_widget(header, chunks[0]);
    
    // Current value
    let current = format!("Current: {:.1} {} / Target: {:.1} {}",
        indicator.current_value.unwrap_or(0.0),
        indicator.unit.as_deref().unwrap_or(""),
        indicator.target_value.unwrap_or(0.0),
        indicator.unit.as_deref().unwrap_or("")
    );
    f.render_widget(Paragraph::new(current), chunks[1]);
    
    // Progress bar
    let progress = Gauge::default()
        .percent((indicator.current_value.unwrap_or(0.0) / 
                 indicator.target_value.unwrap_or(100.0) * 100.0) as u16)
        .style(Style::default().fg(Color::Green));
    f.render_widget(progress, chunks[2]);
    
    // Sparkline
    let sparkline = Sparkline::default()
        .data(history)
        .style(Style::default().fg(Color::Yellow));
    f.render_widget(sparkline.block(Block::bordered().title("7-Day Trend")), chunks[3]);
    
    // Actions help
    let help = Paragraph::new("[+/-] Adjust  [Enter] Save  [Esc] Close");
    f.render_widget(help, chunks[4]);
}
```

### Success Criteria:

#### Automated Verification:
- [ ] Progress widgets render correctly: `cargo test widgets`
- [ ] Color coding works by percentage: `cargo test progress_colors`
- [ ] Sparkline handles empty data: `cargo test sparkline_empty`

#### Manual Verification:
- [ ] Progress bars show correct fill percentage
- [ ] Colors change based on completion (red/yellow/green)
- [ ] Sparklines display historical trends
- [ ] Trend arrows show correct direction

---

## Phase 4: Universal Indicator Update Dialog

### Overview
Implement a unified update dialog that adapts to different indicator types (Counter, Duration, Percentage, Boolean).

### Changes Required:

#### 1. Indicator Update Mode
**File**: `src/ui.rs`
**Changes**: Add type-aware update interaction

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum InputMode {
    Normal,
    EditingAction,
    UpdatingIndicator(Uuid),  // NEW: Indicator update mode
}

impl TerminalApp {
    pub fn enter_indicator_update(&mut self, indicator_id: Uuid) {
        self.input_mode = InputMode::UpdatingIndicator(indicator_id);
        self.update_buffer.clear();
        
        // Pre-fill with current value
        if let Some(indicator) = self.get_indicator(indicator_id) {
            self.update_buffer = format!("{:.1}", indicator.current_value);
        }
    }
    
    pub fn apply_indicator_update(&mut self) -> Result<()> {
        if let InputMode::UpdatingIndicator(id) = self.input_mode {
            if let Some(indicator) = self.get_indicator_mut(id) {
                // Parse based on indicator type
                let new_value = match indicator.indicator_type {
                    IndicatorType::Counter => self.update_buffer.parse::<f64>()?,
                    IndicatorType::Duration => self.update_buffer.parse::<f64>()?,
                    IndicatorType::Percentage => {
                        let pct = self.update_buffer.trim_end_matches('%').parse::<f64>()?;
                        pct.min(100.0).max(0.0)
                    },
                    IndicatorType::Boolean => {
                        if self.update_buffer.to_lowercase().starts_with('y') ||
                           self.update_buffer == "1" || 
                           self.update_buffer.to_lowercase() == "true" {
                            1.0
                        } else {
                            0.0
                        }
                    }
                };
                
                // Store history
                indicator.history.push(IndicatorEntry {
                    timestamp: Utc::now(),
                    value: indicator.current_value,
                    note: None,
                });
                
                // Update value
                indicator.current_value = new_value;
                
                // Recalculate objective progress
                self.recalculate_objective_progress()?;
            }
            
            self.input_mode = InputMode::Normal;
            self.save_goals()?;
        }
        Ok(())
    }
}
```

#### 2. Universal Update Dialog
**File**: `src/ui.rs`  
**Changes**: Type-aware update dialog with quick actions

```rust
fn render_update_overlay(f: &mut Frame, area: Rect, app: &TerminalApp) {
    if let InputMode::UpdatingIndicator(id) = app.input_mode {
        let indicator = app.get_indicator(id).unwrap();
        
        // Create centered popup (60% width, 40% height)
        let popup_area = centered_rect(60, 40, area);
        f.render_widget(Clear, popup_area);
        
        let popup = Block::bordered()
            .title(format!("Update: {}", indicator.name))
            .border_style(Style::default().fg(Color::Yellow));
        
        let inner = popup.inner(popup_area);
        f.render_widget(popup, popup_area);
        
        // Layout based on indicator type
        let chunks = Layout::vertical([
            Constraint::Length(3),   // Current/Target display
            Constraint::Length(3),   // Progress bar
            Constraint::Length(4),   // Quick actions
            Constraint::Length(3),   // Custom input
            Constraint::Length(2),   // Help text
        ]).split(inner);
        
        // Current and target values
        let value_display = format!("Current: {} | Target: {}",
            format_indicator_value(indicator),
            format_target_value(indicator)
        );
        f.render_widget(Paragraph::new(value_display).centered(), chunks[0]);
        
        // Progress bar
        let progress = (indicator.current_value / indicator.target_value).min(1.0);
        let gauge = Gauge::default()
            .percent((progress * 100.0) as u16)
            .style(Style::default().fg(Color::Green));
        f.render_widget(gauge, chunks[1]);
        
        // Quick actions based on type
        let quick_actions = match indicator.indicator_type {
            IndicatorType::Counter => {
                vec![
                    "[1] +1 → {}".to_string(),
                    "[3] +3 → {}".to_string(), 
                    "[5] +5 → {}".to_string(),
                ]
            },
            IndicatorType::Duration => {
                vec![
                    "[1] +0.5hr → {}".to_string(),
                    "[2] +1hr → {}".to_string(),
                    "[3] +2hrs → {}".to_string(),
                ]
            },
            IndicatorType::Percentage => {
                vec![
                    "[2] 25%".to_string(),
                    "[5] 50%".to_string(),
                    "[7] 75%".to_string(),
                    "[9] 100%".to_string(),
                ]
            },
            IndicatorType::Boolean => {
                vec![
                    "[Y] Mark Complete".to_string(),
                    "[N] Mark Incomplete".to_string(),
                ]
            }
        };
        
        let actions_text = quick_actions.join("  ");
        f.render_widget(
            Paragraph::new(actions_text).centered(),
            chunks[2]
        );
        
        // Custom input field
        let input = Paragraph::new(app.update_buffer.clone())
            .block(Block::bordered().title("Or enter custom value"))
            .style(Style::default().fg(Color::White));
        f.render_widget(input, chunks[3]);
        
        // Help text
        let help = match indicator.indicator_type {
            IndicatorType::Counter => "[Enter number] [Enter] Save [Esc] Cancel",
            IndicatorType::Duration => "[Enter hours] [Enter] Save [Esc] Cancel", 
            IndicatorType::Percentage => "[Enter 0-100] [Enter] Save [Esc] Cancel",
            IndicatorType::Boolean => "[Y/N] or [Enter] Save [Esc] Cancel",
        };
        f.render_widget(
            Paragraph::new(help)
                .style(Style::default().fg(Color::DarkGray))
                .centered(),
            chunks[4]
        );
    }
}

fn format_target_value(indicator: &Indicator) -> String {
    match indicator.indicator_type {
        IndicatorType::Counter => format!("{}", indicator.target_value as i32),
        IndicatorType::Duration => format!("{:.1} hrs", indicator.target_value),
        IndicatorType::Percentage => "100%".to_string(),
        IndicatorType::Boolean => "Complete".to_string(),
    }
}
```

### Success Criteria:

#### Automated Verification:
- [ ] Update mode transitions work: `cargo test input_modes`
- [ ] Value parsing handles errors: `cargo test parse_updates`
- [ ] History tracking works: `cargo test indicator_history`

#### Manual Verification:
- [ ] +/- keys quickly adjust values
- [ ] Enter on indicator opens update dialog
- [ ] Values persist after updates
- [ ] Update history is maintained

---

## Phase 5: Overall Objective Progress Calculation

### Overview
Implement the system for calculating and displaying overall objective progress based on all its indicators.

### Changes Required:

#### 1. Objective Progress Calculator
**File**: `src/progress.rs` (new file)
**Changes**: Calculate overall objective progress

```rust
use std::collections::HashMap;
use uuid::Uuid;

pub struct ProgressCalculator {
    objectives: HashMap<Uuid, Objective>,
    indicators: HashMap<Uuid, Indicator>,
}

impl ProgressCalculator {
    pub fn calculate_objective_progress(&self, objective_id: Uuid) -> ObjectiveProgress {
        let objective = &self.objectives[&objective_id];
        let mut indicator_progresses = Vec::new();
        let mut total_progress = 0.0;
        let mut completed_count = 0;
        
        for indicator_id in &objective.indicators {
            if let Some(indicator) = self.indicators.get(indicator_id) {
                let progress = (indicator.current_value / indicator.target_value).min(1.0);
                let is_complete = progress >= 1.0;
                
                if is_complete {
                    completed_count += 1;
                }
                
                indicator_progresses.push(IndicatorProgress {
                    indicator_id: *indicator_id,
                    name: indicator.name.clone(),
                    current: indicator.current_value,
                    target: indicator.target_value,
                    progress_percentage: progress * 100.0,
                    is_complete,
                });
                
                total_progress += progress;
            }
        }
        
        let overall_percentage = if !objective.indicators.is_empty() {
            (total_progress / objective.indicators.len() as f64) * 100.0
        } else {
            0.0
        };
        
        ObjectiveProgress {
            objective_id,
            title: objective.title.clone(),
            indicator_progresses,
            overall_percentage,
            completed_indicators: completed_count,
            total_indicators: objective.indicators.len(),
        }
    }
    
    pub fn get_next_focus(&self, objective_id: Uuid) -> Option<String> {
        let objective = &self.objectives[&objective_id];
        
        // Find indicator with lowest progress
        let mut lowest_progress = 1.0;
        let mut focus_indicator = None;
        
        for indicator_id in &objective.indicators {
            if let Some(indicator) = self.indicators.get(indicator_id) {
                let progress = indicator.current_value / indicator.target_value;
                if progress < lowest_progress {
                    lowest_progress = progress;
                    focus_indicator = Some(indicator.name.clone());
                }
            }
        }
        
        focus_indicator
    }
}

#[derive(Debug)]
pub struct ObjectiveProgress {
    pub objective_id: Uuid,
    pub title: String,
    pub indicator_progresses: Vec<IndicatorProgress>,
    pub overall_percentage: f64,
    pub completed_indicators: usize,
    pub total_indicators: usize,
}

#[derive(Debug)]
pub struct IndicatorProgress {
    pub indicator_id: Uuid,
    pub name: String,
    pub current: f64,
    pub target: f64,
    pub progress_percentage: f64,
    pub is_complete: bool,
}
```

#### 2. Progress Summary Display  
**File**: `src/ui.rs`
**Changes**: Show objective progress summary

```rust
fn render_objective_progress(f: &mut Frame, area: Rect, progress: &ObjectiveProgress) {
    let block = Block::bordered()
        .title(format!("Objective: {}", progress.title))
        .border_style(Style::default().fg(Color::Cyan));
    
    let inner = block.inner(area);
    f.render_widget(block, area);
    
    let chunks = Layout::vertical([
        Constraint::Length(3),   // Overall progress
        Constraint::Min(1),      // Indicator list
        Constraint::Length(2),   // Summary stats
    ]).split(inner);
    
    // Overall progress bar
    let overall_gauge = Gauge::default()
        .percent(progress.overall_percentage as u16)
        .label(format!("Overall Progress: {:.0}%", progress.overall_percentage))
        .style(match progress.overall_percentage {
            p if p >= 100.0 => Style::default().fg(Color::Green),
            p if p >= 70.0 => Style::default().fg(Color::Yellow),
            _ => Style::default().fg(Color::Red),
        });
    f.render_widget(overall_gauge, chunks[0]);
    
    // Individual indicator progress
    let mut lines = vec![
        Line::from(Span::styled("Indicators:", Style::default().bold())),
    ];
    
    for ind_progress in &progress.indicator_progresses {
        let status = if ind_progress.is_complete { "✓" } else { " " };
        let bar = render_mini_bar(ind_progress.progress_percentage / 100.0);
        
        let line = Line::from(vec![
            Span::raw(format!(" {} ", status)),
            Span::raw(&ind_progress.name),
            Span::raw(": "),
            Span::raw(bar),
            Span::raw(format!(" {:.0}/{:.0}", 
                ind_progress.current, 
                ind_progress.target
            )),
        ]);
        lines.push(line);
    }
    
    f.render_widget(Paragraph::new(lines), chunks[1]);
    
    // Summary stats
    let summary = format!(
        "Completed: {}/{} indicators | Focus next: {}",
        progress.completed_indicators,
        progress.total_indicators,
        app.progress_calculator.get_next_focus(progress.objective_id)
            .unwrap_or_else(|| "All complete!".to_string())
    );
    f.render_widget(
        Paragraph::new(summary)
            .style(Style::default().fg(Color::DarkGray)),
        chunks[2]
    );
}

fn render_mini_bar(progress: f64) -> String {
    let filled = (progress * 10.0) as usize;
    let empty = 10 - filled;
    format!("[{}{}]",
        "█".repeat(filled),
        "░".repeat(empty)
    )
}
```

#### 3. Integration with Main UI
**File**: `src/ui.rs`
**Changes**: Add progress calculation to main rendering

```rust
impl TerminalApp {
    pub fn recalculate_objective_progress(&mut self) -> Result<()> {
        // Update all objective progress calculations
        for objective_id in self.objectives.keys() {
            let progress = self.progress_calculator
                .calculate_objective_progress(*objective_id);
            self.objective_progress_cache.insert(*objective_id, progress);
        }
        Ok(())
    }
    
    pub fn get_objective_progress(&self, objective_id: Uuid) -> Option<&ObjectiveProgress> {
        self.objective_progress_cache.get(&objective_id)
    }
}
```

### Success Criteria:

#### Automated Verification:
- [ ] Impact calculations are correct: `cargo test impact_calc`
- [ ] Compound scoring works: `cargo test compound_score`
- [ ] Preview renders without errors: `cargo test preview_render`

#### Manual Verification:
- [ ] Impact preview shows before completion
- [ ] Multiple indicators update correctly
- [ ] Compound score reflects overall progress
- [ ] Objective progress updates automatically

---

## Testing Strategy

### Unit Tests:
- Test expandable state management
- Test indicator value updates and history
- Test impact calculations
- Test progress bar rendering logic

### Integration Tests:
- Test full action→objective→indicator flow
- Test markdown parsing with new metadata
- Test UI navigation through expanded items
- Test save/load with enhanced data

### Manual Testing Steps:
1. Create action with objective link in markdown:
   ```markdown
   - [ ] Review 3 SaaS listings
     objective: make-first-loi-offer
   ```
2. Launch TUI and navigate to action
3. Press Enter/e to expand action
4. Verify objective appears with 📎 icon
5. Verify ALL indicators for that objective are shown
6. Select an indicator and press 'u' or Enter
7. Verify update dialog shows with type-appropriate quick actions
8. Update value using quick action or custom input
9. Verify overall objective progress updates
10. Check that progress persists after restart

## Performance Considerations

- **Lazy Loading**: Only calculate impacts when action is selected
- **Caching**: Cache indicator progress calculations
- **Batch Updates**: Update multiple indicators in single operation
- **Render Optimization**: Only re-render changed widgets

## Migration Notes

### Backward Compatibility:
1. Old markdown files without metadata will still load
2. Actions without indicators will display normally
3. UUID generation happens on first load for existing actions

### Migration Steps:
1. Deploy code with backward compatibility
2. Run migration script to add UUIDs to existing data
3. Gradually add indicator metadata to actions
4. Monitor for any parsing errors

## References

- Original research: `research_objectives_indicators.md`
- Ratatui documentation: https://ratatui.rs/
- Current UI implementation: `src/ui.rs:190-221`
- Model definitions: `src/models.rs:45-105`