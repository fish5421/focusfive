# 🎯 FocusFive - Start Here!

## The Most Important Thing to Know

**YOUR GOALS ARE STORED IN YOUR HOME DIRECTORY**, not in this project folder!

- ✅ Correct: `~/FocusFive/goals/` 
- ❌ Wrong: `~/projects/goal_setting/`

---

## Quick Start (3 minutes)

```bash
# 1. Build the app (one time only)
cargo build --release

# 2. Run the app (creates your goals file)
./target/release/focusfive

# 3. See where your goals are
echo "Your goals are at: ~/FocusFive/goals/"
ls ~/FocusFive/goals/

# 4. Edit today's goals
nano ~/FocusFive/goals/$(date +%Y-%m-%d).md

# 5. Check everything is working
./validate_setup.sh
```

---

## What You'll See

When you run the app:
```
FocusFive - Goal Tracking System
Goals directory: /Users/yourname/FocusFive/goals    ← THIS IS WHERE YOUR FILES ARE!
Created goals file: /Users/yourname/FocusFive/goals/2025-08-18.md

Progress: 0/9 actions completed
```

---

## Your Daily Workflow

### Morning (2 minutes)
```bash
# Run app to create today's file
./target/release/focusfive

# Edit your goals for today
nano ~/FocusFive/goals/$(date +%Y-%m-%d).md
```

### During the Day
Complete a task? Mark it done:
- Change `[ ]` to `[x]` in the file

### Evening (30 seconds)
```bash
# See your progress
./target/release/focusfive
# Shows: Progress: 5/9 actions completed
```

---

## File Format

Your markdown file looks like this:
```markdown
# August 18, 2025

## Work (Goal: Ship feature X)
- [x] Write code         ← Completed!
- [ ] Write tests        ← Not done yet
- [ ] Deploy to staging

## Health (Goal: Stay active)
- [x] Morning walk
- [ ] Healthy lunch
- [ ] Evening yoga

## Family (Goal: Be present)
- [ ] Breakfast together
- [x] Call mom
- [ ] Game night
```

---

## Common Confusion

### "I can't find my markdown files!"

They're NOT in the project folder. They're in your HOME directory:

```bash
# WRONG - Don't look here:
ls ~/projects/goal_setting/

# RIGHT - Look here instead:
ls ~/FocusFive/goals/
```

### "How do I edit the files?"

Any text editor works:
```bash
# Easy way (nano):
nano ~/FocusFive/goals/$(date +%Y-%m-%d).md

# Mac way:
open ~/FocusFive/goals/$(date +%Y-%m-%d).md

# VS Code:
code ~/FocusFive/goals/$(date +%Y-%m-%d).md
```

---

## Validation

Run this anytime to check your setup:
```bash
./validate_setup.sh
```

It will tell you exactly what's working and what needs fixing.

---

## Full Documentation

For complete instructions, see: `COMPLETE_USER_GUIDE.md`

---

## Remember

1. **Build once**: `cargo build --release`
2. **Run daily**: `./target/release/focusfive`
3. **Files are in**: `~/FocusFive/goals/` (NOT here!)
4. **Edit with**: Any text editor
5. **Mark complete**: Change `[ ]` to `[x]`

That's it! You're ready to track your goals! 🚀