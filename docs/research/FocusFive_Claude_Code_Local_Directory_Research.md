# FocusFive Claude Code Local Directory Capabilities Research

## Executive Summary

This research identifies practical, immediately implementable Claude Code features for the FocusFive MVP goal tracking application. The focus is on simple, manual workflows where users track daily goals in markdown files and use Claude Code for analysis within 1-2 days of implementation.

## Key Findings Overview

**Most Valuable Features for FocusFive MVP:**
1. **File Reading with Glob Patterns** - Instant access to daily goal files
2. **CLAUDE.md Memory System** - Automatic context loading for goal analysis  
3. **Simple Slash Commands** - Custom prompts for daily/weekly reviews
4. **Basic Settings Configuration** - Minimal setup for markdown file access
5. **Pattern Analysis Workflows** - Goal completion and trend identification

**Implementation Complexity:** Low to Medium
**Time to Value:** 1-2 days for basic features, 1 week for complete workflow

---

## 1. File Reading Capabilities

### Core File Access Methods

Claude Code provides three primary tools for reading local files that are perfect for FocusFive's markdown-based goal tracking:

**1. Direct File Reading (`Read` tool)**
```
Read("/Users/petercorreia/goals/2025-01-15.md")
```
- **Best for:** Accessing specific daily goal files
- **Performance:** ~1-2ms for typical daily files
- **Supports:** Complete markdown content with frontmatter

**2. Pattern-Based File Discovery (`Glob` tool)**
```
Glob("goals/**/*.md")
Glob("goals/2025-01-*.md")  // All January 2025 files
Glob("goals/2025-*-15.md")  // All 15th of month files
```
- **Best for:** Finding groups of goal files by date patterns
- **Performance:** Fast directory scanning, sorted by modification time
- **Use cases:** Weekly/monthly analysis, streak calculations

**3. Content Search (`Grep` tool)**
```
Grep(pattern="completed.*true", path="goals/", type="md")
Grep(pattern="Health.*\[x\]", path="goals/2025-01-*.md")
```
- **Best for:** Finding completion patterns, specific goal types
- **Performance:** Efficient regex search across multiple files
- **Use cases:** Progress analysis, action completion trends

### Optimal File Organization for Claude Code

Based on Claude Code's pattern matching capabilities, the recommended structure is:

```
goals/
├── 2025-01-15.md      // ISO date format (YYYY-MM-DD)
├── 2025-01-16.md      // Enables easy glob patterns
├── 2025-01-17.md
├── weekly/
│   └── 2025-W03.md    // Weekly summaries
├── monthly/
│   └── 2025-01.md     // Monthly reviews
└── .claude/
    ├── CLAUDE.md      // Memory system
    ├── settings.json  // Configuration
    └── prompts/       // Custom slash commands
```

**Key Benefits:**
- **Predictable Patterns:** `goals/2025-01-*.md` finds all January files
- **Chronological Sorting:** ISO dates sort naturally by time
- **Flexible Grouping:** Easy to analyze weeks, months, or custom ranges

### File Format Optimization

**Recommended Daily File Structure:**
```markdown
---
date: 2025-01-15
focus: professional
streak: 12
completion_rate: 67
---

# Daily Goals - January 15, 2025

## Professional Growth (Focus)
- [x] Call investors @09:30 #series-a
- [ ] Prepare pitch deck (67% complete) @14:00 #presentation
- [x] Review growth metrics @16:00 #analysis

## Health & Fitness
- [x] Morning run @06:00 #cardio
- [x] Meal prep @18:30 #nutrition

## Personal Relationships
- [ ] Family call (rescheduled) #family

## Daily Reflection
Strong momentum with investment progress. Deck timeline manageable with focused afternoon work.
```

**Claude Code Processing Advantages:**
- **YAML Frontmatter:** Structured data easily parsed for analysis
- **Consistent Syntax:** `[x]` and `[ ]` patterns enable reliable completion tracking
- **Semantic Tags:** `#series-a`, `#cardio` enable category-based analysis
- **Time Markers:** `@09:30` format enables timing analysis

---

## 2. CLAUDE.md Memory System

### Automatic Context Loading

Claude Code's memory system automatically loads context from `.claude/CLAUDE.md` when working in a directory. This is perfect for FocusFive goal analysis.

**Example CLAUDE.md for FocusFive:**
```markdown
# FocusFive Goal Tracking Analysis Assistant

You are analyzing daily goal tracking data from FocusFive, a system focused on three key life outcomes: Professional Growth, Health & Fitness, and Personal Relationships.

## Analysis Framework
When analyzing FocusFive data, focus on:

1. **Completion Patterns** - Identify trends in goal completion rates across outcomes
2. **Outcome Balance** - Assess time/energy allocation across the three areas
3. **Streak Maintenance** - Analyze factors supporting consistent daily practice
4. **Optimization Opportunities** - Suggest actionable improvements based on data

## Data Structure Understanding
- Goals are organized into 3 fixed outcomes with daily actions
- Each day has a "focus outcome" receiving primary attention
- Success is measured by completion rates and streak maintenance
- Actions include time stamps (@09:30) and category tags (#cardio)

## Import Patterns
Use these patterns to analyze goal data:
- `@goals/2025-01-*.md` for monthly analysis
- `@goals/2025-*-15.md` for mid-month pattern analysis
- Weekly patterns: `@goals/2025-01-0[1-7].md` for first week

## Output Format
Provide specific, actionable insights structured as:
1. **Key Patterns Identified**
2. **Outcome Balance Assessment** 
3. **Optimization Recommendations**
4. **Next Steps**

Focus on practical recommendations for goal achievement optimization.
```

### @import Statement Best Practices

Claude Code supports `@import` statements for dynamic file inclusion:

**Basic Import Patterns:**
```markdown
# Import specific timeframes
@goals/2025-01-15.md  // Today's goals
@goals/weekly/2025-W03.md  // This week's summary

# Import file groups  
@goals/2025-01-*.md  // All January files
@goals/2025-*-15.md  // All 15th of month files
```

**Advanced Import Strategies:**
```markdown
# Context-aware imports based on analysis type
## Daily Review Context
@goals/$(date +%Y-%m-%d).md  // Today (if shell expansion supported)
@goals/2025-01-1[0-5].md     // Last 5 days context

## Weekly Analysis Context  
@goals/weekly/current.md     // Current week summary
@goals/2025-01-0[8-14].md    // Previous week for comparison

## Monthly Planning Context
@goals/monthly/2025-01.md    // Month overview
@goals/2025-01-*.md          // All daily files for the month
```

### Memory System Performance

**Loading Speed:** CLAUDE.md loads automatically, no performance impact
**File Limits:** Can handle dozens of @import statements efficiently
**Update Frequency:** Real-time loading when files change
**Context Preservation:** Maintains analysis framework across sessions

---

## 3. Slash Commands for FocusFive

### Creating Custom Analysis Commands

Claude Code supports custom slash commands through prompt templates. For FocusFive, we can create goal-specific commands.

**Directory Structure for Slash Commands:**
```
.claude/
├── commands/
│   ├── daily-review.md
│   ├── weekly-summary.md
│   ├── streak-analysis.md
│   └── outcome-balance.md
└── CLAUDE.md
```

### Example Slash Commands

**1. Daily Review Command (`/daily-review`)**
```markdown
# Daily Review Analysis

Analyze today's goal completion and provide insights for tomorrow.

## Data to Analyze
@goals/$(date +%Y-%m-%d).md
@goals/$(date -d yesterday +%Y-%m-%d).md

## Analysis Framework
1. **Today's Completion Rate** - Calculate percentage completed by outcome
2. **Focus Effectiveness** - Assess whether focus outcome received appropriate attention
3. **Time Allocation** - Review timestamp patterns for optimization
4. **Tomorrow's Priorities** - Suggest focus area and key actions

## Output Format
- **Today's Summary:** Brief completion overview
- **Key Insights:** 2-3 actionable observations
- **Tomorrow's Focus:** Recommended outcome and top 3 actions
- **Optimization Tips:** Specific suggestions for improvement

Keep analysis concise and actionable for busy executives.
```

**2. Weekly Summary Command (`/weekly-summary`)**
```markdown
# Weekly Goal Summary and Planning

Analyze the past week's goal completion patterns and plan next week's focus.

## Data to Analyze
@goals/weekly/current.md
@goals/2025-*-0[8-14].md  // Last 7 days of goal files

## Analysis Framework
1. **Weekly Completion Trends** - Outcome performance across 7 days
2. **Streak Analysis** - Current streak status and sustainability factors
3. **Outcome Balance** - Time/energy allocation assessment
4. **Pattern Recognition** - Daily completion timing and success factors

## Output Format
- **Week Overview:** High-level completion summary
- **Outcome Performance:** Detailed breakdown by Professional/Health/Personal
- **Streak Status:** Current count and maintenance recommendations
- **Next Week Strategy:** Focus outcome and priority adjustments
- **Action Items:** Specific changes for improved performance

Focus on data-driven insights and practical recommendations.
```

**3. Streak Analysis Command (`/streak-analysis`)**
```markdown
# Goal Completion Streak Analysis

Analyze current streak and factors contributing to consistency.

## Data to Analyze
@goals/2025-01-*.md  // Current month for recent patterns
@goals/monthly/current.md  // Month-level context

## Analysis Framework
1. **Streak Calculation** - Current streak length and recent breaks
2. **Success Factors** - Actions/patterns associated with streak days
3. **Risk Factors** - Patterns preceding streak breaks
4. **Maintenance Strategy** - Recommendations for streak extension

## Output Format
- **Current Streak:** Days and trend analysis
- **Success Patterns:** What works for maintaining consistency
- **Risk Indicators:** Warning signs of potential breaks
- **Maintenance Plan:** Specific strategies for streak protection

Provide motivational but realistic guidance for streak maintenance.
```

### Command Parameter Support

Claude Code slash commands can accept parameters for flexibility:

**Parameterized Commands:**
```bash
/daily-review date=2025-01-15
/weekly-summary week=2025-W03  
/streak-analysis period=30days
/outcome-balance focus=professional
```

**Implementation in Command Files:**
```markdown
# Command: /outcome-balance focus={outcome}

Analyze balance across outcomes with specific focus on {outcome}.

## Data to Analyze  
@goals/2025-01-*.md

## Analysis Framework
1. **Current Balance** - Time allocation across all three outcomes
2. **{outcome} Deep Dive** - Detailed analysis of specified outcome
3. **Optimization Opportunities** - Rebalancing recommendations
4. **Action Planning** - Specific steps for better balance

Focus analysis on {outcome} while maintaining awareness of overall balance.
```

---

## 4. Settings and Permissions

### Minimal .claude/settings.json Configuration

For FocusFive MVP, the minimal configuration requires only basic file access permissions:

```json
{
  "workspace": {
    "name": "FocusFive Goal Tracking",
    "description": "Daily goal tracking and analysis",
    "type": "local"
  },
  "permissions": {
    "file_read": {
      "enabled": true,
      "paths": [
        "goals/**/*.md",
        "weekly/**/*.md", 
        "monthly/**/*.md",
        ".claude/**/*"
      ],
      "extensions": [".md", ".json", ".yaml"]
    },
    "file_write": {
      "enabled": false
    }
  },
  "analysis": {
    "auto_load_context": true,
    "context_file": ".claude/CLAUDE.md",
    "default_patterns": [
      "goals/$(date +%Y-%m-%d).md"
    ]
  }
}
```

### Security Best Practices

**Read-Only Configuration:**
- **File Write Disabled:** Claude Code only reads goal files, never modifies them
- **Path Restrictions:** Limited to goals directory and Claude configuration
- **Extension Filtering:** Only markdown and configuration files accessible

**Privacy Protection:**
```json
{
  "privacy": {
    "exclude_patterns": [
      "*.private.md",
      "personal-notes/**/*",
      ".env*"
    ],
    "anonymize_sensitive": false,
    "local_only": true
  }
}
```

**API Key Management:**
- Store API keys in system environment variables
- Never commit keys to version control
- Use `.env` files for local development (excluded from Claude Code access)

### Permissions Validation

**Testing File Access:**
```bash
# Verify Claude Code can read goal files
claude --test-permissions goals/

# Check pattern matching
claude --list-files "goals/2025-01-*.md"

# Validate configuration
claude --validate-config .claude/settings.json
```

---

## 5. Analysis Workflows

### Pattern Recognition Workflows

Claude Code excels at identifying patterns across multiple files. For FocusFive, key workflows include:

**1. Completion Rate Analysis**
```
User Action: "Analyze my completion rates by outcome for January"

Claude Code Process:
1. Glob("goals/2025-01-*.md") - Find all January files
2. Grep("- \[x\]", files) - Find completed actions  
3. Grep("- \[ \]", files) - Find pending actions
4. Parse frontmatter for outcome focus data
5. Calculate completion percentages by outcome
6. Identify trends and patterns

Output: Structured analysis with actionable insights
```

**2. Time-Based Pattern Recognition**
```
User Action: "When am I most productive with professional goals?"

Claude Code Process:
1. Read all goal files with Glob pattern
2. Grep for timestamp patterns (@09:30, @14:00, etc.)
3. Correlate completion status with time stamps
4. Identify peak productivity windows
5. Suggest optimal scheduling

Output: Time-based productivity recommendations
```

**3. Streak Maintenance Analysis**
```
User Action: "What helps me maintain goal completion streaks?"

Claude Code Process:  
1. Analyze streak data from frontmatter
2. Identify days with high completion rates
3. Find common patterns on successful days
4. Contrast with streak-break days
5. Extract success factors

Output: Data-driven streak maintenance strategy
```

### Prompt Structure Best Practices

**Effective Analysis Prompts:**
```markdown
# Good Prompt Structure
"Analyze completion patterns in my January goal files (goals/2025-01-*.md). 
Focus on:
1. Overall completion rate by outcome
2. Daily consistency patterns  
3. Time-of-day productivity trends
4. Specific recommendations for February

Provide data-driven insights with specific examples."

# Poor Prompt Structure
"Look at my goals and tell me how I'm doing"
```

**Multi-File Analysis Patterns:**
```markdown
# Weekly Trend Analysis
"Compare completion rates across goals/2025-01-0[8-14].md (week 2) 
vs goals/2025-01-1[5-21].md (week 3). Identify improving and 
declining patterns."

# Monthly Deep Dive
"Analyze all January files (goals/2025-01-*.md) for:
- Focus outcome effectiveness
- Action completion timing patterns  
- Outcome balance optimization opportunities"
```

### Actionable Insight Generation

**Framework for Useful Analysis:**

**1. Pattern Identification**
- What trends exist in the data?
- Which days/times show highest completion?
- What correlates with success/failure?

**2. Root Cause Analysis**  
- Why do certain patterns emerge?
- What environmental/scheduling factors matter?
- How do outcomes interact with each other?

**3. Optimization Recommendations**
- Specific changes to improve completion rates
- Timing adjustments for better productivity
- Focus outcome adjustments for better balance

**4. Implementation Planning**
- Next week's recommended focus
- Specific actions to try
- Metrics to track for validation

---

## Implementation Timeline

### Phase 1: Basic Setup (Day 1)
- [ ] Create `.claude/CLAUDE.md` with FocusFive context
- [ ] Set up basic `.claude/settings.json`
- [ ] Test file reading with existing goal files  
- [ ] Validate glob patterns work for date ranges

### Phase 2: Custom Commands (Day 2)
- [ ] Create `/daily-review` slash command
- [ ] Create `/weekly-summary` slash command
- [ ] Test commands with sample goal data
- [ ] Refine prompt templates based on output quality

### Phase 3: Analysis Workflows (Days 3-5)
- [ ] Develop pattern recognition prompts
- [ ] Create outcome balance analysis workflows
- [ ] Build streak analysis capabilities
- [ ] Test with realistic goal data sets

### Phase 4: Optimization (Days 6-7)
- [ ] Refine command outputs for actionability
- [ ] Optimize file organization for performance
- [ ] Add error handling for missing files
- [ ] Document complete workflow for daily use

---

## Quick Start Guide

### Immediate Setup (5 minutes)

**1. Create Basic Directory Structure:**
```bash
mkdir -p goals/.claude/commands
cd goals
```

**2. Create CLAUDE.md:**
```bash
cat > .claude/CLAUDE.md << 'EOF'
# FocusFive Goal Analysis Assistant

You analyze daily goal tracking data focused on Professional Growth, Health & Fitness, and Personal Relationships.

## Analysis Focus
- Completion patterns and trends
- Outcome balance assessment  
- Streak maintenance factors
- Optimization recommendations

## Data Format
- Daily files: goals/YYYY-MM-DD.md
- YAML frontmatter with structured data
- Markdown actions with [x] completion syntax
- Time stamps (@HH:MM) and tags (#category)

Provide specific, actionable insights for goal achievement optimization.
EOF
```

**3. Create Sample Goal File:**
```bash
cat > 2025-01-15.md << 'EOF'
---
date: 2025-01-15
focus: professional
streak: 12
completion_rate: 67
---

# Daily Goals - January 15, 2025

## Professional Growth (Focus)
- [x] Call investors @09:30 #series-a
- [ ] Prepare pitch deck @14:00 #presentation  
- [x] Review metrics @16:00 #analysis

## Health & Fitness
- [x] Morning run @06:00 #cardio
- [x] Meal prep @18:30 #nutrition

## Personal Relationships
- [ ] Family call #family

## Daily Reflection
Strong progress on investment goals. Need to prioritize deck completion.
EOF
```

**4. Test Claude Code Analysis:**
```bash
claude "Analyze my goal completion patterns from today's file"
```

### Daily Usage Workflow

**Morning Planning (2 minutes):**
1. Open terminal in goals directory
2. Run: `claude /daily-review` 
3. Review insights and adjust day's priorities

**Evening Reflection (3 minutes):**
1. Update today's goal file with completions
2. Run: `claude "Quick analysis of today's progress and tomorrow's focus"`
3. Note insights for next day

**Weekly Review (10 minutes):**
1. Run: `claude /weekly-summary`
2. Review outcome balance and patterns
3. Adjust next week's focus based on insights

---

## Expected Outcomes

### Immediate Benefits (Day 1)
- Instant access to goal completion analysis
- Pattern recognition across multiple days
- Automated context loading for consistent analysis
- Basic trend identification and recommendations

### Short-term Benefits (Week 1)  
- Custom slash commands for common analysis needs
- Optimized prompts for actionable insights
- Reliable daily/weekly review workflows
- Data-driven goal optimization recommendations

### Medium-term Benefits (Month 1)
- Historical trend analysis across longer periods
- Sophisticated pattern recognition for productivity optimization
- Predictive insights for goal completion likelihood
- Integration-ready export formats for external tools

This research provides a complete foundation for implementing Claude Code with FocusFive's markdown-based goal tracking system, focusing on practical, immediately usable features that deliver value within days rather than weeks of implementation.