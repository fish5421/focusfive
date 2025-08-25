# FocusFive Developer Documentation

## Project Overview

FocusFive is a terminal-based goal tracking application written in Rust. It provides an interactive Terminal User Interface (TUI) for managing daily goals across three life outcomes: Work, Health, and Family. The application supports variable actions (1-5 per outcome), templates, visions, and ritual phases for morning planning and evening reflection.

## Architecture

The application follows a Model-View-Controller pattern:
- **Models** (`models.rs`): Data structures for goals, outcomes, actions
- **View** (`ui.rs`): Terminal UI rendering using ratatui
- **Controller** (`app.rs`): Application state and event handling
- **Data Layer** (`data.rs`): File I/O and markdown parsing
- **Entry Point** (`main.rs`): Event loop and initialization

## File Structure

```
src/
├── main.rs         # Application entry point and event loop
├── app.rs          # Application state management and input handling
├── ui.rs           # Terminal UI rendering and layout
├── models.rs       # Data structures and configuration
├── data.rs         # File I/O, markdown parsing, and persistence
└── lib.rs          # Library exports for testing
```

## Key Technologies

- **Language**: Rust
- **TUI Framework**: ratatui (terminal UI)
- **Cross-platform Terminal**: crossterm
- **Date Handling**: chrono
- **Serialization**: serde_json
- **Error Handling**: anyhow
- **Regex**: For markdown parsing

## Core Concepts

### 1. Variable Actions System
Unlike fixed 3-action systems, FocusFive supports 1-5 actions per outcome, allowing flexibility based on daily needs.

### 2. Ritual Phases
- **Morning Phase**: Set intentions, apply templates, plan the day
- **Evening Phase**: Quick completion with letter keys, reflection, review stats

### 3. Two-Pane Layout
- Left pane: Outcomes (Work, Health, Family) with progress indicators
- Right pane: Actions for selected outcome with checkboxes

### 4. Templates
Save and reuse common action sets for different types of days (e.g., "Deep Work Day", "Meeting Heavy").

### 5. Goals & Visions
- **Goals**: Short-term objectives (100 char max) shown in headers
- **Visions**: Long-term aspirations for each outcome

## File-by-File Documentation

---

## `src/main.rs` - Application Entry Point

**Purpose**: Initializes the application, sets up the terminal, and runs the main event loop.

**Key Components**:
- Terminal initialization with alternate screen
- Event loop that handles keyboard input
- Graceful shutdown and cleanup
- Auto-save on quit

**Main Flow**:
1. Load configuration and create directories
2. Load or create today's goals file
3. Initialize terminal UI
4. Run event loop (handle keys, update UI, render)
5. Save changes on exit

**Key Functions**:
- `main()`: Entry point, sets up and runs the app
- Event loop: Processes keyboard events and updates UI

---

## `src/app.rs` - Application State & Logic

**Purpose**: Manages application state, handles user input, and coordinates between data and UI layers.

**Key Structures**:
- `App`: Main application state containing goals, UI state, and configuration
- `InputMode`: Enum for different input modes (Normal, Editing, VisionEditing, etc.)
- `Pane`: Active pane (Outcomes or Actions)
- `RitualPhase`: Morning, Day, or Evening phase

**Major Features Implemented**:
1. **Goal Editing** (`handle_goal_edit_mode`): Edit outcome goals with 'g' key
2. **Action Management**: Add (up to 5) and delete (min 1) actions
3. **Template System**: Save/load action templates
4. **Vision Editing**: Multi-line vision text for each outcome
5. **Reflection Mode**: End-of-day reflection writing
6. **Quick Complete**: Evening phase letter-key completion

**Input Handlers**:
- `handle_normal_mode()`: Navigation and commands
- `handle_edit_mode()`: Action text editing
- `handle_vision_edit_mode()`: Multi-line vision editing
- `handle_goal_edit_mode()`: Goal text editing
- `handle_reflection_mode()`: Reflection writing

**Save System**:
- Multiple save key combinations (F2, Ctrl+S, Cmd+Enter)
- Auto-save on certain operations
- Manual save with 's' key

---

## `src/ui.rs` - Terminal UI Rendering

**Purpose**: Renders the terminal interface using ratatui, creating the visual layout and components.

**Layout Structure**:
```
┌─────────────────┬──────────────────┐
│  OUTCOMES       │  ACTIONS         │
├─────────────────┼──────────────────┤
│ > Work [2/4]    │  [x] Task 1      │
│   Health [1/3]  │  [ ] Task 2      │
│   Family [0/2]  │  ...             │
└─────────────────┴──────────────────┘
```

**Key Functions**:
- `ui()`: Main render function, coordinates all UI elements
- `render_outcomes()`: Left pane with outcome list
- `render_actions()`: Right pane with action checkboxes
- `render_goal_editor()`: Modal for goal editing
- `render_vision_editor()`: Modal for vision editing
- `render_reflection_editor()`: Reflection input modal
- `render_help()`: Help screen with keyboard shortcuts
- `render_info()`: Green info notifications
- `render_error()`: Red error dialogs

**Dynamic Elements**:
- Progress indicators update based on actual action count
- Template options show available templates (1-9)
- Evening phase shows letter keys for quick complete
- Status bar changes based on current mode

---

## `src/models.rs` - Data Structures

**Purpose**: Defines all data structures used throughout the application.

**Core Structures**:

```rust
pub struct DailyGoals {
    pub date: NaiveDate,
    pub day_number: Option<u32>,
    pub work: Outcome,
    pub health: Outcome,
    pub family: Outcome,
}

pub struct Outcome {
    pub outcome_type: OutcomeType,
    pub goal: Option<String>,        // "Ship feature X"
    pub actions: Vec<Action>,        // 1-5 actions
}

pub struct Action {
    pub text: String,
    pub completed: bool,
}
```

**Configuration**:
- `Config`: Manages file paths and directories
- Falls back gracefully if HOME directory not found

**Helper Structures**:
- `CompletionStats`: Tracks completion percentages
- `OutcomeType`: Enum for Work/Health/Family

---

## `src/data.rs` - File I/O & Persistence

**Purpose**: Handles all file operations, markdown parsing, and data persistence.

**Key Features**:
1. **Markdown Parser**: Regex-based parser for goal files
2. **Atomic Writes**: Temp file + rename pattern for safety
3. **Flexible Parsing**: Handles various markdown formats
4. **Vision/Reflection Storage**: Separate files for each
5. **Template Management**: JSON-based template storage

**Main Functions**:
- `read_goals_file()`: Parse markdown into DailyGoals
- `write_goals_file()`: Convert DailyGoals to markdown
- `parse_markdown()`: Core parsing logic with regex
- `save_vision()`: Store vision text
- `load_vision()`: Retrieve vision text
- `save_reflection()`: Store daily reflection
- `load_reflection()`: Retrieve reflection

**File Locations**:
- Goals: `~/FocusFive/goals/YYYY-MM-DD.md`
- Visions: `~/FocusFive/visions/outcome_YYYY-MM-DD.txt`
- Reflections: `~/FocusFive/reflections/YYYY-MM-DD.txt`
- Templates: `~/.focusfive_templates.json`

---

## `src/lib.rs` - Library Interface

**Purpose**: Exports public modules for testing and external use.

```rust
pub mod models;
pub mod data;
```

Simple library interface that makes the core modules available for integration testing.

---

## Enhancement Opportunities

### For Intermediate Developers

1. **Add Persistence for Settings**
   - Create a settings file for user preferences
   - Save theme, default action count, etc.

2. **Implement Statistics View**
   - Weekly/monthly completion rates
   - Streak tracking
   - Pattern analysis

3. **Add Export Features**
   - Export to CSV/JSON
   - Generate weekly reports
   - Email summaries

4. **Enhance Templates**
   - Template categories
   - Smart template suggestions
   - Template sharing

5. **Add Keyboard Macros**
   - Record and replay common operations
   - Custom key bindings

6. **Implement Themes**
   - Color scheme customization
   - Font size options (if terminal supports)

7. **Add Search Functionality**
   - Search through past goals
   - Find patterns in completions

8. **Cloud Sync (Optional)**
   - Encrypted backup to cloud
   - Multi-device sync

## Development Setup

```bash
# Clone the repository
git clone <repository>
cd goal_setting

# Build the project
cargo build --release

# Run tests
cargo test

# Run the application
./target/release/focusfive
```

## Testing Strategy

1. **Unit Tests**: Test individual functions in data.rs and models.rs
2. **Integration Tests**: Test file I/O and parsing with real files
3. **UI Tests**: Manual testing of keyboard interactions
4. **Regression Tests**: Ensure fixes stay fixed

## Common Patterns

### Error Handling
```rust
// Always use Result<T> with context
operation()
    .context("Failed to perform operation")?
```

### File Operations
```rust
// Atomic writes pattern
let temp_path = format!("{}.tmp", path);
fs::write(&temp_path, content)?;
fs::rename(temp_path, path)?;
```

### UI Updates
```rust
// Pattern for UI updates
self.needs_save = true;
self.info_message = Some("Saved".to_string());
```

## Debugging Tips

1. **Check file locations**: Goals are in `~/FocusFive/goals/`, not project directory
2. **Use `--nocapture`**: Run tests with output visible
3. **Check regexes**: Most parsing issues are regex-related
4. **Terminal compatibility**: Some keys (Cmd+Enter) don't work in all terminals

## Code Style Guidelines

- Use `Result<T>` everywhere, never `.unwrap()` in production
- Provide meaningful error contexts with `.context()`
- Keep functions focused and under 50 lines when possible
- Comment complex regex patterns
- Use descriptive variable names
- Follow Rust naming conventions (snake_case for functions/variables)

## Performance Considerations

- File I/O is the main bottleneck
- Regex compilation is cached where possible
- UI rendering is efficient (only redraws on change)
- Memory usage is minimal (< 10MB typical)

## Security Considerations

- No network access required
- All data stored locally
- No telemetry or tracking
- Atomic writes prevent corruption
- Input validation on all user text

---

## Contact & Support

For questions about the codebase or architecture decisions, refer to:
- `CLAUDE.md` - AI assistant context
- `USER_GUIDE.md` - User documentation
- `tests/` - Example usage patterns

This documentation provides the foundation for understanding and enhancing FocusFive. The codebase is well-structured and ready for additional features while maintaining its core philosophy of simplicity and local-first operation.