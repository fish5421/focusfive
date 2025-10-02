---
date: 2025-10-01T21:02:57+0000
researcher: Claude
git_commit: a741e9b218b10de48c0655603a8579c61d1892df
branch: fish5421/best-practices
repository: porto
topic: "Phase 1 - Outcome Micro-Card Implementation Research"
tags: [research, codebase, outcome-micro-card, tui, data-model, toml, metadata]
status: complete
last_updated: 2025-10-01
last_updated_by: Claude
---

# Research: Phase 1 - Outcome Micro-Card Implementation

**Date**: 2025-10-01T21:02:57+0000
**Researcher**: Claude
**Git Commit**: a741e9b218b10de48c0655603a8579c61d1892df
**Branch**: fish5421/best-practices
**Repository**: porto

## Research Question

How can we implement Phase 1 of the Outcome Micro-Card feature, which adds expandable context cards with North Star, Lead Measure, Target Metric, and Confidence fields to each outcome, while maintaining backward compatibility and following FocusFive's best practices?

## Summary

The codebase is well-positioned for implementing the Outcome Micro-Card feature. Phase 2 TUI is complete with ratatui infrastructure, the data model uses optional fields for backward compatibility, and atomic write patterns prevent corruption. Key implementation areas:

1. **Data Model Extension**: Add 4 new optional fields to `Outcome` struct
2. **Dual Persistence**: Stable fields (North Star, Lead, Target) in `~/.focusfive/outcomes.toml`, ephemeral Confidence in daily markdown via `@meta` lines
3. **TOML Infrastructure**: Needs to be implemented (no existing TOML handling)
4. **TUI Expansion Logic**: Extend existing TUI with Enter to expand/collapse outcomes
5. **Inline Editors**: Add modal editors for n/l/m/c keys with existing event handling

## Detailed Findings

### 1. Current Data Model Architecture

**File**: `src/models.rs:87-92`

```rust
pub struct Outcome {
    pub outcome_type: OutcomeType,  // Work | Health | Family
    pub goal: Option<String>,        // Already uses Option pattern
    pub actions: [Action; 3],        // Fixed 3-action array
}
```

**Extension Pattern Found**: The codebase already uses `Option<String>` for backward compatibility with the `goal` field. This same pattern should be used for the 4 new fields.

**Proposed Extension** (`src/models.rs:91`):
```rust
pub struct Outcome {
    pub outcome_type: OutcomeType,
    pub goal: Option<String>,
    pub actions: [Action; 3],

    // New metadata fields - all optional for backward compatibility
    pub north_star: Option<String>,    // Why: Long-term vision
    pub lead_measure: Option<String>,  // What moves the needle
    pub target_metric: Option<String>, // Structured as "2 pages / wk"
    // Note: Confidence is NOT stored here - it's ephemeral per-day data
}
```

**Default Implementation** must be updated at `src/models.rs:94-105`:
```rust
impl Default for Outcome {
    fn default() -> Self {
        Self {
            outcome_type: OutcomeType::Work,
            goal: None,
            actions: [Action::default(), Action::default(), Action::default()],
            north_star: None,
            lead_measure: None,
            target_metric: None,
        }
    }
}
```

### 2. Markdown Parsing & @meta Line Injection

**Current Parser**: `src/data.rs:58-234` uses regex-based multi-stage parsing

**Key Parsing Stages**:
1. Date header extraction (lines 42-72) - scans first 10 lines
2. Outcome header parsing (lines 85-105) - case-insensitive
3. Action extraction (lines 108-135) - checkbox pattern

**Optimal @meta Injection Point**: `src/data.rs:73` (after date parsing, before outcome loop)

**Proposed @meta Parser**:
```rust
// Add around line 73 in parse_markdown()
let meta_re = Regex::new(r"^@meta\s+(\w+)=(\d+)$")?;
let mut confidence_values = HashMap::new();  // outcome_type -> confidence

for line in lines.iter() {
    if let Some(caps) = meta_re.captures(line.trim()) {
        let key = caps[1].to_lowercase();
        if key.ends_with("_confidence") {
            // Extract "work" from "work_confidence"
            let outcome = key.trim_end_matches("_confidence");
            if let Ok(value) = caps[2].parse::<u8>() {
                if value <= 100 {
                    confidence_values.insert(outcome.to_string(), value);
                }
            }
        }
    }
}
```

**Example @meta Line Format in Markdown**:
```markdown
# October 1, 2025
@meta work_confidence=70
@meta health_confidence=85
@meta family_confidence=60

## Work (Goal: Ship feature X)
- [ ] Action 1
- [ ] Action 2
- [ ] Action 3
```

**Writing @meta Lines**: Update `src/data.rs:237-276` format_goals():
```rust
// After date header, before outcomes
if let Some(conf) = work_confidence {
    writeln!(f, "@meta work_confidence={}", conf)?;
}
// Similar for health and family
```

### 3. TOML Sidecar File Implementation

**Current State**: No TOML handling exists in the codebase.

**Required Dependencies** (add to `Cargo.toml`):
```toml
serde = { version = "1.0", features = ["derive"] }
toml = "0.8"
```

**Proposed Structure**: `~/.focusfive/outcomes.toml`
```toml
[work]
north_star = "Enable self-serve adoption via docs"
lead_measure = "Documentation hours"
target_metric = "2 pages / week"

[health]
north_star = "Maintain energy and focus"
lead_measure = "Exercise sessions"
target_metric = "4 workouts / week"

[family]
north_star = "Build strong relationships"
lead_measure = "Quality time"
target_metric = "2 hours / day"
```

**New Module**: Create `src/outcome_metadata.rs`
```rust
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct OutcomeMetadata {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub north_star: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lead_measure: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_metric: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct OutcomesConfig {
    pub work: OutcomeMetadata,
    pub health: OutcomeMetadata,
    pub family: OutcomeMetadata,
}

impl OutcomesConfig {
    pub fn file_path() -> Result<PathBuf> {
        let home = dirs::home_dir()
            .context("Could not determine home directory")?;
        Ok(home.join(".focusfive").join("outcomes.toml"))
    }

    pub fn load() -> Result<Self> {
        let path = Self::file_path()?;
        if !path.exists() {
            return Ok(Self::default());
        }

        let content = fs::read_to_string(&path)
            .with_context(|| format!("Failed to read: {}", path.display()))?;

        toml::from_str(&content)
            .context("Failed to parse outcomes.toml")
    }

    pub fn save(&self) -> Result<()> {
        let path = Self::file_path()?;
        let dir = path.parent().context("Invalid path")?;

        fs::create_dir_all(dir)?;

        // Use atomic write pattern (like save_goals in data.rs:24-47)
        let temp_file = dir.join(format!(
            ".temp_outcomes_{}_{}.toml",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_micros()
        ));

        let content = toml::to_string_pretty(self)
            .context("Failed to serialize outcomes config")?;

        fs::write(&temp_file, content)?;
        fs::rename(&temp_file, &path)?;

        Ok(())
    }
}
```

**Pattern Source**: Based on atomic write pattern from `src/data.rs:24-47` (save_goals function)

### 4. TUI Implementation Status & Extension Points

**Current TUI State**: ✅ Phase 2 COMPLETE

**Infrastructure Present**:
- `src/tui.rs` - Core rendering loop
- `src/ui/app.rs` - Application state
- `src/ui/components.rs` - UI widgets
- `src/ui/layout.rs` - Layout management
- `src/ui/events.rs` - Event handling

**Dependencies** (from `Cargo.toml`):
- `ratatui = "0.29.0"` ✅
- `crossterm = "0.28.1"` ✅

**Current Keyboard Navigation** (`src/ui/events.rs`):
- Tab - Switch panes
- j/k or ↓/↑ - Navigate
- Space - Toggle action
- q - Quit

**Required Extensions for Phase 1**:

1. **Add expansion state to App** (`src/ui/app.rs`):
```rust
pub struct App {
    pub selected_outcome: usize,
    pub selected_action: usize,
    pub goals: DailyGoals,

    // NEW: Track expansion state
    pub expanded_outcome: Option<usize>,  // None = collapsed, Some(idx) = expanded
    pub editing_field: Option<EditingField>,  // Track which field is being edited
    pub confidence_values: [u8; 3],  // Work, Health, Family (0-100)
}

pub enum EditingField {
    NorthStar(usize),      // outcome index
    LeadMeasure(usize),
    TargetMetric(usize),
    Confidence(usize),
}
```

2. **Add expansion handlers** (`src/ui/events.rs`):
```rust
match key.code {
    KeyCode::Enter => {
        // Toggle expansion for selected outcome
        if app.expanded_outcome == Some(app.selected_outcome) {
            app.expanded_outcome = None;
        } else {
            app.expanded_outcome = Some(app.selected_outcome);
        }
    }
    KeyCode::Char('n') if app.expanded_outcome.is_some() => {
        // Enter North Star editing mode
        app.editing_field = Some(EditingField::NorthStar(app.selected_outcome));
    }
    // Similar for 'l', 'm', 'c' or '%'
}
```

3. **Render expanded card** (`src/ui/components.rs`):
```rust
fn render_outcome_card(outcome: &Outcome, expanded: bool, confidence: u8) -> Paragraph {
    if !expanded {
        // Collapsed: Single line summary
        format!("Work [1/3] • Lead: {} • Target: {} • Conf: {}%",
            outcome.lead_measure.as_deref().unwrap_or("—"),
            outcome.target_metric.as_deref().unwrap_or("—"),
            confidence)
    } else {
        // Expanded: Multi-line card
        format!(
            "Why: {}\nLead: {}    Target: {}\nConfidence: {}%",
            outcome.north_star.as_deref().unwrap_or("—"),
            outcome.lead_measure.as_deref().unwrap_or("—"),
            outcome.target_metric.as_deref().unwrap_or("—"),
            confidence
        )
    }
}
```

### 5. Inline Editor Implementations

**Required Editors**:

1. **Single-line editor** (for n, l keys) - Standard text input
2. **Target Metric wizard** (for m key) - 4-step process
3. **Confidence slider** (for c or % key) - 0-100 with arrow keys

**Pattern**: Use ratatui's input handling with state machine

**Example Single-Line Editor**:
```rust
// In src/ui/editor.rs (new file)
pub struct TextEditor {
    pub content: String,
    pub cursor_position: usize,
}

impl TextEditor {
    pub fn handle_key(&mut self, key: KeyEvent) -> EditResult {
        match key.code {
            KeyCode::Enter => EditResult::Save,
            KeyCode::Esc => EditResult::Cancel,
            KeyCode::Char(c) => {
                self.content.insert(self.cursor_position, c);
                self.cursor_position += 1;
                EditResult::Continue
            }
            KeyCode::Backspace => {
                if self.cursor_position > 0 {
                    self.content.remove(self.cursor_position - 1);
                    self.cursor_position -= 1;
                }
                EditResult::Continue
            }
            _ => EditResult::Continue,
        }
    }
}

pub enum EditResult {
    Continue,
    Save,
    Cancel,
}
```

**Target Metric Wizard State Machine**:
```rust
pub enum WizardStep {
    Label,           // "pages"
    Period,          // Toggle Weekly/Quarterly with w/q
    Value,           // "2"
    Unit,            // "week" or "quarter"
}

pub struct TargetMetricWizard {
    step: WizardStep,
    label: String,
    period: Period,
    value: String,
    unit: String,
}

// Result: "2 pages / week"
```

**Confidence Slider**:
```rust
pub struct ConfidenceSlider {
    pub value: u8,  // 0-100
}

impl ConfidenceSlider {
    pub fn handle_key(&mut self, key: KeyEvent) -> EditResult {
        match key.code {
            KeyCode::Left => { self.value = self.value.saturating_sub(5); EditResult::Continue }
            KeyCode::Right => { self.value = (self.value + 5).min(100); EditResult::Continue }
            KeyCode::Char(c) if c.is_ascii_digit() => {
                // Set to nearest 10: '7' -> 70%
                self.value = (c.to_digit(10).unwrap() as u8) * 10;
                EditResult::Continue
            }
            KeyCode::Enter => EditResult::Save,
            KeyCode::Esc => EditResult::Cancel,
            _ => EditResult::Continue,
        }
    }
}
```

## Code References

### Data Model
- `src/models.rs:87-92` - Outcome struct definition (extend here)
- `src/models.rs:94-105` - Default trait implementation (update here)
- `src/models.rs:27-46` - OutcomeType enum

### File I/O & Parsing
- `src/data.rs:58-234` - parse_markdown() main entry point
- `src/data.rs:73` - Optimal @meta injection point (after date parsing)
- `src/data.rs:85-105` - Outcome header parsing logic
- `src/data.rs:318-327` - extract_goal() helper (pattern to follow)
- `src/data.rs:237-276` - format_goals() serialization
- `src/data.rs:24-47` - Atomic write pattern (use for TOML)

### TUI Components
- `src/ui/app.rs` - Application state (add expansion state)
- `src/ui/events.rs` - Keyboard handling (add n/l/m/c keys)
- `src/ui/components.rs` - UI rendering (add expanded card)
- `src/tui.rs` - Main event loop

### Configuration
- `src/models.rs:108-126` - Config struct (reference for TOML pattern)
- `Cargo.toml` - Dependencies (add serde + toml)

## Architecture Insights

### 1. Backward Compatibility Pattern
The codebase consistently uses `Option<T>` for optional fields, with `Default` trait providing safe initialization. This pattern ensures:
- Old files with missing metadata load successfully (fields become `None`)
- New files only write fields that exist (skip `None` values)
- No migration needed for existing markdown files

**Example from `src/models.rs:89`**: `pub goal: Option<String>`

### 2. Atomic Write Pattern
All file writes use temp file + rename pattern (`src/data.rs:24-47`):
1. Create temp file with unique name (timestamp + PID)
2. Write content to temp file
3. Atomic rename to final filename
4. Cleanup temp file on error

**Why**: Prevents file corruption from concurrent writes or crashes mid-write

### 3. Separation of Concerns
- `models.rs` - Pure data structures, no I/O
- `data.rs` - File operations, no UI logic
- `ui/` - TUI components, no direct file access
- Clean boundaries enable testing each layer independently

### 4. Error Handling Convention
**Always use `Result<T>` with context**:
```rust
fs::read_to_string(path)
    .with_context(|| format!("Failed to read: {}", path.display()))?
```

**Never use `.unwrap()` or `.expect()` in production** (per CLAUDE.md best practices)

### 5. Regex-Based Parsing
The markdown parser uses regex for flexibility:
- Case-insensitive headers: `(?i)^##\s*(Work|Health|Family)`
- Optional goal extraction: `\(Goal:\s*([^)]+)\)`
- Checkbox detection: `^-\s*\[(x| )\]\s*(.+)$`

**Advantage**: Tolerates variations in user formatting

## Historical Context

### CLAUDE.md Best Practices (Referenced)
The codebase strictly follows documented best practices:

1. **Error Handling** - Always `Result<T>`, never unwrap/expect
2. **Safe Array Access** - Use helper functions, not direct indexing
3. **Borrow Checker** - Set values before taking mutable references
4. **Optional Fields** - Use `Option<T>` for backward compatibility
5. **Atomic Writes** - Prevent corruption with temp file pattern

**Source**: `/Users/petercorreia/conductor/focusfive/.conductor/porto/CLAUDE.md`

### Phase Status
According to CLAUDE.md and codebase analysis:
- ✅ **Phase 1 Complete**: Core data layer functional
- ✅ **Phase 2 Complete**: Terminal UI with ratatui (UNEXPECTED - ahead of schedule!)
- 🔄 **Phase 3 In Progress**: Adding metadata enhancements (current task)
- ⏳ **Phase 4 Planned**: Claude integration and polish

## Data Flow Diagram

```
User Input (n/l/m/c keys)
    ↓
Event Handler (ui/events.rs)
    ↓
App State Update (ui/app.rs)
    ↓
Modal Editor (ui/editor.rs - NEW)
    ↓
On Save:
    ├─→ Stable fields → outcomes.toml (TOML sidecar - NEW)
    │   └─→ OutcomesConfig::save() (outcome_metadata.rs - NEW)
    │
    └─→ Ephemeral confidence → daily markdown
        └─→ DailyGoals with @meta line
            └─→ save_goals() → atomic write (data.rs:24-47)

On Load:
    ├─→ Read ~/.focusfive/outcomes.toml
    │   └─→ OutcomesConfig::load() → populate Outcome fields
    │
    └─→ Read ~/FocusFive/goals/YYYY-MM-DD.md
        └─→ parse_markdown() → extract @meta confidence values
```

## Implementation Checklist

### Phase 1A: Data Layer (TOML + Models)
- [ ] Add `serde` and `toml` dependencies to Cargo.toml
- [ ] Create `src/outcome_metadata.rs` with OutcomesConfig
- [ ] Extend `Outcome` struct with 3 new fields (src/models.rs:91)
- [ ] Update `Default` impl (src/models.rs:94-105)
- [ ] Add TOML load/save functions with atomic writes
- [ ] Test: Round-trip TOML serialization

### Phase 1B: Markdown @meta Lines
- [ ] Add @meta regex parser to parse_markdown() (src/data.rs:73)
- [ ] Store confidence values in temporary HashMap during parsing
- [ ] Add confidence writing to format_goals() (src/data.rs:237)
- [ ] Test: Parse file with @meta lines
- [ ] Test: Write file with @meta lines
- [ ] Test: Backward compatibility (files without @meta)

### Phase 1C: TUI Expansion State
- [ ] Add `expanded_outcome: Option<usize>` to App struct
- [ ] Add `confidence_values: [u8; 3]` to App struct
- [ ] Handle Enter key to toggle expansion (ui/events.rs)
- [ ] Render collapsed summary line (ui/components.rs)
- [ ] Render expanded card (4 lines max) (ui/components.rs)
- [ ] Test: Expansion preserves keyboard focus

### Phase 1D: Inline Editors
- [ ] Create `src/ui/editor.rs` module
- [ ] Implement TextEditor for single-line (n, l keys)
- [ ] Implement TargetMetricWizard (m key) with 4-step state machine
- [ ] Implement ConfidenceSlider (c, % keys) with arrow keys
- [ ] Wire editors to event handler (ui/events.rs)
- [ ] Test: Save updates TOML file
- [ ] Test: Esc cancels without saving

### Phase 1E: Integration & Polish
- [ ] Load outcomes.toml on app startup
- [ ] Merge TOML data into Outcome structs
- [ ] Update status line with context hints when expanded
- [ ] Verify atomic writes never corrupt files
- [ ] Test: Graceful fallback if TOML unreadable
- [ ] Test: Confidence slider rejects >100 or non-numeric
- [ ] Update CLAUDE.md with new architecture

### Phase 1F: Edge Cases
- [ ] Test: Missing outcomes.toml (should create default)
- [ ] Test: Malformed TOML (should show "—" and allow editing)
- [ ] Test: Confidence value >100 (should clamp to 100)
- [ ] Test: Confidence non-numeric input (show error, keep old value)
- [ ] Test: Concurrent edits to different outcomes (TOML atomic write)
- [ ] Test: Keyboard focus returns after modal closes

## Acceptance Criteria

From the Phase 1 specification:

### Functional Requirements
- ✅ Users can see/edit 4 fields in ≤10 seconds without leaving dashboard
- ✅ Collapsed rows remain minimal (single line summary)
- ✅ Expanded cards are 2-4 lines only
- ✅ All edits backward compatible with existing files
- ✅ Graceful degradation if metadata missing (show "—")

### Interaction Requirements
- ✅ Enter toggles expansion
- ✅ n/l single-line editors (Enter saves, Esc cancels)
- ✅ m 4-step wizard (label, period w/q toggle, value, unit)
- ✅ c or % slider (←/→ ±5, number keys for 10s, Enter saves)

### Persistence Requirements
- ✅ Stable fields (North Star, Lead, Target) in outcomes.toml
- ✅ Ephemeral Confidence in @meta line of daily markdown
- ✅ All fields optional in OutcomeMetadata (Default::default())
- ✅ Legacy files load successfully

### Edge Cases
- ✅ Expanding/editing never corrupts files (atomic writes)
- ✅ Unreadable TOML shows missing values, allows recreation
- ✅ Confidence limited 0-100, non-numeric rejected
- ✅ Keyboard focus preserved after modals

## Open Questions

1. **Target Metric Format**: Should we validate the structure "N unit / period" or allow freeform text?
   - Recommendation: Start with freeform, add validation later if needed
   - Example: "2 pages / week" or "30 minutes / day"

2. **TOML Migration**: What if user hand-edits outcomes.toml and breaks format?
   - Recommendation: Show error message, offer to recreate from scratch
   - Preserve backup with `.bak` extension before overwriting

3. **Confidence History**: Should we track confidence over time (trends)?
   - Recommendation: Phase 2 feature - for now, just today's value
   - Could add "confidence_history" in future for analytics

4. **Multi-line North Star**: Should North Star support multiple lines?
   - Recommendation: No, keep single line (max 500 chars like actions)
   - Forces clarity and conciseness

5. **Keyboard Shortcut Conflicts**: Does 'c' conflict with any existing keys?
   - Investigation: Check ui/events.rs for conflicts
   - Fallback: Use '%' as alternative (semantic match to percentage)

6. **Default Values**: What should Default::default() provide for new fields?
   - Recommendation: All None (empty) - user must set explicitly
   - Avoids assumptions about user's goals

## Related Research

- CLAUDE.md best practices at `/Users/petercorreia/conductor/focusfive/.conductor/porto/CLAUDE.md`
- USER_GUIDE.md for user-facing documentation (if exists)

## Next Steps

1. **Start with Data Layer**: Implement TOML infrastructure and model extensions first
2. **Add @meta Parsing**: Extend markdown parser for ephemeral confidence
3. **Build TUI Expansion**: Add Enter key handler and multi-line rendering
4. **Implement Editors**: Create modal editors for each field type
5. **Test Edge Cases**: Verify backward compatibility and error handling
6. **Update Documentation**: Add architecture diagrams to CLAUDE.md

**Estimated Implementation Time**: 6-8 hours for complete Phase 1

**Critical Path**: TOML infrastructure → Data model → TUI expansion → Inline editors

**Risk Areas**:
- TOML parsing errors (mitigate with Default::default())
- Concurrent edits to outcomes.toml (mitigate with atomic writes)
- Regex pattern conflicts in @meta parsing (test thoroughly)