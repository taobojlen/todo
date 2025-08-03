use super::models::{ListItem, TodoList};
use anyhow::{Context, Result};
use std::fs;

pub fn parse_todo_file(file_path: &str) -> Result<TodoList> {
    let content = fs::read_to_string(file_path)
        .with_context(|| format!("Failed to read TODO file: {}", file_path))?;

    let mut todo_list = TodoList::new(file_path.to_string());
    let mut in_yaml_frontmatter = false;

    for (line_number, line) in content.lines().enumerate() {
        // Skip YAML frontmatter
        if line.trim() == "---" {
            in_yaml_frontmatter = !in_yaml_frontmatter;
            continue;
        }
        if in_yaml_frontmatter {
            continue;
        }

        if let Some(item) = parse_line(line, line_number) {
            todo_list.add_item(item);
        }
    }

    Ok(todo_list)
}

fn parse_line(line: &str, line_number: usize) -> Option<ListItem> {
    let trimmed = line.trim();
    
    // Skip empty lines
    if trimmed.is_empty() {
        return None;
    }

    // Check for headings first
    if let Some((level, content)) = extract_heading_content(trimmed) {
        return Some(ListItem::new_heading(content, level, line_number));
    }

    // Check for todo items
    let trimmed_start = line.trim_start();
    let indent_level = calculate_indent_level(line);

    // Check for checkbox patterns: - [ ] or - [x] or - [X]
    if let Some(content) = extract_checkbox_content(trimmed_start) {
        let completed = is_checkbox_completed(trimmed_start);
        return Some(ListItem::new_todo(content, completed, indent_level, line_number));
    }

    None
}

fn calculate_indent_level(line: &str) -> usize {
    let mut indent_level = 0;
    
    for ch in line.chars() {
        match ch {
            '\t' => indent_level += 1,  // Tab = 1 level
            ' ' => {
                // Count spaces, 2 spaces = 1 level
                // We'll count all leading spaces and divide by 2
                let leading_spaces = line.chars()
                    .take_while(|&c| c == ' ')
                    .count();
                return leading_spaces / 2;
            }
            _ => break, // First non-whitespace character
        }
    }
    
    indent_level
}

fn extract_heading_content(line: &str) -> Option<(usize, String)> {
    if line.starts_with('#') {
        let mut level = 0;
        let mut chars = line.chars();
        
        // Count the number of # characters
        while let Some(ch) = chars.next() {
            if ch == '#' {
                level += 1;
            } else {
                break;
            }
        }
        
        // Extract content after the #'s and any following space
        let content_start = level;
        if line.len() > content_start {
            let content = line[content_start..].trim_start();
            if !content.is_empty() {
                return Some((level, content.to_string()));
            }
        }
    }
    None
}

fn extract_checkbox_content(line: &str) -> Option<String> {
    // Match patterns like "- [ ] content" or "- [x] content"
    if line.starts_with("- [") && line.len() > 5 {
        let checkbox_end = line.find(']')?;
        // For "- [ ]" pattern, ] should be at position 4
        // For "- [x]" pattern, ] should be at position 4 
        if checkbox_end == 4 {
            // Extract content after "]" (skip the space if present)
            let start_pos = if line.len() > checkbox_end + 1 && line.chars().nth(checkbox_end + 1) == Some(' ') {
                checkbox_end + 2
            } else {
                checkbox_end + 1
            };
            
            if line.len() > start_pos {
                let content = line[start_pos..].trim();
                if !content.is_empty() {
                    return Some(content.to_string());
                }
            }
        }
    }
    None
}

fn is_checkbox_completed(line: &str) -> bool {
    if line.len() > 4 {
        let checkbox_char = line.chars().nth(3).unwrap_or(' ');
        checkbox_char == 'x' || checkbox_char == 'X'
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_uncompleted_checkbox() {
        let item = parse_line("- [ ] Buy groceries", 0);
        assert!(item.is_some());
        let item = item.unwrap();
        match item {
            ListItem::Todo { content, completed, indent_level, .. } => {
                assert_eq!(content, "Buy groceries");
                assert!(!completed);
                assert_eq!(indent_level, 0);
            }
            _ => panic!("Expected Todo item"),
        }
    }

    #[test]
    fn test_parse_completed_checkbox() {
        let item = parse_line("- [x] Finish project", 1);
        assert!(item.is_some());
        let item = item.unwrap();
        match item {
            ListItem::Todo { content, completed, indent_level, .. } => {
                assert_eq!(content, "Finish project");
                assert!(completed);
                assert_eq!(indent_level, 0);
            }
            _ => panic!("Expected Todo item"),
        }
    }

    #[test]
    fn test_parse_indented_checkbox() {
        let item = parse_line("  - [ ] Subtask", 2);
        assert!(item.is_some());
        let item = item.unwrap();
        match item {
            ListItem::Todo { content, completed, indent_level, .. } => {
                assert_eq!(content, "Subtask");
                assert!(!completed);
                assert_eq!(indent_level, 1);
            }
            _ => panic!("Expected Todo item"),
        }
    }

    #[test]
    fn test_parse_heading() {
        let item = parse_line("# Main Section", 0);
        assert!(item.is_some());
        let item = item.unwrap();
        match item {
            ListItem::Heading { content, level, .. } => {
                assert_eq!(content, "Main Section");
                assert_eq!(level, 1);
            }
            _ => panic!("Expected Heading item"),
        }
    }

    #[test]
    fn test_parse_nested_heading() {
        let item = parse_line("## Subsection", 0);
        assert!(item.is_some());
        let item = item.unwrap();
        match item {
            ListItem::Heading { content, level, .. } => {
                assert_eq!(content, "Subsection");
                assert_eq!(level, 2);
            }
            _ => panic!("Expected Heading item"),
        }
    }

    #[test]
    fn test_parse_non_checkbox_line() {
        let item = parse_line("This is just a note", 0);
        assert!(item.is_none());
    }

    #[test]
    fn test_parse_invalid_checkbox() {
        let item = parse_line("- [invalid] content", 0);
        assert!(item.is_none());
    }

    #[test]
    fn test_parse_tab_indented_checkbox() {
        let item = parse_line("\t- [ ] Tab indented task", 0);
        assert!(item.is_some());
        let item = item.unwrap();
        match item {
            ListItem::Todo { content, completed, indent_level, .. } => {
                assert_eq!(content, "Tab indented task");
                assert!(!completed);
                assert_eq!(indent_level, 1);
            }
            _ => panic!("Expected Todo item"),
        }
    }

    #[test]
    fn test_parse_double_tab_indented_checkbox() {
        let item = parse_line("\t\t- [ ] Double tab indented task", 0);
        assert!(item.is_some());
        let item = item.unwrap();
        match item {
            ListItem::Todo { content, completed, indent_level, .. } => {
                assert_eq!(content, "Double tab indented task");
                assert!(!completed);
                assert_eq!(indent_level, 2);
            }
            _ => panic!("Expected Todo item"),
        }
    }

    #[test]
    fn test_calculate_indent_level() {
        assert_eq!(calculate_indent_level("- [ ] No indent"), 0);
        assert_eq!(calculate_indent_level("  - [ ] Two spaces"), 1);
        assert_eq!(calculate_indent_level("    - [ ] Four spaces"), 2);
        assert_eq!(calculate_indent_level("\t- [ ] One tab"), 1);
        assert_eq!(calculate_indent_level("\t\t- [ ] Two tabs"), 2);
    }
}