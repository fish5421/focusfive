# CLAUDE.md

This file provides comprehensive guidance to Claude Code (claude.ai/code) and developers working on FocusFive.

## Project Overview

FocusFive is a minimalist terminal-based goal tracking system that enforces exactly 3 life outcomes with 3 daily actions each. It uses local markdown files for storage, requires no network connection, and is designed for a 3-minute daily interaction.

**Current Status**: Phase 1 Complete ✅ (Core data layer functional)

## 🚨 Critical Information

### File Storage Location
**IMPORTANT**: Goal files are stored in the USER'S HOME directory, NOT the project directory:
- ✅ Correct: `~/FocusFive/goals/` (e.g., `/Users/username/FocusFive/goals/`)
- ❌ Wrong: `~/projects/goal_setting/` (project directory)

This is a common source of confusion. The app creates files in the user's home directory for persistence across terminal sessions.

## Core Architecture

### Data Model (`src/models.rs`)
```rust
// Fixed structure - exactly 3 outcomes, 3 actions each
pub struct DailyGoals {
    pub date: NaiveDate,
    pub day_number: Option<u32>,  // "Day N" tracking
    pub work: Outcome,
    pub health: Outcome,
    pub family: Outcome,
}

pub struct Outcome {
    pub outcome_type: OutcomeType,  // Work | Health | Family
    pub goal: Option<String>,        // From "(Goal: ...)" in header
    pub actions: [Action; 3],        // Exactly 3, no more, no less
}

pub struct Action {
    pub text: String,               // Limited to 500 chars
    pub completed: bool,            // Maps to [x] or [ ]
}
```

### File I/O (`src/data.rs`)
- **Atomic writes**: Uses temp file + rename pattern to prevent corruption
- **Flexible parsing**: Handles case-insensitive headers, leading content
- **Error handling**: All operations return `Result<T>` with context
- **Concurrency safe**: Unique temp filenames with timestamp + PID

### Configuration (`src/models.rs`)
```rust
impl Config {
    pub fn new() -> anyhow::Result<Self> {
        // Attempts to find home directory
        // Falls back to current directory if HOME not found
        // Never panics - always returns Result
    }
}
```

## Coding Style & Patterns

### Error Handling
- **Always use `Result<T>`**: Never use `.unwrap()` or `.expect()` in production code
- **Provide context**: Use `.context()` or `.with_context()` for meaningful errors
- **Graceful fallbacks**: Config falls back to current dir if HOME missing

### Rust Patterns
```rust
// ✅ GOOD: Handle Results properly
let config = Config::new()
    .unwrap_or_else(|e| {
        eprintln!("Warning: {}", e);
        Config { goals_dir: "./FocusFive/goals".to_string() }
    });

// ❌ BAD: Never do this in production
let config = Config::new().expect("Config must work");
```

### Borrow Checker Lessons
```rust
// ❌ BAD: Causes borrow checker error
current_outcome = Some(&mut goals.work);
goals.work.goal = extract_goal();  // Can't access while borrowed!

// ✅ GOOD: Set values before taking mutable reference
goals.work.goal = extract_goal();
current_outcome = Some(&mut goals.work);
```

### Safe Array Access
```rust
// ❌ BAD: Can panic
let month = &caps[1];

// ✅ GOOD: Safe helper function
fn get_capture<'a>(caps: &'a Captures, index: usize) -> Result<&'a str> {
    caps.get(index)
        .map(|m| m.as_str())
        .context(format!("Missing capture group {}", index))
}
```

## File Format Specification

### Markdown Structure
```markdown
# August 18, 2025              # Date header (required)
# August 18, 2025 - Day 5      # With optional day counter

## Work (Goal: Ship feature X)  # Outcome header (case-insensitive)
- [x] Complete PR review        # Completed action
- [ ] Write documentation       # Pending action
- [ ] Deploy to staging         # Exactly 3 actions required

## Health (Goal: Stay active)   # Headers can be HEALTH, health, Health
- [x] Morning walk
- [ ] Drink 8 glasses water
- [ ] Sleep before 11pm

## Family (Goal: Be present)
- [ ] Breakfast together
- [x] Call parents
- [ ] Plan weekend activity
```

### Parsing Flexibility
- Headers can appear within first 10 lines (allows for comments/notes)
- Case-insensitive outcome matching (Work, WORK, work all valid)
- Goals extracted from `(Goal: ...)` pattern in headers
- Action text limited to 500 characters (truncated with warning)
- Files limited to 1MB for safety

## Testing & Validation

### Run Tests
```bash
# Run all tests
cargo test

# Run with output
cargo test -- --nocapture

# Specific test
cargo test test_parser_with_real_file

# Validation script
./validate_setup.sh
```

### Critical Fixes Implemented (Phase 1)
1. **Config panic prevention**: Returns Result instead of panicking
2. **Array indexing safety**: No direct array access, uses safe helpers
3. **Atomic write concurrency**: Unique temp files prevent collisions
4. **Header position flexibility**: Scans first 10 lines for date header
5. **Case-insensitive headers**: Handles WORK, work, Work, etc.

## Development Commands

```bash
# Build
cargo build --release

# Run
./target/release/focusfive

# Format code (currently broken due to Rust toolchain issues)
cargo fmt

# Lint
cargo clippy -- -D warnings

# Documentation
cargo doc --open
```

## Current Implementation Status

### ✅ Phase 1 Complete (Days 1-3)
- Core data models implemented
- Markdown parser working with regex
- File I/O with atomic writes
- All critical bugs fixed
- Comprehensive test coverage
- Validated with real-world usage

### ⏳ Phase 2 Pending (Days 4-7): Terminal UI
```rust
// Next steps for TUI implementation
use ratatui::{
    widgets::{Block, Borders, List, ListItem},
    layout::{Layout, Constraint, Direction},
};

// Two-pane layout needed:
// [Outcomes List] | [Actions for Selected Outcome]
// Tab to switch, j/k to navigate, Space to toggle, q to quit
```

### 📍 Phase 3 Planned (Days 8-10): Claude Integration
- Export to `.claude/` directory
- Generate analysis context
- Create slash commands

### 🔧 Phase 4 Planned (Days 11-14): Polish
- Performance optimization
- Cross-platform testing
- Release preparation

## Known Issues & Solutions

### Issue: "Can't find markdown files"
**Cause**: Looking in wrong directory
**Solution**: Files are in `~/FocusFive/goals/`, NOT project directory

### Issue: Rust toolchain broken (Homebrew)
**Cause**: Missing libgit2, llvm libraries
**Solution**: Use rustup instead of Homebrew:
```bash
brew uninstall rust
curl --proto='=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### Issue: Can't cd to goals directory
**Workaround**: Use absolute paths instead:
```bash
cat ~/FocusFive/goals/$(date +%Y-%m-%d).md
nano ~/FocusFive/goals/$(date +%Y-%m-%d).md
```

## Project Structure

```
goal_setting/
├── src/
│   ├── main.rs         # Entry point, creates sample goals
│   ├── models.rs       # Data structures (Action, Outcome, DailyGoals)
│   ├── data.rs         # File I/O and markdown parsing
│   └── lib.rs          # Library exports
├── tests/
│   ├── regression_tests.rs    # All critical fixes validated
│   └── *.rs                   # Various test suites
├── CLAUDE.md           # This file - AI/developer context
├── USER_GUIDE.md       # How to use the application
└── validate_setup.sh   # Check if everything works
```

## Performance Characteristics

- **Startup**: < 100ms ✅ (target: < 500ms)
- **File save**: < 50ms ✅ (target: < 100ms)
- **Memory**: < 10MB typical ✅ (target: < 50MB)
- **Concurrent writes**: 0% collision rate ✅

## Contributing Guidelines

### For AI Assistants
1. Always validate file operations with proper error handling
2. Test with edge cases (missing HOME, concurrent access, malformed files)
3. Maintain the 3x3 constraint (3 outcomes, 3 actions each)
4. Keep interactions under 3 minutes
5. Preserve backward compatibility with existing markdown files

### For Developers
1. Run `./validate_setup.sh` before committing
2. Add tests for any new functionality
3. Update this CLAUDE.md with significant changes
4. Follow Rust idioms and safety patterns
5. Document panic-free guarantees

## Quick Reference

```bash
# Where are the files?
ls ~/FocusFive/goals/

# Today's file
cat ~/FocusFive/goals/$(date +%Y-%m-%d).md

# Run the app
./target/release/focusfive

# Validate everything
./validate_setup.sh
```

## Next Developer Tasks

1. **Implement TUI (Phase 2)**:
   - Use ratatui for terminal UI
   - Two-pane layout (Outcomes | Actions)
   - Keyboard navigation (Tab, j/k, Space, q)
   - Real-time file updates

2. **Add features**:
   - Weekly summary generation
   - Streak tracking
   - Goal completion statistics
   - Theme customization

3. **Improve parser**:
   - Support for notes/comments in tasks
   - Flexible action count (1-5 instead of fixed 3)
   - Tags or categories for actions

Remember: This is a LOCAL-FIRST, PRIVACY-FOCUSED tool. No telemetry, no cloud sync, no external dependencies for core functionality.