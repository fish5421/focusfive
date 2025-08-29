#!/bin/bash

echo "=== Creating Test Data for Data Capture Layer ==="
echo

# Run the app to create today's goals
echo "1. Creating today's goals file..."
cat > ~/Library/Application\ Support/FocusFive/goals/$(date +%Y-%m-%d).md << 'EOF'
# January 29, 2025 - Day 1

## Work (Goal: Test new features)
- [x] Implement data capture layer
- [ ] Write integration tests  
- [ ] Document changes

## Health (Goal: Stay active)
- [x] Morning walk
- [ ] Gym session
- [ ] Evening yoga

## Family (Goal: Quality time)
- [ ] Breakfast together
- [x] Help with homework
- [ ] Game night
EOF

echo "✓ Created today's markdown file"

# Create a test metadata file
echo "2. Creating metadata sidecar..."
cat > ~/Library/Application\ Support/FocusFive/meta/$(date +%Y-%m-%d).meta.json << EOF
{
  "version": 1,
  "date": "$(date +%Y-%m-%d)",
  "day_number": 1,
  "actions": [
    {
      "id": "550e8400-e29b-41d4-a716-446655440001",
      "text": "Implement data capture layer",
      "completed": true,
      "status": "Completed",
      "effort_minutes": 120,
      "notes": "Successfully implemented all core features",
      "created_at": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
      "completed_at": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
      "outcome_type": "Work",
      "action_index": 0
    },
    {
      "id": "550e8400-e29b-41d4-a716-446655440002",
      "text": "Write integration tests",
      "completed": false,
      "status": "InProgress",
      "effort_minutes": 45,
      "notes": "Working on test scenarios",
      "created_at": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
      "completed_at": null,
      "outcome_type": "Work",
      "action_index": 1
    }
  ],
  "notes": "First day using the new data capture system",
  "created_at": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
  "updated_at": "$(date -u +%Y-%m-%dT%H:%M:%SZ)"
}
EOF

echo "✓ Created metadata file"

# Create objectives file
echo "3. Creating objectives..."
cat > ~/Library/Application\ Support/FocusFive/objectives.json << 'EOF'
{
  "version": 1,
  "objectives": [
    {
      "id": "550e8400-e29b-41d4-a716-446655440010",
      "title": "Complete FocusFive v2.0",
      "description": "Ship the enhanced version with data capture",
      "outcome_type": "Work",
      "target_date": "2025-03-31",
      "key_results": [
        {
          "id": "550e8400-e29b-41d4-a716-446655440011",
          "description": "Data capture layer complete",
          "target_value": 100.0,
          "current_value": 85.0,
          "unit": "%"
        }
      ],
      "created_at": "2025-01-29T10:00:00Z",
      "updated_at": "2025-01-29T10:00:00Z",
      "archived": false
    }
  ]
}
EOF

echo "✓ Created objectives file"

# Create indicators file
echo "4. Creating indicators..."
cat > ~/Library/Application\ Support/FocusFive/indicators.json << 'EOF'
{
  "version": 1,
  "indicators": [
    {
      "id": "550e8400-e29b-41d4-a716-446655440020",
      "name": "Daily Focus Hours",
      "description": "Deep work hours per day",
      "outcome_type": "Work",
      "unit": "hours",
      "target_value": 4.0,
      "frequency": "Daily",
      "created_at": "2025-01-29T10:00:00Z",
      "archived": false
    }
  ]
}
EOF

echo "✓ Created indicators file"

# Create some observations
echo "5. Creating observations..."
cat > ~/Library/Application\ Support/FocusFive/observations.ndjson << 'EOF'
{"id":"550e8400-e29b-41d4-a716-446655440030","indicator_id":"550e8400-e29b-41d4-a716-446655440020","value":3.5,"notes":"Good focus session","observed_at":"2025-01-29T14:00:00Z","created_at":"2025-01-29T14:00:00Z"}
{"id":"550e8400-e29b-41d4-a716-446655440031","indicator_id":"550e8400-e29b-41d4-a716-446655440020","value":4.2,"notes":"Excellent deep work","observed_at":"2025-01-29T18:00:00Z","created_at":"2025-01-29T18:00:00Z"}
EOF

echo "✓ Created observations file"

echo
echo "=== Test Data Created Successfully ==="
echo
echo "Files created:"
ls -la ~/Library/Application\ Support/FocusFive/goals/$(date +%Y-%m-%d).md
ls -la ~/Library/Application\ Support/FocusFive/meta/$(date +%Y-%m-%d).meta.json
ls -la ~/Library/Application\ Support/FocusFive/objectives.json
ls -la ~/Library/Application\ Support/FocusFive/indicators.json
ls -la ~/Library/Application\ Support/FocusFive/observations.ndjson

echo
echo "You can view the data with:"
echo "  cat ~/Library/Application\ Support/FocusFive/meta/\$(date +%Y-%m-%d).meta.json | python3 -m json.tool"