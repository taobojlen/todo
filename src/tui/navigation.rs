use crate::todo::models::ListItem;
use std::collections::HashSet;

pub struct NavigationState {
    pub selected_index: usize,
    pub scroll_offset: usize,
    pub selected_items: HashSet<usize>,
}

impl NavigationState {
    pub fn new() -> Self {
        Self {
            selected_index: 0,
            scroll_offset: 0,
            selected_items: HashSet::new(),
        }
    }

    pub fn move_selection_up(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
            self.update_scroll();
        }
    }

    pub fn move_selection_down(&mut self, max_items: usize) {
        if self.selected_index < max_items.saturating_sub(1) {
            self.selected_index += 1;
            self.update_scroll();
        }
    }

    pub fn update_scroll(&mut self) {
        // Simple scroll logic - keep selected item visible
        const VISIBLE_ITEMS: usize = 20; // Will be dynamic based on terminal height
        
        if self.selected_index < self.scroll_offset {
            self.scroll_offset = self.selected_index;
        } else if self.selected_index >= self.scroll_offset + VISIBLE_ITEMS {
            self.scroll_offset = self.selected_index.saturating_sub(VISIBLE_ITEMS - 1);
        }
    }

    pub fn toggle_item_selection(&mut self, max_items: usize) {
        if self.selected_index < max_items {
            if self.selected_items.contains(&self.selected_index) {
                self.selected_items.remove(&self.selected_index);
            } else {
                self.selected_items.insert(self.selected_index);
            }
        }
    }

    pub fn clear_selection(&mut self) {
        self.selected_items.clear();
    }

    pub fn has_selection(&self) -> bool {
        !self.selected_items.is_empty()
    }

    pub fn selection_count(&self) -> usize {
        self.selected_items.len()
    }

    pub fn is_item_selected(&self, index: usize) -> bool {
        self.selected_items.contains(&index)
    }
}

pub struct ItemCreator;

impl ItemCreator {
    pub fn find_current_heading_context(items: &[ListItem], selected_index: usize) -> usize {
        if items.is_empty() {
            return 0;
        }

        // Look backwards from current position to find the most recent heading
        for i in (0..=selected_index).rev() {
            if let Some(ListItem::Heading { .. }) = items.get(i) {
                // Found a heading, insert right after it
                return i + 1;
            }
        }
        
        // No heading found above current position, insert at the very top
        0
    }

    pub fn get_block_range(items: &[ListItem], start_index: usize) -> (usize, usize) {
        if start_index >= items.len() {
            return (start_index, start_index);
        }

        let start_item = &items[start_index];
        let base_indent = match start_item {
            ListItem::Todo { indent_level, .. } => *indent_level,
            ListItem::Note { indent_level, .. } => *indent_level,
            ListItem::Heading { .. } => 0,
        };

        let mut end_index = start_index;
        
        // Find all items that belong to this block
        for (i, item) in items.iter().enumerate().skip(start_index + 1) {
            match item {
                ListItem::Todo { indent_level, .. } => {
                    if *indent_level > base_indent {
                        // This item is nested under the current item
                        end_index = i;
                    } else {
                        // We've reached a sibling or parent, stop here
                        break;
                    }
                }
                ListItem::Note { indent_level, .. } => {
                    if *indent_level > base_indent {
                        // This item is nested under the current item
                        end_index = i;
                    } else {
                        // We've reached a sibling or parent, stop here
                        break;
                    }
                }
                ListItem::Heading { .. } => {
                    // Headings always break blocks
                    break;
                }
            }
        }

        (start_index, end_index)
    }

    pub fn create_new_todo(content: String, completed: bool, indent_level: usize) -> ListItem {
        ListItem::new_todo(content, completed, indent_level, 0)
    }

    pub fn create_new_note(content: String, indent_level: usize) -> ListItem {
        ListItem::new_note(content, indent_level, 0)
    }

    pub fn determine_insert_position_for_new_todo(
        items: &[ListItem],
        selected_index: usize,
    ) -> (usize, usize) {
        if items.is_empty() {
            return (0, 0); // Position 0, indent level 0
        }

        if selected_index >= items.len() {
            return (items.len(), 0);
        }

        let current_item = &items[selected_index];
        
        match current_item {
            ListItem::Todo { indent_level: current_indent, .. } |
            ListItem::Note { indent_level: current_indent, .. } => {
                // Check if this item has children
                let (_, block_end) = Self::get_block_range(items, selected_index);
                
                if block_end > selected_index {
                    // This item has children, add new child after the last child
                    let child_indent = current_indent + 1;
                    (block_end + 1, child_indent)
                } else {
                    // This item has no children, add sibling with same indentation
                    (selected_index + 1, *current_indent)
                }
            }
            ListItem::Heading { .. } => {
                // New todos under headings start at level 0
                (selected_index + 1, 0)
            }
        }
    }

    pub fn determine_insert_position_for_new_todo_at_top(
        items: &[ListItem],
        selected_index: usize,
    ) -> usize {
        Self::find_current_heading_context(items, selected_index)
    }
}

pub trait Navigable {
    fn get_navigation_state(&self) -> &NavigationState;
    fn get_navigation_state_mut(&mut self) -> &mut NavigationState;
    fn get_item_count(&self) -> usize;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_navigation_state_new() {
        let nav_state = NavigationState::new();
        assert_eq!(nav_state.selected_index, 0);
        assert_eq!(nav_state.scroll_offset, 0);
        assert!(nav_state.selected_items.is_empty());
    }

    #[test]
    fn test_move_selection() {
        let mut nav_state = NavigationState::new();
        
        // Test moving down
        nav_state.move_selection_down(5);
        assert_eq!(nav_state.selected_index, 1);
        
        // Test moving up
        nav_state.move_selection_up();
        assert_eq!(nav_state.selected_index, 0);
        
        // Test can't move up from 0
        nav_state.move_selection_up();
        assert_eq!(nav_state.selected_index, 0);
        
        // Test can't move down beyond max
        nav_state.selected_index = 4;
        nav_state.move_selection_down(5);
        assert_eq!(nav_state.selected_index, 4);
    }

    #[test]
    fn test_toggle_item_selection() {
        let mut nav_state = NavigationState::new();
        
        // Select item 0
        nav_state.toggle_item_selection(5);
        assert!(nav_state.selected_items.contains(&0));
        assert_eq!(nav_state.selection_count(), 1);
        
        // Deselect item 0
        nav_state.toggle_item_selection(5);
        assert!(!nav_state.selected_items.contains(&0));
        assert_eq!(nav_state.selection_count(), 0);
        
        // Select multiple items
        nav_state.selected_index = 1;
        nav_state.toggle_item_selection(5);
        nav_state.selected_index = 3;
        nav_state.toggle_item_selection(5);
        assert_eq!(nav_state.selection_count(), 2);
        assert!(nav_state.is_item_selected(1));
        assert!(nav_state.is_item_selected(3));
    }

    #[test]
    fn test_clear_selection() {
        let mut nav_state = NavigationState::new();
        
        nav_state.toggle_item_selection(5);
        nav_state.selected_index = 2;
        nav_state.toggle_item_selection(5);
        
        assert_eq!(nav_state.selection_count(), 2);
        
        nav_state.clear_selection();
        assert_eq!(nav_state.selection_count(), 0);
        assert!(!nav_state.has_selection());
    }

    #[test]
    fn test_find_current_heading_context() {
        let items = vec![
            ListItem::new_todo("Task 1".to_string(), false, 0, 0),
            ListItem::new_heading("Section A".to_string(), 1, 1),
            ListItem::new_todo("Task 2".to_string(), false, 0, 2),
            ListItem::new_heading("Section B".to_string(), 1, 3),
            ListItem::new_todo("Task 3".to_string(), false, 0, 4),
        ];
        
        // When selected on Task 3 (index 4), should find Section B (index 3) and return 4
        let context = ItemCreator::find_current_heading_context(&items, 4);
        assert_eq!(context, 4); // Insert after Section B
        
        // When selected on Task 2 (index 2), should find Section A (index 1) and return 2
        let context = ItemCreator::find_current_heading_context(&items, 2);
        assert_eq!(context, 2); // Insert after Section A
        
        // When selected on Task 1 (index 0), no heading above, should return 0
        let context = ItemCreator::find_current_heading_context(&items, 0);
        assert_eq!(context, 0); // Insert at top
    }

    #[test]
    fn test_get_block_range() {
        let items = vec![
            ListItem::new_todo("Parent".to_string(), false, 0, 0),
            ListItem::new_todo("Child 1".to_string(), false, 1, 1),
            ListItem::new_todo("Child 2".to_string(), false, 1, 2),
            ListItem::new_todo("Next parent".to_string(), false, 0, 3),
        ];
        
        let (start, end) = ItemCreator::get_block_range(&items, 0);
        assert_eq!(start, 0);
        assert_eq!(end, 2); // Should include both children
        
        let (start, end) = ItemCreator::get_block_range(&items, 3);
        assert_eq!(start, 3);
        assert_eq!(end, 3); // No children
    }

    #[test]
    fn test_determine_insert_position_for_new_todo() {
        let items = vec![
            ListItem::new_todo("Parent".to_string(), false, 0, 0),
            ListItem::new_todo("Child".to_string(), false, 1, 1),
            ListItem::new_todo("Sibling".to_string(), false, 0, 2),
        ];
        
        // Inserting after parent with children should create new child
        let (pos, indent) = ItemCreator::determine_insert_position_for_new_todo(&items, 0);
        assert_eq!(pos, 2); // After the child
        assert_eq!(indent, 1); // Child indentation
        
        // Inserting after item with no children should create sibling
        let (pos, indent) = ItemCreator::determine_insert_position_for_new_todo(&items, 2);
        assert_eq!(pos, 3); // After the sibling
        assert_eq!(indent, 0); // Same level as sibling
    }
}