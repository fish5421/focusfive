#!/bin/bash

# FocusFive Setup Validator
# Run this to check if everything is working correctly

echo "================================================"
echo "         FocusFive Setup Validator"
echo "================================================"
echo ""

# Color codes
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Counter for passed tests
PASSED=0
TOTAL=0

# Function to check a condition
check() {
    TOTAL=$((TOTAL + 1))
    if [ $1 -eq 0 ]; then
        echo -e "${GREEN}✅ $2${NC}"
        PASSED=$((PASSED + 1))
        return 0
    else
        echo -e "${RED}❌ $2${NC}"
        echo -e "   ${YELLOW}Fix: $3${NC}"
        return 1
    fi
}

echo "1. Checking Rust installation..."
rustc --version > /dev/null 2>&1
check $? "Rust is installed" "Install Rust: curl --proto='=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
echo ""

echo "2. Checking project structure..."
if [ -f "src/main.rs" ] && [ -f "Cargo.toml" ]; then
    check 0 "Project files found" ""
else
    check 1 "Project files found" "Make sure you're in the goal_setting directory"
fi
echo ""

echo "3. Checking if app is built..."
if [ -f "target/release/focusfive" ]; then
    check 0 "App is built" ""
else
    check 1 "App is built" "Run: cargo build --release"
fi
echo ""

echo "4. Checking if app runs..."
if [ -f "target/release/focusfive" ]; then
    ./target/release/focusfive > /dev/null 2>&1
    check $? "App runs successfully" "Check error messages above"
else
    echo -e "${YELLOW}⚠️  Skipping (app not built)${NC}"
fi
echo ""

echo "5. Checking goals directory..."
if [ -d "$HOME/FocusFive/goals" ]; then
    check 0 "Goals directory exists at ~/FocusFive/goals" ""
else
    check 1 "Goals directory exists" "Run the app once: ./target/release/focusfive"
fi
echo ""

echo "6. Checking for today's file..."
TODAY=$(date +%Y-%m-%d)
if [ -f "$HOME/FocusFive/goals/$TODAY.md" ]; then
    check 0 "Today's file exists: $TODAY.md" ""
    
    # Check file content
    if grep -q "## Work" "$HOME/FocusFive/goals/$TODAY.md" && \
       grep -q "## Health" "$HOME/FocusFive/goals/$TODAY.md" && \
       grep -q "## Family" "$HOME/FocusFive/goals/$TODAY.md"; then
        check 0 "File has correct structure (Work/Health/Family)" ""
    else
        check 1 "File has correct structure" "File may be corrupted. Delete and run app again."
    fi
else
    check 1 "Today's file exists" "Run the app: ./target/release/focusfive"
fi
echo ""

echo "7. Checking file permissions..."
if [ -r "$HOME/FocusFive/goals/$TODAY.md" ] && [ -w "$HOME/FocusFive/goals/$TODAY.md" ]; then
    check 0 "Goals file is readable and writable" ""
else
    check 1 "Goals file is readable and writable" "Check file permissions"
fi
echo ""

echo "================================================"
echo "                 SUMMARY"
echo "================================================"
echo ""

if [ $PASSED -eq $TOTAL ]; then
    echo -e "${GREEN}🎉 Perfect! Everything is set up correctly!${NC}"
    echo ""
    echo "Next steps:"
    echo "1. Edit your goals: nano ~/FocusFive/goals/$TODAY.md"
    echo "2. Mark tasks complete by changing [ ] to [x]"
    echo "3. Run app to see progress: ./target/release/focusfive"
else
    echo -e "${YELLOW}⚠️  Some issues found: $PASSED/$TOTAL checks passed${NC}"
    echo ""
    echo "Fix the issues above, then run this validator again."
fi
echo ""

echo "================================================"
echo "           QUICK REFERENCE"
echo "================================================"
echo "Your goals location: ~/FocusFive/goals/"
echo "Today's file: ~/FocusFive/goals/$TODAY.md"
echo "Run app: ./target/release/focusfive"
echo "Edit goals: nano ~/FocusFive/goals/$TODAY.md"
echo "================================================"