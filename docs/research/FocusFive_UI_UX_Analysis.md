# FocusFive UI/UX and TUI Workflow Analysis

## Background & Problem Framing

### Core UI/UX Challenges for Terminal-Based Goal Tracking

Building a terminal-based goal tracking system presents unique challenges that span both technical implementation and user experience design:

**Information Density vs. Clarity**: Terminal interfaces must convey complex goal hierarchies, progress indicators, and temporal relationships within severely constrained screen real estate. Unlike GUI applications, every character position matters, requiring careful information prioritization.

**Discoverability vs. Efficiency**: High-performance executives and educators need both rapid daily interactions (3-5 minutes) and intuitive discoverability for less frequent operations. The interface must balance keyboard shortcuts for power users with clear visual cues for occasional users.

**Navigation Model Complexity**: Managing three fixed outcomes with nine daily actions requires sophisticated navigation patterns. Users must efficiently move between outcome levels, drill into specific actions, and maintain situational awareness of their position within the goal hierarchy.

**Temporal Context Switching**: Daily sessions demand quick context restoration. Users need immediate visual feedback about today's priorities, recent progress, and upcoming deadlines without manual navigation overhead.

**Limited Input Methods**: Keyboard-only interaction constrains input patterns to sequential character entry and hotkey combinations, eliminating the parallel interaction patterns possible with mouse/touch interfaces. This fundamentally affects how users can manipulate and explore their goal data.

**Visual Feedback Constraints**: Progress visualization must rely on ASCII characters, basic colors, and layout positioning rather than rich graphics. Effective progress communication becomes a typography and color theory challenge.

**Mobile Terminal Considerations**: SSH sessions and mobile terminal usage introduce additional constraints including smaller screens, potential network latency, and limited color support, requiring graceful degradation strategies.

## Exploration & Brainstorming

### Option 1: Pure GitUI-Style Three-Pane Layout

**Design Overview**: Following the gitui model with three horizontal panes - Outcomes (left), Actions (center), and Context/Detail (right). Users navigate between panes using h/l keys, within panes using j/k keys, and drill into details using Enter.

**Layout Structure**:
```
┌─ FocusFive ─────────────────────────────────────────────────────────────┐
│ [Q] Quit  [Tab] Next Pane  [?] Help                            2025-01-15 │
├─ Outcomes ──────┬─ Actions ─────────────┬─ Details ────────────────────────┤
│ › 1. Professional│ › [ ] Call investors   │ Action: Call investors           │
│   2. Health      │   [ ] Prep deck       │ Target: Wed, Jan 15              │
│   3. Personal    │   [X] Review metrics  │ Context: Q1 funding round       │
│                  │                       │ Notes: Focus on growth metrics  │
│ Progress: 67%    │ Progress: 1/3         │ Last updated: 2 hours ago      │
│                  │                       │                                 │
│ Today's Focus:   │ › [ ] Morning run     │ ┌─ Weekly Overview ───────────┐ │
│ Professional     │   [X] Meal prep       │ │ Mon ██░░ Tue ███░ Wed ░░░░  │ │
│                  │   [ ] Family call     │ │ Thu ░░░░ Fri ░░░░ Sat ░░░░  │ │
│                  │                       │ │ Sun ░░░░                    │ │
│ Streak: 12 days  │ Progress: 1/3         │ └─────────────────────────────┘ │
└──────────────────┴───────────────────────┴─────────────────────────────────┘
```

**Workflow Patterns**:
1. **Morning Session**: User launches FocusFive, sees today's focused outcome highlighted, navigates to actions with Tab or l key
2. **Action Selection**: j/k to navigate actions, Space to toggle completion, Enter for details
3. **Progress Review**: h key returns to outcomes view for high-level progress check
4. **Quick Exit**: q key for immediate quit, preserving state

**Pros**:
- **Familiar Pattern**: Mirrors successful gitui interface that developers already understand
- **Clear Information Hierarchy**: Physical separation prevents cognitive overload
- **Efficient Navigation**: Predictable hjkl movement patterns enable muscle memory
- **Contextual Detail**: Right pane provides expandable detail without losing position
- **Scalable Layout**: Works across terminal sizes with responsive pane widths

**Cons**:
- **Screen Real Estate**: Three panes may feel cramped on narrow terminals (< 100 cols)
- **Context Switching Overhead**: Multiple tab presses required to reach specific panes
- **Limited Immediate Action**: No overview mode showing all outcomes and actions simultaneously
- **Information Fragmentation**: Related information may be separated across panes

**Implementation Considerations**:
- Use Terminal.Gui containers with customizable sizing (30/40/30 default ratios)
- Implement responsive breakpoints for terminal width < 80 characters
- Cache pane state for instant restoration on application restart
- Support vim-style marks for quick position jumping

### Option 2: Modal Interface with Context Switching

**Design Overview**: Single-pane interface with modal overlays for different contexts. Base view shows current day's focus with hotkeys for different modes: outcome selection (o), action details (d), progress review (p), and settings (s).

**Modal Flow**:
```
┌─ FocusFive Daily Focus ─────────────────────────────────────────┐
│ Wednesday, January 15, 2025                     [o] [d] [p] [s] │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  Today's Outcome: Professional Growth                          │
│  ████████████████████████████████████████████████░░░░  89%     │
│                                                                 │
│  Actions for Today:                                             │
│  1. [X] Call investors                         ✓ Completed     │
│  2. [ ] Prepare pitch deck                     → In Progress   │
│  3. [ ] Review growth metrics                  ○ Pending       │
│                                                                 │
│  ┌─ Quick Stats ─────────────────────────────────────────────┐  │
│  │ Current Streak: 12 days                                  │  │
│  │ Weekly Progress: 67% (Professional), 45% (Health)       │  │
│  │ Next Milestone: 15 days (3 days away)                   │  │
│  └───────────────────────────────────────────────────────────┘  │
│                                                                 │
│  [Enter] Toggle Action  [Space] Quick Note  [Tab] Next Action  │
└─────────────────────────────────────────────────────────────────┘

When user presses 'o':
┌─ Select Outcome ─────────────────────────────────┐
│ › 1. Professional Growth              89% │████  │
│   2. Health & Fitness                45% │██▒▒  │  
│   3. Personal Relationships          67% │███▒  │
│                                               │
│ [Enter] Select  [Esc] Cancel                   │
└─────────────────────────────────────────────────┘
```

**Workflow Patterns**:
1. **Launch**: Application opens to today's primary outcome with immediate action visibility
2. **Quick Toggle**: Number keys (1,2,3) or j/k + Enter to toggle action completion
3. **Outcome Switch**: 'o' key opens outcome selector modal, preserving current progress
4. **Deep Dive**: 'd' key opens action detail modal with notes, deadlines, context
5. **Progress Review**: 'p' key shows weekly/monthly progress modal with visualizations

**Pros**:
- **Maximum Screen Utilization**: Full terminal width for primary content display
- **Context Preservation**: Modals overlay without losing base state
- **Rapid Context Switching**: Single hotkey access to all major functions
- **Mobile-Friendly**: Single column layout works well on narrow screens
- **Immediate Action**: Primary workflow (action completion) requires minimal keystrokes

**Cons**:
- **Modal Fatigue**: Frequent modal opening/closing may feel cumbersome
- **Hidden Information**: Secondary data only visible through modal activation
- **Navigation Memory**: Users must remember hotkey combinations for different contexts
- **Limited Overview**: Difficult to see relationships between outcomes simultaneously

**Implementation Considerations**:
- Implement modal stack management for nested overlays
- Use semi-transparent modal backgrounds to maintain context
- Cache modal state for instant reopening with previous selections
- Support ESC key for consistent modal dismissal

### Option 3: Hybrid Approach with Floating Overlays

**Design Overview**: Two-pane base layout (Outcomes left, Actions right) with floating overlays for detailed information. Combines benefits of persistent layout with contextual detail expansion.

**Base Layout + Overlay System**:
```
┌─ FocusFive ─────────────────────────────────────────────────────────┐
│ Today: Wed Jan 15  Streak: 12  [?] Help                            │
├─ Outcomes ──────────────────┬─ Actions ─────────────────────────────┤
│                             │                                       │
│ 1. Professional Growth      │ Focus: Professional Growth            │
│    ████████████████████░░░  │                                       │
│    Progress: 89%            │ 1. [X] Call investors     ⏰ 09:00    │
│                             │    └─ Completed 2h ago                │
│ 2. Health & Fitness         │                                       │
│    ████████░░░░░░░░░░░░░░░░  │ 2. [ ] Prepare pitch deck ⏰ 14:00   │
│    Progress: 45%            │    └─ 67% complete                    │
│                             │                                       │
│ 3. Personal Relations       │ 3. [ ] Review metrics    ⏰ 16:00    │
│    ████████████░░░░░░░░░░░░  │    └─ Pending                        │
│    Progress: 67%            │                                       │
│                             │                                       │
│                             │ [Enter] Details [Space] Toggle        │
└─────────────────────────────┴───────────────────────────────────────┘

When user presses Enter on "Prepare pitch deck":
┌─────────────────────────────┬─ Action Details ─┬─────────────────────┐
│                             │ ┌─ Pitch Deck ──┐ │                     │
│ 3. Personal Relations       │ │ Due: Today 2PM │ │ 3. Review metrics   │
│    ████████████░░░░░░░░░░░░  │ │ Status: 67%    │ │    └─ Pending       │
│    Progress: 67%            │ │ ─────────────  │ │                     │
│                             │ │ Notes:         │ │                     │
│                             │ │ - Slides 1-8   │ │                     │
│                             │ │   completed    │ │                     │
│                             │ │ - Need metrics │ │                     │
│                             │ │   from team    │ │                     │
│                             │ │ - Review at    │ │                     │
│                             │ │   1:30 PM      │ │                     │
│                             │ └───────────────┘ │                     │
└─────────────────────────────┴───────────────────┴─────────────────────┘
```

**Workflow Patterns**:
1. **Overview**: Base layout shows all outcomes and current focus actions simultaneously
2. **Navigation**: h/l to switch panes, j/k within panes for selection
3. **Detail Expansion**: Enter key overlays detail panel without changing base layout
4. **Quick Actions**: Space for toggle, number keys for direct action access
5. **Context Return**: ESC or second Enter press dismisses overlay, returning to exact previous state

**Pros**:
- **Persistent Overview**: Base information always visible, reducing cognitive load
- **Flexible Detail**: Overlays provide deep information without navigation penalty
- **Efficient Space Usage**: Two-pane layout optimizes for most terminal sizes
- **Graceful Degradation**: Overlays can be disabled on narrow terminals
- **Visual Continuity**: Less jarring than full modal switches

**Cons**:
- **Layout Complexity**: Managing overlay positioning and sizing increases complexity
- **Visual Clutter**: Overlays may obscure important background information
- **State Management**: More complex state tracking for overlay positions and content
- **Platform Constraints**: Terminal.Gui overlay support may be limited

**Implementation Considerations**:
- Use Terminal.Gui FrameView with dynamic positioning for overlays
- Implement overlay stack management for nested detail views
- Add configuration options for overlay transparency levels
- Support both overlay and modal modes for different terminal capabilities

## Key Tradeoffs/Limitations

### Information Architecture Constraints

**Hierarchical Display Limitations**: Terminal interfaces struggle with complex hierarchical information display. While goals naturally form trees (5-year → yearly → quarterly → monthly → weekly → daily), TUIs can only effectively show 2-3 levels simultaneously without overwhelming users. This forces design decisions about which levels to prioritize in the default view.

**Progress Visualization Challenges**: Effective progress communication in TUIs is limited to ASCII progress bars, color coding, and numerical percentages. Unlike GUIs, we cannot use rich charts, graphs, or visual metaphors. This constrains how effectively users can understand progress patterns and relationships between different goals.

**Temporal Navigation Complexity**: Moving between time periods (today, this week, this month) requires careful navigation design. Users need to maintain context about which time period they're viewing while having efficient ways to switch between periods. This temporal context switching can become a significant cognitive burden.

### Technical Implementation Constraints

**Platform Compatibility**: Terminal capabilities vary significantly across platforms and SSH sessions. Color support, Unicode character availability, and keyboard input handling differ between Windows Command Prompt, macOS Terminal, and various Linux terminal emulators. This requires defensive programming and graceful degradation strategies.

**Screen Size Variability**: Terminal windows range from mobile SSH sessions (24x80) to large desktop terminals (50x200+). The interface must work effectively across this range while providing enhanced experiences on larger screens. Fixed layouts often fail at the extremes.

**Performance Considerations**: TUI applications must remain responsive even with large goal datasets. Unlike web applications with lazy loading, TUI applications typically load all data at startup. With multiple years of goal data, rendering performance becomes critical.

**State Persistence Challenges**: TUI applications lack the automatic state management of web applications. Users expect their position, selections, and context to be preserved between sessions, requiring careful state serialization and restoration logic.

### User Experience Edge Cases

**Learning Curve vs. Efficiency**: Power users (executives) need maximum efficiency with minimal keystrokes, while occasional users need discoverability and clear instructions. Satisfying both groups simultaneously is challenging, especially with limited screen space for help text.

**Error Recovery**: When users make mistakes (wrong action toggle, incorrect navigation), TUI applications need clear error communication and easy recovery paths. Unlike GUIs, hover tooltips and immediate visual feedback are limited.

**Mobile Terminal Usage**: SSH sessions on mobile devices present unique challenges: smaller screens, touch-based keyboard input, and potential network latency. The interface must remain usable even under these constrained conditions.

**Accessibility Limitations**: Screen readers and terminal-based accessibility tools have limited support for complex TUI layouts. The interface must work effectively for users relying on screen readers or high-contrast displays.

### Domain-Specific Challenges

**Motivation vs. Overwhelm**: Goal tracking interfaces must balance showing progress (motivating) with showing remaining work (potentially overwhelming). Executives in high-pressure contexts need motivation without additional stress from interface complexity.

**Daily Session Time Pressure**: The 3-5 minute daily session constraint eliminates any workflow that requires more than 10-15 keystrokes to complete common tasks. This severely limits acceptable navigation depth and forces aggressive workflow optimization.

**Context Switching Costs**: High-performance users often switch between multiple projects, roles, and contexts throughout their day. The interface must minimize the mental overhead of re-establishing context when returning to goal tracking.

**Integration Requirements**: Goals don't exist in isolation - they connect to calendars, task managers, and communication tools. TUI applications have limited integration capabilities compared to web or desktop applications, potentially creating workflow friction.

## Recommendation & Next Steps

### Recommended Approach: Hybrid with Progressive Disclosure

After thorough analysis of the three approaches, I recommend implementing **Option 3 (Hybrid Approach with Floating Overlays)** with significant enhancements for progressive disclosure and mobile optimization.

This recommendation balances the research findings across successful TUI patterns (gitui's three-pane efficiency), goal tracking domain requirements (rapid daily sessions), and the specific constraints of executive/educator users in high-performance contexts.

### Core Architecture Decision

**Base Layout**: Two-pane horizontal split (Outcomes 40% | Actions 60%) optimized for the primary workflow of action completion within outcome context. This provides immediate visibility into both strategic level (outcomes) and tactical level (daily actions) without requiring navigation.

**Progressive Detail System**: Floating overlays activate on-demand for detailed information, notes, progress analytics, and configuration. Unlike full modals, these overlays preserve the base context while providing deep information access.

**Adaptive Mobile Mode**: When terminal width < 80 characters, automatically switch to single-pane modal mode (Option 2 pattern) to maintain usability on mobile SSH sessions.

### Key Implementation Features

**Smart Focus Management**: 
- Application launches with today's primary outcome pre-selected based on user-defined priorities or AI-suggested focus
- Current action position persists between sessions
- "Smart return" functionality restores exact cursor position after detail views

**Keyboard-Optimized Workflows**:
- Number keys (1-3) for direct outcome selection
- Space bar for action toggle (most common operation)
- Enter for action details/notes
- Tab for outcome switching
- Vim-style movement (hjkl) with arrow key fallback

**Visual Progress Communication**:
- Unicode block characters for progress bars (████░░░░)
- Color coding: Green (on track), Yellow (attention needed), Red (urgent)
- Percentage numbers for precise progress tracking
- Streak counters for motivation

**Context-Aware Information Display**:
- Time-sensitive information (due today, overdue) highlighted prominently
- Recent activity indicators (completed 2h ago, updated yesterday)
- Next action suggestions based on goal dependencies

### Progressive Enhancement Strategy

**Phase 1: Core MVP** (Weeks 1-2)
- Two-pane layout with basic navigation
- Action completion toggling
- Simple progress visualization
- Basic state persistence

**Phase 2: Mobile Optimization** (Weeks 3-4)
- Responsive layout switching
- Touch-friendly navigation
- Network optimization for SSH sessions
- Graceful degradation for limited terminals

**Phase 3: Advanced Features** (Weeks 5-6)
- Floating overlay detail system
- Progress analytics and reporting
- Goal dependency visualization
- Advanced keyboard shortcuts

**Phase 4: Integration & Polish** (Weeks 7-8)
- Configuration management
- Export/import capabilities
- Performance optimization
- Accessibility improvements

### Success Metrics for Validation

**Time-to-Daily-Complete**: Measure average time from application launch to completing daily session. Target: < 3 minutes for experienced users.

**Navigation Efficiency**: Track keystrokes required for common operations. Target: < 5 keystrokes for action toggle, < 8 keystrokes for outcome switching.

**Mobile Usability**: Test effectiveness on various mobile terminal applications and screen sizes. Target: Maintain core functionality on 24x80 displays.

**Learning Curve**: Measure time for new users to complete first successful daily session. Target: < 10 minutes with built-in help system.

### Risk Mitigation Strategies

**Technical Risks**: Implement comprehensive terminal compatibility testing across platforms. Create fallback modes for limited terminal capabilities.

**User Adoption Risks**: Design progressive disclosure system that scales from beginner to expert usage. Provide clear migration paths between complexity levels.

**Performance Risks**: Implement efficient data structures and rendering optimizations. Plan for goal datasets spanning multiple years.

**Integration Risks**: Design export/import systems early to enable data portability. Plan API interfaces for future integrations.

This hybrid approach provides the familiarity of proven TUI patterns while addressing the specific constraints and requirements of the FocusFive use case. The progressive enhancement strategy allows for rapid MVP delivery while providing clear paths for advanced functionality development.