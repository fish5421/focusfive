# FocusFive Data Model Documentation

## Overview

FocusFive uses a hierarchical data model centered around daily goals with exactly three life outcomes (Work, Health, Family), each containing 1-5 variable actions. The model enforces business constraints at the type level where possible and includes support for templates, visions, and ritual phases.

## Core Data Structures

### 1. Action
**Purpose**: Represents a single task/action item with completion status

```rust
pub struct Action {
    pub text: String,        // Action description (max 500 chars)
    pub completed: bool,     // Completion status
}
```

**Constraints**:
- Text limited to 500 characters (MAX_ACTION_LENGTH)
- Automatically truncated if exceeds limit
- Default: empty text, not completed

---

### 2. OutcomeType
**Purpose**: Enum defining the three fixed life areas

```rust
pub enum OutcomeType {
    Work,
    Health,
    Family,
}
```

**Constraints**:
- EXACTLY 3 variants (enforced at compile-time)
- Cannot be extended without code changes
- String representation: "Work", "Health", "Family"

---

### 3. Outcome
**Purpose**: Container for one life area with its goal, actions, and reflection

```rust
pub struct Outcome {
    pub outcome_type: OutcomeType,      // Which life area
    pub goal: Option<String>,            // Optional daily goal (max 100 chars)
    pub actions: Vec<Action>,            // Variable 1-5 actions
    pub reflection: Option<String>,      // Evening reflection note
}
```

**Constraints**:
- Actions: minimum 1, maximum 5
- Goal: optional, max 100 characters
- Reflection: optional, no length limit specified
- Default: 3 empty actions (backward compatibility)

**Methods**:
- `add_action()` - Add up to 5 total
- `remove_action()` - Remove down to 1 minimum
- `count_completed()` - Count finished actions
- `completion_percentage()` - Calculate progress (0-100)

---

### 4. DailyGoals
**Purpose**: Root structure containing date and all three outcomes

```rust
pub struct DailyGoals {
    pub date: NaiveDate,              // The date these goals are for
    pub day_number: Option<u32>,      // Optional "Day N" tracking
    pub work: Outcome,                // Work outcome
    pub health: Outcome,              // Health outcome  
    pub family: Outcome,              // Family outcome
}
```

**Constraints**:
- EXACTLY 3 outcomes (one per OutcomeType)
- Date required
- Day number optional (for streak tracking)

**Methods**:
- `outcomes()` - Get array of all 3 outcomes
- `outcomes_mut()` - Get mutable array
- `completion_stats()` - Calculate statistics

---

### 5. FiveYearVision
**Purpose**: Long-term vision text for each life area

```rust
pub struct FiveYearVision {
    pub work: String,            // Work vision (max 1000 chars)
    pub health: String,          // Health vision (max 1000 chars)
    pub family: String,          // Family vision (max 1000 chars)
    pub created: NaiveDate,      // When created
    pub modified: NaiveDate,     // Last modified
}
```

**Constraints**:
- Each vision max 1000 characters (MAX_VISION_LENGTH)
- Automatically tracks creation/modification dates

---

### 6. ActionTemplates
**Purpose**: Reusable sets of actions for quick daily setup

```rust
pub struct ActionTemplates {
    pub templates: HashMap<String, Vec<String>>,  // Name -> action texts
    pub created: NaiveDate,                       // When created
    pub modified: NaiveDate,                      // Last modified
}
```

**Constraints**:
- Each template limited to 3 actions max
- Action text limited to 500 chars each
- Template names must be unique (HashMap key)

---

### 7. CompletionStats
**Purpose**: Calculated statistics for progress tracking

```rust
pub struct CompletionStats {
    pub completed: usize,                        // Total completed actions
    pub total: usize,                           // Total actions
    pub percentage: u16,                        // Overall percentage (0-100)
    pub by_outcome: Vec<(String, usize, usize)>, // Per-outcome stats
    pub streak_days: Option<u32>,               // Current streak
    pub best_outcome: Option<String>,           // Highest completion %
    pub needs_attention: Vec<String>,           // Outcomes < 50%
}
```

**Calculated Fields**:
- Derived from DailyGoals at runtime
- Not persisted to disk

---

### 8. RitualPhase
**Purpose**: Time-based UI modes for morning/evening rituals

```rust
pub enum RitualPhase {
    Morning,  // 5am-12pm: Set intentions
    Evening,  // 5pm-11pm: Reflect and review
    None,     // Other times: Normal mode
}
```

**Time Windows**:
- Morning: 5:00 AM - 11:59 AM
- Evening: 5:00 PM - 10:59 PM
- None: All other times

---

### 9. Config
**Purpose**: Application configuration

```rust
pub struct Config {
    pub goals_dir: String,  // Directory path for goal files
}
```

**Default Path**:
- Primary: `~/FocusFive/goals/`
- Fallback: `./FocusFive/goals/` (if home not found)

---

## UI-Specific Structures

### 10. Pane
**Purpose**: Active pane in two-pane layout

```rust
pub enum Pane {
    Outcomes,  // Left pane
    Actions,   // Right pane
}
```

---

### 11. InputMode
**Purpose**: Current input/editing state

```rust
pub enum InputMode {
    Normal,
    Editing { buffer: String, original: String },
    GoalEditing { outcome_type: OutcomeType, buffer: String, original: String },
    VisionEditing { outcome_type: OutcomeType, buffer: String, original: String },
    CopyingFromYesterday { /* fields */ },
    TemplateSelection { /* fields */ },
    TemplateSaving { /* fields */ },
    Reflecting { /* fields */ },
}
```

---

### 12. App (Application State)
**Purpose**: Complete application state

```rust
pub struct App {
    // Core data
    pub goals: DailyGoals,
    pub vision: FiveYearVision,
    pub templates: ActionTemplates,
    pub config: Config,
    
    // UI state
    pub active_pane: Pane,
    pub outcome_index: usize,
    pub action_index: usize,
    pub input_mode: InputMode,
    
    // Flags
    pub should_quit: bool,
    pub needs_save: bool,
    pub vision_needs_save: bool,
    pub templates_needs_save: bool,
    pub show_help: bool,
    pub show_morning_prompt: bool,
    pub confirm_delete: bool,
    
    // Messages
    pub error_message: Option<String>,
    pub info_message: Option<String>,
    
    // Ritual phase data
    pub ritual_phase: RitualPhase,
    pub yesterday_context: Option<DailyGoals>,
    pub completion_stats: Option<CompletionStats>,
    pub daily_summary: String,
    
    // Tracking
    pub current_streak: u32,
}
```

---

## Data Relationships

```
DailyGoals (1)
    ├── date: NaiveDate
    ├── day_number: Option<u32>
    └── outcomes (3)
        ├── Work: Outcome
        ├── Health: Outcome
        └── Family: Outcome
            ├── outcome_type: OutcomeType
            ├── goal: Option<String>
            ├── reflection: Option<String>
            └── actions: Vec<Action> (1-5)
                ├── text: String
                └── completed: bool

FiveYearVision (1)
    ├── work: String
    ├── health: String
    ├── family: String
    ├── created: NaiveDate
    └── modified: NaiveDate

ActionTemplates (1)
    ├── templates: HashMap<String, Vec<String>>
    ├── created: NaiveDate
    └── modified: NaiveDate

CompletionStats (calculated)
    ├── completed: usize
    ├── total: usize
    ├── percentage: u16
    ├── by_outcome: Vec<(String, usize, usize)>
    ├── streak_days: Option<u32>
    ├── best_outcome: Option<String>
    └── needs_attention: Vec<String>
```

## Business Rules & Constraints

### Invariants (Cannot be violated)
1. **Exactly 3 Outcomes**: Every DailyGoals has Work, Health, and Family
2. **OutcomeType Enum**: Only 3 possible values
3. **Action Count**: 1-5 actions per outcome
4. **Text Length Limits**:
   - Action text: 500 chars
   - Goal text: 100 chars
   - Vision text: 1000 chars

### Validation Rules
1. Actions cannot be empty (min 1 per outcome)
2. Actions cannot exceed 5 per outcome
3. Text automatically truncated if exceeds limits
4. Dates must be valid NaiveDate values

### Default Values
- New Outcome: 3 empty actions
- New Action: empty text, not completed
- New Vision: empty strings, today's date
- New Templates: empty HashMap
- Config: Home directory or current directory

## File Storage Structure

### Directory Layout
```
~/FocusFive/
├── goals/
│   ├── 2025-01-17.md      # Daily goal files (YYYY-MM-DD.md)
│   ├── 2025-01-18.md
│   └── ...
├── visions/
│   └── vision.json         # Five-year vision
├── reflections/
│   └── 2025-01-17.txt      # Daily reflections
└── templates/
    └── templates.json      # Action templates
```

### File Formats

**Goals (Markdown)**:
```markdown
# January 17, 2025 - Day 5

## Work (Goal: Ship feature X)
- [x] Complete PR review
- [ ] Write documentation
- [ ] Deploy to staging
- [ ] Fix critical bug

## Health (Goal: Stay active)
- [x] Morning walk
- [ ] Drink 8 glasses water

## Family (Goal: Be present)
- [ ] Breakfast together
- [x] Call parents
- [ ] Plan weekend activity
```

**Templates (JSON)**:
```json
{
  "templates": {
    "Deep Work Day": ["Focus session", "Code review", "Documentation"],
    "Meeting Day": ["Prep notes", "Attend meetings", "Follow-ups"]
  },
  "created": "2025-01-17",
  "modified": "2025-01-17"
}
```

## Type Safety Features

### Compile-Time Guarantees
- OutcomeType cannot have invalid values
- DailyGoals always has all 3 outcomes
- Serialization/deserialization maintains structure

### Runtime Validation
- Text length enforcement
- Action count limits (1-5)
- Date validation
- Template limits

## Memory Characteristics
- Action: ~100 bytes
- Outcome: ~500 bytes
- DailyGoals: ~2KB
- OutcomeType: 1 byte (enum)
- Typical session: <10MB RAM

---

This data model provides a type-safe, constraint-enforcing foundation for the FocusFive application while maintaining flexibility through variable actions and optional fields.