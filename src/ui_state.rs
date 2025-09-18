use std::collections::HashSet;

/// State management for expandable action list UI
pub struct ExpandableActionState {
    pub expanded_actions: HashSet<String>,      // Which actions are expanded (by UUID)
    pub selected_action_index: usize,           // Current selection
    pub selected_indicator_index: Option<usize>, // Sub-selection within expanded
    pub indicator_update_mode: bool,            // Whether we're in update mode
    pub update_buffer: String,                  // Input buffer for updates
}

impl Default for ExpandableActionState {
    fn default() -> Self {
        Self::new()
    }
}

impl ExpandableActionState {
    pub fn new() -> Self {
        Self {
            expanded_actions: HashSet::new(),
            selected_action_index: 0,
            selected_indicator_index: None,
            indicator_update_mode: false,
            update_buffer: String::new(),
        }
    }

    /// Toggle expansion of an action
    pub fn toggle_expansion(&mut self, action_id: String) {
        if self.expanded_actions.contains(&action_id) {
            self.expanded_actions.remove(&action_id);
            self.selected_indicator_index = None; // Clear sub-selection when collapsing
        } else {
            self.expanded_actions.insert(action_id);
        }
    }
    
    /// Check if an action is expanded
    pub fn is_expanded(&self, action_id: &str) -> bool {
        self.expanded_actions.contains(action_id)
    }

    /// Clear all expanded states
    pub fn clear_expansions(&mut self) {
        self.expanded_actions.clear();
        self.selected_indicator_index = None;
    }

    /// Enter indicator update mode
    pub fn enter_update_mode(&mut self) {
        self.indicator_update_mode = true;
        self.update_buffer.clear();
    }

    /// Exit indicator update mode
    pub fn exit_update_mode(&mut self) {
        self.indicator_update_mode = false;
        self.update_buffer.clear();
    }

    /// Navigate down through actions and expanded indicators
    pub fn navigate_down(&mut self, max_items: usize) {
        if self.selected_action_index < max_items.saturating_sub(1) {
            self.selected_action_index += 1;
        }
    }

    /// Navigate up through actions and expanded indicators
    pub fn navigate_up(&mut self) {
        if self.selected_action_index > 0 {
            self.selected_action_index -= 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_expansion_toggle() {
        let mut state = ExpandableActionState::new();
        let action_id = "test-uuid-123".to_string();

        // Initially not expanded
        assert!(!state.is_expanded(&action_id));

        // Toggle to expand
        state.toggle_expansion(action_id.clone());
        assert!(state.is_expanded(&action_id));

        // Toggle to collapse
        state.toggle_expansion(action_id.clone());
        assert!(!state.is_expanded(&action_id));
    }

    #[test]
    fn test_multiple_expansions() {
        let mut state = ExpandableActionState::new();
        let id1 = "id-1".to_string();
        let id2 = "id-2".to_string();
        let id3 = "id-3".to_string();

        state.toggle_expansion(id1.clone());
        state.toggle_expansion(id2.clone());
        
        assert!(state.is_expanded(&id1));
        assert!(state.is_expanded(&id2));
        assert!(!state.is_expanded(&id3));

        state.clear_expansions();
        assert!(!state.is_expanded(&id1));
        assert!(!state.is_expanded(&id2));
    }

    #[test]
    fn test_navigation() {
        let mut state = ExpandableActionState::new();
        assert_eq!(state.selected_action_index, 0);

        state.navigate_down(10);
        assert_eq!(state.selected_action_index, 1);

        state.navigate_down(10);
        state.navigate_down(10);
        assert_eq!(state.selected_action_index, 3);

        state.navigate_up();
        assert_eq!(state.selected_action_index, 2);

        // Can't go below 0
        state.selected_action_index = 0;
        state.navigate_up();
        assert_eq!(state.selected_action_index, 0);

        // Can't exceed max
        state.selected_action_index = 8;
        state.navigate_down(9); // max_items is 9, so max index is 8
        assert_eq!(state.selected_action_index, 8);
    }

    #[test]
    fn test_update_mode() {
        let mut state = ExpandableActionState::new();
        assert!(!state.indicator_update_mode);
        assert_eq!(state.update_buffer, "");

        state.enter_update_mode();
        assert!(state.indicator_update_mode);
        assert_eq!(state.update_buffer, "");

        state.update_buffer = "test input".to_string();
        state.exit_update_mode();
        assert!(!state.indicator_update_mode);
        assert_eq!(state.update_buffer, "");
    }
}