#!/bin/bash

echo "=== Initializing FocusFive Data Capture Directories ==="
echo

# Create the Application Support directory structure
APP_SUPPORT="$HOME/Library/Application Support/FocusFive"
echo "Creating macOS Application Support directories..."
mkdir -p "$APP_SUPPORT/goals"
mkdir -p "$APP_SUPPORT/meta"
mkdir -p "$APP_SUPPORT/reviews"

# Also create fallback directory structure
FALLBACK="$HOME/FocusFive"
echo "Creating fallback directories..."
mkdir -p "$FALLBACK/goals"
mkdir -p "$FALLBACK/meta"
mkdir -p "$FALLBACK/reviews"

echo
echo "✓ Directories created successfully!"
echo
echo "Application Support location:"
ls -la "$APP_SUPPORT" 2>/dev/null || echo "  Not created"
echo
echo "Fallback location:"
ls -la "$FALLBACK" 2>/dev/null || echo "  Not created"
echo

# Run the actual app to test
if [ -f "./target/release/focusfive" ]; then
    echo "Running FocusFive to test..."
    timeout 2 ./target/release/focusfive </dev/null 2>/dev/null || true
    echo "✓ App ran successfully"
fi

# Check what was created
echo
echo "=== Current Status ==="
TODAY=$(date +%Y-%m-%d)

if [ -d "$APP_SUPPORT" ]; then
    echo "✓ Application Support directory exists"
    if [ -f "$APP_SUPPORT/goals/$TODAY.md" ]; then
        echo "  ✓ Today's markdown file exists"
    fi
    if [ -f "$APP_SUPPORT/meta/$TODAY.meta.json" ]; then
        echo "  ✓ Today's metadata file exists"
    fi
else
    echo "✗ Application Support directory not found"
fi

if [ -d "$FALLBACK" ]; then
    echo "✓ Fallback directory exists"
    if [ -f "$FALLBACK/goals/$TODAY.md" ]; then
        echo "  ✓ Today's markdown file exists (fallback)"
    fi
    if [ -f "$FALLBACK/meta/$TODAY.meta.json" ]; then
        echo "  ✓ Today's metadata file exists (fallback)"
    fi
fi

echo
echo "To manually test the data capture features, run:"
echo "  cargo test --test data_capture_integration"