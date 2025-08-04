use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use anyhow::Result;

pub struct KeyHandler;

impl KeyHandler {
    pub fn handle_normal_mode_key(key_event: KeyEvent) -> NormalModeAction {
        match key_event.code {
            KeyCode::Char('q') => NormalModeAction::Quit,
            KeyCode::Esc => NormalModeAction::HandleEscape,
            KeyCode::Char('c') if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
                NormalModeAction::Quit
            }
            KeyCode::Up | KeyCode::Char('k') | KeyCode::Char('K') => {
                if key_event.modifiers.contains(KeyModifiers::SHIFT) {
                    NormalModeAction::MoveItemUp
                } else {
                    NormalModeAction::MoveSelectionUp
                }
            }
            KeyCode::Down | KeyCode::Char('j') | KeyCode::Char('J') => {
                if key_event.modifiers.contains(KeyModifiers::SHIFT) {
                    NormalModeAction::MoveItemDown
                } else {
                    NormalModeAction::MoveSelectionDown
                }
            }
            KeyCode::Left | KeyCode::Char('h') | KeyCode::Char('H') => {
                if key_event.modifiers.contains(KeyModifiers::SHIFT) {
                    NormalModeAction::UnindentItem
                } else {
                    NormalModeAction::None
                }
            }
            KeyCode::Right | KeyCode::Char('l') | KeyCode::Char('L') => {
                if key_event.modifiers.contains(KeyModifiers::SHIFT) {
                    NormalModeAction::IndentItem
                } else {
                    NormalModeAction::None
                }
            }
            KeyCode::Enter => NormalModeAction::ToggleSelectedItem,
            KeyCode::Char('e') => NormalModeAction::EnterEditMode,
            KeyCode::Char('a') => NormalModeAction::AddNewTodo,
            KeyCode::Char('A') => NormalModeAction::AddNewTodoAtTop,
            KeyCode::Char('n') => NormalModeAction::HandleN,
            KeyCode::Char('N') => NormalModeAction::HandleShiftN,
            KeyCode::Char(' ') => NormalModeAction::ToggleItemSelection,
            KeyCode::Char('m') => NormalModeAction::MoveSelectedItemsToCursor,
            KeyCode::Char('?') => NormalModeAction::ToggleHelpMode,
            KeyCode::Char('u') => NormalModeAction::Undo,
            KeyCode::Char('/') => NormalModeAction::EnterSearchMode,
            KeyCode::Char('d') => NormalModeAction::DeleteItem,
            _ => NormalModeAction::None,
        }
    }

    pub fn handle_help_mode_key(key_event: KeyEvent) -> HelpModeAction {
        match key_event.code {
            KeyCode::Char('q') | KeyCode::Esc | KeyCode::Char('?') => {
                HelpModeAction::ExitHelpMode
            }
            _ => HelpModeAction::None,
        }
    }

    pub fn handle_search_mode_key(key_event: KeyEvent) -> SearchModeAction {
        match key_event.code {
            KeyCode::Esc => SearchModeAction::CancelSearch,
            KeyCode::Enter => SearchModeAction::ConfirmSearch,
            KeyCode::Backspace => SearchModeAction::Backspace,
            KeyCode::Char(c) => SearchModeAction::InsertChar(c),
            _ => SearchModeAction::None,
        }
    }

    pub fn handle_edit_mode_key(key_event: KeyEvent) -> EditModeAction {
        match key_event.code {
            KeyCode::Esc => EditModeAction::CancelEdit,
            KeyCode::Enter => EditModeAction::ConfirmEdit,
            KeyCode::Backspace => EditModeAction::Backspace,
            KeyCode::Delete => EditModeAction::Delete,
            KeyCode::Left => EditModeAction::MoveCursorLeft,
            KeyCode::Right => EditModeAction::MoveCursorRight,
            KeyCode::Home => EditModeAction::MoveCursorHome,
            KeyCode::End => EditModeAction::MoveCursorEnd,
            KeyCode::Char(c) => EditModeAction::InsertChar(c),
            _ => EditModeAction::None,
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum NormalModeAction {
    None,
    Quit,
    HandleEscape,
    MoveSelectionUp,
    MoveSelectionDown,
    MoveItemUp,
    MoveItemDown,
    IndentItem,
    UnindentItem,
    ToggleSelectedItem,
    EnterEditMode,
    AddNewTodo,
    AddNewTodoAtTop,
    HandleN, // Context-dependent: next match or add note
    HandleShiftN, // Context-dependent: previous match or add note at top
    ToggleItemSelection,
    MoveSelectedItemsToCursor,
    ToggleHelpMode,
    Undo,
    EnterSearchMode,
    DeleteItem,
}

#[derive(Debug, PartialEq)]
pub enum HelpModeAction {
    None,
    ExitHelpMode,
}

#[derive(Debug, PartialEq)]
pub enum SearchModeAction {
    None,
    CancelSearch,
    ConfirmSearch,
    Backspace,
    InsertChar(char),
}

#[derive(Debug, PartialEq)]
pub enum EditModeAction {
    None,
    CancelEdit,
    ConfirmEdit,
    Backspace,
    Delete,
    MoveCursorLeft,
    MoveCursorRight,
    MoveCursorHome,
    MoveCursorEnd,
    InsertChar(char),
}

pub trait KeyEventHandler {
    fn handle_key_event(&mut self, key_event: KeyEvent) -> Result<()>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normal_mode_basic_keys() {
        let key_event = KeyEvent::from(KeyCode::Char('q'));
        assert_eq!(KeyHandler::handle_normal_mode_key(key_event), NormalModeAction::Quit);

        let key_event = KeyEvent::from(KeyCode::Esc);
        assert_eq!(KeyHandler::handle_normal_mode_key(key_event), NormalModeAction::HandleEscape);

        let key_event = KeyEvent::from(KeyCode::Enter);
        assert_eq!(KeyHandler::handle_normal_mode_key(key_event), NormalModeAction::ToggleSelectedItem);

        let key_event = KeyEvent::from(KeyCode::Char('e'));
        assert_eq!(KeyHandler::handle_normal_mode_key(key_event), NormalModeAction::EnterEditMode);
    }

    #[test]
    fn test_normal_mode_navigation_keys() {
        let key_event = KeyEvent::from(KeyCode::Up);
        assert_eq!(KeyHandler::handle_normal_mode_key(key_event), NormalModeAction::MoveSelectionUp);

        let key_event = KeyEvent::from(KeyCode::Char('j'));
        assert_eq!(KeyHandler::handle_normal_mode_key(key_event), NormalModeAction::MoveSelectionDown);

        let key_event = KeyEvent::from(KeyCode::Char('k'));
        assert_eq!(KeyHandler::handle_normal_mode_key(key_event), NormalModeAction::MoveSelectionUp);
    }

    #[test]
    fn test_normal_mode_shift_keys() {
        let mut key_event = KeyEvent::from(KeyCode::Up);
        key_event.modifiers = KeyModifiers::SHIFT;
        assert_eq!(KeyHandler::handle_normal_mode_key(key_event), NormalModeAction::MoveItemUp);

        let mut key_event = KeyEvent::from(KeyCode::Down);
        key_event.modifiers = KeyModifiers::SHIFT;
        assert_eq!(KeyHandler::handle_normal_mode_key(key_event), NormalModeAction::MoveItemDown);

        let mut key_event = KeyEvent::from(KeyCode::Left);
        key_event.modifiers = KeyModifiers::SHIFT;
        assert_eq!(KeyHandler::handle_normal_mode_key(key_event), NormalModeAction::UnindentItem);

        let mut key_event = KeyEvent::from(KeyCode::Right);
        key_event.modifiers = KeyModifiers::SHIFT;
        assert_eq!(KeyHandler::handle_normal_mode_key(key_event), NormalModeAction::IndentItem);
    }

    #[test]
    fn test_normal_mode_ctrl_keys() {
        let mut key_event = KeyEvent::from(KeyCode::Char('c'));
        key_event.modifiers = KeyModifiers::CONTROL;
        assert_eq!(KeyHandler::handle_normal_mode_key(key_event), NormalModeAction::Quit);
    }

    #[test]
    fn test_help_mode_keys() {
        let key_event = KeyEvent::from(KeyCode::Esc);
        assert_eq!(KeyHandler::handle_help_mode_key(key_event), HelpModeAction::ExitHelpMode);

        let key_event = KeyEvent::from(KeyCode::Char('?'));
        assert_eq!(KeyHandler::handle_help_mode_key(key_event), HelpModeAction::ExitHelpMode);

        let key_event = KeyEvent::from(KeyCode::Char('q'));
        assert_eq!(KeyHandler::handle_help_mode_key(key_event), HelpModeAction::ExitHelpMode);

        let key_event = KeyEvent::from(KeyCode::Char('x'));
        assert_eq!(KeyHandler::handle_help_mode_key(key_event), HelpModeAction::None);
    }

    #[test]
    fn test_search_mode_keys() {
        let key_event = KeyEvent::from(KeyCode::Esc);
        assert_eq!(KeyHandler::handle_search_mode_key(key_event), SearchModeAction::CancelSearch);

        let key_event = KeyEvent::from(KeyCode::Enter);
        assert_eq!(KeyHandler::handle_search_mode_key(key_event), SearchModeAction::ConfirmSearch);

        let key_event = KeyEvent::from(KeyCode::Backspace);
        assert_eq!(KeyHandler::handle_search_mode_key(key_event), SearchModeAction::Backspace);

        let key_event = KeyEvent::from(KeyCode::Char('a'));
        assert_eq!(KeyHandler::handle_search_mode_key(key_event), SearchModeAction::InsertChar('a'));
    }

    #[test]
    fn test_edit_mode_keys() {
        let key_event = KeyEvent::from(KeyCode::Esc);
        assert_eq!(KeyHandler::handle_edit_mode_key(key_event), EditModeAction::CancelEdit);

        let key_event = KeyEvent::from(KeyCode::Enter);
        assert_eq!(KeyHandler::handle_edit_mode_key(key_event), EditModeAction::ConfirmEdit);

        let key_event = KeyEvent::from(KeyCode::Backspace);
        assert_eq!(KeyHandler::handle_edit_mode_key(key_event), EditModeAction::Backspace);

        let key_event = KeyEvent::from(KeyCode::Delete);
        assert_eq!(KeyHandler::handle_edit_mode_key(key_event), EditModeAction::Delete);

        let key_event = KeyEvent::from(KeyCode::Left);
        assert_eq!(KeyHandler::handle_edit_mode_key(key_event), EditModeAction::MoveCursorLeft);

        let key_event = KeyEvent::from(KeyCode::Right);
        assert_eq!(KeyHandler::handle_edit_mode_key(key_event), EditModeAction::MoveCursorRight);

        let key_event = KeyEvent::from(KeyCode::Home);
        assert_eq!(KeyHandler::handle_edit_mode_key(key_event), EditModeAction::MoveCursorHome);

        let key_event = KeyEvent::from(KeyCode::End);
        assert_eq!(KeyHandler::handle_edit_mode_key(key_event), EditModeAction::MoveCursorEnd);

        let key_event = KeyEvent::from(KeyCode::Char('x'));
        assert_eq!(KeyHandler::handle_edit_mode_key(key_event), EditModeAction::InsertChar('x'));
    }

    #[test]
    fn test_normal_mode_delete_key() {
        let key_event = KeyEvent::from(KeyCode::Char('d'));
        assert_eq!(KeyHandler::handle_normal_mode_key(key_event), NormalModeAction::DeleteItem);
    }
}