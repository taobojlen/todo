use crate::todo::models::{TodoList, ListItem};
use crate::tui::{
    actions::{ItemActions, ActionPerformer},
    edit::{EditState, Editable},
    handlers::{KeyHandler, KeyEventHandler, NormalModeAction, HelpModeAction, SearchModeAction, EditModeAction},
    navigation::{NavigationState, ItemCreator, Navigable},
    persistence::Persistence,
    search::{SearchState, Searchable},
    state::AppState,
    undo::{UndoManager, UndoableApp},
};
use anyhow::Result;
use crossterm::event::KeyEvent;

pub struct App {
    pub todo_list: TodoList,
    pub should_quit: bool,
    pub help_mode: bool,
    
    // Component states
    navigation: NavigationState,
    edit_state: EditState,
    search_state: SearchState,
    undo_manager: UndoManager,
}

impl App {
    pub fn new(todo_list: TodoList) -> Self {
        Self {
            todo_list,
            should_quit: false,
            help_mode: false,
            navigation: NavigationState::new(),
            edit_state: EditState::new(),
            search_state: SearchState::new(),
            undo_manager: UndoManager::new(),
        }
    }

    pub fn total_items(&self) -> usize {
        self.todo_list.total_items()
    }

    pub fn completed_items(&self) -> usize {
        self.todo_list.completed_items()
    }

    // Delegate to navigation state
    pub fn selected_index(&self) -> usize {
        self.navigation.selected_index
    }

    pub fn scroll_offset(&self) -> usize {
        self.navigation.scroll_offset
    }

    pub fn selected_items(&self) -> &std::collections::HashSet<usize> {
        &self.navigation.selected_items
    }

    // Delegate to edit state
    pub fn edit_mode(&self) -> bool {
        self.edit_state.edit_mode
    }

    pub fn edit_buffer(&self) -> &str {
        &self.edit_state.edit_buffer
    }

    pub fn edit_cursor_position(&self) -> usize {
        self.edit_state.edit_cursor_position
    }

    // Delegate to search state
    pub fn search_mode(&self) -> bool {
        self.search_state.search_mode
    }

    pub fn search_query(&self) -> &str {
        &self.search_state.search_query
    }

    pub fn search_matches(&self) -> &[usize] {
        &self.search_state.search_matches
    }

    pub fn current_match_index(&self) -> Option<usize> {
        self.search_state.current_match_index
    }

    // Handle escape key context
    fn handle_escape(&mut self) {
        if !self.search_state.search_matches.is_empty() {
            // Clear search results if they exist
            self.search_state.clear_results();
        } else {
            // Otherwise clear bulk selection
            self.navigation.clear_selection();
        }
    }

    // Handle 'n' key (context dependent)
    fn handle_n(&mut self) -> Result<()> {
        if !self.search_state.search_matches.is_empty() && self.search_state.current_match_index.is_some() {
            if let Some(index) = self.search_state.next_match() {
                self.navigation.selected_index = index;
                self.navigation.update_scroll();
            }
        } else {
            self.add_new_note()?;
        }
        Ok(())
    }

    // Handle 'N' key (context dependent)
    fn handle_shift_n(&mut self) -> Result<()> {
        if !self.search_state.search_matches.is_empty() && self.search_state.current_match_index.is_some() {
            if let Some(index) = self.search_state.previous_match() {
                self.navigation.selected_index = index;
                self.navigation.update_scroll();
            }
        } else {
            self.add_new_note_at_top()?;
        }
        Ok(())
    }

    fn add_new_note(&mut self) -> Result<()> {
        self.save_current_state();
        self.edit_state.adding_new_todo = true;
        
        if self.todo_list.items.is_empty() {
            let new_note = ItemCreator::create_new_note(String::new(), 0);
            self.todo_list.add_item(new_note);
            self.navigation.selected_index = 0;
            self.enter_edit_mode_for_item(0);
        } else if self.navigation.selected_index < self.todo_list.items.len() {
            let (position, indent) = ItemCreator::determine_insert_position_for_new_todo(&self.todo_list.items, self.navigation.selected_index);
            let new_note = ItemCreator::create_new_note(String::new(), indent);
            self.todo_list.items.insert(position, new_note);
            self.navigation.selected_index = position;
            self.enter_edit_mode_for_item(position);
        }
        Ok(())
    }

    fn add_new_note_at_top(&mut self) -> Result<()> {
        self.save_current_state();
        self.edit_state.adding_new_todo = true;
        
        let new_note = ItemCreator::create_new_note(String::new(), 0);
        let insert_position = ItemCreator::determine_insert_position_for_new_todo_at_top(&self.todo_list.items, self.navigation.selected_index);
        
        self.todo_list.items.insert(insert_position, new_note);
        self.navigation.selected_index = insert_position;
        self.enter_edit_mode_for_item(insert_position);
        Ok(())
    }

    fn add_new_todo(&mut self) -> Result<()> {
        self.save_current_state();
        self.edit_state.adding_new_todo = true;
        
        if self.todo_list.items.is_empty() {
            let new_todo = ItemCreator::create_new_todo(String::new(), false, 0);
            self.todo_list.add_item(new_todo);
            self.navigation.selected_index = 0;
            self.enter_edit_mode_for_item(0);
        } else if self.navigation.selected_index < self.todo_list.items.len() {
            let (position, indent) = ItemCreator::determine_insert_position_for_new_todo(&self.todo_list.items, self.navigation.selected_index);
            let new_todo = ItemCreator::create_new_todo(String::new(), false, indent);
            self.todo_list.items.insert(position, new_todo);
            self.navigation.selected_index = position;
            self.enter_edit_mode_for_item(position);
        }
        Ok(())
    }

    fn add_new_todo_at_top(&mut self) -> Result<()> {
        self.save_current_state();
        self.edit_state.adding_new_todo = true;
        
        let new_todo = ItemCreator::create_new_todo(String::new(), false, 0);
        let insert_position = ItemCreator::determine_insert_position_for_new_todo_at_top(&self.todo_list.items, self.navigation.selected_index);
        
        self.todo_list.items.insert(insert_position, new_todo);
        self.navigation.selected_index = insert_position;
        self.enter_edit_mode_for_item(insert_position);
        Ok(())
    }
}

// Implement all the traits
impl KeyEventHandler for App {
    fn handle_key_event(&mut self, key_event: KeyEvent) -> Result<()> {
        if self.help_mode {
            match KeyHandler::handle_help_mode_key(key_event) {
                HelpModeAction::ExitHelpMode => self.help_mode = false,
                HelpModeAction::None => {}
            }
        } else if self.edit_state.edit_mode {
            match KeyHandler::handle_edit_mode_key(key_event) {
                EditModeAction::CancelEdit => self.cancel_edit()?,
                EditModeAction::ConfirmEdit => self.confirm_edit()?,
                EditModeAction::Backspace => self.edit_state.backspace(),
                EditModeAction::Delete => self.edit_state.delete(),
                EditModeAction::MoveCursorLeft => self.edit_state.move_cursor_left(),
                EditModeAction::MoveCursorRight => self.edit_state.move_cursor_right(),
                EditModeAction::MoveCursorHome => self.edit_state.move_cursor_home(),
                EditModeAction::MoveCursorEnd => self.edit_state.move_cursor_end(),
                EditModeAction::InsertChar(c) => self.edit_state.insert_char(c),
                EditModeAction::None => {}
            }
        } else if self.search_state.search_mode {
            match KeyHandler::handle_search_mode_key(key_event) {
                SearchModeAction::CancelSearch => self.search_state.cancel_search(),
                SearchModeAction::ConfirmSearch => {
                    if let Some(index) = self.search_state.confirm_search() {
                        self.navigation.selected_index = index;
                        self.navigation.update_scroll();
                    }
                }
                SearchModeAction::Backspace => self.search_state.backspace(&self.todo_list.items),
                SearchModeAction::InsertChar(c) => self.search_state.insert_char(c, &self.todo_list.items),
                SearchModeAction::None => {}
            }
        } else {
            match KeyHandler::handle_normal_mode_key(key_event) {
                NormalModeAction::Quit => self.should_quit = true,
                NormalModeAction::HandleEscape => self.handle_escape(),
                NormalModeAction::MoveSelectionUp => self.navigation.move_selection_up(),
                NormalModeAction::MoveSelectionDown => self.navigation.move_selection_down(self.todo_list.items.len()),
                NormalModeAction::MoveItemUp => {
                    if let Some(new_index) = self.perform_move_item_up(self.navigation.selected_index) {
                        self.navigation.selected_index = new_index;
                        self.navigation.update_scroll();
                    }
                }
                NormalModeAction::MoveItemDown => {
                    if let Some(new_index) = self.perform_move_item_down(self.navigation.selected_index) {
                        self.navigation.selected_index = new_index;
                        self.navigation.update_scroll();
                    }
                }
                NormalModeAction::IndentItem => {
                    self.perform_indent_item(self.navigation.selected_index);
                }
                NormalModeAction::UnindentItem => {
                    self.perform_unindent_item(self.navigation.selected_index);
                }
                NormalModeAction::ToggleSelectedItem => {
                    self.perform_toggle_completion(self.navigation.selected_index);
                }
                NormalModeAction::EnterEditMode => self.enter_edit_mode_for_item(self.navigation.selected_index),
                NormalModeAction::AddNewTodo => self.add_new_todo()?,
                NormalModeAction::AddNewTodoAtTop => self.add_new_todo_at_top()?,
                NormalModeAction::HandleN => self.handle_n()?,
                NormalModeAction::HandleShiftN => self.handle_shift_n()?,
                NormalModeAction::ToggleItemSelection => self.navigation.toggle_item_selection(self.todo_list.items.len()),
                NormalModeAction::MoveSelectedItemsToCursor => {
                    if let Some(new_index) = self.perform_bulk_move(&self.navigation.selected_items.clone(), self.navigation.selected_index) {
                        self.navigation.selected_index = new_index;
                        self.navigation.clear_selection();
                        self.navigation.update_scroll();
                    }
                }
                NormalModeAction::ToggleHelpMode => self.help_mode = true,
                NormalModeAction::Undo => self.perform_undo()?,
                NormalModeAction::EnterSearchMode => self.search_state.enter_search_mode(),
                NormalModeAction::None => {}
            }
        }
        Ok(())
    }
}

impl ActionPerformer for App {
    fn perform_toggle_completion(&mut self, index: usize) -> bool {
        if matches!(self.todo_list.items.get(index), Some(ListItem::Todo { .. })) {
            self.save_current_state();
            let result = ItemActions::toggle_todo_completion(&mut self.todo_list.items, index);
            
            if result {
                // Clear search results when items are modified
                self.search_state.clear_results();
                
                // Save changes to file
                if let Err(e) = self.todo_list.save_to_file() {
                    eprintln!("Failed to save file: {}", e);
                }
            }
            result
        } else {
            false
        }
    }

    fn perform_move_item_up(&mut self, index: usize) -> Option<usize> {
        self.save_current_state();
        let result = ItemActions::move_single_item_up(&mut self.todo_list.items, index);
        
        if result.is_some() {
            // Save changes to file
            if let Err(e) = self.todo_list.save_to_file() {
                eprintln!("Failed to save file: {}", e);
            }
        }
        result
    }

    fn perform_move_item_down(&mut self, index: usize) -> Option<usize> {
        self.save_current_state();
        let result = ItemActions::move_single_item_down(&mut self.todo_list.items, index);
        
        if result.is_some() {
            // Save changes to file
            if let Err(e) = self.todo_list.save_to_file() {
                eprintln!("Failed to save file: {}", e);
            }
        }
        result
    }

    fn perform_indent_item(&mut self, index: usize) -> bool {
        self.save_current_state();
        let result = ItemActions::indent_block(&mut self.todo_list.items, index);
        
        if result {
            // Save changes to file
            if let Err(e) = self.todo_list.save_to_file() {
                eprintln!("Failed to save file: {}", e);
            }
        }
        result
    }

    fn perform_unindent_item(&mut self, index: usize) -> bool {
        self.save_current_state();
        let result = ItemActions::unindent_block(&mut self.todo_list.items, index);
        
        if result {
            // Save changes to file
            if let Err(e) = self.todo_list.save_to_file() {
                eprintln!("Failed to save file: {}", e);
            }
        }
        result
    }

    fn perform_bulk_move(&mut self, selected_indices: &std::collections::HashSet<usize>, target_index: usize) -> Option<usize> {
        if selected_indices.is_empty() {
            return None;
        }

        self.save_current_state();
        let result = ItemActions::move_selected_items_to_position(&mut self.todo_list.items, selected_indices, target_index);
        
        if result.is_some() {
            // Save changes to file
            if let Err(e) = self.todo_list.save_to_file() {
                eprintln!("Failed to save file: {}", e);
            }
        }
        result
    }
}

impl Editable for App {
    fn enter_edit_mode_for_item(&mut self, item_index: usize) {
        if item_index < self.todo_list.items.len() {
            if let Some(item) = self.todo_list.items.get(item_index) {
                let content = match item {
                    ListItem::Todo { content, .. } => content.clone(),
                    ListItem::Note { content, .. } => content.clone(),
                    ListItem::Heading { content, .. } => content.clone(),
                };
                self.edit_state.enter_edit_mode(content);
            }
        }
    }

    fn cancel_edit(&mut self) -> Result<()> {
        // If we're canceling edit on an empty todo, remove it
        if self.navigation.selected_index < self.todo_list.items.len() {
            if let Some(ListItem::Todo { content, .. }) = self.todo_list.items.get(self.navigation.selected_index) {
                if content.trim().is_empty() {
                    self.todo_list.items.remove(self.navigation.selected_index);
                    // Adjust selection to stay within bounds
                    if self.navigation.selected_index >= self.todo_list.items.len() && !self.todo_list.items.is_empty() {
                        self.navigation.selected_index = self.todo_list.items.len() - 1;
                    }
                }
            }
        }
        
        self.edit_state.exit_edit_mode();
        
        // Save changes to file (in case we removed an empty todo)
        self.todo_list.save_to_file()
    }

    fn confirm_edit(&mut self) -> Result<()> {
        if self.navigation.selected_index < self.todo_list.items.len() {
            // Only save state if we're not confirming a newly added todo
            if !self.edit_state.adding_new_todo {
                self.save_current_state();
            }

            let should_remove = if let Some(item) = self.todo_list.items.get_mut(self.navigation.selected_index) {
                match item {
                    ListItem::Todo { content, .. } => {
                        *content = self.edit_state.edit_buffer.clone();
                        // Remove todo if it's empty after editing
                        self.edit_state.edit_buffer.trim().is_empty()
                    }
                    ListItem::Note { content, .. } => {
                        *content = self.edit_state.edit_buffer.clone();
                        // Remove note if it's empty after editing
                        self.edit_state.edit_buffer.trim().is_empty()
                    }
                    ListItem::Heading { content, .. } => {
                        *content = self.edit_state.edit_buffer.clone();
                        // Don't remove headings even if empty
                        false
                    }
                }
            } else {
                false
            };

            // Remove the item if it's an empty todo or note
            if should_remove {
                self.todo_list.items.remove(self.navigation.selected_index);
                // Adjust selection to stay within bounds
                if self.navigation.selected_index >= self.todo_list.items.len() && !self.todo_list.items.is_empty() {
                    self.navigation.selected_index = self.todo_list.items.len() - 1;
                }
            }
        }
        
        self.edit_state.exit_edit_mode();
        
        // Clear search results when items are modified
        self.search_state.clear_results();
        
        // Save changes to file
        self.todo_list.save_to_file()
    }
}

impl UndoableApp for App {
    fn save_current_state(&mut self) {
        let state = AppState::new(
            self.todo_list.clone(),
            self.navigation.selected_index,
            self.navigation.selected_items.clone(),
        );
        self.undo_manager.save_state(state);
    }

    fn restore_state(&mut self, state: AppState) -> Result<()> {
        self.todo_list = state.todo_list;
        self.navigation.selected_index = state.selected_index;
        self.navigation.selected_items = state.selected_items;
        self.navigation.update_scroll();
        Ok(())
    }

    fn perform_undo(&mut self) -> Result<()> {
        if let Some(state) = self.undo_manager.undo() {
            self.restore_state(state)?;
            
            // Save changes to file
            self.todo_list.save_to_file()
        } else {
            Ok(())
        }
    }
}

impl Navigable for App {
    fn get_navigation_state(&self) -> &NavigationState {
        &self.navigation
    }

    fn get_navigation_state_mut(&mut self) -> &mut NavigationState {
        &mut self.navigation
    }

    fn get_item_count(&self) -> usize {
        self.todo_list.items.len()
    }
}

impl Searchable for App {
    fn get_search_state(&self) -> &SearchState {
        &self.search_state
    }

    fn get_search_state_mut(&mut self) -> &mut SearchState {
        &mut self.search_state
    }

    fn get_items(&self) -> &[ListItem] {
        &self.todo_list.items
    }
}