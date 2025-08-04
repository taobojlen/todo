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
}