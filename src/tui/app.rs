use crate::todo::models::{TodoList, ListItem};
use crate::todo::writer;
use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};
use std::collections::HashSet;

#[derive(Clone, Debug)]
struct AppState {
    todo_list: TodoList,
    selected_index: usize,
    selected_items: HashSet<usize>,
}

#[derive(Debug)]
pub struct App {
    pub todo_list: TodoList,
    pub selected_index: usize,
    pub scroll_offset: usize,
    pub should_quit: bool,
    pub edit_mode: bool,
    pub edit_buffer: String,
    pub edit_cursor_position: usize,
    pub selected_items: HashSet<usize>,
    pub help_mode: bool,
    pub undo_stack: Vec<AppState>,
    adding_new_todo: bool,
}

impl App {
    pub fn new(todo_list: TodoList) -> Self {
        Self {
            todo_list,
            selected_index: 0,
            scroll_offset: 0,
            should_quit: false,
            edit_mode: false,
            edit_buffer: String::new(),
            edit_cursor_position: 0,
            selected_items: HashSet::new(),
            help_mode: false,
            undo_stack: Vec::new(),
            adding_new_todo: false,
        }
    }

    pub fn handle_key_event(&mut self, key_event: KeyEvent) -> Result<()> {
        if self.help_mode {
            self.handle_help_mode_key(key_event)
        } else if self.edit_mode {
            self.handle_edit_mode_key(key_event)
        } else {
            self.handle_normal_mode_key(key_event)
        }
    }
    
    fn handle_normal_mode_key(&mut self, key_event: KeyEvent) -> Result<()> {
        use crossterm::event::{KeyModifiers};
        
        match key_event.code {
            KeyCode::Char('q') => {
                self.should_quit = true;
            }
            KeyCode::Esc => {
                self.selected_items.clear();
            }
            KeyCode::Char('c') if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
                self.should_quit = true;
            }
            KeyCode::Up | KeyCode::Char('k') | KeyCode::Char('K') => {
                if key_event.modifiers.contains(KeyModifiers::SHIFT) {
                    self.move_item_up();
                } else {
                    self.move_selection_up();
                }
            }
            KeyCode::Down | KeyCode::Char('j') | KeyCode::Char('J') => {
                if key_event.modifiers.contains(KeyModifiers::SHIFT) {
                    self.move_item_down();
                } else {
                    self.move_selection_down();
                }
            }
            KeyCode::Left | KeyCode::Char('h') | KeyCode::Char('H') => {
                if key_event.modifiers.contains(KeyModifiers::SHIFT) {
                    self.unindent_item();
                }
            }
            KeyCode::Right | KeyCode::Char('l') | KeyCode::Char('L') => {
                if key_event.modifiers.contains(KeyModifiers::SHIFT) {
                    self.indent_item();
                }
            }
            KeyCode::Enter => {
                self.toggle_selected_item();
            }
            KeyCode::Char('e') => {
                self.enter_edit_mode();
            }
            KeyCode::Char('a') => {
                self.add_new_todo();
            }
            KeyCode::Char('A') => {
                self.add_new_todo_at_top();
            }
            KeyCode::Char(' ') => {
                self.toggle_item_selection();
            }
            KeyCode::Char('m') => {
                self.move_selected_items_to_cursor();
            }
            KeyCode::Char('?') => {
                self.help_mode = true;
            }
            KeyCode::Char('u') => {
                self.undo();
            }
            _ => {}
        }
        Ok(())
    }

    fn handle_help_mode_key(&mut self, key_event: KeyEvent) -> Result<()> {
        match key_event.code {
            KeyCode::Char('q') | KeyCode::Esc | KeyCode::Char('?') => {
                self.help_mode = false;
            }
            _ => {}
        }
        Ok(())
    }
    
    fn handle_edit_mode_key(&mut self, key_event: KeyEvent) -> Result<()> {
        match key_event.code {
            KeyCode::Esc => {
                self.cancel_edit();
            }
            KeyCode::Enter => {
                self.confirm_edit();
            }
            KeyCode::Backspace => {
                self.edit_backspace();
            }
            KeyCode::Delete => {
                self.edit_delete();
            }
            KeyCode::Left => {
                self.edit_move_cursor_left();
            }
            KeyCode::Right => {
                self.edit_move_cursor_right();
            }
            KeyCode::Home => {
                self.edit_cursor_position = 0;
            }
            KeyCode::End => {
                self.edit_cursor_position = self.edit_buffer.len();
            }
            KeyCode::Char(c) => {
                self.edit_insert_char(c);
            }
            _ => {}
        }
        Ok(())
    }

    fn move_selection_up(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
            self.update_scroll();
        }
    }

    fn move_selection_down(&mut self) {
        if self.selected_index < self.todo_list.items.len().saturating_sub(1) {
            self.selected_index += 1;
            self.update_scroll();
        }
    }

    fn update_scroll(&mut self) {
        // Simple scroll logic - keep selected item visible
        // This will be refined when we implement the UI
        const VISIBLE_ITEMS: usize = 20; // Will be dynamic based on terminal height
        
        if self.selected_index < self.scroll_offset {
            self.scroll_offset = self.selected_index;
        } else if self.selected_index >= self.scroll_offset + VISIBLE_ITEMS {
            self.scroll_offset = self.selected_index.saturating_sub(VISIBLE_ITEMS - 1);
        }
    }

    pub fn total_items(&self) -> usize {
        self.todo_list.total_items()
    }

    pub fn completed_items(&self) -> usize {
        self.todo_list.completed_items()
    }

    fn toggle_selected_item(&mut self) {
        if self.selected_index < self.todo_list.items.len() {
            // Check if it's a todo item first
            let is_todo = matches!(self.todo_list.items[self.selected_index], ListItem::Todo { .. });
            
            if is_todo {
                self.save_state();
                if let Some(ListItem::Todo { completed, .. }) = self.todo_list.items.get_mut(self.selected_index) {
                    *completed = !*completed;
                }
                
                // Save changes to file
                if let Err(e) = self.save_to_file() {
                    eprintln!("Failed to save file: {}", e);
                }
            }
        }
    }

    fn move_item_up(&mut self) {
        self.save_state();
        self.move_single_item_up();
        
        // Save changes to file
        if let Err(e) = self.save_to_file() {
            eprintln!("Failed to save file: {}", e);
        }
    }

    fn move_item_down(&mut self) {
        self.save_state();
        self.move_single_item_down();
        
        // Save changes to file
        if let Err(e) = self.save_to_file() {
            eprintln!("Failed to save file: {}", e);
        }
    }

    fn indent_item(&mut self) {
        if self.selected_index < self.todo_list.items.len() {
            self.save_state();
            // Get the block range to indent all items in the subtree
            let (block_start, block_end) = self.get_block_range(self.selected_index);
            
            // First calculate max indent level for the parent item
            let max_indent = if block_start > 0 {
                match &self.todo_list.items[block_start - 1] {
                    ListItem::Todo { indent_level: prev_indent, .. } => prev_indent + 1,
                    ListItem::Heading { .. } => 1, // Can indent under headings
                }
            } else {
                0 // First item can't be indented
            };
            
            // Check if the parent item can be indented
            if let Some(ListItem::Todo { indent_level: parent_indent, .. }) = self.todo_list.items.get(block_start) {
                if *parent_indent < max_indent {
                    // Indent the entire block
                    for i in block_start..=block_end {
                        if let Some(item) = self.todo_list.items.get_mut(i) {
                            match item {
                                ListItem::Todo { indent_level, .. } => {
                                    *indent_level += 1;
                                }
                                ListItem::Heading { .. } => {
                                    // Don't indent headings
                                }
                            }
                        }
                    }
                }
            }
            
            // Save changes to file
            if let Err(e) = self.save_to_file() {
                eprintln!("Failed to save file: {}", e);
            }
        }
    }

    fn unindent_item(&mut self) {
        if self.selected_index < self.todo_list.items.len() {
            self.save_state();
            // Get the block range to unindent all items in the subtree
            let (block_start, block_end) = self.get_block_range(self.selected_index);
            
            // Check if the parent item can be unindented
            if let Some(ListItem::Todo { indent_level: parent_indent, .. }) = self.todo_list.items.get(block_start) {
                if *parent_indent > 0 {
                    // Unindent the entire block
                    for i in block_start..=block_end {
                        if let Some(item) = self.todo_list.items.get_mut(i) {
                            match item {
                                ListItem::Todo { indent_level, .. } => {
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
                }
            }
            
            // Save changes to file
            if let Err(e) = self.save_to_file() {
                eprintln!("Failed to save file: {}", e);
            }
        }
    }

    fn get_block_range(&self, start_index: usize) -> (usize, usize) {
        if start_index >= self.todo_list.items.len() {
            return (start_index, start_index);
        }

        let start_item = &self.todo_list.items[start_index];
        let base_indent = match start_item {
            ListItem::Todo { indent_level, .. } => *indent_level,
            ListItem::Heading { .. } => 0,
        };

        let mut end_index = start_index;
        
        // Find all items that belong to this block
        for (i, item) in self.todo_list.items.iter().enumerate().skip(start_index + 1) {
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
                ListItem::Heading { .. } => {
                    // Headings always break blocks
                    break;
                }
            }
        }

        (start_index, end_index)
    }

    fn move_single_item_up(&mut self) {
        if self.selected_index > 0 && self.selected_index < self.todo_list.items.len() {
            // Simply swap the current item with the one above it
            self.todo_list.items.swap(self.selected_index - 1, self.selected_index);
            self.selected_index -= 1;
            self.update_scroll();
        }
    }

    fn move_single_item_down(&mut self) {
        if self.selected_index < self.todo_list.items.len().saturating_sub(1) {
            // Simply swap the current item with the one below it
            self.todo_list.items.swap(self.selected_index, self.selected_index + 1);
            self.selected_index += 1;
            self.update_scroll();
        }
    }

    fn toggle_item_selection(&mut self) {
        if self.selected_index < self.todo_list.items.len() {
            if self.selected_items.contains(&self.selected_index) {
                self.selected_items.remove(&self.selected_index);
            } else {
                self.selected_items.insert(self.selected_index);
            }
        }
    }

    fn move_selected_items_to_cursor(&mut self) {
        if self.selected_items.is_empty() {
            return;
        }

        self.save_state();

        // Get indices in sorted order (highest to lowest for removal)
        let mut indices: Vec<usize> = self.selected_items.iter().cloned().collect();
        indices.sort_by(|a, b| b.cmp(a)); // Sort descending

        // Extract the selected items
        let mut items_to_move = Vec::new();
        for &index in &indices {
            if index < self.todo_list.items.len() {
                items_to_move.push(self.todo_list.items.remove(index));
            }
        }
        
        // Reverse to maintain original order when inserting
        items_to_move.reverse();

        // Calculate insertion point (adjust for removed items)
        // Start with position after the current cursor (insert below)
        let mut insertion_point = self.selected_index + 1;
        for &removed_index in &indices {
            if removed_index < insertion_point {
                insertion_point = insertion_point.saturating_sub(1);
            }
        }

        // Insert items below the cursor position
        for (i, item) in items_to_move.into_iter().enumerate() {
            self.todo_list.items.insert(insertion_point + i, item);
        }

        // Clear selection after moving
        self.selected_items.clear();
        
        // Update cursor position to the first moved item
        self.selected_index = insertion_point;
        self.update_scroll();
        
        // Save changes to file
        if let Err(e) = self.save_to_file() {
            eprintln!("Failed to save file: {}", e);
        }
    }

    fn save_state(&mut self) {
        let state = AppState {
            todo_list: self.todo_list.clone(),
            selected_index: self.selected_index,
            selected_items: self.selected_items.clone(),
        };
        
        self.undo_stack.push(state);
        
        // Limit undo stack to 20 items
        if self.undo_stack.len() > 20 {
            self.undo_stack.remove(0);
        }
    }

    fn undo(&mut self) {
        if let Some(state) = self.undo_stack.pop() {
            self.todo_list = state.todo_list;
            self.selected_index = state.selected_index;
            self.selected_items = state.selected_items;
            self.update_scroll();
            
            // Save changes to file
            if let Err(e) = self.save_to_file() {
                eprintln!("Failed to save file: {}", e);
            }
        }
    }

    fn save_to_file(&self) -> Result<()> {
        writer::write_todo_file(&self.todo_list)
    }

    fn enter_edit_mode(&mut self) {
        if self.selected_index < self.todo_list.items.len() {
            if let Some(item) = self.todo_list.items.get(self.selected_index) {
                let content = match item {
                    ListItem::Todo { content, .. } => content.clone(),
                    ListItem::Heading { content, .. } => content.clone(),
                };
                self.edit_buffer = content;
                self.edit_cursor_position = self.edit_buffer.len();
                self.edit_mode = true;
            }
        }
    }

    fn cancel_edit(&mut self) {
        // If we're canceling edit on an empty todo, remove it
        if self.selected_index < self.todo_list.items.len() {
            if let Some(ListItem::Todo { content, .. }) = self.todo_list.items.get(self.selected_index) {
                if content.trim().is_empty() {
                    self.todo_list.items.remove(self.selected_index);
                    // Adjust selection to stay within bounds
                    if self.selected_index >= self.todo_list.items.len() && !self.todo_list.items.is_empty() {
                        self.selected_index = self.todo_list.items.len() - 1;
                    }
                }
            }
        }
        
        self.edit_mode = false;
        self.edit_buffer.clear();
        self.edit_cursor_position = 0;
        self.adding_new_todo = false;
        
        // Save changes to file (in case we removed an empty todo)
        if let Err(e) = self.save_to_file() {
            eprintln!("Failed to save file: {}", e);
        }
    }

    fn confirm_edit(&mut self) {
        if self.selected_index < self.todo_list.items.len() {
            // Only save state if we're not confirming a newly added todo (which already saved state)
            if !self.adding_new_todo {
                self.save_state();
            }
            let should_remove = if let Some(item) = self.todo_list.items.get_mut(self.selected_index) {
                match item {
                    ListItem::Todo { content, .. } => {
                        *content = self.edit_buffer.clone();
                        // Remove todo if it's empty after editing
                        self.edit_buffer.trim().is_empty()
                    }
                    ListItem::Heading { content, .. } => {
                        *content = self.edit_buffer.clone();
                        // Don't remove headings even if empty (they serve as section markers)
                        false
                    }
                }
            } else {
                false
            };

            // Remove the item if it's an empty todo
            if should_remove {
                self.todo_list.items.remove(self.selected_index);
                // Adjust selection to stay within bounds
                if self.selected_index >= self.todo_list.items.len() && !self.todo_list.items.is_empty() {
                    self.selected_index = self.todo_list.items.len() - 1;
                }
            }
        }
        
        self.edit_mode = false;
        self.edit_buffer.clear();
        self.edit_cursor_position = 0;
        self.adding_new_todo = false;
        
        // Save changes to file
        if let Err(e) = self.save_to_file() {
            eprintln!("Failed to save file: {}", e);
        }
    }

    fn edit_insert_char(&mut self, c: char) {
        self.edit_buffer.insert(self.edit_cursor_position, c);
        self.edit_cursor_position += c.len_utf8();
    }

    fn edit_backspace(&mut self) {
        if self.edit_cursor_position > 0 {
            // Find the previous character boundary
            let chars: Vec<char> = self.edit_buffer.chars().collect();
            let mut byte_pos = 0;
            let mut char_index = 0;
            
            // Find which character we're at
            for (i, ch) in chars.iter().enumerate() {
                if byte_pos >= self.edit_cursor_position {
                    char_index = i;
                    break;
                }
                byte_pos += ch.len_utf8();
                char_index = i + 1;
            }
            
            if char_index > 0 {
                let char_to_remove = chars[char_index - 1];
                self.edit_cursor_position -= char_to_remove.len_utf8();
                self.edit_buffer.remove(self.edit_cursor_position);
            }
        }
    }

    fn edit_delete(&mut self) {
        if self.edit_cursor_position < self.edit_buffer.len() {
            self.edit_buffer.remove(self.edit_cursor_position);
        }
    }

    fn edit_move_cursor_left(&mut self) {
        if self.edit_cursor_position > 0 {
            // Find the previous character boundary
            let chars: Vec<char> = self.edit_buffer.chars().collect();
            let mut byte_pos = 0;
            
            for ch in chars.iter() {
                if byte_pos >= self.edit_cursor_position {
                    break;
                }
                if byte_pos + ch.len_utf8() >= self.edit_cursor_position {
                    self.edit_cursor_position = byte_pos;
                    return;
                }
                byte_pos += ch.len_utf8();
            }
        }
    }

    fn edit_move_cursor_right(&mut self) {
        if self.edit_cursor_position < self.edit_buffer.len() {
            // Find the next character boundary
            let chars: Vec<char> = self.edit_buffer.chars().collect();
            let mut byte_pos = 0;
            
            for ch in chars.iter() {
                if byte_pos >= self.edit_cursor_position {
                    self.edit_cursor_position = byte_pos + ch.len_utf8();
                    return;
                }
                byte_pos += ch.len_utf8();
            }
        }
    }

    fn add_new_todo(&mut self) {
        self.save_state();
        self.adding_new_todo = true;
        if self.todo_list.items.is_empty() {
            // If there are no items, add the first one at level 0
            let new_todo = ListItem::new_todo(String::new(), false, 0, 0);
            self.todo_list.add_item(new_todo);
            self.selected_index = 0;
            self.enter_edit_mode();
        } else if self.selected_index < self.todo_list.items.len() {
            let current_item = &self.todo_list.items[self.selected_index];
            
            match current_item {
                ListItem::Todo { indent_level: current_indent, .. } => {
                    // Check if this todo has children
                    let (_, block_end) = self.get_block_range(self.selected_index);
                    
                    if block_end > self.selected_index {
                        // This todo has children, add new child after the last child
                        let child_indent = current_indent + 1;
                        let new_todo = ListItem::new_todo(String::new(), false, child_indent, 0);
                        let insert_position = block_end + 1;
                        
                        self.todo_list.items.insert(insert_position, new_todo);
                        self.selected_index = insert_position;
                    } else {
                        // This todo has no children, add sibling with same indentation
                        let new_todo = ListItem::new_todo(String::new(), false, *current_indent, 0);
                        let insert_position = self.selected_index + 1;
                        
                        self.todo_list.items.insert(insert_position, new_todo);
                        self.selected_index = insert_position;
                    }
                }
                ListItem::Heading { .. } => {
                    // New todos under headings start at level 0
                    let new_todo = ListItem::new_todo(String::new(), false, 0, 0);
                    let insert_position = self.selected_index + 1;
                    
                    self.todo_list.items.insert(insert_position, new_todo);
                    self.selected_index = insert_position;
                }
            }
            
            self.enter_edit_mode();
        }
    }

    fn add_new_todo_at_top(&mut self) {
        self.save_state();
        self.adding_new_todo = true;
        // Create new todo at level 0 (top level)
        let new_todo = ListItem::new_todo(String::new(), false, 0, 0);
        
        // Find the current heading context
        let insert_position = self.find_current_heading_context();
        
        // Insert the new todo
        self.todo_list.items.insert(insert_position, new_todo);
        
        // Move selection to the new todo and enter edit mode
        self.selected_index = insert_position;
        self.enter_edit_mode();
    }

    fn find_current_heading_context(&self) -> usize {
        if self.todo_list.items.is_empty() {
            return 0;
        }

        // Look backwards from current position to find the most recent heading
        for i in (0..=self.selected_index).rev() {
            if let Some(ListItem::Heading { .. }) = self.todo_list.items.get(i) {
                // Found a heading, insert right after it
                return i + 1;
            }
        }
        
        // No heading found above current position, insert at the very top
        0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::todo::models::ListItem;

    #[test]
    fn test_toggle_todo_item() {
        let mut todo_list = TodoList::new("test".to_string());
        todo_list.add_item(ListItem::new_todo("Test task".to_string(), false, 0, 0));
        
        let mut app = App::new(todo_list);
        
        // Initially uncompleted
        match &app.todo_list.items[0] {
            ListItem::Todo { completed, .. } => assert!(!completed),
            _ => panic!("Expected Todo item"),
        }
        
        // Toggle to completed
        app.toggle_selected_item();
        match &app.todo_list.items[0] {
            ListItem::Todo { completed, .. } => assert!(*completed),
            _ => panic!("Expected Todo item"),
        }
        
        // Toggle back to uncompleted
        app.toggle_selected_item();
        match &app.todo_list.items[0] {
            ListItem::Todo { completed, .. } => assert!(!completed),
            _ => panic!("Expected Todo item"),
        }
    }

    #[test]
    fn test_toggle_heading_does_nothing() {
        let mut todo_list = TodoList::new("test".to_string());
        todo_list.add_item(ListItem::new_heading("Test heading".to_string(), 1, 0));
        
        let mut app = App::new(todo_list);
        
        // Should remain a heading
        match &app.todo_list.items[0] {
            ListItem::Heading { content, .. } => assert_eq!(content, "Test heading"),
            _ => panic!("Expected Heading item"),
        }
        
        app.toggle_selected_item();
        
        // Should still be a heading
        match &app.todo_list.items[0] {
            ListItem::Heading { content, .. } => assert_eq!(content, "Test heading"),
            _ => panic!("Expected Heading item"),
        }
    }

    #[test]
    fn test_move_item_down() {
        let mut todo_list = TodoList::new("test".to_string());
        todo_list.add_item(ListItem::new_todo("First task".to_string(), false, 0, 0));
        todo_list.add_item(ListItem::new_todo("Second task".to_string(), false, 0, 1));
        
        let mut app = App::new(todo_list);
        app.selected_index = 0;
        
        app.move_item_down();
        
        // First item should now be second
        match &app.todo_list.items[1] {
            ListItem::Todo { content, .. } => assert_eq!(content, "First task"),
            _ => panic!("Expected Todo item"),
        }
        
        // Selection should follow the moved item
        assert_eq!(app.selected_index, 1);
    }

    #[test]
    fn test_move_item_up() {
        let mut todo_list = TodoList::new("test".to_string());
        todo_list.add_item(ListItem::new_todo("First task".to_string(), false, 0, 0));
        todo_list.add_item(ListItem::new_todo("Second task".to_string(), false, 0, 1));
        
        let mut app = App::new(todo_list);
        app.selected_index = 1;
        
        app.move_item_up();
        
        // Second item should now be first
        match &app.todo_list.items[0] {
            ListItem::Todo { content, .. } => assert_eq!(content, "Second task"),
            _ => panic!("Expected Todo item"),
        }
        
        // Selection should follow the moved item
        assert_eq!(app.selected_index, 0);
    }

    #[test]
    fn test_indent_item() {
        let mut todo_list = TodoList::new("test".to_string());
        todo_list.add_item(ListItem::new_todo("Parent task".to_string(), false, 0, 0));
        todo_list.add_item(ListItem::new_todo("Child task".to_string(), false, 0, 1));
        
        let mut app = App::new(todo_list);
        app.selected_index = 1;
        
        app.indent_item();
        
        // Second item should be indented
        match &app.todo_list.items[1] {
            ListItem::Todo { content, indent_level, .. } => {
                assert_eq!(content, "Child task");
                assert_eq!(*indent_level, 1);
            }
            _ => panic!("Expected Todo item"),
        }
    }

    #[test]
    fn test_unindent_item() {
        let mut todo_list = TodoList::new("test".to_string());
        todo_list.add_item(ListItem::new_todo("Parent task".to_string(), false, 0, 0));
        todo_list.add_item(ListItem::new_todo("Child task".to_string(), false, 1, 1));
        
        let mut app = App::new(todo_list);
        app.selected_index = 1;
        
        app.unindent_item();
        
        // Second item should be unindented
        match &app.todo_list.items[1] {
            ListItem::Todo { content, indent_level, .. } => {
                assert_eq!(content, "Child task");
                assert_eq!(*indent_level, 0);
            }
            _ => panic!("Expected Todo item"),
        }
    }

    #[test]
    fn test_get_block_range_single_item() {
        let mut todo_list = TodoList::new("test".to_string());
        todo_list.add_item(ListItem::new_todo("Single task".to_string(), false, 0, 0));
        
        let app = App::new(todo_list);
        
        let (start, end) = app.get_block_range(0);
        assert_eq!(start, 0);
        assert_eq!(end, 0);
    }

    #[test]
    fn test_get_block_range_with_children() {
        let mut todo_list = TodoList::new("test".to_string());
        todo_list.add_item(ListItem::new_todo("Parent".to_string(), false, 0, 0));
        todo_list.add_item(ListItem::new_todo("Child 1".to_string(), false, 1, 1));
        todo_list.add_item(ListItem::new_todo("Child 2".to_string(), false, 1, 2));
        todo_list.add_item(ListItem::new_todo("Next parent".to_string(), false, 0, 3));
        
        let app = App::new(todo_list);
        
        let (start, end) = app.get_block_range(0);
        assert_eq!(start, 0);
        assert_eq!(end, 2); // Should include both children
    }

    #[test]
    fn test_move_single_item_down() {
        let mut todo_list = TodoList::new("test".to_string());
        todo_list.add_item(ListItem::new_todo("Parent A".to_string(), false, 0, 0));
        todo_list.add_item(ListItem::new_todo("Child A1".to_string(), false, 1, 1));
        todo_list.add_item(ListItem::new_todo("Parent B".to_string(), false, 0, 2));
        
        let mut app = App::new(todo_list);
        app.selected_index = 0; // Select "Parent A"
        
        app.move_item_down();
        
        // With single-item movement, only "Parent A" moves down, children stay in place
        // Order should be: Child A1, Parent A, Parent B
        match &app.todo_list.items[0] {
            ListItem::Todo { content, .. } => assert_eq!(content, "Child A1"),
            _ => panic!("Expected Todo item"),
        }
        match &app.todo_list.items[1] {
            ListItem::Todo { content, .. } => assert_eq!(content, "Parent A"),
            _ => panic!("Expected Todo item"),
        }
        match &app.todo_list.items[2] {
            ListItem::Todo { content, .. } => assert_eq!(content, "Parent B"),
            _ => panic!("Expected Todo item"),
        }
        
        // Selection should follow the moved item
        assert_eq!(app.selected_index, 1);
    }

    #[test]
    fn test_move_single_item_up() {
        let mut todo_list = TodoList::new("test".to_string());
        todo_list.add_item(ListItem::new_todo("Parent A".to_string(), false, 0, 0));
        todo_list.add_item(ListItem::new_todo("Parent B".to_string(), false, 0, 1));
        todo_list.add_item(ListItem::new_todo("Child B1".to_string(), false, 1, 2));
        
        let mut app = App::new(todo_list);
        app.selected_index = 1; // Select "Parent B"
        
        app.move_item_up();
        
        // With single-item movement, only "Parent B" moves up, children stay in place
        // Order should be: Parent B, Parent A, Child B1
        match &app.todo_list.items[0] {
            ListItem::Todo { content, .. } => assert_eq!(content, "Parent B"),
            _ => panic!("Expected Todo item"),
        }
        match &app.todo_list.items[1] {
            ListItem::Todo { content, .. } => assert_eq!(content, "Parent A"),
            _ => panic!("Expected Todo item"),
        }
        match &app.todo_list.items[2] {
            ListItem::Todo { content, .. } => assert_eq!(content, "Child B1"),
            _ => panic!("Expected Todo item"),
        }
        
        // Selection should follow the moved item
        assert_eq!(app.selected_index, 0);
    }

    #[test]
    fn test_move_item_down_does_not_adopt_children() {
        // This test reproduces the exact bug scenario the user reported:
        // "if i am moving a to-do item, and i want to move it down beneath an item that has children, 
        // then when i move it to be directly above the children, then it 'takes over' as the parent 
        // of the children and i move the children with it. i don't want that!"
        
        let mut todo_list = TodoList::new("test".to_string());
        todo_list.add_item(ListItem::new_todo("Task A".to_string(), false, 0, 0));
        todo_list.add_item(ListItem::new_todo("Task B".to_string(), false, 0, 1));
        todo_list.add_item(ListItem::new_todo("Child B1".to_string(), false, 1, 2));
        todo_list.add_item(ListItem::new_todo("Child B2".to_string(), false, 1, 3));
        
        let mut app = App::new(todo_list);
        app.selected_index = 0; // Select "Task A"
        
        // Move Task A down (swapping with Task B)
        app.move_item_down();
        // Now order is: Task B, Task A, Child B1, Child B2
        // Selection is now on index 1 (Task A)
        
        // Move Task A down again (swapping with Child B1)
        app.move_item_down();
        // Now order should be: Task B, Child B1, Task A, Child B2
        // NOT: Task B, Task A, Child B1, Child B2 (which would be block movement)
        
        match &app.todo_list.items[0] {
            ListItem::Todo { content, .. } => assert_eq!(content, "Task B"),
            _ => panic!("Expected Todo item"),
        }
        match &app.todo_list.items[1] {
            ListItem::Todo { content, .. } => assert_eq!(content, "Child B1"),
            _ => panic!("Expected Todo item"),
        }
        match &app.todo_list.items[2] {
            ListItem::Todo { content, .. } => assert_eq!(content, "Task A"),
            _ => panic!("Expected Todo item"),
        }
        match &app.todo_list.items[3] {
            ListItem::Todo { content, .. } => assert_eq!(content, "Child B2"),
            _ => panic!("Expected Todo item"),
        }
        
        // Selection should follow the moved item
        assert_eq!(app.selected_index, 2);
    }

    #[test]
    fn test_indent_block() {
        let mut todo_list = TodoList::new("test".to_string());
        todo_list.add_item(ListItem::new_todo("Parent A".to_string(), false, 0, 0));
        todo_list.add_item(ListItem::new_todo("Parent B".to_string(), false, 0, 1));
        todo_list.add_item(ListItem::new_todo("Child B1".to_string(), false, 1, 2));
        todo_list.add_item(ListItem::new_todo("Child B2".to_string(), false, 1, 3));
        
        let mut app = App::new(todo_list);
        app.selected_index = 1; // Select "Parent B"
        
        app.indent_item();
        
        // Parent B should be indented to level 1
        match &app.todo_list.items[1] {
            ListItem::Todo { content, indent_level, .. } => {
                assert_eq!(content, "Parent B");
                assert_eq!(*indent_level, 1);
            }
            _ => panic!("Expected Todo item"),
        }
        
        // Children should also be indented (maintaining relative hierarchy)
        match &app.todo_list.items[2] {
            ListItem::Todo { content, indent_level, .. } => {
                assert_eq!(content, "Child B1");
                assert_eq!(*indent_level, 2);
            }
            _ => panic!("Expected Todo item"),
        }
        
        match &app.todo_list.items[3] {
            ListItem::Todo { content, indent_level, .. } => {
                assert_eq!(content, "Child B2");
                assert_eq!(*indent_level, 2);
            }
            _ => panic!("Expected Todo item"),
        }
    }

    #[test]
    fn test_unindent_block() {
        let mut todo_list = TodoList::new("test".to_string());
        todo_list.add_item(ListItem::new_todo("Parent A".to_string(), false, 0, 0));
        todo_list.add_item(ListItem::new_todo("Parent B".to_string(), false, 1, 1));
        todo_list.add_item(ListItem::new_todo("Child B1".to_string(), false, 2, 2));
        todo_list.add_item(ListItem::new_todo("Child B2".to_string(), false, 2, 3));
        
        let mut app = App::new(todo_list);
        app.selected_index = 1; // Select "Parent B"
        
        app.unindent_item();
        
        // Parent B should be unindented to level 0
        match &app.todo_list.items[1] {
            ListItem::Todo { content, indent_level, .. } => {
                assert_eq!(content, "Parent B");
                assert_eq!(*indent_level, 0);
            }
            _ => panic!("Expected Todo item"),
        }
        
        // Children should also be unindented (maintaining relative hierarchy)
        match &app.todo_list.items[2] {
            ListItem::Todo { content, indent_level, .. } => {
                assert_eq!(content, "Child B1");
                assert_eq!(*indent_level, 1);
            }
            _ => panic!("Expected Todo item"),
        }
        
        match &app.todo_list.items[3] {
            ListItem::Todo { content, indent_level, .. } => {
                assert_eq!(content, "Child B2");
                assert_eq!(*indent_level, 1);
            }
            _ => panic!("Expected Todo item"),
        }
    }

    #[test]
    fn test_indent_block_cannot_exceed_max_level() {
        let mut todo_list = TodoList::new("test".to_string());
        todo_list.add_item(ListItem::new_todo("Parent A".to_string(), false, 2, 0));
        todo_list.add_item(ListItem::new_todo("Parent B".to_string(), false, 2, 1));
        todo_list.add_item(ListItem::new_todo("Child B1".to_string(), false, 3, 2));
        
        let mut app = App::new(todo_list);
        app.selected_index = 1; // Select "Parent B"
        
        app.indent_item();
        
        // Parent B should be indented to level 3 (one more than Parent A)
        match &app.todo_list.items[1] {
            ListItem::Todo { content, indent_level, .. } => {
                assert_eq!(content, "Parent B");
                assert_eq!(*indent_level, 3);
            }
            _ => panic!("Expected Todo item"),
        }
        
        // Child should maintain relative hierarchy
        match &app.todo_list.items[2] {
            ListItem::Todo { content, indent_level, .. } => {
                assert_eq!(content, "Child B1");
                assert_eq!(*indent_level, 4);
            }
            _ => panic!("Expected Todo item"),
        }
    }

    #[test]
    fn test_unindent_block_cannot_go_below_zero() {
        let mut todo_list = TodoList::new("test".to_string());
        todo_list.add_item(ListItem::new_todo("Parent A".to_string(), false, 0, 0));
        todo_list.add_item(ListItem::new_todo("Child A1".to_string(), false, 1, 1));
        
        let mut app = App::new(todo_list);
        app.selected_index = 0; // Select "Parent A"
        
        app.unindent_item();
        
        // Parent A should remain at level 0 (cannot go below 0)
        match &app.todo_list.items[0] {
            ListItem::Todo { content, indent_level, .. } => {
                assert_eq!(content, "Parent A");
                assert_eq!(*indent_level, 0);
            }
            _ => panic!("Expected Todo item"),
        }
        
        // Child should also remain unchanged
        match &app.todo_list.items[1] {
            ListItem::Todo { content, indent_level, .. } => {
                assert_eq!(content, "Child A1");
                assert_eq!(*indent_level, 1);
            }
            _ => panic!("Expected Todo item"),
        }
    }

    #[test]
    fn test_indent_block_under_heading() {
        let mut todo_list = TodoList::new("test".to_string());
        todo_list.add_item(ListItem::new_heading("Section A".to_string(), 1, 0));
        todo_list.add_item(ListItem::new_todo("Task A".to_string(), false, 0, 1));
        todo_list.add_item(ListItem::new_todo("Subtask A1".to_string(), false, 1, 2));
        
        let mut app = App::new(todo_list);
        app.selected_index = 1; // Select "Task A"
        
        app.indent_item();
        
        // Task A should be indented to level 1 (can indent under headings)
        match &app.todo_list.items[1] {
            ListItem::Todo { content, indent_level, .. } => {
                assert_eq!(content, "Task A");
                assert_eq!(*indent_level, 1);
            }
            _ => panic!("Expected Todo item"),
        }
        
        // Subtask should maintain relative hierarchy
        match &app.todo_list.items[2] {
            ListItem::Todo { content, indent_level, .. } => {
                assert_eq!(content, "Subtask A1");
                assert_eq!(*indent_level, 2);
            }
            _ => panic!("Expected Todo item"),
        }
    }

    #[test]
    fn test_enter_edit_mode_todo() {
        let mut todo_list = TodoList::new("test".to_string());
        todo_list.add_item(ListItem::new_todo("Test task".to_string(), false, 0, 0));
        
        let mut app = App::new(todo_list);
        app.selected_index = 0;
        
        // Should not be in edit mode initially
        assert!(!app.edit_mode);
        assert!(app.edit_buffer.is_empty());
        
        app.enter_edit_mode();
        
        // Should now be in edit mode with content loaded
        assert!(app.edit_mode);
        assert_eq!(app.edit_buffer, "Test task");
        assert_eq!(app.edit_cursor_position, "Test task".len());
    }

    #[test]
    fn test_enter_edit_mode_heading() {
        let mut todo_list = TodoList::new("test".to_string());
        todo_list.add_item(ListItem::new_heading("Test heading".to_string(), 1, 0));
        
        let mut app = App::new(todo_list);
        app.selected_index = 0;
        
        app.enter_edit_mode();
        
        // Should be in edit mode with heading content loaded
        assert!(app.edit_mode);
        assert_eq!(app.edit_buffer, "Test heading");
        assert_eq!(app.edit_cursor_position, "Test heading".len());
    }

    #[test]
    fn test_cancel_edit() {
        let mut todo_list = TodoList::new("test".to_string());
        todo_list.add_item(ListItem::new_todo("Test task".to_string(), false, 0, 0));
        
        let mut app = App::new(todo_list);
        app.selected_index = 0;
        
        app.enter_edit_mode();
        app.edit_insert_char('!');
        
        // Verify we're in edit mode with changes
        assert!(app.edit_mode);
        assert_eq!(app.edit_buffer, "Test task!");
        
        app.cancel_edit();
        
        // Should exit edit mode and clear buffer
        assert!(!app.edit_mode);
        assert!(app.edit_buffer.is_empty());
        assert_eq!(app.edit_cursor_position, 0);
        
        // Original content should be unchanged
        match &app.todo_list.items[0] {
            ListItem::Todo { content, .. } => assert_eq!(content, "Test task"),
            _ => panic!("Expected Todo item"),
        }
    }

    #[test]
    fn test_confirm_edit_todo() {
        let mut todo_list = TodoList::new("test".to_string());
        todo_list.add_item(ListItem::new_todo("Test task".to_string(), false, 0, 0));
        
        let mut app = App::new(todo_list);
        app.selected_index = 0;
        
        app.enter_edit_mode();
        app.edit_insert_char('!');
        
        app.confirm_edit();
        
        // Should exit edit mode
        assert!(!app.edit_mode);
        assert!(app.edit_buffer.is_empty());
        assert_eq!(app.edit_cursor_position, 0);
        
        // Content should be updated
        match &app.todo_list.items[0] {
            ListItem::Todo { content, .. } => assert_eq!(content, "Test task!"),
            _ => panic!("Expected Todo item"),
        }
    }

    #[test]
    fn test_confirm_edit_heading() {
        let mut todo_list = TodoList::new("test".to_string());
        todo_list.add_item(ListItem::new_heading("Test heading".to_string(), 1, 0));
        
        let mut app = App::new(todo_list);
        app.selected_index = 0;
        
        app.enter_edit_mode();
        app.edit_insert_char('!');
        
        app.confirm_edit();
        
        // Content should be updated
        match &app.todo_list.items[0] {
            ListItem::Heading { content, .. } => assert_eq!(content, "Test heading!"),
            _ => panic!("Expected Heading item"),
        }
    }

    #[test]
    fn test_edit_text_operations() {
        let mut todo_list = TodoList::new("test".to_string());
        todo_list.add_item(ListItem::new_todo("Hello World".to_string(), false, 0, 0));
        
        let mut app = App::new(todo_list);
        app.selected_index = 0;
        
        app.enter_edit_mode();
        assert_eq!(app.edit_buffer, "Hello World");
        assert_eq!(app.edit_cursor_position, "Hello World".len());
        
        // Test cursor movement
        app.edit_move_cursor_left(); // Move to before 'd'
        assert_eq!(app.edit_cursor_position, "Hello Worl".len());
        app.edit_move_cursor_left(); // Move to before 'l'
        assert_eq!(app.edit_cursor_position, "Hello Wor".len());
        app.edit_move_cursor_left(); // Move to before 'r'
        assert_eq!(app.edit_cursor_position, "Hello Wo".len());
        
        // Test inserting - when cursor is at "Hello Wo|rld", inserting chars will give "Hello Wo<chars>rld"
        app.edit_insert_char('n');
        assert_eq!(app.edit_buffer, "Hello Wonrld");
        app.edit_insert_char('d');
        assert_eq!(app.edit_buffer, "Hello Wondrld");
        app.edit_insert_char('e');
        assert_eq!(app.edit_buffer, "Hello Wonderld");
        app.edit_insert_char('r');
        assert_eq!(app.edit_buffer, "Hello Wonderrld");
        app.edit_insert_char('f');
        assert_eq!(app.edit_buffer, "Hello Wonderfrld");
        app.edit_insert_char('u');
        assert_eq!(app.edit_buffer, "Hello Wonderfurld");
        app.edit_insert_char('l');
        assert_eq!(app.edit_buffer, "Hello Wonderfulrld");
        app.edit_insert_char(' ');
        assert_eq!(app.edit_buffer, "Hello Wonderful rld");
        
        // Test backspace
        app.edit_backspace(); // Remove space
        assert_eq!(app.edit_buffer, "Hello Wonderfulrld");
        app.edit_backspace(); // Remove 'l'
        assert_eq!(app.edit_buffer, "Hello Wonderfurld");
        
        // Test delete
        app.edit_delete(); // Remove 'r'
        assert_eq!(app.edit_buffer, "Hello Wonderfuld");
    }

    #[test]
    fn test_edit_cursor_boundaries() {
        let mut todo_list = TodoList::new("test".to_string());
        todo_list.add_item(ListItem::new_todo("Hi".to_string(), false, 0, 0));
        
        let mut app = App::new(todo_list);
        app.selected_index = 0;
        
        app.enter_edit_mode();
        
        // Test moving cursor left beyond start
        app.edit_cursor_position = 0;
        app.edit_move_cursor_left();
        assert_eq!(app.edit_cursor_position, 0);
        
        // Test moving cursor right beyond end
        app.edit_cursor_position = app.edit_buffer.len();
        app.edit_move_cursor_right();
        assert_eq!(app.edit_cursor_position, app.edit_buffer.len());
        
        // Test backspace at start
        app.edit_cursor_position = 0;
        app.edit_backspace();
        assert_eq!(app.edit_buffer, "Hi");
        
        // Test delete at end
        app.edit_cursor_position = app.edit_buffer.len();
        app.edit_delete();
        assert_eq!(app.edit_buffer, "Hi");
    }

    #[test]
    fn test_add_new_todo_empty_list() {
        let todo_list = TodoList::new("test".to_string());
        let mut app = App::new(todo_list);
        
        app.add_new_todo();
        
        // Should have one item at level 0
        assert_eq!(app.todo_list.items.len(), 1);
        match &app.todo_list.items[0] {
            ListItem::Todo { content, completed, indent_level, .. } => {
                assert_eq!(content, "");
                assert!(!completed);
                assert_eq!(*indent_level, 0);
            }
            _ => panic!("Expected Todo item"),
        }
        
        // Should be selected and in edit mode
        assert_eq!(app.selected_index, 0);
        assert!(app.edit_mode);
    }

    #[test]
    fn test_add_new_todo_after_existing_todo() {
        let mut todo_list = TodoList::new("test".to_string());
        todo_list.add_item(ListItem::new_todo("Existing task".to_string(), false, 1, 0));
        
        let mut app = App::new(todo_list);
        app.selected_index = 0;
        
        app.add_new_todo();
        
        // Should have two items
        assert_eq!(app.todo_list.items.len(), 2);
        
        // First item should remain unchanged
        match &app.todo_list.items[0] {
            ListItem::Todo { content, indent_level, .. } => {
                assert_eq!(content, "Existing task");
                assert_eq!(*indent_level, 1);
            }
            _ => panic!("Expected Todo item"),
        }
        
        // Second item should be new and empty with same indent level
        match &app.todo_list.items[1] {
            ListItem::Todo { content, completed, indent_level, .. } => {
                assert_eq!(content, "");
                assert!(!completed);
                assert_eq!(*indent_level, 1);
            }
            _ => panic!("Expected Todo item"),
        }
        
        // Should be selected on new item and in edit mode
        assert_eq!(app.selected_index, 1);
        assert!(app.edit_mode);
    }

    #[test]
    fn test_add_new_todo_after_heading() {
        let mut todo_list = TodoList::new("test".to_string());
        todo_list.add_item(ListItem::new_heading("Section".to_string(), 1, 0));
        
        let mut app = App::new(todo_list);
        app.selected_index = 0;
        
        app.add_new_todo();
        
        // Should have two items
        assert_eq!(app.todo_list.items.len(), 2);
        
        // First item should remain unchanged
        match &app.todo_list.items[0] {
            ListItem::Heading { content, level, .. } => {
                assert_eq!(content, "Section");
                assert_eq!(*level, 1);
            }
            _ => panic!("Expected Heading item"),
        }
        
        // Second item should be new todo at level 0 (todos under headings start at 0)
        match &app.todo_list.items[1] {
            ListItem::Todo { content, completed, indent_level, .. } => {
                assert_eq!(content, "");
                assert!(!completed);
                assert_eq!(*indent_level, 0);
            }
            _ => panic!("Expected Todo item"),
        }
        
        // Should be selected on new item and in edit mode
        assert_eq!(app.selected_index, 1);
        assert!(app.edit_mode);
    }

    #[test]
    fn test_add_new_todo_preserves_indentation() {
        let mut todo_list = TodoList::new("test".to_string());
        todo_list.add_item(ListItem::new_todo("Parent".to_string(), false, 0, 0));
        todo_list.add_item(ListItem::new_todo("Child".to_string(), false, 2, 1));
        todo_list.add_item(ListItem::new_todo("Sibling".to_string(), false, 0, 2));
        
        let mut app = App::new(todo_list);
        app.selected_index = 1; // Select "Child" at level 2
        
        app.add_new_todo();
        
        // Should have four items
        assert_eq!(app.todo_list.items.len(), 4);
        
        // New item should be inserted at position 2 with same indent level as "Child"
        match &app.todo_list.items[2] {
            ListItem::Todo { content, completed, indent_level, .. } => {
                assert_eq!(content, "");
                assert!(!completed);
                assert_eq!(*indent_level, 2);
            }
            _ => panic!("Expected Todo item"),
        }
        
        // Original "Sibling" should now be at position 3
        match &app.todo_list.items[3] {
            ListItem::Todo { content, .. } => {
                assert_eq!(content, "Sibling");
            }
            _ => panic!("Expected Todo item"),
        }
        
        // Should be selected on new item and in edit mode
        assert_eq!(app.selected_index, 2);
        assert!(app.edit_mode);
    }

    #[test]
    fn test_confirm_edit_removes_empty_todo() {
        let mut todo_list = TodoList::new("test".to_string());
        todo_list.add_item(ListItem::new_todo("Test task".to_string(), false, 0, 0));
        todo_list.add_item(ListItem::new_todo("Another task".to_string(), false, 0, 1));
        
        let mut app = App::new(todo_list);
        app.selected_index = 0;
        
        // Enter edit mode and clear the content
        app.enter_edit_mode();
        app.edit_buffer.clear();
        app.edit_cursor_position = 0;
        
        app.confirm_edit();
        
        // First todo should be removed, second should move up
        assert_eq!(app.todo_list.items.len(), 1);
        match &app.todo_list.items[0] {
            ListItem::Todo { content, .. } => assert_eq!(content, "Another task"),
            _ => panic!("Expected Todo item"),
        }
        
        // Selection should adjust to stay in bounds
        assert_eq!(app.selected_index, 0);
        assert!(!app.edit_mode);
    }

    #[test]
    fn test_confirm_edit_preserves_whitespace_only_todo() {
        let mut todo_list = TodoList::new("test".to_string());
        todo_list.add_item(ListItem::new_todo("Test task".to_string(), false, 0, 0));
        
        let mut app = App::new(todo_list);
        app.selected_index = 0;
        
        // Enter edit mode and set content to only spaces
        app.enter_edit_mode();
        app.edit_buffer = "   ".to_string();
        
        app.confirm_edit();
        
        // Todo with only whitespace should be removed
        assert_eq!(app.todo_list.items.len(), 0);
        assert_eq!(app.selected_index, 0);
        assert!(!app.edit_mode);
    }

    #[test]
    fn test_confirm_edit_preserves_empty_heading() {
        let mut todo_list = TodoList::new("test".to_string());
        todo_list.add_item(ListItem::new_heading("Section".to_string(), 1, 0));
        
        let mut app = App::new(todo_list);
        app.selected_index = 0;
        
        // Enter edit mode and clear the heading content
        app.enter_edit_mode();
        app.edit_buffer.clear();
        app.edit_cursor_position = 0;
        
        app.confirm_edit();
        
        // Empty heading should be preserved (not removed)
        assert_eq!(app.todo_list.items.len(), 1);
        match &app.todo_list.items[0] {
            ListItem::Heading { content, .. } => assert_eq!(content, ""),
            _ => panic!("Expected Heading item"),
        }
        
        assert!(!app.edit_mode);
    }

    #[test]
    fn test_cancel_edit_removes_empty_todo() {
        let mut todo_list = TodoList::new("test".to_string());
        todo_list.add_item(ListItem::new_todo("Test task".to_string(), false, 0, 0));
        
        let mut app = App::new(todo_list);
        app.selected_index = 0;
        
        // Clear the content first, then enter edit mode
        if let Some(ListItem::Todo { content, .. }) = app.todo_list.items.get_mut(0) {
            *content = String::new();
        }
        
        app.enter_edit_mode();
        app.cancel_edit();
        
        // Empty todo should be removed when canceling edit
        assert_eq!(app.todo_list.items.len(), 0);
        assert!(!app.edit_mode);
    }

    #[test]
    fn test_cancel_edit_preserves_non_empty_todo() {
        let mut todo_list = TodoList::new("test".to_string());
        todo_list.add_item(ListItem::new_todo("Test task".to_string(), false, 0, 0));
        
        let mut app = App::new(todo_list);
        app.selected_index = 0;
        
        app.enter_edit_mode();
        app.edit_insert_char('!');
        app.cancel_edit();
        
        // Non-empty todo should be preserved with original content
        assert_eq!(app.todo_list.items.len(), 1);
        match &app.todo_list.items[0] {
            ListItem::Todo { content, .. } => assert_eq!(content, "Test task"),
            _ => panic!("Expected Todo item"),
        }
        
        assert!(!app.edit_mode);
    }

    #[test]
    fn test_add_new_todo_then_cancel_removes_it() {
        let mut todo_list = TodoList::new("test".to_string());
        todo_list.add_item(ListItem::new_todo("Existing task".to_string(), false, 0, 0));
        
        let mut app = App::new(todo_list);
        app.selected_index = 0;
        
        // Add new todo (which enters edit mode automatically)
        app.add_new_todo();
        
        // Should have 2 items and be in edit mode on the new one
        assert_eq!(app.todo_list.items.len(), 2);
        assert_eq!(app.selected_index, 1);
        assert!(app.edit_mode);
        
        // Cancel without typing anything
        app.cancel_edit();
        
        // Empty new todo should be removed
        assert_eq!(app.todo_list.items.len(), 1);
        match &app.todo_list.items[0] {
            ListItem::Todo { content, .. } => assert_eq!(content, "Existing task"),
            _ => panic!("Expected Todo item"),
        }
        
        // Selection should adjust back to the remaining item
        assert_eq!(app.selected_index, 0);
        assert!(!app.edit_mode);
    }

    #[test]
    fn test_add_new_todo_then_confirm_empty_removes_it() {
        let mut todo_list = TodoList::new("test".to_string());
        todo_list.add_item(ListItem::new_todo("Existing task".to_string(), false, 0, 0));
        
        let mut app = App::new(todo_list);
        app.selected_index = 0;
        
        // Add new todo
        app.add_new_todo();
        
        // Confirm without typing anything (edit_buffer is empty)
        app.confirm_edit();
        
        // Empty new todo should be removed
        assert_eq!(app.todo_list.items.len(), 1);
        match &app.todo_list.items[0] {
            ListItem::Todo { content, .. } => assert_eq!(content, "Existing task"),
            _ => panic!("Expected Todo item"),
        }
        
        assert_eq!(app.selected_index, 0);
        assert!(!app.edit_mode);
    }

    #[test]
    fn test_add_new_todo_to_parent_with_children() {
        let mut todo_list = TodoList::new("test".to_string());
        todo_list.add_item(ListItem::new_todo("Parent".to_string(), false, 0, 0));
        todo_list.add_item(ListItem::new_todo("Child 1".to_string(), false, 1, 1));
        todo_list.add_item(ListItem::new_todo("Child 2".to_string(), false, 1, 2));
        todo_list.add_item(ListItem::new_todo("Sibling".to_string(), false, 0, 3));
        
        let mut app = App::new(todo_list);
        app.selected_index = 0; // Select "Parent"
        
        app.add_new_todo();
        
        // Should have 5 items
        assert_eq!(app.todo_list.items.len(), 5);
        
        // New child should be inserted after last child (position 3)
        match &app.todo_list.items[3] {
            ListItem::Todo { content, completed, indent_level, .. } => {
                assert_eq!(content, "");
                assert!(!completed);
                assert_eq!(*indent_level, 1); // Child indentation
            }
            _ => panic!("Expected Todo item"),
        }
        
        // "Sibling" should now be at position 4
        match &app.todo_list.items[4] {
            ListItem::Todo { content, .. } => assert_eq!(content, "Sibling"),
            _ => panic!("Expected Todo item"),
        }
        
        // Should be selected on new child and in edit mode
        assert_eq!(app.selected_index, 3);
        assert!(app.edit_mode);
    }

    #[test]
    fn test_add_new_todo_to_parent_without_children() {
        let mut todo_list = TodoList::new("test".to_string());
        todo_list.add_item(ListItem::new_todo("Parent 1".to_string(), false, 0, 0));
        todo_list.add_item(ListItem::new_todo("Parent 2".to_string(), false, 0, 1));
        
        let mut app = App::new(todo_list);
        app.selected_index = 0; // Select "Parent 1" (no children)
        
        app.add_new_todo();
        
        // Should have 3 items
        assert_eq!(app.todo_list.items.len(), 3);
        
        // New sibling should be inserted at position 1
        match &app.todo_list.items[1] {
            ListItem::Todo { content, completed, indent_level, .. } => {
                assert_eq!(content, "");
                assert!(!completed);
                assert_eq!(*indent_level, 0); // Same indentation as parent
            }
            _ => panic!("Expected Todo item"),
        }
        
        // "Parent 2" should now be at position 2
        match &app.todo_list.items[2] {
            ListItem::Todo { content, .. } => assert_eq!(content, "Parent 2"),
            _ => panic!("Expected Todo item"),
        }
        
        // Should be selected on new sibling and in edit mode
        assert_eq!(app.selected_index, 1);
        assert!(app.edit_mode);
    }

    #[test]
    fn test_add_new_todo_to_nested_parent_with_children() {
        let mut todo_list = TodoList::new("test".to_string());
        todo_list.add_item(ListItem::new_todo("Root".to_string(), false, 0, 0));
        todo_list.add_item(ListItem::new_todo("Parent".to_string(), false, 1, 1));
        todo_list.add_item(ListItem::new_todo("Child A".to_string(), false, 2, 2));
        todo_list.add_item(ListItem::new_todo("Child B".to_string(), false, 2, 3));
        todo_list.add_item(ListItem::new_todo("Other".to_string(), false, 0, 4));
        
        let mut app = App::new(todo_list);
        app.selected_index = 1; // Select "Parent" (has children at level 2)
        
        app.add_new_todo();
        
        // Should have 6 items
        assert_eq!(app.todo_list.items.len(), 6);
        
        // New child should be inserted after "Child B" (position 4)
        match &app.todo_list.items[4] {
            ListItem::Todo { content, completed, indent_level, .. } => {
                assert_eq!(content, "");
                assert!(!completed);
                assert_eq!(*indent_level, 2); // Child indentation (parent + 1)
            }
            _ => panic!("Expected Todo item"),
        }
        
        // "Other" should now be at position 5
        match &app.todo_list.items[5] {
            ListItem::Todo { content, .. } => assert_eq!(content, "Other"),
            _ => panic!("Expected Todo item"),
        }
        
        // Should be selected on new child and in edit mode
        assert_eq!(app.selected_index, 4);
        assert!(app.edit_mode);
    }

    #[test]
    fn test_add_new_todo_to_multi_level_parent() {
        let mut todo_list = TodoList::new("test".to_string());
        todo_list.add_item(ListItem::new_todo("Parent".to_string(), false, 0, 0));
        todo_list.add_item(ListItem::new_todo("Child".to_string(), false, 1, 1));
        todo_list.add_item(ListItem::new_todo("Grandchild".to_string(), false, 2, 2));
        todo_list.add_item(ListItem::new_todo("Great-grandchild".to_string(), false, 3, 3));
        todo_list.add_item(ListItem::new_todo("Next parent".to_string(), false, 0, 4));
        
        let mut app = App::new(todo_list);
        app.selected_index = 0; // Select "Parent" (has multi-level children)
        
        app.add_new_todo();
        
        // Should have 6 items
        assert_eq!(app.todo_list.items.len(), 6);
        
        // New child should be inserted after "Great-grandchild" (position 4)
        match &app.todo_list.items[4] {
            ListItem::Todo { content, completed, indent_level, .. } => {
                assert_eq!(content, "");
                assert!(!completed);
                assert_eq!(*indent_level, 1); // Direct child of parent (level 0 + 1)
            }
            _ => panic!("Expected Todo item"),
        }
        
        // "Next parent" should now be at position 5
        match &app.todo_list.items[5] {
            ListItem::Todo { content, .. } => assert_eq!(content, "Next parent"),
            _ => panic!("Expected Todo item"),
        }
        
        // Should be selected on new child and in edit mode
        assert_eq!(app.selected_index, 4);
        assert!(app.edit_mode);
    }

    #[test]
    fn test_add_new_todo_preserves_existing_behavior_for_childless_items() {
        // Test that the existing tests still pass with the new logic
        let mut todo_list = TodoList::new("test".to_string());
        todo_list.add_item(ListItem::new_todo("Task 1".to_string(), false, 2, 0));
        todo_list.add_item(ListItem::new_todo("Task 2".to_string(), false, 1, 1));
        
        let mut app = App::new(todo_list);
        app.selected_index = 0; // Select "Task 1" (no children)
        
        app.add_new_todo();
        
        // Should add sibling with same indentation
        assert_eq!(app.todo_list.items.len(), 3);
        match &app.todo_list.items[1] {
            ListItem::Todo { content, indent_level, .. } => {
                assert_eq!(content, "");
                assert_eq!(*indent_level, 2); // Same as parent
            }
            _ => panic!("Expected Todo item"),
        }
        
        assert_eq!(app.selected_index, 1);
        assert!(app.edit_mode);
    }

    #[test]
    fn test_add_new_todo_at_top_empty_list() {
        let todo_list = TodoList::new("test".to_string());
        let mut app = App::new(todo_list);
        
        app.add_new_todo_at_top();
        
        // Should have one item at position 0
        assert_eq!(app.todo_list.items.len(), 1);
        match &app.todo_list.items[0] {
            ListItem::Todo { content, completed, indent_level, .. } => {
                assert_eq!(content, "");
                assert!(!completed);
                assert_eq!(*indent_level, 0);
            }
            _ => panic!("Expected Todo item"),
        }
        
        // Should be selected and in edit mode
        assert_eq!(app.selected_index, 0);
        assert!(app.edit_mode);
    }

    #[test]
    fn test_add_new_todo_at_top_no_headings() {
        let mut todo_list = TodoList::new("test".to_string());
        todo_list.add_item(ListItem::new_todo("First task".to_string(), false, 0, 0));
        todo_list.add_item(ListItem::new_todo("Second task".to_string(), false, 1, 1));
        
        let mut app = App::new(todo_list);
        app.selected_index = 1; // Select "Second task"
        
        app.add_new_todo_at_top();
        
        // Should have 3 items with new one at the very top
        assert_eq!(app.todo_list.items.len(), 3);
        
        // New todo should be at position 0
        match &app.todo_list.items[0] {
            ListItem::Todo { content, completed, indent_level, .. } => {
                assert_eq!(content, "");
                assert!(!completed);
                assert_eq!(*indent_level, 0);
            }
            _ => panic!("Expected Todo item"),
        }
        
        // Original items should be shifted down
        match &app.todo_list.items[1] {
            ListItem::Todo { content, .. } => assert_eq!(content, "First task"),
            _ => panic!("Expected Todo item"),
        }
        
        match &app.todo_list.items[2] {
            ListItem::Todo { content, .. } => assert_eq!(content, "Second task"),
            _ => panic!("Expected Todo item"),
        }
        
        // Should be selected on new todo and in edit mode
        assert_eq!(app.selected_index, 0);
        assert!(app.edit_mode);
    }

    #[test]
    fn test_add_new_todo_at_top_under_current_heading() {
        let mut todo_list = TodoList::new("test".to_string());
        todo_list.add_item(ListItem::new_heading("Section A".to_string(), 1, 0));
        todo_list.add_item(ListItem::new_todo("Task A1".to_string(), false, 0, 1));
        todo_list.add_item(ListItem::new_todo("Task A2".to_string(), false, 0, 2));
        todo_list.add_item(ListItem::new_heading("Section B".to_string(), 1, 3));
        todo_list.add_item(ListItem::new_todo("Task B1".to_string(), false, 0, 4));
        
        let mut app = App::new(todo_list);
        app.selected_index = 2; // Select "Task A2" (under Section A)
        
        app.add_new_todo_at_top();
        
        // Should have 6 items
        assert_eq!(app.todo_list.items.len(), 6);
        
        // New todo should be inserted right after "Section A" (position 1)
        match &app.todo_list.items[1] {
            ListItem::Todo { content, completed, indent_level, .. } => {
                assert_eq!(content, "");
                assert!(!completed);
                assert_eq!(*indent_level, 0);
            }
            _ => panic!("Expected Todo item"),
        }
        
        // "Task A1" should now be at position 2
        match &app.todo_list.items[2] {
            ListItem::Todo { content, .. } => assert_eq!(content, "Task A1"),
            _ => panic!("Expected Todo item"),
        }
        
        // Should be selected on new todo and in edit mode
        assert_eq!(app.selected_index, 1);
        assert!(app.edit_mode);
    }

    #[test]
    fn test_add_new_todo_at_top_before_any_heading() {
        let mut todo_list = TodoList::new("test".to_string());
        todo_list.add_item(ListItem::new_todo("Preamble task".to_string(), false, 0, 0));
        todo_list.add_item(ListItem::new_heading("Section A".to_string(), 1, 1));
        todo_list.add_item(ListItem::new_todo("Task A1".to_string(), false, 0, 2));
        
        let mut app = App::new(todo_list);
        app.selected_index = 0; // Select "Preamble task" (before any heading)
        
        app.add_new_todo_at_top();
        
        // Should have 4 items
        assert_eq!(app.todo_list.items.len(), 4);
        
        // New todo should be at the very top (position 0)
        match &app.todo_list.items[0] {
            ListItem::Todo { content, completed, indent_level, .. } => {
                assert_eq!(content, "");
                assert!(!completed);
                assert_eq!(*indent_level, 0);
            }
            _ => panic!("Expected Todo item"),
        }
        
        // "Preamble task" should now be at position 1
        match &app.todo_list.items[1] {
            ListItem::Todo { content, .. } => assert_eq!(content, "Preamble task"),
            _ => panic!("Expected Todo item"),
        }
        
        // Should be selected on new todo and in edit mode
        assert_eq!(app.selected_index, 0);
        assert!(app.edit_mode);
    }

    #[test]
    fn test_add_new_todo_at_top_on_heading_itself() {
        let mut todo_list = TodoList::new("test".to_string());
        todo_list.add_item(ListItem::new_heading("Section A".to_string(), 1, 0));
        todo_list.add_item(ListItem::new_todo("Task A1".to_string(), false, 0, 1));
        todo_list.add_item(ListItem::new_heading("Section B".to_string(), 1, 2));
        
        let mut app = App::new(todo_list);
        app.selected_index = 0; // Select "Section A" heading itself
        
        app.add_new_todo_at_top();
        
        // Should have 4 items
        assert_eq!(app.todo_list.items.len(), 4);
        
        // New todo should be inserted right after "Section A" (position 1)
        match &app.todo_list.items[1] {
            ListItem::Todo { content, completed, indent_level, .. } => {
                assert_eq!(content, "");
                assert!(!completed);
                assert_eq!(*indent_level, 0);
            }
            _ => panic!("Expected Todo item"),
        }
        
        // Should be selected on new todo and in edit mode
        assert_eq!(app.selected_index, 1);
        assert!(app.edit_mode);
    }

    #[test]
    fn test_add_new_todo_at_top_multiple_heading_levels() {
        let mut todo_list = TodoList::new("test".to_string());
        todo_list.add_item(ListItem::new_heading("Main Section".to_string(), 1, 0));
        todo_list.add_item(ListItem::new_heading("Subsection A".to_string(), 2, 1));
        todo_list.add_item(ListItem::new_todo("Task A1".to_string(), false, 0, 2));
        todo_list.add_item(ListItem::new_heading("Subsection B".to_string(), 2, 3));
        todo_list.add_item(ListItem::new_todo("Task B1".to_string(), false, 0, 4));
        
        let mut app = App::new(todo_list);
        app.selected_index = 4; // Select "Task B1" (under Subsection B, which is under Main Section)
        
        app.add_new_todo_at_top();
        
        // Should have 6 items
        assert_eq!(app.todo_list.items.len(), 6);
        
        // New todo should be inserted right after the most recent heading found backwards
        // When at position 4 (Task B1), looking backwards we find:
        // Position 3: Subsection B (heading) - this is the most recent heading
        // So new todo should be inserted at position 4, shifting Task B1 to position 5
        match &app.todo_list.items[4] {
            ListItem::Todo { content, completed, indent_level, .. } => {
                assert_eq!(content, "");
                assert!(!completed);
                assert_eq!(*indent_level, 0);
            }
            _ => panic!("Expected Todo item"),
        }
        
        // Task B1 should now be at position 5
        match &app.todo_list.items[5] {
            ListItem::Todo { content, .. } => assert_eq!(content, "Task B1"),
            _ => panic!("Expected Todo item"),
        }
        
        // Should be selected on new todo and in edit mode
        assert_eq!(app.selected_index, 4);
        assert!(app.edit_mode);
    }

    #[test]
    fn test_toggle_item_selection() {
        let mut todo_list = TodoList::new("test".to_string());
        todo_list.add_item(ListItem::new_todo("Task A".to_string(), false, 0, 0));
        todo_list.add_item(ListItem::new_todo("Task B".to_string(), false, 0, 1));
        todo_list.add_item(ListItem::new_todo("Task C".to_string(), false, 0, 2));
        
        let mut app = App::new(todo_list);
        app.selected_index = 0; // Select "Task A"
        
        // Initially, no items should be selected
        assert!(app.selected_items.is_empty());
        
        // Select item 0
        app.toggle_item_selection();
        assert!(app.selected_items.contains(&0));
        assert_eq!(app.selected_items.len(), 1);
        
        // Select item 2
        app.selected_index = 2;
        app.toggle_item_selection();
        assert!(app.selected_items.contains(&0));
        assert!(app.selected_items.contains(&2));
        assert_eq!(app.selected_items.len(), 2);
        
        // Deselect item 0
        app.selected_index = 0;
        app.toggle_item_selection();
        assert!(!app.selected_items.contains(&0));
        assert!(app.selected_items.contains(&2));
        assert_eq!(app.selected_items.len(), 1);
    }

    #[test]
    fn test_bulk_move_to_different_position() {
        let mut todo_list = TodoList::new("test".to_string());
        todo_list.add_item(ListItem::new_todo("Task A".to_string(), false, 0, 0));
        todo_list.add_item(ListItem::new_todo("Task B".to_string(), false, 0, 1));
        todo_list.add_item(ListItem::new_todo("Task C".to_string(), false, 0, 2));
        todo_list.add_item(ListItem::new_todo("Task D".to_string(), false, 0, 3));
        todo_list.add_item(ListItem::new_todo("Task E".to_string(), false, 0, 4));
        
        let mut app = App::new(todo_list);
        
        // Select items 0 and 2 (Task A and Task C)
        app.selected_index = 0;
        app.toggle_item_selection(); // Select Task A
        app.selected_index = 2;
        app.toggle_item_selection(); // Select Task C
        
        // Move cursor to position 4 (Task E) and bulk move
        app.selected_index = 4;
        app.move_selected_items_to_cursor();
        
        // Expected order: Task B, Task D, Task E, Task A, Task C
        // (Items A and C should be inserted after position 4, which becomes position 3 after removals)
        match &app.todo_list.items[0] {
            ListItem::Todo { content, .. } => assert_eq!(content, "Task B"),
            _ => panic!("Expected Todo item"),
        }
        match &app.todo_list.items[1] {
            ListItem::Todo { content, .. } => assert_eq!(content, "Task D"),
            _ => panic!("Expected Todo item"),
        }
        match &app.todo_list.items[2] {
            ListItem::Todo { content, .. } => assert_eq!(content, "Task E"),
            _ => panic!("Expected Todo item"),
        }
        match &app.todo_list.items[3] {
            ListItem::Todo { content, .. } => assert_eq!(content, "Task A"),
            _ => panic!("Expected Todo item"),
        }
        match &app.todo_list.items[4] {
            ListItem::Todo { content, .. } => assert_eq!(content, "Task C"),
            _ => panic!("Expected Todo item"),
        }
        
        // Selection should be cleared and cursor at first moved item
        assert!(app.selected_items.is_empty());
        assert_eq!(app.selected_index, 3);
    }

    #[test]
    fn test_bulk_move_preserves_order() {
        let mut todo_list = TodoList::new("test".to_string());
        todo_list.add_item(ListItem::new_todo("Task A".to_string(), false, 0, 0));
        todo_list.add_item(ListItem::new_todo("Task B".to_string(), false, 0, 1));
        todo_list.add_item(ListItem::new_todo("Task C".to_string(), false, 0, 2));
        todo_list.add_item(ListItem::new_todo("Task D".to_string(), false, 0, 3));
        
        let mut app = App::new(todo_list);
        
        // Select items 0, 1, and 3 in different order
        app.selected_index = 3;
        app.toggle_item_selection(); // Select Task D
        app.selected_index = 0;
        app.toggle_item_selection(); // Select Task A
        app.selected_index = 1;
        app.toggle_item_selection(); // Select Task B
        
        // Move cursor to position 2 (Task C) and bulk move
        app.selected_index = 2;
        app.move_selected_items_to_cursor();
        
        // Expected order: Task C, Task A, Task B, Task D
        // (Selected items A, B, D are inserted after cursor position 2, which becomes position 1 after removals)
        match &app.todo_list.items[0] {
            ListItem::Todo { content, .. } => assert_eq!(content, "Task C"),
            _ => panic!("Expected Todo item"),
        }
        match &app.todo_list.items[1] {
            ListItem::Todo { content, .. } => assert_eq!(content, "Task A"),
            _ => panic!("Expected Todo item"),
        }
        match &app.todo_list.items[2] {
            ListItem::Todo { content, .. } => assert_eq!(content, "Task B"),
            _ => panic!("Expected Todo item"),
        }
        match &app.todo_list.items[3] {
            ListItem::Todo { content, .. } => assert_eq!(content, "Task D"),
            _ => panic!("Expected Todo item"),
        }
        
        // Selection should be cleared
        assert!(app.selected_items.is_empty());
    }

    #[test]
    fn test_bulk_move_with_no_selection_does_nothing() {
        let mut todo_list = TodoList::new("test".to_string());
        todo_list.add_item(ListItem::new_todo("Task A".to_string(), false, 0, 0));
        todo_list.add_item(ListItem::new_todo("Task B".to_string(), false, 0, 1));
        
        let mut app = App::new(todo_list);
        app.selected_index = 1;
        
        // Try to bulk move without any selection
        app.move_selected_items_to_cursor();
        
        // Order should remain unchanged
        match &app.todo_list.items[0] {
            ListItem::Todo { content, .. } => assert_eq!(content, "Task A"),
            _ => panic!("Expected Todo item"),
        }
        match &app.todo_list.items[1] {
            ListItem::Todo { content, .. } => assert_eq!(content, "Task B"),
            _ => panic!("Expected Todo item"),
        }
        
        // Cursor should remain at same position
        assert_eq!(app.selected_index, 1);
    }

    #[test]
    fn test_bulk_move_with_mixed_item_types() {
        let mut todo_list = TodoList::new("test".to_string());
        todo_list.add_item(ListItem::new_heading("Heading 1".to_string(), 1, 0));
        todo_list.add_item(ListItem::new_todo("Task A".to_string(), false, 0, 1));
        todo_list.add_item(ListItem::new_todo("Task B".to_string(), false, 0, 2));
        todo_list.add_item(ListItem::new_heading("Heading 2".to_string(), 1, 3));
        
        let mut app = App::new(todo_list);
        
        // Select heading and todo item
        app.selected_index = 0;
        app.toggle_item_selection(); // Select Heading 1
        app.selected_index = 2;
        app.toggle_item_selection(); // Select Task B
        
        // Move cursor to end and bulk move
        app.selected_index = 3;
        app.move_selected_items_to_cursor();
        
        // Expected order: Task A, Heading 2, Heading 1, Task B
        match &app.todo_list.items[0] {
            ListItem::Todo { content, .. } => assert_eq!(content, "Task A"),
            _ => panic!("Expected Todo item"),
        }
        match &app.todo_list.items[1] {
            ListItem::Heading { content, .. } => assert_eq!(content, "Heading 2"),
            _ => panic!("Expected Heading item"),
        }
        match &app.todo_list.items[2] {
            ListItem::Heading { content, .. } => assert_eq!(content, "Heading 1"),
            _ => panic!("Expected Heading item"),
        }
        match &app.todo_list.items[3] {
            ListItem::Todo { content, .. } => assert_eq!(content, "Task B"),
            _ => panic!("Expected Todo item"),
        }
        
        assert!(app.selected_items.is_empty());
    }

    #[test]
    fn test_help_mode_toggle() {
        let todo_list = TodoList::new("test".to_string());
        let mut app = App::new(todo_list);
        
        // Initially not in help mode
        assert!(!app.help_mode);
        
        // Press '?' to enter help mode
        let key_event = KeyEvent::from(KeyCode::Char('?'));
        app.handle_key_event(key_event).unwrap();
        assert!(app.help_mode);
        
        // Press '?' again to exit help mode
        let key_event = KeyEvent::from(KeyCode::Char('?'));
        app.handle_key_event(key_event).unwrap();
        assert!(!app.help_mode);
        
        // Test Esc key to exit help mode
        let key_event = KeyEvent::from(KeyCode::Char('?'));
        app.handle_key_event(key_event).unwrap();
        assert!(app.help_mode);
        
        let key_event = KeyEvent::from(KeyCode::Esc);
        app.handle_key_event(key_event).unwrap();
        assert!(!app.help_mode);
    }

    #[test]
    fn test_undo_toggle_todo() {
        let mut todo_list = TodoList::new("test".to_string());
        todo_list.add_item(ListItem::new_todo("Task A".to_string(), false, 0, 0));
        
        let mut app = App::new(todo_list);
        app.selected_index = 0;
        
        // Initial state: not completed
        assert!(!matches!(app.todo_list.items[0], ListItem::Todo { completed: true, .. }));
        assert!(app.undo_stack.is_empty());
        
        // Toggle item (this should save state before changing)
        app.toggle_selected_item();
        
        // Now it should be completed and have an undo state
        assert!(matches!(app.todo_list.items[0], ListItem::Todo { completed: true, .. }));
        assert_eq!(app.undo_stack.len(), 1);
        
        // Undo the toggle
        app.undo();
        
        // Should be back to initial state
        assert!(!matches!(app.todo_list.items[0], ListItem::Todo { completed: true, .. }));
        assert!(app.undo_stack.is_empty());
    }

    #[test]
    fn test_undo_edit_todo() {
        let mut todo_list = TodoList::new("test".to_string());
        todo_list.add_item(ListItem::new_todo("Original".to_string(), false, 0, 0));
        
        let mut app = App::new(todo_list);
        app.selected_index = 0;
        
        // Enter edit mode and change content
        app.enter_edit_mode();
        app.edit_buffer = "Modified".to_string();
        
        // Confirm edit (this should save state before changing)
        app.confirm_edit();
        
        // Content should be changed
        match &app.todo_list.items[0] {
            ListItem::Todo { content, .. } => assert_eq!(content, "Modified"),
            _ => panic!("Expected Todo item"),
        }
        assert_eq!(app.undo_stack.len(), 1);
        
        // Undo the edit
        app.undo();
        
        // Should be back to original content
        match &app.todo_list.items[0] {
            ListItem::Todo { content, .. } => assert_eq!(content, "Original"),
            _ => panic!("Expected Todo item"),
        }
        assert!(app.undo_stack.is_empty());
    }

    #[test]
    fn test_undo_move_item() {
        let mut todo_list = TodoList::new("test".to_string());
        todo_list.add_item(ListItem::new_todo("Task A".to_string(), false, 0, 0));
        todo_list.add_item(ListItem::new_todo("Task B".to_string(), false, 0, 1));
        
        let mut app = App::new(todo_list);
        app.selected_index = 0; // Select first item
        
        // Move item down
        app.move_item_down();
        
        // Order should be: Task B, Task A
        match &app.todo_list.items[0] {
            ListItem::Todo { content, .. } => assert_eq!(content, "Task B"),
            _ => panic!("Expected Todo item"),
        }
        match &app.todo_list.items[1] {
            ListItem::Todo { content, .. } => assert_eq!(content, "Task A"),
            _ => panic!("Expected Todo item"),
        }
        assert_eq!(app.undo_stack.len(), 1);
        
        // Undo the move
        app.undo();
        
        // Should be back to original order: Task A, Task B
        match &app.todo_list.items[0] {
            ListItem::Todo { content, .. } => assert_eq!(content, "Task A"),
            _ => panic!("Expected Todo item"),
        }
        match &app.todo_list.items[1] {
            ListItem::Todo { content, .. } => assert_eq!(content, "Task B"),
            _ => panic!("Expected Todo item"),
        }
        assert_eq!(app.selected_index, 0); // Cursor should be restored too
        assert!(app.undo_stack.is_empty());
    }

    #[test]
    fn test_undo_add_todo() {
        let mut todo_list = TodoList::new("test".to_string());
        todo_list.add_item(ListItem::new_todo("Existing".to_string(), false, 0, 0));
        
        let mut app = App::new(todo_list);
        app.selected_index = 0;
        
        // Add new todo (this saves state before adding)
        app.add_new_todo();
        
        // Should have 2 items now and 1 undo state saved
        assert_eq!(app.todo_list.items.len(), 2);
        assert_eq!(app.undo_stack.len(), 1);
        
        // Confirm the edit (this doesn't save state since adding_new_todo is true)
        app.edit_buffer = "New todo".to_string();
        app.confirm_edit();
        
        // Should still have 1 undo state
        assert_eq!(app.undo_stack.len(), 1);
        
        // Undo the addition
        app.undo();
        
        // Should be back to 1 item
        assert_eq!(app.todo_list.items.len(), 1);
        match &app.todo_list.items[0] {
            ListItem::Todo { content, .. } => assert_eq!(content, "Existing"),
            _ => panic!("Expected Todo item"),
        }
        assert!(app.undo_stack.is_empty());
    }

    #[test]
    fn test_undo_bulk_move() {
        let mut todo_list = TodoList::new("test".to_string());
        todo_list.add_item(ListItem::new_todo("Task A".to_string(), false, 0, 0));
        todo_list.add_item(ListItem::new_todo("Task B".to_string(), false, 0, 1));
        todo_list.add_item(ListItem::new_todo("Task C".to_string(), false, 0, 2));
        
        let mut app = App::new(todo_list);
        
        // Select items 0 and 2
        app.selected_index = 0;
        app.toggle_item_selection(); // Select Task A
        app.selected_index = 2;
        app.toggle_item_selection(); // Select Task C
        
        // Move to position 1 and bulk move
        app.selected_index = 1;
        app.move_selected_items_to_cursor();
        
        // Order should be changed
        assert_eq!(app.todo_list.items.len(), 3);
        assert_eq!(app.undo_stack.len(), 1); // Only the bulk move saves state, not selections
        
        // Undo the bulk move
        app.undo();
        
        // Should be back to original order
        match &app.todo_list.items[0] {
            ListItem::Todo { content, .. } => assert_eq!(content, "Task A"),
            _ => panic!("Expected Todo item"),
        }
        match &app.todo_list.items[1] {
            ListItem::Todo { content, .. } => assert_eq!(content, "Task B"),
            _ => panic!("Expected Todo item"),
        }
        match &app.todo_list.items[2] {
            ListItem::Todo { content, .. } => assert_eq!(content, "Task C"),
            _ => panic!("Expected Todo item"),
        }
    }

    #[test]
    fn test_undo_stack_limit() {
        let todo_list = TodoList::new("test".to_string());
        let mut app = App::new(todo_list);
        
        // Add 25 empty states to exceed the limit of 20
        for i in 0..25 {
            app.save_state();
            // Modify something small to differentiate states
            app.selected_index = i % 10;
        }
        
        // Should be limited to 20 items
        assert_eq!(app.undo_stack.len(), 20);
    }

    #[test]
    fn test_undo_with_empty_stack() {
        let todo_list = TodoList::new("test".to_string());
        let mut app = App::new(todo_list);
        
        let original_index = app.selected_index;
        
        // Try to undo with empty stack - should do nothing
        app.undo();
        
        assert_eq!(app.selected_index, original_index);
        assert!(app.undo_stack.is_empty());
    }

    #[test]
    fn test_escape_clears_selection() {
        let mut todo_list = TodoList::new("test".to_string());
        todo_list.add_item(ListItem::new_todo("Task A".to_string(), false, 0, 0));
        todo_list.add_item(ListItem::new_todo("Task B".to_string(), false, 0, 1));
        
        let mut app = App::new(todo_list);
        
        // Select some items
        app.selected_index = 0;
        app.toggle_item_selection();
        app.selected_index = 1;
        app.toggle_item_selection();
        
        assert_eq!(app.selected_items.len(), 2);
        assert!(!app.should_quit);
        
        // Press Escape - should clear selection, not quit
        let key_event = KeyEvent::from(KeyCode::Esc);
        app.handle_key_event(key_event).unwrap();
        
        assert!(app.selected_items.is_empty());
        assert!(!app.should_quit);
    }

    #[test]
    fn test_escape_never_quits() {
        let todo_list = TodoList::new("test".to_string());
        let mut app = App::new(todo_list);
        
        assert!(app.selected_items.is_empty());
        assert!(!app.should_quit);
        
        // Press Escape with no selection - should NOT quit
        let key_event = KeyEvent::from(KeyCode::Esc);
        app.handle_key_event(key_event).unwrap();
        
        assert!(!app.should_quit);
        assert!(app.selected_items.is_empty());
    }
}