# FocusFive Architecture Mindmap

## Comprehensive System Architecture Diagram

```mermaid
graph TB
    %% Core Entry Point
    Main[main.rs<br/>Entry Point]
    
    %% Primary Components
    Main --> App[App State<br/>app.rs]
    Main --> TUI[Terminal UI<br/>ui.rs]
    Main --> EventLoop[Event Loop<br/>250ms polling]
    
    %% Data Model Layer
    subgraph DataModels[" 📊 Data Models - models.rs "]
        CoreModels[Core Models]
        CoreModels --> DailyGoals[DailyGoals<br/>Fixed 3 Outcomes]
        CoreModels --> Outcome[Outcome<br/>1-5 Actions<br/>Variable Count]
        CoreModels --> Action[Action<br/>ID, Text, Status<br/>500 char limit]
        
        EnhancedModels[Enhanced Models]
        EnhancedModels --> ActionMeta[ActionMeta<br/>UUID, Status, Origin<br/>Time Tracking]
        EnhancedModels --> DayMeta[DayMeta<br/>Metadata Container<br/>Version Control]
        EnhancedModels --> Objectives[Objectives<br/>Long-term Goals<br/>Hierarchical]
        EnhancedModels --> Indicators[Indicators<br/>KPIs & Metrics<br/>Leading/Lagging]
        EnhancedModels --> Observations[Observations<br/>Time-series Data<br/>Measurements]
        
        Constraints[Constraints & Validation]
        Constraints --> FixedOutcomes[3 Fixed Outcomes<br/>Work, Health, Family]
        Constraints --> ActionLimits[1-5 Actions<br/>per Outcome]
        Constraints --> TextLimits[Text Limits<br/>Action: 500<br/>Goal: 100<br/>Vision: 1000]
    end
    
    %% Storage Layer
    subgraph Storage[" 💾 Storage Layer "]
        FileSystem[File System Structure]
        FileSystem --> GoalsDir[~/FocusFive/goals/<br/>Markdown Files]
        FileSystem --> MetaDir[~/FocusFive/meta/<br/>JSON Sidecars]
        FileSystem --> DataFiles[Data Files<br/>vision.json<br/>templates.json<br/>objectives.json<br/>indicators.json<br/>observations.ndjson]
        
        IOPatterns[I/O Patterns<br/>data.rs]
        IOPatterns --> AtomicWrite[Atomic Write<br/>Temp + Rename<br/>Concurrency Safe]
        IOPatterns --> MarkdownParser[Markdown Parser<br/>Flexible Headers<br/>Case-insensitive]
        IOPatterns --> JSONHandler[JSON Handler<br/>Schema Versioned<br/>Backward Compatible]
        
        DataCapture[Data Capture<br/>data_capture.rs]
        DataCapture --> Reconciliation[Reconciliation<br/>Sync Markdown ↔ JSON]
        DataCapture --> StreamingObs[Streaming NDJSON<br/>Append-only<br/>Time-series]
    end
    
    %% UI Layer
    subgraph UILayer[" 🖥️ Terminal UI Layer "]
        UIArchitecture[UI Architecture]
        UIArchitecture --> Ratatui[Ratatui Framework<br/>Event-driven<br/>60fps rendering]
        UIArchitecture --> Layout[Layout System<br/>Header 3 lines<br/>Content flex<br/>Footer 1 line]
        UIArchitecture --> TwoPanes[Two-Pane Layout<br/>Outcomes 40%<br/>Actions 60%]
        
        InputModes[12 Input Modes]
        InputModes --> Normal[Normal<br/>Navigation]
        InputModes --> Editing[Editing Modes<br/>Action, Goal, Vision]
        InputModes --> Modals[Modal Overlays<br/>Templates<br/>Objectives<br/>Indicators<br/>Copy Yesterday]
        
        Navigation[Navigation]
        Navigation --> PaneSwitch[Tab: Switch Panes]
        Navigation --> Movement[j/k: Up/Down<br/>Space: Toggle Status]
        Navigation --> Shortcuts[Shortcuts<br/>t: Templates<br/>o: Objectives<br/>y: Yesterday<br/>i: Indicators]
        
        Phases[Ritual Phases]
        Phases --> Morning[Morning Phase<br/>5am-12pm<br/>☀️ Yellow/Green<br/>Planning Focus]
        Phases --> Evening[Evening Phase<br/>5pm-11pm<br/>🌙 Blue/Magenta<br/>Reflection Focus]
    end
    
    %% State Management
    subgraph StateManagement[" 🔄 State Management "]
        AppState[App State<br/>app.rs:89-118]
        AppState --> CoreState[Core Data<br/>Goals, Config<br/>Vision, Templates]
        AppState --> NavState[Navigation<br/>Pane, Indices<br/>Input Mode]
        AppState --> SaveFlags[Save Flags<br/>needs_save<br/>atomic saves]
        
        StateTransitions[State Transitions]
        StateTransitions --> ActionCycle[Action Status Cycle<br/>Planned→InProgress<br/>→Done→Skipped<br/>→Blocked→Planned]
        StateTransitions --> ModalChain[Modal Chains<br/>Objective→Indicator]
        StateTransitions --> SaveCascade[Save Cascade<br/>Goals→Meta→Vision<br/>→Templates→Objectives<br/>→Indicators]
    end
    
    %% Integration Points
    subgraph Integration[" 🔗 Integration Points "]
        EventHandling[Event Handling<br/>main.rs:64-217]
        EventHandling --> KeyEvents[Key Events<br/>handle_key function]
        EventHandling --> SaveTriggers[Save Triggers<br/>On modification]
        EventHandling --> ErrorHandling[Error Handling<br/>Result propagation<br/>Graceful fallbacks]
        
        DataFlow[Data Flow]
        DataFlow --> UserToStorage[User Input →<br/>App State →<br/>Markdown →<br/>Filesystem]
        DataFlow --> StorageToUI[Filesystem →<br/>Parse →<br/>App State →<br/>UI Render]
        
        Validation[Validation]
        Validation --> CompileTime[Compile-time<br/>Enum exhaustiveness<br/>Array bounds]
        Validation --> Runtime[Runtime<br/>Text limits<br/>Action counts<br/>Input validation]
    end
    
    %% Key Features
    subgraph Features[" ✨ Key Features "]
        Templates[Templates System<br/>Reusable Actions<br/>Quick Application]
        YesterdayCopy[Yesterday Copy<br/>Smart Selection<br/>Incomplete Focus]
        StreakTracking[Streak Tracking<br/>Daily Calculation<br/>365 day limit]
        Statistics[Statistics<br/>Completion %<br/>Best/Worst<br/>Progress Gauge]
        ObjectiveLink[Objective Linking<br/>Action→Objective<br/>Hierarchical Goals]
        IndicatorTrack[Indicator Tracking<br/>KPI Management<br/>Observations]
    end
    
    %% Performance Characteristics
    subgraph Performance[" ⚡ Performance "]
        Metrics[Performance Metrics]
        Metrics --> StartupTime[Startup < 100ms]
        Metrics --> SaveTime[Save < 50ms]
        Metrics --> Memory[Memory < 10MB]
        Metrics --> RenderCycle[Render < 16ms<br/>60fps achieved]
        Metrics --> Concurrency[0% collision rate<br/>Unique temp files]
    end
    
    %% Connections between major components
    App --> DataModels
    App --> StateManagement
    App --> UILayer
    
    EventLoop --> EventHandling
    EventLoop --> SaveCascade
    
    TUI --> UILayer
    TUI --> Ratatui
    
    DataModels --> Storage
    StateManagement --> DataFlow
    UILayer --> EventHandling
    
    Storage --> AtomicWrite
    Storage --> Reconciliation
    
    Features --> Templates
    Features --> YesterdayCopy
    Features --> Statistics
    
    style DataModels fill:#e1f5fe
    style Storage fill:#fff3e0
    style UILayer fill:#f3e5f5
    style StateManagement fill:#e8f5e9
    style Integration fill:#fce4ec
    style Features fill:#fff9c4
    style Performance fill:#e0f2f1
```

## Architecture Summary

### Core Design Principles

1. **Fixed 3x3 Constraint**: Exactly 3 life outcomes (Work, Health, Family) with 1-5 actions each
2. **Dual Storage**: Human-readable markdown + machine-readable JSON metadata
3. **Atomic Operations**: All writes use temp file + rename for data integrity
4. **Event-Driven UI**: Responsive terminal interface with modal overlays
5. **Type Safety**: Compile-time constraints with runtime validation

### Key Architectural Patterns

#### Data Layer
- **Enum-enforced constraints** for the 3 fixed outcomes
- **Variable action support** (1-5 per outcome) with backward compatibility
- **UUID-based identity** for stable action references
- **Schema versioning** for forward compatibility

#### Storage Layer
- **Atomic write pattern** prevents data corruption
- **Dual-schema strategy** balances human and machine needs
- **NDJSON streaming** for efficient time-series data
- **Graceful fallbacks** for missing or corrupted files

#### UI Layer
- **Single-threaded event loop** with 250ms polling
- **Modal state machine** for complex interactions
- **Phase-aware interface** (Morning/Evening modes)
- **Two-pane layout** with keyboard navigation

#### Integration Layer
- **Save cascade** ensures data consistency
- **Reconciliation system** keeps markdown and JSON in sync
- **Error propagation** with Result types throughout
- **Validation at multiple levels** (compile-time, parse-time, runtime)

### Data Flow Paths

1. **User Input → Storage**:
   - Keyboard event → App state change → Save flag set → Markdown generation → Atomic write → Metadata sync

2. **Storage → Display**:
   - File read → Parse markdown → Create data structures → Load metadata → Reconcile → Render UI

3. **Cross-Component Communication**:
   - Actions link to objectives via UUID
   - Templates apply to empty action slots
   - Indicators track via observations
   - Metadata enhances markdown data

### Performance Characteristics

- **Startup**: < 100ms (target: < 500ms) ✅
- **Save**: < 50ms (target: < 100ms) ✅  
- **Memory**: < 10MB (target: < 50MB) ✅
- **Rendering**: < 16ms per frame (60fps) ✅
- **Concurrency**: 0% collision rate ✅

### File Organization

```
~/FocusFive/                    # User data directory
├── goals/                      # Daily markdown files
├── meta/                       # JSON metadata sidecars  
├── reviews/                    # Weekly/monthly reviews
├── vision.json                 # 5-year vision
├── templates.json              # Action templates
├── objectives.json             # Long-term goals
├── indicators.json             # KPI definitions
└── observations.ndjson         # Time-series data

/project/src/                   # Source code
├── main.rs                     # Entry point & event loop
├── app.rs                      # Application state
├── ui.rs                       # Terminal UI rendering
├── models.rs                   # Data structures
├── data.rs                     # File I/O & parsing
└── data_capture.rs             # Enhanced storage
```

This architecture successfully balances simplicity (3-minute daily use) with sophistication (objectives, indicators, templates) while maintaining local-first privacy and robust data integrity.