use super::models::{ListItem, TodoList};
use anyhow::{Context, Result};
use std::fs;

pub fn write_todo_file(todo_list: &TodoList) -> Result<()> {
    let content = serialize_todo_list(todo_list);
    fs::write(&todo_list.file_path, content)
        .with_context(|| format!("Failed to write TODO file: {}", todo_list.file_path))?;
    Ok(())
}

pub fn serialize_todo_list(todo_list: &TodoList) -> String {
    let mut lines = Vec::new();
    
    for item in &todo_list.items {
        match item {
            ListItem::Todo { content, completed, indent_level, .. } => {
                let indent = "  ".repeat(*indent_level);
                let checkbox = if *completed { "- [x]" } else { "- [ ]" };
                lines.push(format!("{}{} {}", indent, checkbox, content));
            }
            ListItem::Note { content, indent_level, .. } => {
                let indent = "  ".repeat(*indent_level);
                lines.push(format!("{}- {}", indent, content));
            }
            ListItem::Heading { content, level, .. } => {
                let prefix = "#".repeat(*level);
                lines.push(format!("{} {}", prefix, content));
            }
        }
    }
    
    lines.join("\n") + "\n"
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::todo::parser;

    #[test]
    fn test_serialize_empty_list() {
        let todo_list = TodoList::new("test.md".to_string());
        let result = serialize_todo_list(&todo_list);
        assert_eq!(result, "\n");
    }

    #[test]
    fn test_serialize_single_todo() {
        let mut todo_list = TodoList::new("test.md".to_string());
        todo_list.add_item(ListItem::new_todo("Buy groceries".to_string(), false, 0, 0));
        
        let result = serialize_todo_list(&todo_list);
        assert_eq!(result, "- [ ] Buy groceries\n");
    }

    #[test]
    fn test_serialize_completed_todo() {
        let mut todo_list = TodoList::new("test.md".to_string());
        todo_list.add_item(ListItem::new_todo("Finish project".to_string(), true, 0, 0));
        
        let result = serialize_todo_list(&todo_list);
        assert_eq!(result, "- [x] Finish project\n");
    }

    #[test]
    fn test_serialize_indented_todo() {
        let mut todo_list = TodoList::new("test.md".to_string());
        todo_list.add_item(ListItem::new_todo("Subtask".to_string(), false, 2, 0));
        
        let result = serialize_todo_list(&todo_list);
        assert_eq!(result, "    - [ ] Subtask\n");
    }

    #[test]
    fn test_serialize_heading() {
        let mut todo_list = TodoList::new("test.md".to_string());
        todo_list.add_item(ListItem::new_heading("Main Section".to_string(), 1, 0));
        
        let result = serialize_todo_list(&todo_list);
        assert_eq!(result, "# Main Section\n");
    }

    #[test]
    fn test_serialize_nested_heading() {
        let mut todo_list = TodoList::new("test.md".to_string());
        todo_list.add_item(ListItem::new_heading("Subsection".to_string(), 2, 0));
        
        let result = serialize_todo_list(&todo_list);
        assert_eq!(result, "## Subsection\n");
    }

    #[test]
    fn test_serialize_note() {
        let mut todo_list = TodoList::new("test.md".to_string());
        todo_list.add_item(ListItem::new_note("This is a note".to_string(), 0, 0));
        
        let result = serialize_todo_list(&todo_list);
        assert_eq!(result, "- This is a note\n");
    }

    #[test]
    fn test_serialize_indented_note() {
        let mut todo_list = TodoList::new("test.md".to_string());
        todo_list.add_item(ListItem::new_note("Indented note".to_string(), 1, 0));
        
        let result = serialize_todo_list(&todo_list);
        assert_eq!(result, "  - Indented note\n");
    }

    #[test]
    fn test_serialize_mixed_content() {
        let mut todo_list = TodoList::new("test.md".to_string());
        todo_list.add_item(ListItem::new_heading("Project".to_string(), 1, 0));
        todo_list.add_item(ListItem::new_todo("Task 1".to_string(), false, 0, 1));
        todo_list.add_item(ListItem::new_note("Project notes".to_string(), 0, 2));
        todo_list.add_item(ListItem::new_todo("Task 2".to_string(), true, 0, 3));
        todo_list.add_item(ListItem::new_todo("Subtask".to_string(), false, 1, 4));
        todo_list.add_item(ListItem::new_note("Nested note".to_string(), 1, 5));
        
        let result = serialize_todo_list(&todo_list);
        let expected = "# Project\n- [ ] Task 1\n- Project notes\n- [x] Task 2\n  - [ ] Subtask\n  - Nested note\n";
        assert_eq!(result, expected);
    }

    #[test]
    fn test_roundtrip_parse_and_serialize() {
        use std::fs;
        
        // Create test content
        let original_content = "# Test Project\n\n- [ ] First task\n- [x] Completed task\n  - [ ] Subtask\n## Section 2\n- [ ] Another task\n";
        
        // Create temporary file
        let temp_file = "/tmp/test_roundtrip.md";
        fs::write(temp_file, original_content).unwrap();
        
        // Parse the file
        let todo_list = parser::parse_todo_file(temp_file).unwrap();
        
        // Serialize it back
        let serialized = serialize_todo_list(&todo_list);
        
        // The output should contain all the essential information
        // (might differ slightly in whitespace but should have same structure)
        assert!(serialized.contains("# Test Project"));
        assert!(serialized.contains("- [ ] First task"));
        assert!(serialized.contains("- [x] Completed task"));
        assert!(serialized.contains("  - [ ] Subtask"));
        assert!(serialized.contains("## Section 2"));
        assert!(serialized.contains("- [ ] Another task"));
        
        // Clean up
        fs::remove_file(temp_file).ok();
    }
}