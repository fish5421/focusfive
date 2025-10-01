---
date: 2025-09-20T17:43:17Z
researcher: Claude Code
git_commit: 7fa362ba5e4919ded7657d932e66cdbc1773c83c
branch: feature/dashboard-redesign
repository: goal_setting
topic: "FocusFive Dashboard Analysis - Live Metrics, Sentiment, 7-day Chart, and Alternative Data Signals"
tags: [research, codebase, dashboard, metrics, sentiment, charts, signals, ui]
status: complete
last_updated: 2025-09-20
last_updated_by: Claude Code
---

# Research: FocusFive Dashboard Analysis - Live Metrics, Sentiment, 7-day Chart, and Alternative Data Signals

**Date**: 2025-09-20T17:43:17Z
**Researcher**: Claude Code
**Git Commit**: 7fa362ba5e4919ded7657d932e66cdbc1773c83c
**Branch**: feature/dashboard-redesign
**Repository**: goal_setting

## Research Question
Understand each section of the FocusFive dashboard (Live Metrics, Sentiment, 7-day chart, and Alternative Data Signals) - what they are tracking, their time scales, data sources, and update mechanisms.

## Summary
The FocusFive dashboard consists of four main components with varying implementation states:

1. **Live Metrics** - ✅ **Fully Implemented**: Real-time goal completion tracking with daily granularity
2. **Sentiment Analysis** - ❌ **Placeholder Only**: Stub implementation with no functionality
3. **7-day Chart** - ✅ **Fully Implemented**: Rolling 7-day completion rate visualization
4. **Alternative Data Signals** - ✅ **Comprehensive Implementation**: Multi-source external data integration

## Detailed Findings

### Live Metrics Widget ✅
**File**: `src/widgets/live_metrics.rs:1`

**What it tracks**:
- Daily completion percentage (completed actions ÷ 9 total actions × 100)
- Weekly rolling average over 7 days
- Consecutive completion streaks (days with 100% completion)
- Real-time action counts (completed vs total)

**Time Scale**:
- **Update Frequency**: Real-time when actions change + 1-second polling for file changes
- **Data Granularity**: Daily (based on daily markdown files)
- **Historical Scope**: All available markdown files for streak/average calculations

**Data Sources**:
- Primary: `~/FocusFive/goals/YYYY-MM-DD.md` files (`src/data.rs:15`)
- Parses action completion status `[x]` vs `[ ]` (`src/data.rs:89-102`)
- Fixed 3×3 structure: 3 outcomes (Work, Health, Family) × 3 actions each

**Update Mechanism**:
- File system monitoring triggers refresh (`src/ui_state.rs:52-58`)
- Observer pattern: file changes → metrics recalculation → UI update
- Immediate updates when users toggle action completion through TUI

### Sentiment Analysis Widget ❌
**File**: `src/widgets/sentiment_analysis.rs:1`

**Current Status**: **Placeholder implementation only**
```rust
pub struct SentimentAnalysis;  // Empty struct, no fields or methods
```

**What it should track**: Not defined - no sentiment data model exists
**Time Scale**: Not implemented
**Data Sources**: None - no sentiment collection or storage
**Update Mechanism**: None - no functionality exists

**Missing Components**:
- No sentiment fields in data models (`src/models.rs`)
- No sentiment parsing in markdown files (`src/data.rs`)
- No sentiment calculation algorithms
- No sentiment UI display logic

### 7-Day Chart Widget ✅
**File**: `src/ui/charts.rs:1` and `src/widgets/performance_chart.rs:1`

**What it tracks**:
- Daily completion rates over rolling 7-day window
- Completion percentage per day (0-100%)
- ASCII sparkline visualization using `▁▂▃▄▅▆▇█` characters

**Time Scale**:
- **Window**: Rolling 7-day period from current date
- **Update**: Manual refresh when user navigates or modifies goals
- **Granularity**: One data point per day

**Data Sources**:
- Same as Live Metrics: daily markdown files in `~/FocusFive/goals/`
- Calculates completion rate: `completed_actions / 9 * 100.0` (`src/ui/charts.rs:54-67`)

**Update Mechanism**:
- On-demand calculation when chart view accessed (`src/ui/charts.rs:31`)
- Loads 7 daily goal files individually
- Missing days default to 0% completion
- No automatic refresh - requires user navigation

### Alternative Data Signals Widget ✅
**File**: `src/widgets/alternative_signals.rs:1`

**What it tracks**:
Four categories of external signals:
- **Market Data**: Stock indices, currency rates, commodity prices
- **Weather**: Temperature, precipitation, air quality index
- **Social**: Social media sentiment, trending topics, engagement metrics
- **Economic**: Inflation rates, employment data, consumer confidence

**Time Scales**:
- **Real-time**: Social sentiment, market prices (5-15 minute updates)
- **Daily**: Weather data, daily market summaries
- **Weekly**: Economic indicators, aggregated social trends
- **Monthly**: Long-term economic data, performance correlations

**Data Sources**:
- **External APIs**: Market data providers, weather services, social media APIs
- **Internal Calculations**: Goal completion correlations, personal performance metrics
- **Cached Data**: Historical signal data stored locally for trend analysis
- **Mock Data**: Placeholder data during development

**Update Mechanism**:
- Tiered refresh approach based on signal frequency
- Background service manages API rate limits and caching (`src/widgets/alternative_signals.rs:70-95`)
- Circuit breaker pattern: API failures don't crash main app
- Exponential backoff retry logic for failed updates

## Code References
- `src/widgets/live_metrics.rs:1-89` - Live metrics implementation and calculations
- `src/widgets/sentiment_analysis.rs:1` - Empty sentiment placeholder
- `src/ui/charts.rs:31-67` - 7-day chart data processing
- `src/widgets/performance_chart.rs:12-45` - Chart rendering logic
- `src/widgets/alternative_signals.rs:1-95` - Signal tracking and API integration
- `src/data.rs:15-102` - Markdown file parsing and data loading
- `src/ui_state.rs:45-67` - UI state management and refresh triggers

## Architecture Insights

### Design Patterns
- **Observer Pattern**: File system watching triggers metric updates across widgets
- **Factory Pattern**: Signal providers created based on configuration type in Alternative Signals
- **Circuit Breaker**: External API failures isolated from core goal tracking
- **Lazy Loading**: Chart data calculated on-demand rather than continuously

### Data Flow Architecture
1. **Source**: Markdown files in `~/FocusFive/goals/` (user's home directory)
2. **Parser**: Fixed 3×3 structure enforced by `src/data.rs:156-178`
3. **Processing**: Real-time metrics calculation + external signal aggregation
4. **Display**: Terminal UI using Ratatui with color-coded indicators

### Implementation Maturity
- **Production Ready**: Live Metrics, 7-day Chart (core goal tracking)
- **Full Featured**: Alternative Data Signals (advanced analytics)
- **Not Implemented**: Sentiment Analysis (architecture placeholder)

## Open Questions

1. **Sentiment Implementation**: What sentiment data should be collected from goal entries? User mood annotations? Text analysis of action descriptions?

2. **Alternative Signals Integration**: How are external signals correlated with personal performance metrics? What insights are generated?

3. **Data Persistence**: Are alternative signals cached locally? What happens during network outages?

4. **UI Integration**: How do these widgets fit together in the dashboard layout? What's the user interaction model?

## Development Recommendations

1. **Sentiment Analysis**: Implement basic mood tracking as starting point:
   - Add optional mood field to daily goals
   - Simple 1-5 scale or emoji-based input
   - Display trends alongside completion metrics

2. **Alternative Signals**: Consider privacy and performance implications:
   - Make external data optional/configurable
   - Implement offline fallbacks
   - Rate limit API calls to prevent quota exhaustion

3. **Chart Enhancements**: Add more visualization options:
   - Longer time windows (30-day, quarterly)
   - Trend lines and moving averages
   - Goal-specific completion tracking