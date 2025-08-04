use crate::todo::models::ListItem;
use crate::tui::navigation::ItemCreator;
use std::collections::HashSet;

pub struct ItemActions;

impl ItemActions {
    pub fn toggle_todo_completion(items: &mut [ListItem], index: usize) -> bool {
        if index < items.len() {
            if let Some(ListItem::Todo { completed, .. }) = items.get_mut(index) {
                *completed = !*completed;
                return true;
            }
        }
        false
    }

    pub fn move_single_item_up(items: &mut Vec<ListItem>, index: usize) -> Option<usize> {
        if index > 0 && index < items.len() {
            items.swap(index - 1, index);
            Some(index - 1)
        } else {
            None
        }
    }

    pub fn move_single_item_down(items: &mut Vec<ListItem>, index: usize) -> Option<usize> {
        if index < items.len().saturating_sub(1) {
            items.swap(index, index + 1);
            Some(index + 1)
        } else {
            None
        }
    }

    pub fn indent_block(items: &mut [ListItem], start_index: usize) -> bool {
        if start_index >= items.len() {
            return false;
        }

        // Get the block range to indent all items in the subtree
        let (block_start, block_end) = ItemCreator::get_block_range(items, start_index);
        
        // First calculate max indent level for the parent item
        let max_indent = if block_start > 0 {
            match &items[block_start - 1] {
                ListItem::Todo { indent_level: prev_indent, .. } => prev_indent + 1,
                ListItem::Note { indent_level: prev_indent, .. } => prev_indent + 1,
                ListItem::Heading { .. } => 1, // Can indent under headings
            }
        } else {
            0 // First item can't be indented
        };
        
        // Check if the parent item can be indented
        if let Some(item) = items.get(block_start) {
            let parent_indent = match item {
                ListItem::Todo { indent_level, .. } => *indent_level,
                ListItem::Note { indent_level, .. } => *indent_level,
                ListItem::Heading { .. } => return false, // Can't indent headings
            };

            if parent_indent < max_indent {
                // Indent the entire block
                for i in block_start..=block_end {
                    if let Some(item) = items.get_mut(i) {
                        match item {
                            ListItem::Todo { indent_level, .. } => {
                                *indent_level += 1;
                            }
                            ListItem::Note { indent_level, .. } => {
                                *indent_level += 1;
                            }
                            ListItem::Heading { .. } => {
                                // Don't indent headings
                            }
                        }
                    }
                }
                return true;
            }
        }
        false
    }

    pub fn unindent_block(items: &mut [ListItem], start_index: usize) -> bool {
        if start_index >= items.len() {
            return false;
        }

        // Get the block range to unindent all items in the subtree
        let (block_start, block_end) = ItemCreator::get_block_range(items, start_index);
        
        // Check if the parent item can be unindented
        if let Some(item) = items.get(block_start) {
            let parent_indent = match item {
                ListItem::Todo { indent_level, .. } => *indent_level,
                ListItem::Note { indent_level, .. } => *indent_level,
                ListItem::Heading { .. } => return false, // Can't unindent headings
            };

            if parent_indent > 0 {
                // Unindent the entire block
                for i in block_start..=block_end {
                    if let Some(item) = items.get_mut(i) {
                        match item {
                            ListItem::Todo { indent_level, .. } => {
                                if *indent_level > 0 {
                                    *indent_level -= 1;
                                }
                            }
                            ListItem::Note { indent_level, .. } => {
                                if *indent_level > 0 {
                                    *indent_level -= 1;
                                }
                            }
                            ListItem::Heading { .. } => {
                                // Don't unindent headings
                            }
                        }
                    }
                }
                return true;
            }
        }
        false
    }

    pub fn move_selected_items_to_position(
        items: &mut Vec<ListItem>,
        selected_indices: &HashSet<usize>,
        target_position: usize,
    ) -> Option<usize> {
        if selected_indices.is_empty() {
            return None;
        }

        // Get indices in sorted order (highest to lowest for removal)
        let mut indices: Vec<usize> = selected_indices.iter().cloned().collect();
        indices.sort_by(|a, b| b.cmp(a)); // Sort descending

        // Extract the selected items
        let mut items_to_move = Vec::new();
        for &index in &indices {
            if index < items.len() {
                items_to_move.push(items.remove(index));
            }
        }
        
        // Reverse to maintain original order when inserting
        items_to_move.reverse();

        // Calculate insertion point (adjust for removed items)
        // Start with position after the current cursor (insert below)
        let mut insertion_point = target_position + 1;
        for &removed_index in &indices {
            if removed_index < insertion_point {
                insertion_point = insertion_point.saturating_sub(1);
            }
        }

        // Insert items below the cursor position
        for (i, item) in items_to_move.into_iter().enumerate() {
            items.insert(insertion_point + i, item);
        }

        Some(insertion_point)
    }
}

pub trait ActionPerformer {
    fn perform_toggle_completion(&mut self, index: usize) -> bool;
    fn perform_move_item_up(&mut self, index: usize) -> Option<usize>;
    fn perform_move_item_down(&mut self, index: usize) -> Option<usize>;
    fn perform_indent_item(&mut self, index: usize) -> bool;
    fn perform_unindent_item(&mut self, index: usize) -> bool;
    fn perform_bulk_move(&mut self, selected_indices: &HashSet<usize>, target_index: usize) -> Option<usize>;
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_items() -> Vec<ListItem> {
        vec![
            ListItem::new_todo("Task A".to_string(), false, 0),
            ListItem::new_todo("Task B".to_string(), false, 0),
            ListItem::new_todo("Task C".to_string(), false, 1),
            ListItem::new_todo("Task D".to_string(), false, 0),
        ]
    }

    #[test]
    fn test_toggle_todo_completion() {
        let mut items = create_test_items();
        
        // Toggle first item
        let result = ItemActions::toggle_todo_completion(&mut items, 0);
        assert!(result);
        
        if let ListItem::Todo { completed, .. } = &items[0] {
            assert!(*completed);
        } else {
            panic!("Expected Todo item");
        }
        
        // Toggle it back
        let result = ItemActions::toggle_todo_completion(&mut items, 0);
        assert!(result);
        
        if let ListItem::Todo { completed, .. } = &items[0] {
            assert!(!*completed);
        } else {
            panic!("Expected Todo item");
        }
        
        // Try invalid index
        let result = ItemActions::toggle_todo_completion(&mut items, 10);
        assert!(!result);
    }

    #[test]
    fn test_move_single_item_up() {
        let mut items = create_test_items();
        
        // Move second item up
        let new_index = ItemActions::move_single_item_up(&mut items, 1);
        assert_eq!(new_index, Some(0));
        
        // Check order
        if let ListItem::Todo { content, .. } = &items[0] {
            assert_eq!(content, "Task B");
        }
        if let ListItem::Todo { content, .. } = &items[1] {
            assert_eq!(content, "Task A");
        }
        
        // Try to move first item up (should fail)
        let new_index = ItemActions::move_single_item_up(&mut items, 0);
        assert_eq!(new_index, None);
    }

    #[test]
    fn test_move_single_item_down() {
        let mut items = create_test_items();
        
        // Move first item down
        let new_index = ItemActions::move_single_item_down(&mut items, 0);
        assert_eq!(new_index, Some(1));
        
        // Check order
        if let ListItem::Todo { content, .. } = &items[0] {
            assert_eq!(content, "Task B");
        }
        if let ListItem::Todo { content, .. } = &items[1] {
            assert_eq!(content, "Task A");
        }
        
        // Try to move last item down (should fail)
        let items_len = items.len();
        let new_index = ItemActions::move_single_item_down(&mut items, items_len - 1);
        assert_eq!(new_index, None);
    }

    #[test]
    fn test_indent_block() {
        let mut items = vec![
            ListItem::new_todo("Parent".to_string(), false, 0),
            ListItem::new_todo("Child".to_string(), false, 0),
        ];
        
        // Indent second item under first
        let result = ItemActions::indent_block(&mut items, 1);
        assert!(result);
        
        if let ListItem::Todo { indent_level, .. } = &items[1] {
            assert_eq!(*indent_level, 1);
        }
        
        // Try to indent first item (should fail - no parent)
        let result = ItemActions::indent_block(&mut items, 0);
        assert!(!result);
    }

    #[test]
    fn test_unindent_block() {
        let mut items = vec![
            ListItem::new_todo("Parent".to_string(), false, 0),
            ListItem::new_todo("Child".to_string(), false, 1),
        ];
        
        // Unindent child
        let result = ItemActions::unindent_block(&mut items, 1);
        assert!(result);
        
        if let ListItem::Todo { indent_level, .. } = &items[1] {
            assert_eq!(*indent_level, 0);
        }
        
        // Try to unindent further (should fail - already at 0)
        let result = ItemActions::unindent_block(&mut items, 1);
        assert!(!result);
    }

    #[test]
    fn test_move_selected_items_to_position() {
        let mut items = create_test_items();
        let mut selected = HashSet::new();
        selected.insert(0); // Task A
        selected.insert(2); // Task C
        
        // Move to position after Task B (index 1)
        let result = ItemActions::move_selected_items_to_position(&mut items, &selected, 1);
        assert!(result.is_some());
        
        // Check new order: Task B, Task A, Task C, Task D
        // Original: Task A(0), Task B(1), Task C(2), Task D(3)
        // Selected: Task A(0), Task C(2)
        // After removal: Task B, Task D (remaining)
        // After insertion at position 1+1=2 -> 1 (adjusted): Task B, Task A, Task C, Task D
        if let ListItem::Todo { content, .. } = &items[0] {
            assert_eq!(content, "Task B");
        }
        if let ListItem::Todo { content, .. } = &items[1] {
            assert_eq!(content, "Task A");
        }
        if let ListItem::Todo { content, .. } = &items[2] {
            assert_eq!(content, "Task C");
        }
        if let ListItem::Todo { content, .. } = &items[3] {
            assert_eq!(content, "Task D");
        }
    }

    #[test]
    fn test_move_selected_items_empty_selection() {
        let mut items = create_test_items();
        let selected = HashSet::new();
        
        let result = ItemActions::move_selected_items_to_position(&mut items, &selected, 1);
        assert!(result.is_none());
        
        // Items should remain unchanged
        assert_eq!(items.len(), 4);
    }
}