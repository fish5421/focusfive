# FocusFive MVP Build Plan

## Executive Summary
A step-by-step implementation plan for building FocusFive, a minimalist terminal-based goal tracking application with Claude Code integration. Total build time: 2 weeks for core MVP.

## MVP Core Principles
1. **One user, local-only** - No multi-user complexity, no cloud sync
2. **3-minute daily habit** - Optimize for speed of daily entry
3. **Plain markdown files** - Human-readable, Git-friendly, tool-agnostic
4. **Claude Code integration** - Simple but powerful AI analysis built-in
5. **Zero configuration start** - Default settings work immediately

## Technical Stack
```toml
# Cargo.toml
[dependencies]
ratatui = "0.26"        # TUI framework
crossterm = "0.27"      # Terminal control
chrono = "0.4"          # Date handling
serde = "1"             # Serialization
serde_yaml = "0.9"      # YAML config
anyhow = "1"            # Error handling
regex = "1"             # Markdown parsing
```

## Project Structure
```
~/FocusFive/
├── goals/                      # User data directory
│   ├── 2025-01-15.md          # Daily goal files
│   ├── 2025-01-16.md
│   ├── config.yaml            # User's 3 outcomes
│   ├── .claude/               # Claude Code config
│   │   ├── settings.json      # Permissions
│   │   └── commands/          # Slash commands
│   │       ├── daily-review.md
│   │       └── weekly-summary.md
│   └── CLAUDE.md              # Auto-loaded context
├── src/                       # Application source
│   ├── main.rs               # Entry point & TUI loop
│   ├── ui.rs                 # Terminal interface
│   ├── data.rs               # File I/O
│   ├── models.rs             # Data structures
│   └── export.rs             # Claude integration
└── Cargo.toml
```

## Build Timeline: 14 Days

### Phase 1: Core Data Layer (Days 1-3)

#### Day 1: Project Setup & Data Models
**Morning (4 hours)**
- [ ] Initialize Rust project with dependencies
- [ ] Create basic data structures:
```rust
struct DailyGoals {
    date: NaiveDate,
    work_actions: Vec<Action>,
    health_actions: Vec<Action>,
    family_actions: Vec<Action>,
}

struct Action {
    text: String,
    completed: bool,
}

struct Config {
    work_goal: String,    // "Ship product v1"
    health_goal: String,  // "Run 5k"
    family_goal: String,  // "Be present"
}
```

**Afternoon (4 hours)**
- [ ] Implement file I/O functions:
  - `read_daily_file(date: NaiveDate) -> Result<DailyGoals>`
  - `write_daily_file(goals: &DailyGoals) -> Result<()>`
  - `list_goal_files() -> Vec<PathBuf>`
- [ ] Test with sample markdown files

#### Day 2: Markdown Parsing
**Morning (4 hours)**
- [ ] Build regex-based markdown parser:
```rust
// Parse: "- [x] Call investors" -> Action { text: "Call investors", completed: true }
// Parse: "- [ ] Meal prep" -> Action { text: "Meal prep", completed: false }
```
- [ ] Handle file creation for new days
- [ ] Implement atomic file writes (temp file + rename)

**Afternoon (4 hours)**
- [ ] Build markdown generator:
```rust
fn generate_markdown(goals: &DailyGoals) -> String {
    // Produces clean, human-readable markdown
}
```
- [ ] Add config.yaml reading/writing
- [ ] Test round-trip parsing (read → modify → write → read)

#### Day 3: Streak & Analytics
**Morning (4 hours)**
- [ ] Calculate current streak from file history:
```rust
fn calculate_streak(goal_dir: &Path) -> u32 {
    // Count consecutive days with at least one completion
}
```
- [ ] Calculate completion percentages per outcome
- [ ] Find most recent N days of data

**Afternoon (4 hours)**
- [ ] Build weekly summary generator
- [ ] Add basic caching for performance (in-memory only)
- [ ] Create test data generator for development

### Phase 2: Terminal UI (Days 4-7)

#### Day 4: Basic TUI Layout
**Morning (4 hours)**
- [ ] Set up ratatui application skeleton
- [ ] Create two-pane layout:
```
┌─────────────┬──────────────────┐
│  Outcomes   │  Today's Actions │
│  (Left)     │  (Right)         │
└─────────────┴──────────────────┘
```
- [ ] Implement basic event loop (keyboard input, rendering)

**Afternoon (4 hours)**
- [ ] Add navigation between panes (Tab key)
- [ ] Display static test data in panes
- [ ] Add quit functionality (q key saves and exits)

#### Day 5: Interactive Features
**Morning (4 hours)**
- [ ] Implement action selection (j/k or arrow keys)
- [ ] Add checkbox toggling (Space key)
- [ ] Show real data from markdown files

**Afternoon (4 hours)**
- [ ] Add visual feedback for selections
- [ ] Implement completion indicators (progress bars)
- [ ] Display current streak in header

#### Day 6: Data Persistence
**Morning (4 hours)**
- [ ] Connect UI actions to file writes
- [ ] Implement auto-save on changes
- [ ] Add dirty state tracking

**Afternoon (4 hours)**
- [ ] Handle date changes (new day = new file)
- [ ] Add keyboard shortcuts display
- [ ] Implement proper error handling with user feedback

#### Day 7: Polish & Testing
**Morning (4 hours)**
- [ ] Add colors and styling
- [ ] Optimize rendering performance
- [ ] Handle terminal resize events

**Afternoon (4 hours)**
- [ ] Test on different terminal sizes
- [ ] Add help screen (? key)
- [ ] Fix any UI glitches

### Phase 3: Claude Code Integration (Days 8-10)

#### Day 8: Claude Configuration
**Morning (4 hours)**
- [ ] Create CLAUDE.md template:
```markdown
# FocusFive Goal Tracking Context

You are analyzing daily goal tracking data for a single user tracking three life outcomes: Work, Health, and Family.

## Goal Structure
- 3 outcomes with specific goals
- 3 daily actions per outcome  
- Simple completion tracking with [x] or [ ]
- Streak maintenance is key metric

## Current Goals
@goals/config.yaml

## Analysis Framework
When analyzing, focus on:
1. Completion patterns and trends
2. Balance across three life areas
3. Streak maintenance factors
4. Specific, actionable recommendations
```

**Afternoon (4 hours)**
- [ ] Create .claude/settings.json:
```json
{
  "permissions": {
    "allow": ["Read(./goals/*.md)", "Read(./goals/config.yaml)"],
    "deny": ["Write(**)", "Bash(**)", "Edit(**)", "Delete(***)"]
  }
}
```
- [ ] Set up directory structure for Claude
- [ ] Test Claude Code can read the files

#### Day 9: Slash Commands
**Morning (4 hours)**
- [ ] Create /daily-review command:
```markdown
---
description: Review today's progress and plan tomorrow
allowed-tools: Read, Glob, Grep
---
# Daily Review Analysis

Review @goals/{{TODAY}}.md and recent files.

Provide:
1. Today's completion rate per outcome
2. What patterns led to success/failure
3. Three specific actions for tomorrow
4. One process improvement suggestion
```

**Afternoon (4 hours)**
- [ ] Create /weekly-summary command
- [ ] Create /streak-analysis command
- [ ] Test all commands with sample data

#### Day 10: Export Features
**Morning (4 hours)**
- [ ] Build export command in TUI ('e' key)
- [ ] Generate analysis-ready markdown:
```rust
fn generate_claude_export(days: u32) -> String {
    // Creates comprehensive export for Claude analysis
}
```

**Afternoon (4 hours)**
- [ ] Add export success feedback
- [ ] Create instruction file for Claude usage
- [ ] Test full workflow: track → export → analyze

### Phase 4: MVP Finalization (Days 11-14)

#### Day 11: Installation & Setup
**Morning (4 hours)**
- [ ] Create installation script
- [ ] Build first-run wizard:
```rust
fn first_run_setup() {
    // 1. Ask for three outcome goals
    // 2. Create config.yaml
    // 3. Create today's file
    // 4. Set up Claude directory
}
```

**Afternoon (4 hours)**
- [ ] Add command-line arguments
- [ ] Create man page / help documentation
- [ ] Set up cargo package metadata

#### Day 12: Edge Cases & Robustness
**Morning (4 hours)**
- [ ] Handle missing/corrupted files gracefully
- [ ] Add backup before modifications
- [ ] Implement config migration for updates

**Afternoon (4 hours)**
- [ ] Test with 1+ years of data
- [ ] Optimize performance for large datasets
- [ ] Add data recovery options

#### Day 13: Documentation
**Morning (4 hours)**
- [ ] Write README.md with:
  - Installation instructions
  - Quick start guide
  - Claude Code setup
  - Keyboard shortcuts

**Afternoon (4 hours)**
- [ ] Create example goal files
- [ ] Document Claude analysis workflows
- [ ] Add troubleshooting guide

#### Day 14: Testing & Release
**Morning (4 hours)**
- [ ] Full integration testing
- [ ] Test on macOS, Linux, Windows (WSL)
- [ ] Performance profiling

**Afternoon (4 hours)**
- [ ] Create GitHub release
- [ ] Build binaries for platforms
- [ ] Publish to crates.io

## Critical Path Items

### Must Have for MVP
✅ Daily markdown files with checkbox format
✅ Two-pane TUI with keyboard navigation  
✅ Streak calculation
✅ Claude Code integration with slash commands
✅ Zero-config startup

### Nice to Have (Defer)
⏸ Historical graphs
⏸ Multiple goal templates
⏸ Cloud backup
⏸ Mobile app
⏸ Team features

## Success Metrics

### Technical
- [ ] Daily entry completes in < 10 seconds
- [ ] Application starts in < 500ms
- [ ] File saves are atomic (no data loss)
- [ ] Works on 80-column terminals

### User Experience  
- [ ] First run to tracking goals: < 1 minute
- [ ] Daily habit completion: < 3 minutes
- [ ] Claude analysis provides actionable insights
- [ ] 30-day retention (self-testing)

## Risk Mitigation

### Technical Risks
| Risk | Mitigation |
|------|------------|
| File corruption | Atomic writes, automatic backups |
| Performance with many files | Lazy loading, date-based filtering |
| Terminal compatibility | Test on multiple terminals, fallback rendering |
| Claude integration breaks | Graceful degradation, manual export option |

### User Risks
| Risk | Mitigation |
|------|------------|
| Too complex to use daily | Obsessive simplification, user testing |
| Forgetting to track | OS notifications (future feature) |
| Losing motivation | Streak display, Claude encouragement |
| Data privacy concerns | Local-only, clear documentation |

## Development Environment Setup

```bash
# Prerequisites
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
cargo install cargo-watch

# Development
git clone https://github.com/YOU/focusfive
cd focusfive
cargo watch -x run  # Auto-rebuild on changes

# Testing
cargo test
cargo build --release

# Claude Code testing
npm install -g @anthropic-ai/claude-code
cd ~/FocusFive/goals
claude  # Should auto-load CLAUDE.md context
```

## Post-MVP Roadmap (Future)

### Version 1.1 (Week 3-4)
- Git auto-commit integration
- Weekly email summaries
- Outcome customization

### Version 1.2 (Month 2)
- Historical analysis views
- Goal templates library
- Export to other formats

### Version 2.0 (Month 3+)
- Web dashboard
- Team features
- Mobile sync

## Daily Development Checklist

### Morning Standup (5 min)
- [ ] Review yesterday's progress
- [ ] Identify today's critical path
- [ ] Note any blockers

### Coding Sessions
- [ ] 2-hour focused blocks
- [ ] Test after each feature
- [ ] Commit working code frequently

### Evening Review (5 min)
- [ ] Update build plan checkboxes
- [ ] Document any decisions/changes
- [ ] Prep tomorrow's tasks

## Final Notes

**Philosophy**: Build the smallest thing that creates the daily habit. Every feature must earn its complexity by providing clear user value.

**Testing**: You are user #1. If you don't want to use it daily, neither will anyone else.

**Success**: MVP succeeds when one person tracks goals for 30 consecutive days and gets value from Claude's analysis.

---

*Start Date: [YOUR START DATE]*  
*Target Completion: [START DATE + 14 days]*  
*Daily Build Time: 8 hours*  
*Total Effort: 112 hours*