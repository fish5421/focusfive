# Debug (Rust TUI)

You are tasked with helping debug issues during manual testing or implementation of the FocusFive Rust TUI application. This command guides investigation of logs-like data, persisted state, and git history without editing files. Use it to bootstrap a focused debugging session without burning the primary window's context.

## Initial Response

When invoked WITH a plan/ticket file:
```
I'll help debug issues with [file name]. Let me understand the current state.

What specific problem are you encountering?
- What were you trying to test/implement?
- What went wrong?
- Any error messages or unexpected behavior?

I'll investigate persisted data, recent activity, and git state to figure out what's happening.
```

When invoked WITHOUT parameters:
```
I'll help debug your current TUI issue.

Please describe what's going wrong:
- What are you working on?
- What specific problem occurred?
- When did it last work as expected?

I can inspect persisted files (goals, meta, objectives), recent observations, and recent code changes to identify the issue.
```

## Environment Information (Rust TUI)

FocusFive is a Rust TUI using ratatui + crossterm. It persists state to local files (no DB server or background daemon):

- Goals directory: default `~/FocusFive/goals` (fallback `./FocusFive/goals` if home not resolved)
- Data root directory (OS-specific via `directories::ProjectDirs::from("com", "Correia", "FocusFive")`):
  - macOS: `~/Library/Application Support/com.Correia.FocusFive`
  - Linux: `~/.local/share/FocusFive`
  - Fallback: parent of goals dir (e.g., `~/FocusFive`)

Key files under these locations:
- Goals markdown: `<goals_dir>/YYYY-MM-DD.md`
- Day metadata: `<data_root>/meta/YYYY-MM-DD.meta.json`
- Vision: `<parent_of_goals_dir>/vision.json`
- Templates: `<parent_of_goals_dir>/templates.json`
- Objectives: `<data_root>/objectives.json`
- Indicators: `<data_root>/indicators.json`
- Observations (append-only NDJSON): `<data_root>/observations.ndjson`
- Weekly reviews: `<data_root>/reviews/YYYY-Www.json`

Git is used for source control; there are no daemon/WUI processes in this app.

## Process Steps

### Step 1: Understand the Problem

After the user describes the issue:

1. Read any provided context (plan/ticket):
   - Understand what they're implementing/testing
   - Note which feature area or step they're on
   - Identify expected vs actual behavior

2. Quick state check:
   - Current git branch and recent commits
   - Any uncommitted changes
   - When the issue started occurring

### Step 2: Investigate the Issue

Spawn parallel Task agents for efficient investigation:

Task 1 - Recent Activity and Errors:
1. Resolve data locations (non-destructive checks):
   - macOS: `ls -d "$HOME/Library/Application Support/com.Correia.FocusFive" 2>/dev/null || true`
   - Linux: `ls -d "$HOME/.local/share/FocusFive" 2>/dev/null || true`
   - Fallback: `ls -d "$HOME/FocusFive" 2>/dev/null || true`
2. Inspect observations (if present):
   - `tail -n 100 "$DATA_ROOT/observations.ndjson" 2>/dev/null`
   - If `jq` is available: `jq -c . "$DATA_ROOT/observations.ndjson" | tail -n 50`
3. Check for warnings/errors emitted previously (TUI uses stderr):
   - Reproduction guidance: run `cargo run` from a separate terminal, observe stderr messages printed (e.g., save failures, parse warnings). The app does not write a structured log file by default.
4. Find latest goals and meta files to correlate timestamps:
   - Latest goals: `ls -t "$GOALS_DIR"/*.md | head -1`
   - Matching meta: `ls -t "$DATA_ROOT"/meta/*.meta.json | head -1`

Return: Recent observations (if any), latest file timestamps, and any reproduction stderr.

Task 2 - Persisted Data State:
1. Goals integrity:
   - Verify today’s file exists: `ls "$GOALS_DIR/$(date +%F).md" 2>/dev/null || echo "No goals for today"`
   - Open and skim: `sed -n '1,80p' "$GOALS_DIR/$(date +%F).md" 2>/dev/null`
2. Metadata consistency:
   - `sed -n '1,200p' "$DATA_ROOT/meta/$(date +%F).meta.json" 2>/dev/null`
   - Check counts align with actions per outcome; look for obvious nulls/shape mismatches
3. Global stores:
   - Objectives: `sed -n '1,200p' "$DATA_ROOT/objectives.json" 2>/dev/null`
   - Indicators: `sed -n '1,200p' "$DATA_ROOT/indicators.json" 2>/dev/null`
   - Templates/Vision: `sed -n '1,200p' "$(dirname "$GOALS_DIR")/templates.json" 2>/dev/null`; `sed -n '1,160p' "$(dirname "$GOALS_DIR")/vision.json" 2>/dev/null`
4. Look for stuck states or anomalies (mismatched lengths, invalid enum strings, missing files).

Return: Data anomalies, missing files, and schema/shape mismatches.

Task 3 - Git and File State:
1. Git status: `git status --porcelain=v1 -b`
2. Recent commits: `git log --oneline -n 15`
3. Uncommitted diff (scan, but do not paste entire diff unless asked): `git diff --stat`
4. Verify expected files exist (e.g., `src/app.rs`, `src/data.rs`, `src/ui.rs`)

Return: Git branch, recent changes that might be related, any file state issues.

Optional Task 4 - Build/Test Sanity (non-interactive):
- `cargo check` or `cargo test -q` if safe to run in the environment; capture failures and error messages. Do not run the interactive TUI inside the same window.

### Step 3: Present Findings

Use this structure to report back:

````markdown
## Debug Report

### What's Wrong
[Clear statement of the issue based on evidence]

### Evidence Found

**From Observations** (`$DATA_ROOT/observations.ndjson`):
- [Recent event/timestamp]
- [Patterns or repeated anomalies]

**From Persisted Data**:
- goals: `<goals_dir>/YYYY-MM-DD.md` — [finding]
- meta: `<data_root>/meta/YYYY-MM-DD.meta.json` — [finding]
- objectives/indicators — [counts, missing fields]

**From Git/Files**:
- [Recent changes that might be related]
- [Missing or unexpected files]

### Root Cause
[Most likely explanation based on evidence]

### Next Steps

1. Try this first:
   ```bash
   [specific check or file to fix (no edits here), or a reproduction command]
   ```
2. If that doesn't work:
   - Run `cargo check` (or `cargo test -q`) and share the exact error
   - Re-run the TUI in a separate terminal and capture stderr
   - Confirm actual `goals_dir` and `data_root` paths with the OS-specific checks

### Can't Access?
Some things are outside this command's reach:
- Live TUI interaction in this window (run it in a separate terminal)
- OS keychain/permissions dialogs
- System-level or terminal configuration issues

Would you like me to investigate something specific further?
````

## Important Notes

- This app does not run background services; ignore daemon/WUI steps from prior projects
- No SQLite database; state is in markdown/JSON/NDJSON files
- Focus on manual testing scenarios and persisted state verification
- Always request a problem description; debugging without specifics leads to guesswork
- No file editing in this command; propose fixes but do not modify files here

## Quick Reference (Rust TUI)

- Locate directories:
  - macOS data root: `echo "$HOME/Library/Application Support/com.Correia.FocusFive"`
  - Linux data root: `echo "$HOME/.local/share/FocusFive"`
  - Goals dir: `echo "$HOME/FocusFive/goals"`

- Latest files:
  ```bash
  ls -t "$GOALS_DIR"/*.md | head -1
  ls -t "$DATA_ROOT"/meta/*.meta.json | head -1
  ```

- Inspect stores quickly (safe reads):
  ```bash
  sed -n '1,80p' "$GOALS_DIR/$(date +%F).md"
  sed -n '1,120p' "$DATA_ROOT/objectives.json"
  sed -n '1,120p' "$DATA_ROOT/indicators.json"
  tail -n 50 "$DATA_ROOT/observations.ndjson"
  ```

- Git state:
  ```bash
  git status --porcelain=v1 -b
  git log --oneline -n 15
  git diff --stat
  ```

- Repro guidance (separate terminal):
  ```bash
  cargo run   # 'q' to quit, watch stderr for warnings
  ```

---

Implementation details referenced by code:
- Paths: `Config::new()` in `src/models.rs` sets `goals_dir` and `data_root`
- Files used across the app: `src/data.rs` reads/writes goals, meta, objectives, indicators, observations, reviews
- TUI entrypoint: `src/main.rs` (ratatui + crossterm), no background services

This command is updated for the Rust TUI environment and replaces prior WUI/daemon log/database steps.

Helper scripts in repo:
- `./debug_goals.sh` prints detected goals locations and shows today’s file if present.
