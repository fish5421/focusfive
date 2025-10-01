---
date: 2025-09-24T13:48:43Z
researcher: Claude Code
git_commit: 7fa362ba5e4919ded7657d932e66cdbc1773c83c
branch: feature/dashboard-redesign
repository: goal_setting
topic: "Comprehensive FocusFive Application Guide for Users and Developers"
tags: [research, codebase, focusfive, application-guide, user-guide, developer-guide]
status: complete
last_updated: 2025-09-24
last_updated_by: Claude Code
---

# Research: Comprehensive FocusFive Application Guide for Users and Developers

**Date**: 2025-09-24T13:48:43Z
**Researcher**: Claude Code
**Git Commit**: 7fa362ba5e4919ded7657d932e66cdbc1773c83c
**Branch**: feature/dashboard-redesign
**Repository**: goal_setting

## Research Question
Generate a comprehensive markdown document that explains the FocusFive application in enough detail for new users to understand what it's used for and make the most use out of it, and for intermediate developers to come into the codebase and make changes.

## Summary
FocusFive is a sophisticated terminal-based goal tracking system that enforces a 3x3 structure (3 life outcomes with 3 daily actions each) using local markdown files. Currently at Phase 2 completion, it features a full Terminal User Interface (TUI), real-time progress tracking, advanced widgets, and a comprehensive data architecture designed for 3-minute daily interactions.

# 🎯 FocusFive: The Complete Application Guide

## What is FocusFive?

FocusFive is a **minimalist terminal-based goal tracking system** that helps you maintain laser focus on what matters most in your life. Built on the philosophy that **constraint breeds clarity**, it enforces exactly **3 life outcomes with 3 daily actions each**.

### Core Philosophy
- **3 Outcomes**: Focus on exactly three life areas (typically Work, Health, Family/Personal)
- **3 Actions per Outcome**: Maximum 3 concrete actions per area each day
- **3-Minute Habit**: Daily tracking takes less than 3 minutes total
- **Local-First**: Your data stays on your machine, stored as human-readable markdown files
- **Zero Configuration**: Works immediately after installation

### Why FocusFive Exists
Most productivity tools overwhelm you with options. FocusFive does the opposite - it constrains your choices to force clarity and prevent decision paralysis. By limiting yourself to 9 total daily actions across 3 life areas, you're forced to choose what truly matters.

---

## 🚀 Getting Started

### Installation & First Run

```bash
# 1. Build the application (one-time setup)
git clone https://github.com/YOUR_USERNAME/goal_setting.git
cd goal_setting
cargo build --release

# 2. Run FocusFive (creates your first goal file)
./target/release/focusfive

# 3. Verify setup
./validate_setup.sh
```

**Important**: Your goal files are stored in `~/FocusFive/goals/`, NOT in the project directory.

### Your First Day with FocusFive

When you first run FocusFive, you'll see a **two-pane Terminal User Interface**:

```
┌─────────────────────────┬──────────────────────────────────┐
│  OUTCOMES               │  ACTIONS                         │
├─────────────────────────┼──────────────────────────────────┤
│ > Work [0/3]            │  Work Actions:                   │
│   Health [0/3]          │  [ ] Write project proposal      │
│   Family [0/3]          │  [ ] Review team feedback        │
│                         │  [ ] Deploy staging build        │
│                         │                                  │
│                         │  Goal: Ship new feature          │
│                         │  Press 'g' to edit goal          │
└─────────────────────────┴──────────────────────────────────┘
Status: Tab: Switch | j/k: Navigate | Space: Toggle | q: Quit
```

**Basic Navigation:**
- **Tab** - Switch between Outcomes (left) and Actions (right) panes
- **j/k** or **↑/↓** - Move up and down in lists
- **Space** - Mark an action as complete/incomplete
- **q** - Quit and save your progress

---

## 📝 Understanding the Data Structure

### The Markdown Format
Behind the scenes, FocusFive stores your goals as simple markdown files in `~/FocusFive/goals/`:

```markdown
# September 24, 2025 - Day 15

## Work (Goal: Ship new feature)
- [x] Write project proposal
- [ ] Review team feedback
- [ ] Deploy staging build

## Health (Goal: Exercise daily)
- [x] Morning workout
- [ ] Healthy lunch
- [ ] Evening walk

## Family (Goal: Quality time)
- [ ] Call parents
- [x] Plan weekend activity
- [ ] Help with homework
```

### File Structure
- **Date Header**: `# September 24, 2025 - Day 15` (Day counter is optional)
- **Outcome Sections**: `## Work (Goal: Ship new feature)` (Goals in parentheses are optional)
- **Actions**: `- [x] Completed action` or `- [ ] Pending action`
- **Location**: Files are named `YYYY-MM-DD.md` in `~/FocusFive/goals/`

### Data Constraints
- **Exactly 3 outcomes**: Work, Health, Family (or your chosen categories)
- **Exactly 3 actions per outcome**: No more, no less (enforced by the system)
- **500 character limit**: Action text is automatically truncated with warning
- **1MB file limit**: Safety limit to prevent corruption

---

## 🎮 Daily Workflow Guide

### Morning Ritual (2 minutes)
1. **Launch FocusFive**: `./target/release/focusfive`
2. **Review yesterday**: See what you accomplished
3. **Set today's focus**: Edit your 9 actions for the day
   - Use **Tab** to switch between outcomes
   - Press **e** to edit an action
   - Press **g** to set/edit goals for each outcome

### Throughout the Day (30 seconds)
- **Quick check-ins**: Launch FocusFive to see your progress
- **Mark completions**: Press **Space** on completed actions
- **Stay focused**: The constraint of only 9 actions keeps you on track

### Evening Review (30 seconds)
- **Final updates**: Mark any remaining completions
- **Reflect**: See your daily completion percentage
- **Streaks**: Track consecutive high-performance days

### Weekly Review (optional)
FocusFive automatically tracks your completion patterns across days, helping you identify what's working and what needs adjustment.

---

## 🔧 Advanced Features

### Goals & Visions
- **Goals**: Short-term objectives for each outcome (press **g**)
- **Visions**: Long-term purpose statements (press **v**)
- **Templates**: Save common action patterns for reuse (press **T**)

### Progress Tracking
- **Completion percentages**: Real-time calculation of daily progress
- **Streak tracking**: Days with high completion rates
- **7-day trends**: Rolling averages and patterns
- **Historical data**: All stored in searchable markdown files

### Customization
- **Action editing**: Press **e** to modify action text
- **Goal setting**: Press **g** to set outcome goals
- **Help system**: Press **?** for complete keyboard reference
- **Themes**: Built-in color schemes for different preferences

---

## 💻 For Developers: Architecture Overview

### Technology Stack
```toml
# Core Technologies
ratatui = "0.28"        # Modern Terminal UI framework
crossterm = "0.28"      # Cross-platform terminal control
chrono = "0.4"          # Date/time handling
anyhow = "1.0"          # Error handling
regex = "1.0"           # Markdown parsing
serde = "1.0"           # Data serialization
```

### Project Structure
```
goal_setting/
├── src/
│   ├── main.rs                    # Application entry point
│   ├── app.rs                     # Main application controller (3,074 lines)
│   ├── models.rs                  # Core data structures
│   ├── data.rs                    # File I/O and markdown parsing
│   ├── ui_state.rs                # Global UI state management
│   ├── ui/                        # Terminal UI components
│   │   ├── app.rs                 # UI application logic
│   │   ├── dashboard_layout.rs    # Two-pane layout system
│   │   ├── charts.rs              # Progress visualization
│   │   ├── help.rs                # Help system
│   │   ├── popup.rs               # Modal dialogs
│   │   ├── stats.rs               # Statistics display
│   │   ├── terminal.rs            # Terminal initialization
│   │   └── theme.rs               # Styling system
│   └── widgets/                   # Custom UI components
│       ├── progress.rs            # Progress bars and indicators
│       ├── live_metrics.rs        # Real-time metrics
│       ├── performance_chart.rs   # Historical charts
│       ├── status_line.rs         # Status bar
│       └── [various other widgets]
├── tests/                         # Comprehensive test suite (32 files)
├── thoughts/                      # Design documentation and research
└── docs/                          # User and developer documentation
```

---

## 🏗️ Core Architecture Deep Dive

### 1. Data Model (`src/models.rs`)

The heart of FocusFive is its **constrained data structure**:

```rust
// Fixed structure - exactly 3 outcomes, 3 actions each
#[derive(Debug, Clone, PartialEq)]
pub struct DailyGoals {
    pub date: NaiveDate,
    pub day_number: Option<u32>,    // "Day N" tracking
    pub work: Outcome,
    pub health: Outcome,
    pub family: Outcome,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Outcome {
    pub outcome_type: OutcomeType,   // Work | Health | Family
    pub goal: Option<String>,        // From "(Goal: ...)" in header
    pub actions: [Action; 3],        // Exactly 3, no more, no less
}

#[derive(Debug, Clone, PartialEq)]
pub struct Action {
    pub text: String,               // Limited to 500 chars
    pub completed: bool,            // Maps to [x] or [ ]
}
```

**Key Design Decisions:**
- **Fixed arrays** instead of vectors enforce the 3x3 constraint at compile time
- **Clone + PartialEq** traits enable efficient TUI state management
- **Optional goals** allow flexibility while maintaining structure
- **Type safety** through enums prevents invalid outcome categories

### 2. File I/O System (`src/data.rs`)

FocusFive implements **atomic write operations** to prevent data corruption:

```rust
pub fn save_daily_goals(goals: &DailyGoals, config: &Config) -> anyhow::Result<()> {
    // Create unique temp file with PID + timestamp for concurrency safety
    let temp_filename = format!("{}.tmp.{}.{}",
        filename,
        std::process::id(),
        SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis()
    );

    // Write to temp file first
    fs::write(&temp_path, content)?;

    // Atomic rename ensures no partial writes
    fs::rename(&temp_path, &file_path)?;
}
```

**Safety Guarantees:**
- **Atomic writes**: Either complete file or no change (never partial)
- **Concurrency safe**: Unique temp files prevent collisions
- **Error recovery**: All operations return `Result<T>`, no panics
- **Data integrity**: Validation during parsing with graceful fallbacks

### 3. Terminal UI Architecture (`src/ui/`)

The TUI follows a **modular, event-driven architecture**:

```
Terminal → App → Layout → Widgets
    ↓         ↓      ↓        ↓
 Events   State   Render  Components
```

**Core Components:**
- **`terminal.rs`**: Raw terminal I/O and event loop management
- **`app.rs`**: Central controller with application state routing
- **`dashboard_layout.rs`**: Two-pane layout implementation
- **`ui_state.rs`**: Global state management with focus modes

**Event Flow:**
1. Terminal captures keyboard/mouse events
2. App processes events and updates state
3. Layout system positions widgets
4. Widgets render their content to terminal buffer

### 4. Widget System (`src/widgets/`)

FocusFive implements a **custom widget architecture** built on ratatui:

```rust
impl Widget for CustomProgress {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Custom rendering logic using ratatui primitives
    }
}
```

**Available Widgets:**
- **Progress indicators**: Multiple styles (bars, circles, percentages)
- **Live metrics**: Real-time completion tracking
- **Performance charts**: Historical data visualization
- **Status line**: Context-aware information display
- **Alternative signals**: External data integration

---

## 🧪 Testing & Quality Assurance

### Test Suite Architecture (32 test files)

FocusFive has a **comprehensive testing strategy**:

#### Test Categories
1. **Regression Tests** - Validates critical bug fixes and prevents regressions
2. **Integration Tests** - End-to-end workflow testing
3. **Unit Tests** - Individual component validation
4. **Edge Case Tests** - Error conditions and boundary cases
5. **Performance Tests** - Benchmark validation

#### Key Testing Patterns
```rust
// Real filesystem testing with temporary directories
#[test]
fn test_atomic_write_safety() {
    let temp_dir = TempDir::new().unwrap();
    let config = Config {
        goals_dir: temp_dir.path().to_str().unwrap().to_string(),
    };

    // Test concurrent writes without mocking
    let handles = (0..50).map(|_| {
        thread::spawn(|| save_daily_goals(&create_sample_goals(), &config))
    }).collect::<Vec<_>>();

    // All should succeed without corruption
    assert!(handles.into_iter().all(|h| h.join().unwrap().is_ok()));
}
```

**Testing Philosophy:**
- **No panics**: Every test validates graceful error handling
- **Real filesystem**: Uses temporary directories instead of mocks
- **Concurrent safety**: Validates atomic operations under load
- **Round-trip validation**: Ensures data integrity through save/load cycles

---

## 🚀 Development Status & Roadmap

### Current Status: Phase 2 Complete ✅

**Achievements:**
- ✅ Full TUI implementation with two-pane layout
- ✅ Keyboard navigation and event handling
- ✅ Real-time progress tracking and streak calculation
- ✅ Comprehensive widget system
- ✅ Robust file I/O with atomic writes
- ✅ 100% Phase 2 validation rate (20/20 test criteria)

**Performance Metrics:**
- **Binary size**: 2.1MB (target: <10MB) ✅
- **Startup time**: ~50ms (target: <500ms) ✅
- **Memory usage**: ~5MB (target: <50MB) ✅
- **File operations**: <50ms (target: <100ms) ✅

### Next Phase: Phase 3 - Claude Integration

**Planned Features:**
- Export to `.claude/` directory for AI analysis
- Generate weekly and monthly insights
- Custom slash commands for goal analysis
- Predictive suggestions based on completion patterns

---

## 🛠️ Developer Contribution Guide

### Setting Up Development Environment

```bash
# 1. Prerequisites
# - Rust 1.75+
# - Git

# 2. Clone and build
git clone https://github.com/YOUR_USERNAME/goal_setting.git
cd goal_setting
cargo build --release

# 3. Run tests
cargo test
cargo clippy -- -D warnings

# 4. Validate setup
./validate_setup.sh
```

### Key Development Patterns

#### Error Handling
```rust
// ✅ GOOD: Always use Result<T>
pub fn load_goals(date: NaiveDate) -> anyhow::Result<DailyGoals> {
    let file_path = get_file_path(date);

    if !file_path.exists() {
        return create_default_goals(date); // Graceful fallback
    }

    let content = fs::read_to_string(&file_path)
        .with_context(|| format!("Failed to read {}", file_path.display()))?;

    parse_markdown(&content)
        .with_context(|| "Failed to parse goals")
}

// ❌ BAD: Never use unwrap/expect in production
let goals = load_goals(date).unwrap(); // Don't do this!
```

#### Safe Array Access
```rust
// ✅ GOOD: Safe iteration
for (i, action) in outcome.actions.iter().enumerate() {
    if i >= 3 { break; } // Safety check
    process_action(action);
}

// ❌ BAD: Direct indexing can panic
let action = outcome.actions[index]; // Don't do this!
```

### Adding New Features

#### 1. New Widget Development
```rust
// Create src/widgets/new_widget.rs
pub struct NewWidget {
    pub data: WidgetData,
    pub style: Style,
}

impl Widget for NewWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Implement rendering using ratatui primitives
    }
}

// Add to src/widgets/mod.rs
pub mod new_widget;
pub use new_widget::NewWidget;
```

#### 2. Extending the Data Model
```rust
// When adding fields, maintain backward compatibility
#[derive(Debug, Clone, PartialEq)]
pub struct Action {
    pub text: String,
    pub completed: bool,
    pub priority: Option<Priority>, // New field must be Optional
}
```

#### 3. UI State Management
```rust
// Extend UIState for new features
pub enum FocusMode {
    Dashboard,
    Statistics,
    NewFeature,  // Add your new mode here
}

impl UIState {
    pub fn handle_new_feature_event(&mut self, event: KeyEvent) -> Result<()> {
        // Handle events for your new feature
    }
}
```

### Testing Guidelines

#### Writing Tests
```rust
use tempfile::TempDir;
use crate::*;

#[test]
fn test_new_feature() {
    // 1. Set up isolated test environment
    let temp_dir = TempDir::new().unwrap();
    let config = Config {
        goals_dir: temp_dir.path().to_str().unwrap().to_string(),
    };

    // 2. Create test data
    let test_goals = create_sample_goals();

    // 3. Execute feature
    let result = your_new_feature(&test_goals, &config);

    // 4. Validate results
    assert!(result.is_ok());

    // 5. Verify side effects
    let loaded = load_daily_goals(test_goals.date, &config).unwrap();
    assert_eq!(loaded.work.actions[0].completed, true);
}
```

---

## 📈 Performance & Scalability

### Performance Characteristics
- **Startup**: < 100ms consistently
- **File I/O**: Atomic operations with < 50ms save times
- **Memory**: ~5MB typical usage, ~10MB maximum
- **UI Rendering**: 60fps capable terminal updates

### Scalability Considerations
- **File Count**: Tested with 1000+ daily goal files
- **Concurrent Access**: Atomic writes handle multiple instances
- **Large Actions**: 500-character limit prevents memory issues
- **File Size**: 1MB limit per file prevents corruption

---

## 🔐 Security & Privacy

### Local-First Design
- **No Network**: Zero external dependencies for core functionality
- **No Telemetry**: No data collection or reporting
- **File Permissions**: Standard user file permissions only
- **Data Location**: All files in user-controlled directories

### Security Features
- **Input Validation**: All user input validated and sanitized
- **Path Safety**: File operations use safe path construction
- **Memory Safety**: Rust's ownership system prevents common vulnerabilities
- **Error Handling**: No information leakage through error messages

---

## 🤝 Community & Support

### Getting Help
- **Documentation**: Comprehensive guides in `/docs/` directory
- **Validation Script**: `./validate_setup.sh` diagnoses issues
- **Issue Tracking**: GitHub issues for bug reports and features
- **Code Examples**: Extensive examples in test suite

### Contributing
1. **Issues**: Report bugs and request features via GitHub issues
2. **Pull Requests**: Follow existing code patterns and include tests
3. **Documentation**: Update relevant documentation with changes
4. **Testing**: Ensure all tests pass and add new test coverage

---

## 📚 Additional Resources

### Documentation
- `CLAUDE.md` - Comprehensive AI/developer context
- `USER_GUIDE.md` - Complete terminal UI guide
- `ARCHITECTURE_MINDMAP.md` - System architecture overview
- `DATA_MODEL.md` - Data structure specifications
- `TESTING_GUIDE.md` - Testing procedures and standards

### Historical Context
- `thoughts/shared/research/` - Analysis and findings
- `thoughts/shared/plans/` - Implementation roadmaps
- `thoughts/shared/summaries/` - Progress summaries

### Quick Reference Commands
```bash
# Build and run
cargo build --release && ./target/release/focusfive

# Today's goals file
cat ~/FocusFive/goals/$(date +%Y-%m-%d).md

# Run all tests
cargo test

# Check everything works
./validate_setup.sh

# View goals directory
ls -la ~/FocusFive/goals/
```

---

## 🎯 Core Principles & Philosophy

### Design Philosophy
1. **Constraint Breeds Clarity**: Limiting choices forces better decisions
2. **Local-First**: Your data belongs to you, stays with you
3. **Simplicity Enables Consistency**: 3-minute daily habit over complex systems
4. **Human-Readable**: Markdown format is future-proof and portable
5. **Fail Gracefully**: System degrades gracefully, never loses data

### The 3x3 Constraint
The core insight of FocusFive is that **unlimited options lead to paralysis**. By constraining yourself to:
- **3 life outcomes** (Work, Health, Family/Personal)
- **3 daily actions per outcome**
- **3 minutes total daily interaction**

You're forced to identify what truly matters and maintain sustainable focus over time.

This constraint isn't arbitrary - it's based on cognitive load research suggesting humans can effectively track 5-9 concurrent items. FocusFive uses 9 actions across 3 categories to maximize focus while remaining cognitively manageable.

---

## Code References

Key implementation files with line references:
- Core data models: `src/models.rs:15-59`
- Atomic file operations: `src/data.rs:47-85`
- TUI main controller: `src/ui/app.rs:25-70`
- Two-pane layout: `src/ui/dashboard_layout.rs:1-200`
- Widget architecture: `src/widgets/mod.rs:1-50`
- Test suite patterns: `tests/regression_tests.rs:1-100`
- Configuration setup: `src/models.rs:61-84`
- Progress tracking: `src/widgets/live_metrics.rs:1-150`

## Architecture Insights

FocusFive successfully balances **simplicity with sophistication**. The constrained 3x3 data model creates artificial limitations that improve user outcomes, while the underlying architecture provides professional-grade reliability, performance, and extensibility.

The combination of **compile-time safety** (Rust's type system), **runtime safety** (comprehensive error handling), and **data safety** (atomic file operations) creates a robust foundation for daily productivity tracking.

## Historical Context (from thoughts/)

The project has evolved through careful architectural planning documented in:
- Dashboard enhancement plans with live metrics and performance visualization
- UI schema foundation work establishing the two-pane layout
- Statistics framework implementation for long-term tracking
- Professional TUI implementation with comprehensive widget system

## Related Research

This comprehensive analysis builds upon previous research documented in:
- `thoughts/shared/research/2025-09-20_17-43-17_focusfive-dashboard-analysis.md` - Dashboard component analysis

## Open Questions

Areas for future investigation:
1. **Plugin Architecture**: How to enable third-party widgets while maintaining simplicity
2. **Cross-Platform Polish**: Optimizing performance across different terminal environments
3. **AI Integration**: Balancing intelligent insights with local-first privacy principles
4. **Scaling Patterns**: Supporting years of daily goal data efficiently