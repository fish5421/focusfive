---
date: 2025-09-18T13:53:10Z
researcher: Claude Code
git_commit: 5a3055be9847acd0a397a1f16b7a2a0bea00955a
branch: feature/phase1-tui-enhancement
repository: goal_setting
topic: "Missing Dashboard Functionality After UI Change"
tags: [research, codebase, ui, data-model, objectives, kpis, gap-analysis]
status: complete
last_updated: 2025-09-18
last_updated_by: Claude Code
---

# Research: Missing Dashboard Functionality After UI Change

**Date**: 2025-09-18T13:53:10Z
**Researcher**: Claude Code
**Git Commit**: 5a3055be9847acd0a397a1f16b7a2a0bea00955a
**Branch**: feature/phase1-tui-enhancement
**Repository**: goal_setting

## Research Question

After implementing the UI changes from PROFESSIONAL_TUI_IMPLEMENTATION_PLAN.md, several critical features are missing from the Dashboard:
1. Cannot edit actual outcomes (5-year timeline goals)
2. Cannot attach outcomes to Actions
3. Cannot tie objectives to actions
4. Cannot set KPIs against objectives for goal tracking

## Summary

The research reveals a significant **implementation gap**: The data model layer has comprehensive support for objectives, KPIs, 5-year visions, and complex goal tracking, but the current UI implementation only exposes basic action editing and completion toggling. The missing functionality is not a data layer limitation but rather incomplete UI implementation.

## Detailed Findings

### Data Model Capabilities (Fully Implemented)

The data models in `src/models.rs` support extensive functionality:

#### 1. Enhanced Outcome Structure (`src/models.rs:52-70`)
```rust
pub struct Outcome {
    pub outcome_type: OutcomeType,
    pub goal: Option<String>,           // 5-year vision
    pub actions: [Action; 3],           // Daily actions
    pub objectives: Vec<Objective>,     // Annual objectives
    pub metrics: OutcomeMetrics,        // Performance tracking
    pub five_year_vision: Option<String>, // Long-term vision
}
```

#### 2. Objectives with KPIs (`src/models.rs:85-120`)
```rust
pub struct Objective {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub target_date: Option<NaiveDate>,
    pub status: ObjectiveStatus,
    pub progress_percentage: f32,
    pub kpis: Vec<KPI>,
    pub related_actions: Vec<String>,    // Links to action IDs
}

pub struct KPI {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub target_value: f32,
    pub current_value: f32,
    pub unit: String,
    pub measurement_frequency: MeasurementFrequency,
}
```

#### 3. Action-Objective Linking (`src/models.rs:150-170`)
```rust
pub struct Action {
    pub id: Option<String>,
    pub text: String,
    pub completed: bool,
    pub linked_objective_id: Option<String>, // Links to objectives
    // ... other fields
}
```

### Current UI Implementation Gaps

The UI implementation in `src/app.rs` and `src/ui/` only exposes:

#### ✅ Currently Available in UI:
- Action text editing (`src/app.rs:105-120`)
- Action completion toggling (`src/app.rs:192-204`)
- Basic outcome goal editing (`src/app.rs:121-139`)

#### ❌ Missing from UI (but supported in data model):
1. **5-Year Vision Editing**
   - Field exists: `Outcome.five_year_vision`
   - UI support: None

2. **Objectives Management**
   - Data structure: `Vec<Objective>` per outcome
   - UI support: None (no CRUD operations)

3. **KPI Setting and Tracking**
   - Data structure: `Vec<KPI>` per objective
   - UI support: None (no create/edit/track)

4. **Action-Objective Linking**
   - Fields exist: `Action.linked_objective_id`, `Objective.related_actions`
   - UI support: None (no linking UI)

5. **Metrics Dashboard**
   - Data available: `OutcomeMetrics`, `DailyStats`, `WeeklySummary`
   - UI support: Basic completion percentage only

### Code References

- `src/models.rs:52-70` - Enhanced Outcome structure with objectives and metrics
- `src/models.rs:85-120` - Objective and KPI definitions
- `src/models.rs:150-170` - Action structure with objective linking
- `src/app.rs:68-139` - Current input handling (limited to basic editing)
- `src/app.rs:15-38` - App state structure (no objective/KPI state management)
- `src/widgets/popup_editor.rs:8-17` - Popup editor (only used for text)

## Architecture Insights

### 1. Data Layer is Feature-Complete
The data models have full support for complex goal management including hierarchical objectives, quantifiable KPIs, and action-objective relationships. The implementation follows best practices with type safety and validation.

### 2. UI Layer is Minimal
The UI implementation focused on Phase 1 (basic TUI) from the implementation plan but didn't incorporate the advanced data capture features that exist in the model layer.

### 3. Widget Infrastructure Exists
The codebase has reusable widgets (`popup_editor.rs`, `action_editor.rs`) that could be extended or replicated for objective and KPI editing.

## Implementation Path Forward

### Quick Wins (1-2 hours each):
1. **Add 5-Year Vision Editing**: Copy goal editing pattern, add 'v' key handler
2. **Display Objectives List**: Read-only view in stats panel
3. **Show KPI Values**: Display current/target values in UI

### Medium Effort (4-8 hours each):
1. **Objectives CRUD**: New widget for add/edit/delete objectives
2. **KPI Management**: Widget for setting and updating KPI values
3. **Action-Objective Linking**: Dropdown or selection UI for linking

### Full Implementation (2-3 days):
1. **Complete Data Capture UI**: All fields editable
2. **Metrics Dashboard**: Visual charts and progress tracking
3. **Navigation Between Days**: Historical data viewing/editing

## Related Implementation Files

### Existing Patterns to Reuse:
- `/src/widgets/popup_editor.rs` - Text editing popup pattern
- `/src/app.rs:121-139` - Goal editing flow
- `/src/app.rs:68-86` - Input handling pattern

### Files Needing Modification:
- `/src/app.rs` - Add new input modes for objectives/KPIs
- `/src/ui/app.rs` - Render objectives and KPIs
- `/src/widgets/` - New widgets for objectives and KPIs

### Data Persistence:
- `/src/data.rs:173-216` - Extend markdown serialization for objectives/KPIs

## Open Questions

1. **Markdown Format**: How should objectives and KPIs be represented in the markdown files?
2. **UI Layout**: Where in the TUI should objectives and KPIs be displayed/edited?
3. **Migration**: How to handle existing files without objectives/KPIs?
4. **Keyboard Shortcuts**: What keys should trigger objective/KPI editing?

## Recommendations

### Immediate Action:
1. **Prioritize 5-Year Vision**: This is the simplest missing feature - just needs a new key handler
2. **Add Objectives Display**: Start with read-only display to validate the UI layout
3. **Implement One KPI**: Build the pattern with a single KPI before scaling

### Long-term Strategy:
1. **Phase the Implementation**: Don't try to add all features at once
2. **Maintain Backward Compatibility**: Ensure existing markdown files still work
3. **Test Incrementally**: Each new feature should be tested before adding the next

The gap between the data model capabilities and UI exposure represents significant untapped potential in the application. The foundation is solid - it just needs the UI layer to catch up.