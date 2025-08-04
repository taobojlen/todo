use anyhow::Result;

pub struct EditState {
    pub edit_mode: bool,
    pub edit_buffer: String,
    pub edit_cursor_position: usize,
    pub adding_new_todo: bool,
}

impl EditState {
    pub fn new() -> Self {
        Self {
            edit_mode: false,
            edit_buffer: String::new(),
            edit_cursor_position: 0,
            adding_new_todo: false,
        }
    }

    pub fn enter_edit_mode(&mut self, content: String) {
        self.edit_buffer = content;
        self.edit_cursor_position = self.edit_buffer.len();
        self.edit_mode = true;
    }

    pub fn exit_edit_mode(&mut self) {
        self.edit_mode = false;
        self.edit_buffer.clear();
        self.edit_cursor_position = 0;
        self.adding_new_todo = false;
    }

    pub fn insert_char(&mut self, c: char) {
        self.edit_buffer.insert(self.edit_cursor_position, c);
        self.edit_cursor_position += c.len_utf8();
    }

    pub fn backspace(&mut self) {
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

    pub fn delete(&mut self) {
        if self.edit_cursor_position < self.edit_buffer.len() {
            self.edit_buffer.remove(self.edit_cursor_position);
        }
    }

    pub fn move_cursor_left(&mut self) {
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

    pub fn move_cursor_right(&mut self) {
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

    pub fn move_cursor_home(&mut self) {
        self.edit_cursor_position = 0;
    }

    pub fn move_cursor_end(&mut self) {
        self.edit_cursor_position = self.edit_buffer.len();
    }

    pub fn delete_word_backward(&mut self) {
        if self.edit_cursor_position == 0 {
            return;
        }

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

        // Find the start of the word to delete
        let mut word_start = char_index;
        let mut in_word = false;
        
        // Move backward from current position
        for i in (0..char_index).rev() {
            let ch = chars[i];
            if ch.is_whitespace() {
                if in_word {
                    // Found whitespace after word chars, stop here
                    word_start = i + 1;
                    break;
                }
                // Still in whitespace, continue
            } else {
                // Found a word character
                in_word = true;
                word_start = i;
            }
        }

        // Calculate byte positions for deletion
        let mut delete_start_byte = 0;
        for i in 0..word_start {
            delete_start_byte += chars[i].len_utf8();
        }

        // Delete the range
        let delete_len = self.edit_cursor_position - delete_start_byte;
        if delete_len > 0 {
            for _ in 0..delete_len {
                if delete_start_byte < self.edit_buffer.len() {
                    self.edit_buffer.remove(delete_start_byte);
                }
            }
            self.edit_cursor_position = delete_start_byte;
        }
    }

    pub fn move_to_previous_word(&mut self) {
        if self.edit_cursor_position == 0 {
            return;
        }

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

        // Find the start of the previous word
        let mut target_pos = 0;
        let mut found_word = false;
        
        // Move backward from current position
        for i in (0..char_index).rev() {
            let ch = chars[i];
            if ch.is_whitespace() {
                if found_word {
                    // Found whitespace after word chars, stop at next position
                    target_pos = i + 1;
                    break;
                }
                // Still in whitespace, continue
            } else {
                // Found a word character
                found_word = true;
                target_pos = i;
            }
        }

        // Calculate byte position for target
        let mut target_byte_pos = 0;
        for i in 0..target_pos {
            target_byte_pos += chars[i].len_utf8();
        }
        
        self.edit_cursor_position = target_byte_pos;
    }

    pub fn move_to_next_word(&mut self) {
        if self.edit_cursor_position >= self.edit_buffer.len() {
            return;
        }

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

        // First skip any non-whitespace characters (current word)
        let mut i = char_index;
        while i < chars.len() && !chars[i].is_whitespace() {
            i += 1;
        }
        
        // Then skip any whitespace
        while i < chars.len() && chars[i].is_whitespace() {
            i += 1;
        }

        // Calculate byte position for target
        let mut target_byte_pos = 0;
        for j in 0..i {
            if j < chars.len() {
                target_byte_pos += chars[j].len_utf8();
            }
        }
        
        self.edit_cursor_position = target_byte_pos;
    }
}

pub trait Editable {
    fn enter_edit_mode_for_item(&mut self, item_index: usize);
    fn cancel_edit(&mut self) -> Result<()>;
    fn confirm_edit(&mut self) -> Result<()>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_edit_state_new() {
        let edit_state = EditState::new();
        assert!(!edit_state.edit_mode);
        assert!(edit_state.edit_buffer.is_empty());
        assert_eq!(edit_state.edit_cursor_position, 0);
        assert!(!edit_state.adding_new_todo);
    }

    #[test]
    fn test_enter_edit_mode() {
        let mut edit_state = EditState::new();
        edit_state.enter_edit_mode("Hello World".to_string());
        
        assert!(edit_state.edit_mode);
        assert_eq!(edit_state.edit_buffer, "Hello World");
        assert_eq!(edit_state.edit_cursor_position, "Hello World".len());
    }

    #[test]
    fn test_exit_edit_mode() {
        let mut edit_state = EditState::new();
        edit_state.enter_edit_mode("Hello World".to_string());
        edit_state.exit_edit_mode();
        
        assert!(!edit_state.edit_mode);
        assert!(edit_state.edit_buffer.is_empty());
        assert_eq!(edit_state.edit_cursor_position, 0);
        assert!(!edit_state.adding_new_todo);
    }

    #[test]
    fn test_insert_char() {
        let mut edit_state = EditState::new();
        edit_state.enter_edit_mode("Hello".to_string());
        edit_state.edit_cursor_position = 5; // At end
        edit_state.insert_char('!');
        
        assert_eq!(edit_state.edit_buffer, "Hello!");
        assert_eq!(edit_state.edit_cursor_position, 6);
    }

    #[test]
    fn test_backspace() {
        let mut edit_state = EditState::new();
        edit_state.enter_edit_mode("Hello".to_string());
        edit_state.backspace();
        
        assert_eq!(edit_state.edit_buffer, "Hell");
        assert_eq!(edit_state.edit_cursor_position, 4);
    }

    #[test]
    fn test_delete() {
        let mut edit_state = EditState::new();
        edit_state.enter_edit_mode("Hello".to_string());
        edit_state.edit_cursor_position = 0; // At start
        edit_state.delete();
        
        assert_eq!(edit_state.edit_buffer, "ello");
        assert_eq!(edit_state.edit_cursor_position, 0);
    }

    #[test]
    fn test_cursor_movement() {
        let mut edit_state = EditState::new();
        edit_state.enter_edit_mode("Hello".to_string());
        
        // Test moving left
        edit_state.move_cursor_left();
        assert_eq!(edit_state.edit_cursor_position, 4);
        
        // Test moving right
        edit_state.move_cursor_right();
        assert_eq!(edit_state.edit_cursor_position, 5);
        
        // Test home
        edit_state.move_cursor_home();
        assert_eq!(edit_state.edit_cursor_position, 0);
        
        // Test end
        edit_state.move_cursor_end();
        assert_eq!(edit_state.edit_cursor_position, 5);
    }

    #[test]
    fn test_delete_word_backward() {
        let mut edit_state = EditState::new();
        
        // Test deleting a word at the end of text
        edit_state.enter_edit_mode("Hello world".to_string());
        edit_state.delete_word_backward();
        assert_eq!(edit_state.edit_buffer, "Hello ");
        assert_eq!(edit_state.edit_cursor_position, 6);
        
        // Test deleting another word
        edit_state.delete_word_backward();
        assert_eq!(edit_state.edit_buffer, "");
        assert_eq!(edit_state.edit_cursor_position, 0);
        
        // Test with cursor in middle of text
        edit_state.enter_edit_mode("foo bar baz".to_string());
        edit_state.edit_cursor_position = 7; // Between "bar" and " baz"
        edit_state.delete_word_backward();
        assert_eq!(edit_state.edit_buffer, "foo  baz");
        assert_eq!(edit_state.edit_cursor_position, 4);
        
        // Test with multiple spaces
        edit_state.enter_edit_mode("word   test".to_string());
        edit_state.delete_word_backward();
        assert_eq!(edit_state.edit_buffer, "word   ");
        assert_eq!(edit_state.edit_cursor_position, 7);
        
        // Test at beginning of buffer
        edit_state.enter_edit_mode("test".to_string());
        edit_state.edit_cursor_position = 0;
        edit_state.delete_word_backward();
        assert_eq!(edit_state.edit_buffer, "test");
        assert_eq!(edit_state.edit_cursor_position, 0);
    }

    #[test]
    fn test_move_to_previous_word() {
        let mut edit_state = EditState::new();
        
        // Test moving to previous word from end
        edit_state.enter_edit_mode("hello world test".to_string());
        edit_state.move_to_previous_word();
        assert_eq!(edit_state.edit_cursor_position, 12); // Start of "test"
        
        // Test moving to another previous word
        edit_state.move_to_previous_word();
        assert_eq!(edit_state.edit_cursor_position, 6); // Start of "world"
        
        // Test moving to the first word
        edit_state.move_to_previous_word();
        assert_eq!(edit_state.edit_cursor_position, 0); // Start of "hello"
        
        // Test at beginning - should stay at beginning
        edit_state.move_to_previous_word();
        assert_eq!(edit_state.edit_cursor_position, 0);
        
        // Test with cursor in middle of a word
        edit_state.enter_edit_mode("foo bar baz".to_string());
        edit_state.edit_cursor_position = 6; // Middle of "bar"
        edit_state.move_to_previous_word();
        assert_eq!(edit_state.edit_cursor_position, 4); // Start of "bar"
        
        // Test with multiple spaces
        edit_state.enter_edit_mode("word   test".to_string());
        edit_state.move_to_previous_word();
        assert_eq!(edit_state.edit_cursor_position, 7); // Start of "test"
    }

    #[test]
    fn test_move_to_next_word() {
        let mut edit_state = EditState::new();
        
        // Test moving to next word from beginning
        edit_state.enter_edit_mode("hello world test".to_string());
        edit_state.edit_cursor_position = 0; // Start of "hello"
        edit_state.move_to_next_word();
        assert_eq!(edit_state.edit_cursor_position, 6); // Start of "world"
        
        // Test moving to another next word
        edit_state.move_to_next_word();
        assert_eq!(edit_state.edit_cursor_position, 12); // Start of "test"
        
        // Test at end - should stay at end
        edit_state.move_to_next_word();
        assert_eq!(edit_state.edit_cursor_position, 16); // End of buffer
        
        edit_state.move_to_next_word();
        assert_eq!(edit_state.edit_cursor_position, 16); // Still at end
        
        // Test with cursor in middle of a word
        edit_state.enter_edit_mode("foo bar baz".to_string());
        edit_state.edit_cursor_position = 1; // Middle of "foo"
        edit_state.move_to_next_word();
        assert_eq!(edit_state.edit_cursor_position, 4); // Start of "bar"
        
        // Test with multiple spaces
        edit_state.enter_edit_mode("word   test".to_string());
        edit_state.edit_cursor_position = 0;
        edit_state.move_to_next_word();
        assert_eq!(edit_state.edit_cursor_position, 7); // Start of "test"
    }
}