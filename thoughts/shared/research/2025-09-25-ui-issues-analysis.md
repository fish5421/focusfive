---
date: 2025-09-25T10:05:15-04:00
researcher: Claude
git_commit: 7fa362ba5e4919ded7657d932e66cdbc1773c83c
branch: feature/dashboard-redesign
repository: goal_setting
topic: "FocusFive Dashboard UI Issues and Improvements Analysis"
tags: [research, codebase, ui, charts, navigation, dashboard, indicators, layout]
status: complete
last_updated: 2025-09-25
last_updated_by: Claude
---

# Research: FocusFive Dashboard UI Issues and Improvements Analysis

**Date**: 2025-09-25T10:05:15-04:00
**Researcher**: Claude
**Git Commit**: 7fa362ba5e4919ded7657d932e66cdbc1773c83c
**Branch**: feature/dashboard-redesign
**Repository**: goal_setting

## Research Question
Analysis of current FocusFive application UI issues including indicator popup key interactions, chart sizing and viewport utilization, Live Metrics column layout, and main dashboard day navigation functionality.

## Summary
The FocusFive TUI application has four major UI implementation issues:

1. **Indicator Popup Key Handling**: Help keys (up/down arrows, tab) don't interact with popup due to missing event delegation
2. **Chart Viewport Utilization**: Bar charts too narrow, 7-day view doesn't fill viewport due to fixed width calculations
3. **Live Metrics Column Sizing**: Indicator names truncated at 18 chars despite available space, hardcoded column widths
4. **Day Navigation Missing**: No ability to cycle through days of the week, only displays current day

All issues are architectural with clear paths to resolution through existing framework capabilities.

## Detailed Findings

### 1. Indicator Popup Key Interaction Issues

**Root Cause**: Event delegation architecture missing - all keys processed by main app instead of active popup.

**Key Files**:
- `src/ui/indicator_popup.rs:80-120` - Contains proper key handling logic but never called
- `src/ui/app.rs:200-250` - Main event handler lacks popup delegation check
- `src/ui_state.rs:45-65` - Tracks popup state but no delegation mechanism

**Current Broken Flow**:
1. Key event → `src/ui/app.rs:200`
2. Processed directly by main app logic
3. Popup's `handle_key_event()` method never called
4. Help keys ignored or mishandled

**Fix Required**: Add popup priority check in `handle_key_event()`:
```rust
// Missing from src/ui/app.rs:200
if let Some(ref mut popup) = self.ui_state.indicator_popup {
    if popup.handle_key_event(key) {
        return Ok(()); // Popup handled the event
    }
}
```

### 2. Chart Sizing and Viewport Issues

**Root Cause**: Fixed bar width calculation and inflexible layout constraints prevent full viewport utilization.

**Key Implementation Problems**:

**Bar Width Calculation** (`src/widgets/performance_chart.rs:58`):
```rust
let bar_width = std::cmp::max(1, (area.width as usize - 10) / data.len().max(1));
```
- Divides width by 7 days, creates narrow bars
- 80-char terminal → (70)/7 = 10 chars per bar (underutilizes space)
- No minimum width enforcement for readability

**Fixed Layout Constraints** (`src/ui/dashboard_layout.rs:13`):
```rust
.constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
```
- Charts always get exactly 50% of screen width
- No dynamic allocation based on content needs
- 7-day charts need more horizontal space than other content

**Day-of-Week Labels**: Generated correctly at `src/widgets/performance_chart.rs:40-48` but bars too narrow to display properly.

### 3. Live Metrics Column Layout Issues

**Root Cause**: Hardcoded column constraints with premature text truncation.

**Column Width Problem** (`src/widgets/live_metrics.rs:85-89`):
```rust
let widths = [
    Constraint::Length(20), // Indicator name - FIXED 20 chars
    Constraint::Length(8),  // Current value - FIXED 8 chars
    // ... more fixed constraints
];
```

**Text Truncation Logic** (`src/widgets/live_metrics.rs:67-70`):
```rust
let name_cell = Cell::from(if indicator.name.len() > 18 {
    format!("{}...", &indicator.name[..15])  // Truncates to 15 + "..."
} else {
    indicator.name.clone()
});
```
- Truncates at 18 chars but column allows 20 chars (wastes 2 characters)
- No consideration of available terminal width
- Fixed behavior regardless of screen size

### 4. Day Navigation Missing Implementation

**Root Cause**: App designed for single-date operation, lacks navigation state management.

**Current State Management** (`src/app.rs:15-45`):
```rust
pub struct App {
    pub goals: DailyGoals,  // Only current day's goals
    // MISSING: current_date: NaiveDate
    // MISSING: max_date restriction
}
```

**Key Event Handling** (`src/ui/app.rs:180-220`):
- No Left/Right arrow key handlers for date navigation
- No date cycling logic implemented
- No future date restriction checks

**Available Foundation**:
- `src/data.rs:25-50` - `load_goals_for_date()` supports any date
- `src/models.rs:55-75` - DailyGoals has proper date field
- Auto-save functionality exists for current date changes

## Code References

### Indicator Popup
- `src/ui/indicator_popup.rs:80-120` - Unused key handling implementation
- `src/ui/app.rs:200-250` - Main event handler needing delegation
- `src/ui_state.rs:45-65` - Popup state tracking

### Chart Rendering
- `src/widgets/performance_chart.rs:58` - Bar width calculation logic
- `src/ui/dashboard_layout.rs:13` - Fixed 50/50 layout constraints
- `src/ui/charts.rs:15-89` - Chart rendering entry points

### Live Metrics Layout
- `src/widgets/live_metrics.rs:85-89` - Column width definitions
- `src/widgets/live_metrics.rs:67-70` - Text truncation logic
- `src/ui/dashboard_layout.rs:86-95` - Metrics positioning in dashboard

### Navigation Infrastructure
- `src/app.rs:15-45` - App state structure (needs extensions)
- `src/ui/app.rs:180-220` - Event handling (needs date navigation)
- `src/data.rs:25-50` - Date-based loading (ready for use)

## Architecture Insights

### Event Handling Patterns
- Uses centralized key event processing in main app
- Lacks hierarchical event delegation for UI components
- Popup system exists but not integrated with event flow

### Layout System Design
- Built on ratatui's constraint-based layout system
- Currently uses fixed constraints, supports flexible alternatives
- Dashboard uses percentage-based splits, widgets use fixed lengths

### Data Layer Capabilities
- Supports loading goals for any date via `load_goals_for_date()`
- Auto-saves changes to appropriate date files
- Creates default goal structures for missing dates

## Recommended Implementation Priorities

### High Priority (Functionality Blockers)
1. **Event Delegation**: Fix indicator popup key handling
2. **Day Navigation**: Implement basic Left/Right arrow date cycling
3. **Chart Viewport**: Make bars fill 7-day view width properly

### Medium Priority (User Experience)
4. **Live Metrics Columns**: Make indicator names wider and responsive
5. **Weekly Progress Charts**: Convert from bar to line charts
6. **Future Date Restriction**: Prevent navigation beyond today

### Low Priority (Polish)
7. **Rolling 7-Day Labels**: Always show current day as 7th day
8. **Indicator Reset**: Add functionality to reset indicator data
9. **Context-Aware Help**: Show appropriate keys based on active UI component

## Implementation Notes

All identified issues have clear paths to resolution using existing ratatui capabilities:
- Event delegation through conditional key handling
- Responsive layouts using `Constraint::Percentage()` and `Constraint::Min()`
- Date navigation using existing data layer functions
- Dynamic text sizing based on available area calculations

The foundation is solid - issues are primarily in the integration layer rather than core functionality gaps.