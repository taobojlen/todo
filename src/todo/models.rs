#[derive(Debug, Clone)]
pub enum ListItem {
    Todo {
        content: String,
        completed: bool,
        indent_level: usize,
        line_number: usize,
    },
    Heading {
        content: String,
        level: usize, // 1 for #, 2 for ##, etc.
        line_number: usize,
    },
}

impl ListItem {
    pub fn new_todo(content: String, completed: bool, indent_level: usize, line_number: usize) -> Self {
        Self::Todo {
            content,
            completed,
            indent_level,
            line_number,
        }
    }

    pub fn new_heading(content: String, level: usize, line_number: usize) -> Self {
        Self::Heading {
            content,
            level,
            line_number,
        }
    }

    pub fn is_completed(&self) -> bool {
        match self {
            Self::Todo { completed, .. } => *completed,
            Self::Heading { .. } => false,
        }
    }

    pub fn content(&self) -> &str {
        match self {
            Self::Todo { content, .. } => content,
            Self::Heading { content, .. } => content,
        }
    }
}

#[derive(Debug, Clone)]
pub struct TodoList {
    pub items: Vec<ListItem>,
    pub file_path: String,
}

impl TodoList {
    pub fn new(file_path: String) -> Self {
        Self {
            items: Vec::new(),
            file_path,
        }
    }

    pub fn add_item(&mut self, item: ListItem) {
        self.items.push(item);
    }

    pub fn total_items(&self) -> usize {
        self.items.iter().filter(|item| matches!(item, ListItem::Todo { .. })).count()
    }

    pub fn completed_items(&self) -> usize {
        self.items.iter().filter(|item| item.is_completed()).count()
    }
}