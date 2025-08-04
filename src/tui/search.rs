use crate::todo::models::ListItem;

pub struct SearchState {
    pub search_mode: bool,
    pub search_query: String,
    pub search_matches: Vec<usize>,
    pub current_match_index: Option<usize>,
}

impl SearchState {
    pub fn new() -> Self {
        Self {
            search_mode: false,
            search_query: String::new(),
            search_matches: Vec::new(),
            current_match_index: None,
        }
    }

    pub fn enter_search_mode(&mut self) {
        self.search_mode = true;
        self.search_query.clear();
        self.search_matches.clear();
        self.current_match_index = None;
    }

    pub fn cancel_search(&mut self) {
        self.search_mode = false;
        self.search_query.clear();
        self.search_matches.clear();
        self.current_match_index = None;
    }

    pub fn confirm_search(&mut self) -> Option<usize> {
        self.search_mode = false;
        if !self.search_matches.is_empty() {
            self.current_match_index = Some(0);
            Some(self.search_matches[0])
        } else {
            None
        }
    }

    pub fn insert_char(&mut self, c: char, items: &[ListItem]) {
        self.search_query.push(c);
        self.update_search_matches(items);
    }

    pub fn backspace(&mut self, items: &[ListItem]) {
        if !self.search_query.is_empty() {
            self.search_query.pop();
            self.update_search_matches(items);
        }
    }

    pub fn update_search_matches(&mut self, items: &[ListItem]) {
        self.search_matches.clear();
        self.current_match_index = None;
        
        if self.search_query.is_empty() {
            return;
        }

        let query_lower = self.search_query.to_lowercase();
        
        for (index, item) in items.iter().enumerate() {
            let content = match item {
                ListItem::Todo { content, .. } => content,
                ListItem::Note { content, .. } => content,
                ListItem::Heading { content, .. } => content,
            };
            
            if content.to_lowercase().contains(&query_lower) {
                self.search_matches.push(index);
            }
        }
    }

    pub fn next_match(&mut self) -> Option<usize> {
        if self.search_matches.is_empty() {
            return None;
        }
        
        if let Some(current_match) = self.current_match_index {
            let next_match = (current_match + 1) % self.search_matches.len();
            self.current_match_index = Some(next_match);
            Some(self.search_matches[next_match])
        } else {
            self.current_match_index = Some(0);
            Some(self.search_matches[0])
        }
    }

    pub fn previous_match(&mut self) -> Option<usize> {
        if self.search_matches.is_empty() {
            return None;
        }
        
        if let Some(current_match) = self.current_match_index {
            let prev_match = if current_match == 0 {
                self.search_matches.len() - 1
            } else {
                current_match - 1
            };
            self.current_match_index = Some(prev_match);
            Some(self.search_matches[prev_match])
        } else {
            let last_match = self.search_matches.len() - 1;
            self.current_match_index = Some(last_match);
            Some(self.search_matches[last_match])
        }
    }

    pub fn clear_results(&mut self) {
        self.search_matches.clear();
        self.current_match_index = None;
    }

}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::todo::models::ListItem;

    fn create_test_items() -> Vec<ListItem> {
        vec![
            ListItem::new_todo("Buy groceries".to_string(), false, 0),
            ListItem::new_todo("Walk the dog".to_string(), false, 0),
            ListItem::new_note("Remember to buy milk".to_string(), 0),
            ListItem::new_heading("Work Tasks".to_string(), 1),
            ListItem::new_todo("Finish project".to_string(), false, 0),
        ]
    }

    #[test]
    fn test_search_state_new() {
        let search_state = SearchState::new();
        assert!(!search_state.search_mode);
        assert!(search_state.search_query.is_empty());
        assert!(search_state.search_matches.is_empty());
        assert!(search_state.current_match_index.is_none());
    }

    #[test]
    fn test_enter_search_mode() {
        let mut search_state = SearchState::new();
        search_state.enter_search_mode();
        
        assert!(search_state.search_mode);
        assert!(search_state.search_query.is_empty());
        assert!(search_state.search_matches.is_empty());
        assert!(search_state.current_match_index.is_none());
    }

    #[test]
    fn test_cancel_search() {
        let mut search_state = SearchState::new();
        let items = create_test_items();
        
        search_state.enter_search_mode();
        search_state.insert_char('b', &items);
        search_state.cancel_search();
        
        assert!(!search_state.search_mode);
        assert!(search_state.search_query.is_empty());
        assert!(search_state.search_matches.is_empty());
        assert!(search_state.current_match_index.is_none());
    }

    #[test]
    fn test_search_matches() {
        let mut search_state = SearchState::new();
        let items = create_test_items();
        
        search_state.enter_search_mode();
        search_state.insert_char('b', &items);
        search_state.insert_char('u', &items);
        search_state.insert_char('y', &items);
        
        assert_eq!(search_state.search_query, "buy");
        assert_eq!(search_state.search_matches, vec![0, 2]); // "Buy groceries" and "Remember to buy milk"
    }

    #[test]
    fn test_next_and_previous_match() {
        let mut search_state = SearchState::new();
        let items = create_test_items();
        
        search_state.enter_search_mode();
        search_state.insert_char('t', &items); // Should match "Walk the dog" (1), "Remember to buy milk" (2), "Work Tasks" (3), "Finish project" (4)
        
        assert_eq!(search_state.search_matches, vec![1, 2, 3, 4]);
        
        // Test next match
        let first_match = search_state.next_match();
        assert_eq!(first_match, Some(1));
        assert_eq!(search_state.current_match_index, Some(0));
        
        let second_match = search_state.next_match();
        assert_eq!(second_match, Some(2));
        assert_eq!(search_state.current_match_index, Some(1));
        
        // Test wrap around
        let third_match = search_state.next_match();
        assert_eq!(third_match, Some(3));
        let fourth_match = search_state.next_match();
        assert_eq!(fourth_match, Some(4));
        let wrap_match = search_state.next_match();
        assert_eq!(wrap_match, Some(1)); // Should wrap to first
        
        // Test previous match
        let prev_match = search_state.previous_match();
        assert_eq!(prev_match, Some(4)); // Should go back
    }

    #[test]
    fn test_backspace() {
        let mut search_state = SearchState::new();
        let items = create_test_items();
        
        search_state.enter_search_mode();
        search_state.insert_char('b', &items);
        search_state.insert_char('u', &items);
        search_state.insert_char('y', &items);
        
        assert_eq!(search_state.search_matches.len(), 2);
        
        search_state.backspace(&items);
        assert_eq!(search_state.search_query, "bu");
        // Should still match the same items since "bu" is contained in both
        assert_eq!(search_state.search_matches.len(), 2);
    }

    #[test]
    fn test_confirm_search() {
        let mut search_state = SearchState::new();
        let items = create_test_items();
        
        search_state.enter_search_mode();
        search_state.insert_char('d', &items); // Should match "Walk the dog"
        
        let result = search_state.confirm_search();
        assert_eq!(result, Some(1));
        assert!(!search_state.search_mode);
        assert_eq!(search_state.current_match_index, Some(0));
    }
}