use crate::todo::models::TodoList;
use std::collections::HashSet;

#[derive(Clone, Debug)]
pub struct AppState {
    pub todo_list: TodoList,
    pub selected_index: usize,
    pub selected_items: HashSet<usize>,
}

impl AppState {
    pub fn new(todo_list: TodoList, selected_index: usize, selected_items: HashSet<usize>) -> Self {
        Self {
            todo_list,
            selected_index,
            selected_items,
        }
    }
}