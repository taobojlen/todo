use crate::tui::state::AppState;
use anyhow::Result;

pub struct UndoManager {
    pub undo_stack: Vec<AppState>,
}

impl UndoManager {
    pub fn new() -> Self {
        Self {
            undo_stack: Vec::new(),
        }
    }

    pub fn save_state(&mut self, state: AppState) {
        self.undo_stack.push(state);
        
        // Limit undo stack to 20 items
        if self.undo_stack.len() > 20 {
            self.undo_stack.remove(0);
        }
    }

    pub fn undo(&mut self) -> Option<AppState> {
        self.undo_stack.pop()
    }

}

pub trait UndoableApp {
    fn save_current_state(&mut self);
    fn restore_state(&mut self, state: AppState) -> Result<()>;
    fn perform_undo(&mut self) -> Result<()>;
}