# FocusFive Quick Start Guide

## Understanding the Features

### 1. Ritual Phases (m/n keys)

**What it is:** A time-aware greeting system that encourages different mindsets based on time of day.

- **Morning Phase (5am-11am):** "Good Morning! Time to set today's intentions"
- **Evening Phase (5pm-10pm):** "Evening Review - Reflect on your day"  
- **Normal Phase (other times):** "FocusFive - Daily Goal Tracker"

**How to use:**
- The app automatically detects the phase based on current time
- Press `m` to manually switch to Morning phase
- Press `n` to manually switch to Evening phase
- This only changes the greeting text at the top - functionality remains the same

### 2. Action Templates System

**What it is:** Save and reuse common action patterns to avoid retyping frequent tasks.

**How to CREATE a template:**
1. First, you need to have actions filled in
2. Navigate to an outcome (Work/Health/Family) using `j`/`k`
3. Press `e` to edit an action, type your action text, press Enter
4. Repeat for all 3 actions in that outcome
5. Once you have actions filled in, press `T` (capital T)
6. Enter a name for your template (e.g., "morning_routine")
7. Press Enter to save

**How to USE a template:**
1. Navigate to an outcome where you want to apply a template
2. Press `t` (lowercase t) to open template selector
3. Use `j`/`k` to select a template
4. Press Enter to apply it (fills empty action slots)
5. Press Esc to cancel

### 3. Copy from Yesterday

**What it is:** Quickly copy uncompleted tasks from yesterday's goals.

**How to use:**
1. Press `y` to open yesterday's actions
2. Use `j`/`k` to navigate through yesterday's actions
3. Press Space to select/deselect actions you want to copy
4. Press Enter to copy selected actions to today
5. Only fills empty slots - won't overwrite existing actions

## Step-by-Step Tutorial

### Your First Session

1. **Start the app:**
   ```bash
   ./target/release/focusfive
   ```

2. **Add your first actions:**
   - The cursor starts on "Work" outcome
   - Press `Tab` to move to the actions pane
   - Press `e` to edit the first action
   - Type: "Check emails and respond to urgent ones"
   - Press Enter to save
   - Press `j` to move to the next action
   - Press `e` and add another action
   - Repeat for all 3 actions

3. **Save as a template:**
   - With Work outcome selected and actions filled
   - Press `T` (capital T)
   - Type template name: "work_morning"
   - Press Enter

4. **Try the template tomorrow:**
   - Next day, navigate to Work outcome
   - Press `t` to see your saved templates
   - Select "work_morning" and press Enter
   - Your standard morning tasks are filled in!

5. **Mark actions complete:**
   - Navigate to an action
   - Press Space to toggle completion
   - Completed actions show [x] instead of [ ]

6. **Save and quit:**
   - Press `s` to save your progress
   - Press `q` to quit

## Common Issues

### "I press T but nothing happens"
- Make sure you have actions filled in first
- You need at least one non-empty action to save as template
- Use capital T, not lowercase t

### "Templates aren't showing up"
- Templates are saved in `~/.focusfive_templates.json`
- Check if the file exists: `ls -la ~ | grep focusfive`
- Templates persist between sessions

### "Phase doesn't seem to do anything"
- The phase only changes the greeting message at the top
- It's a subtle motivational feature, not a functional change
- Morning phase encourages planning, evening encourages reflection

## Keyboard Shortcuts Reference

| Key | Action | Context |
|-----|--------|---------|
| `Tab` | Switch between panes | Normal mode |
| `j`/`k` | Navigate up/down | Any list |
| `e` | Edit action | Actions pane |
| `Space` | Toggle completion | Actions pane |
| `v` | Edit 5-year vision | Outcomes pane |
| `g` | Edit goal | Outcomes pane |
| `y` | Copy from yesterday | Normal mode |
| `t` | Use template | Normal mode |
| `T` | Save as template | Normal mode |
| `m` | Morning phase | Normal mode |
| `n` | Evening phase | Normal mode |
| `s` | Save | Normal mode |
| `q` | Quit | Normal mode |
| `?` | Toggle help | Normal mode |

## Tips

1. **Build templates gradually:** Start with one outcome, create a template, test it works
2. **Use phases as reminders:** Morning = plan your day, Evening = review progress
3. **Copy from yesterday:** Great for recurring tasks or unfinished work
4. **Save often:** Press `s` regularly to save your progress

## File Locations

- Daily goals: `~/FocusFive/goals/YYYY-MM-DD.md`
- Templates: `~/.focusfive_templates.json`
- 5-year vision: `~/.focusfive_vision.json`

Need more help? Check the full USER_GUIDE.md or press `?` in the app for help.