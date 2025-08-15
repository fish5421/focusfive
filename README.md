# FocusFive

A minimalist terminal-based goal tracking system with AI-powered insights through Claude Code integration.

## Overview

FocusFive is a local-first, privacy-preserving goal tracking application that helps you maintain focus on three key life outcomes through daily actions. It combines a fast terminal interface with sophisticated AI analysis capabilities.

### Core Philosophy
- **3 Outcomes**: Focus on exactly three life areas (e.g., Professional, Health, Personal)
- **Daily Actions**: Three concrete actions per outcome each day
- **3-Minute Habit**: Daily tracking takes less than 3 minutes
- **AI Insights**: Claude Code provides intelligent analysis without compromising privacy

## Features

### Current (MVP)
- ✅ Terminal UI for rapid daily goal tracking
- ✅ Markdown-based storage (human-readable, Git-friendly)
- ✅ Streak tracking and completion metrics
- ✅ Claude Code integration for AI analysis
- ✅ Zero configuration startup

### Planned (V2)
- 🔄 Adaptive AI guidance that evolves with user sophistication
- 🔄 Causal attribution engine to identify what drives success
- 🔄 Psychological state monitoring for burnout prevention
- 🔄 Predictive interventions and optimization suggestions

## Quick Start

```bash
# Install (once Cargo package is published)
cargo install focusfive

# Run for the first time
focusfive

# Set up your three outcomes and start tracking!
```

## Project Structure

```
goal_setting/
├── docs/                           # Documentation and research
│   ├── FocusFive_MVP_Build_Plan.md
│   ├── FocusFive_V2_PRD.md
│   └── research/
│       ├── FocusFive_UI_UX_Analysis.md
│       ├── FocusFive_Data_Architecture_Research.md
│       └── FocusFive_Claude_Code_Integration_Research.md
├── src/                           # Application source (Rust)
│   ├── main.rs                   # Entry point
│   ├── ui.rs                     # Terminal interface
│   ├── data.rs                   # File I/O
│   ├── models.rs                 # Data structures
│   └── export.rs                 # Claude integration
├── examples/                      # Example goal files
│   └── sample-goals/
└── tests/                        # Test suite
```

## How It Works

### Daily Workflow
1. **Morning (1 minute)**: Open FocusFive, review today's goals
2. **Throughout Day**: Check off completed actions with Space key
3. **Evening (2 minutes)**: Review progress, add reflection
4. **Weekly**: Run Claude Code analysis for insights

### Data Format
Goals are stored as simple markdown files:
```markdown
# January 15, 2025 - Day 12

## Work (Goal: Ship v1)
- [x] Call investors
- [x] Prep deck  
- [ ] Team standup

## Health (Goal: Run 5k)
- [x] Morning run
- [ ] Meal prep
- [ ] Sleep by 10pm

## Family (Goal: Be present)
- [ ] Call parents
- [x] Plan weekend
- [x] Homework help
```

### Claude Code Integration
Open Claude Code in your goals directory for intelligent analysis:
```bash
cd ~/FocusFive/goals
claude /daily-review    # Analyze today's progress
claude /weekly-summary  # Review weekly patterns
```

## Development

### Prerequisites
- Rust 1.75+ 
- Node.js 18+ (for Claude Code)

### Building from Source
```bash
git clone https://github.com/YOUR_USERNAME/goal_setting.git
cd goal_setting
cargo build --release
./target/release/focusfive
```

### Running Tests
```bash
cargo test
cargo clippy -- -D warnings
cargo fmt -- --check
```

## Documentation

- [MVP Build Plan](docs/FocusFive_MVP_Build_Plan.md) - 14-day implementation guide
- [V2 Product Requirements](docs/FocusFive_V2_PRD.md) - Advanced AI features roadmap
- [UI/UX Research](docs/research/FocusFive_UI_UX_Analysis.md) - Interface design decisions
- [Data Architecture](docs/research/FocusFive_Data_Architecture_Research.md) - Storage strategy
- [Claude Integration](docs/research/FocusFive_Claude_Code_Integration_Research.md) - AI capabilities

## Philosophy

FocusFive is built on the belief that:
- **Less is more**: Three outcomes prevent overwhelm
- **Daily habits compound**: Small consistent actions create big results  
- **Privacy matters**: Your goals stay on your machine
- **AI should enhance, not replace**: Claude provides insights, you make decisions
- **Simplicity enables consistency**: If it takes more than 3 minutes, it won't become a habit

## Contributing

This project is in early development. Contributions are welcome but please open an issue first to discuss major changes.

### Development Principles
- **User First**: Every feature must provide clear user value
- **Privacy Always**: No telemetry, no cloud requirements
- **Speed Matters**: Daily interaction must be instant
- **Code Quality**: Clean, tested, documented code only

## License

MIT License - See [LICENSE](LICENSE) file for details

## Acknowledgments

- Inspired by GitUI's excellent terminal interface design
- Built with [Ratatui](https://github.com/ratatui-org/ratatui) TUI framework
- Powered by [Claude Code](https://github.com/anthropics/claude-code) for AI analysis

## Status

🚧 **Pre-Alpha Development** - Not yet ready for production use

Follow development progress in the [Build Plan](docs/FocusFive_MVP_Build_Plan.md).

---

*Building the tool I wish existed for my own goal tracking - simple, private, intelligent.*