# FocusFive User Guide - Complete Terminal UI Guide

## 🚀 Quick Start (2 minutes)

### Step 1: Build and Install
```bash
# Build FocusFive
cd /Users/petercorreia/projects/goal_setting
cargo build --release

# Create an alias for easy access
echo "alias focus='$(pwd)/target/release/focusfive'" >> ~/.zshrc
source ~/.zshrc
```

### Step 2: Launch the Terminal UI
```bash
# Run the app - opens interactive terminal interface
./target/release/focusfive

# Or if you set up the alias:
focus
```

## 🎮 Terminal UI Overview

FocusFive now features a full **Terminal User Interface (TUI)** with two-pane layout:

```
┌─────────────────────────┬──────────────────────────────────┐
│  OUTCOMES               │  ACTIONS                         │
├─────────────────────────┼──────────────────────────────────┤
│ > Work [2/4]            │  Work Actions:                   │
│   Health [1/3]          │  [x] Fix critical bug            │
│   Family [0/2]          │  [ ] Code review                 │
│                         │  [x] Update docs                 │
│                         │  [ ] Deploy to staging           │
│                         │                                  │
│                         │  Goal: Ship new feature          │
│                         │  Vision: Deliver value...        │
└─────────────────────────┴──────────────────────────────────┘
Status: Tab: Switch | j/k: Navigate | Space: Toggle | a: Add | d: Delete | q: Quit
```

## 🎯 Key Features

### Variable Actions (1-5 per outcome)
- **Not limited to 3 actions anymore!** Each outcome can have 1-5 actions
- Press `a` to add new actions (up to 5)
- Press `d` to delete actions (minimum 1 required)
- Progress dynamically updates (e.g., [2/4] means 2 of 4 completed)

### Morning Ritual Phase
When you start the app each day:
1. **Set Intentions**: Review and adjust your daily actions
2. **Apply Templates**: Press `1-9` to apply saved templates
3. **Edit Actions**: Press `e` to modify any action
4. **Add/Remove**: Use `a` and `d` to adjust action count

### Evening Ritual Phase
At day's end, the app enters reflection mode:
- **Easy Complete**: Press `a-z` to quickly mark actions done
- **Reflection**: Press `r` to write end-of-day reflection
- **Review Stats**: See completion percentage for each outcome

## ⌨️ Complete Keyboard Shortcuts

### Navigation
- **Tab** - Switch between Outcomes and Actions panes
- **j/k** or **↓/↑** - Navigate up/down in lists
- **Space** - Toggle action completion (check/uncheck)

### Editing Features
- **e** - Edit selected action text
- **g** - Edit goal for selected outcome (max 100 chars)
- **v** - Edit vision for selected outcome (multi-line)
- **r** - Add/edit reflection (end of day thoughts)
- **a** - Add new action (max 5 per outcome)
- **d** - Delete selected action (min 1 required)

### Templates
- **T** (Shift+T) - Save current actions as template
- **1-9** - Apply template by number (morning phase)
- **0** - Clear all templates

### Save Options
Multiple ways to save when editing:
- **Enter** - Save (in action edit mode)
- **F2** - Universal save key (works everywhere)
- **Ctrl+S** - Standard save shortcut
- **Ctrl+Enter** - Windows/Linux save
- **Cmd+Enter** - Mac save (if supported)
- **Esc** - Cancel without saving
- **s** - Manual save all changes (in normal mode)

### Other Commands
- **?** - Show help screen
- **q** - Quit application

## 🌅 Daily Workflow with TUI

### Morning Ritual (2 minutes)
1. **Launch FocusFive**
   ```bash
   focus
   ```

2. **Morning Phase automatically starts**
   - Review yesterday's uncompleted tasks
   - Apply templates with number keys (1-9)
   - Edit actions with 'e'
   - Add/remove actions with 'a'/'d'
   - Set goals with 'g'
   - Define vision with 'v'

3. **Navigate and Edit**
   - Use Tab to switch panes
   - Use j/k to move between items
   - Press 'e' to edit any action
   - Press Space to mark complete

### Throughout the Day
- Quick launch to toggle completions
- No manual file editing needed!
- Auto-saves on every change

### Evening Reflection (1 minute)
1. **Launch for Evening Phase**
   ```bash
   focus
   ```

2. **Quick Complete Mode**
   - Actions show with letter shortcuts (a, b, c...)
   - Press the letter to mark complete
   - See real-time progress updates

3. **Add Reflection**
   - Press 'r' to write reflection
   - Multi-line text supported
   - Save with F2 or Ctrl+S

## 📝 The Markdown Format

Behind the scenes, FocusFive stores your data in clean markdown files:

```markdown
# January 17, 2025

## Work (Goal: Ship new feature)
- [x] Fix critical bug
- [ ] Code review
- [x] Update documentation  
- [ ] Deploy to staging

## Health (Goal: Exercise daily)
- [x] Morning workout
- [ ] Healthy lunch
- [ ] Evening walk

## Family (Goal: Quality time)
- [ ] Family dinner
- [x] Call parents
```

### Variable Actions Support
Each outcome can now have **1-5 actions** instead of fixed 3:
- Work might have 4 critical tasks
- Health might have 2 simple goals
- Family might have 5 activities planned

## 🎯 Advanced Features

### Goals
- Press **g** in Outcomes pane to set/edit goals
- Goals appear next to outcome: "Work [2/4] - Ship new feature"
- Max 100 characters per goal
- Saved in markdown as: `## Work (Goal: Ship new feature)`

### Visions
- Press **v** to define long-term vision for each outcome
- Multi-line text supported
- Helps maintain focus on bigger picture
- Appears below actions in the UI

### Templates System
Save time with reusable action templates:

1. **Create Template**: Press **T** when you have good actions set up
2. **Name It**: Give template a memorable name
3. **Apply Later**: Press **1-9** in morning phase to apply
4. **Clear All**: Press **0** to remove all templates

Example templates:
- "Deep Work Day" - Focus on coding tasks
- "Meeting Heavy" - Communication focused
- "Recovery Day" - Light tasks for rest days

### Reflections
End each day with insights:
- Press **r** in evening phase
- Write multi-line reflection
- Review patterns over time
- Stored in: `~/FocusFive/reflections/`

## 🎯 Real Usage Examples

### Example 1: Software Developer's Day
```markdown
# January 17, 2025 - Day 5

## Work (Goal: Launch MVP)
- [x] Fix login bug
- [x] Deploy to staging
- [ ] Write release notes

## Health (Goal: Reduce stress)
- [x] 5-minute meditation
- [ ] Lunch away from desk
- [ ] Evening walk

## Family (Goal: Be present)
- [x] Morning coffee with spouse
- [ ] Call parents
- [ ] Read bedtime story
```

### Example 2: Student's Day
```markdown
# January 17, 2025 - Day 12

## Work (Goal: Ace midterms)
- [x] Study Chapter 5
- [x] Complete problem set
- [ ] Review notes

## Health (Goal: Better sleep)
- [ ] No caffeine after 2pm
- [x] 30-min exercise
- [ ] Bed by 11pm

## Family (Goal: Stay connected)
- [x] Call mom
- [ ] Game night with roommates
- [ ] Plan weekend trip
```

## 💡 Pro Tips

### Tip 1: Quick Access Aliases
```bash
# Add to ~/.zshrc or ~/.bashrc
alias focus='~/projects/goal_setting/target/release/focusfive'
alias goals='cd ~/FocusFive/goals'
alias today='vim ~/FocusFive/goals/$(date +%Y-%m-%d).md'
alias progress='grep -c "\[x\]" ~/FocusFive/goals/$(date +%Y-%m-%d).md'
```

### Tip 2: Quick Edit Commands
```bash
# Mark task complete (if you know the line number)
sed -i '' '7s/\[ \]/\[x\]/' ~/FocusFive/goals/$(date +%Y-%m-%d).md

# See today's goals
cat ~/FocusFive/goals/$(date +%Y-%m-%d).md

# See this week's files
ls -la ~/FocusFive/goals/*.md | tail -7
```

### Tip 3: Track Your Streak
```bash
# Count consecutive days with goals
ls ~/FocusFive/goals/*.md | wc -l

# See completion rate over time
for f in ~/FocusFive/goals/*.md; do
    echo -n "$(basename $f): "
    echo "$(grep -c '\[x\]' $f)/9 completed"
done
```

### Tip 4: Git Integration
```bash
cd ~/FocusFive/goals
git init
git add .
git commit -m "Goals for $(date +%Y-%m-%d)"

# Daily commit
echo "git add . && git commit -m 'EOD $(date +%Y-%m-%d)'" >> ~/.zshrc
alias eod='cd ~/FocusFive/goals && git add . && git commit -m "EOD $(date +%Y-%m-%d)"'
```

## 🗓️ Weekly Review

### See the Week's Progress
```bash
# Create a weekly summary
for f in ~/FocusFive/goals/*.md; do
    if [ -f "$f" ]; then
        echo "=== $(basename $f .md) ==="
        echo "Completed: $(grep -c '\[x\]' $f)/9"
        echo "Work: $(grep -A3 '^## Work' $f | grep -c '\[x\]')/3"
        echo "Health: $(grep -A3 '^## Health' $f | grep -c '\[x\]')/3"
        echo "Family: $(grep -A3 '^## Family' $f | grep -c '\[x\]')/3"
        echo ""
    fi
done
```

## 🚨 Troubleshooting

### Issue: Command+Enter not saving
**Solution**: Use **F2** or **Ctrl+S** instead - these work universally across all terminals

### Issue: Can't save vision/reflection (Enter adds newlines)
**Solution**: Use **F2**, **Ctrl+S**, or **Ctrl+Enter** to save multi-line content

### Issue: Template creation with 'T' not working
**Solution**: Make sure you're pressing capital T (Shift+T). If that doesn't work, check if your terminal passes through shift modifiers

### Issue: Actions still showing after delete
**Solution**: Press **s** to manually save, or changes will auto-save when you quit with **q**

### Issue: "No goal set" showing
**Solution**: Press **g** in the Outcomes pane to edit the goal for that outcome

### Issue: Progress count not updating
**Solution**: The count updates dynamically as you add/remove actions. If stuck, press **s** to save and refresh

### Issue: Can't add more than 3 actions
**Solution**: Press **a** when in Actions pane to add up to 5 actions per outcome

### Issue: Info messages showing as errors
**Solution**: Green messages are info, red are errors. This is working correctly in latest version

## 📱 Mobile Access (Bonus)

### Using iCloud Drive
```bash
# Move goals to iCloud
mv ~/FocusFive ~/Library/Mobile\ Documents/com~apple~CloudDocs/
ln -s ~/Library/Mobile\ Documents/com~apple~CloudDocs/FocusFive ~/FocusFive

# Now edit on iPhone/iPad with any markdown app
```

### Using Git + Working Copy (iOS)
```bash
# Push to GitHub
cd ~/FocusFive/goals
git remote add origin https://github.com/yourusername/goals.git
git push -u origin main

# Clone in Working Copy app on iOS
# Edit goals on the go!
```

## 🎉 You're Ready!

Start using the Terminal UI now:
```bash
# Just one command needed!
focus

# That's it! Everything else is done through the TUI:
# - Edit actions with 'e'
# - Set goals with 'g'  
# - Define visions with 'v'
# - Add/remove actions with 'a'/'d'
# - Save templates with 'T'
# - Mark complete with Space
# - Quick complete in evening with letter keys
# - Save changes with 's' or auto-save on quit
```

## 📊 File Locations

FocusFive creates these directories:
- `~/FocusFive/goals/` - Daily markdown files
- `~/FocusFive/reflections/` - End-of-day reflections
- `~/FocusFive/visions/` - Long-term visions
- `~/.focusfive_templates.json` - Saved templates

## 🔑 Key Principles

1. **Variable Actions**: 1-5 actions per outcome (not fixed at 3)
2. **Ritual Phases**: Morning for planning, Evening for reflection
3. **Templates**: Save time with reusable action sets
4. **Goals & Visions**: Short-term goals + long-term visions
5. **3-Minute Daily**: Quick morning setup, quick evening review

Remember: **3 outcomes, variable actions, 3 minutes per day!**