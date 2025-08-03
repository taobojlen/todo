use crate::todo::models::ListItem as TodoListItem;
use crate::tui::app::App;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Clear},
};

pub fn draw(frame: &mut Frame, app: &mut App) {
    if app.help_mode {
        draw_help_window(frame, app);
    } else {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([
                Constraint::Length(3), // Header
                Constraint::Min(0),    // Main content
                Constraint::Length(3), // Footer
            ])
            .split(frame.size());

        draw_header(frame, chunks[0], app);
        draw_todo_list(frame, chunks[1], app);
        draw_footer(frame, chunks[2], app);
    }
}

fn draw_header(frame: &mut Frame, area: ratatui::layout::Rect, app: &App) {
    let header_text = format!("TODO List - {}", app.todo_list.file_path);
    let header = Paragraph::new(header_text)
        .block(Block::default().borders(Borders::ALL).title("Todo"))
        .style(Style::default().fg(Color::Cyan));

    frame.render_widget(header, area);
}

fn draw_todo_list(frame: &mut Frame, area: ratatui::layout::Rect, app: &mut App) {
    let items: Vec<ListItem> = app
        .todo_list
        .items
        .iter()
        .enumerate()
        .map(|(i, list_item)| {
            // Check if this item is being edited or selected for bulk operation
            let is_editing = app.edit_mode && i == app.selected_index;
            let is_bulk_selected = app.selected_items.contains(&i);
            
            match list_item {
                TodoListItem::Todo {
                    content,
                    completed,
                    indent_level,
                    ..
                } => {
                    let checkbox = if *completed { "☑" } else { "☐" };
                    let indent = "  ".repeat(*indent_level);
                    let selection_indicator = if is_bulk_selected { "●" } else { " " };
                    
                    let display_content = if is_editing {
                        // Show edit buffer with cursor
                        let (before_cursor, after_cursor) = app.edit_buffer.split_at(app.edit_cursor_position);
                        format!("{}{}{} {}█{}", selection_indicator, indent, checkbox, before_cursor, after_cursor)
                    } else {
                        format!("{}{}{} {}", selection_indicator, indent, checkbox, content)
                    };

                    let style = if is_editing {
                        Style::default()
                            .bg(Color::Blue)
                            .fg(Color::White)
                            .add_modifier(Modifier::BOLD)
                    } else if is_bulk_selected {
                        Style::default()
                            .fg(Color::Cyan)
                            .add_modifier(Modifier::BOLD)
                    } else if *completed {
                        Style::default()
                            .fg(Color::DarkGray)
                            .add_modifier(Modifier::CROSSED_OUT)
                    } else {
                        Style::default().fg(Color::White)
                    };

                    let line = Line::from(Span::styled(display_content, style));
                    ListItem::new(line)
                }
                TodoListItem::Heading { content, level, .. } => {
                    let prefix = "#".repeat(*level);
                    let selection_indicator = if is_bulk_selected { "●" } else { " " };
                    
                    let display_content = if is_editing {
                        // Show edit buffer with cursor for headings
                        let (before_cursor, after_cursor) = app.edit_buffer.split_at(app.edit_cursor_position);
                        format!("{}{} {}█{}", selection_indicator, prefix, before_cursor, after_cursor)
                    } else {
                        format!("{}{} {}", selection_indicator, prefix, content)
                    };

                    let (color, modifier) = if is_editing {
                        (Color::White, Modifier::BOLD)
                    } else if is_bulk_selected {
                        (Color::Cyan, Modifier::BOLD)
                    } else {
                        match level {
                            1 => (Color::Yellow, Modifier::BOLD | Modifier::UNDERLINED),
                            2 => (Color::Cyan, Modifier::BOLD),
                            3 => (Color::Green, Modifier::BOLD),
                            _ => (Color::Blue, Modifier::BOLD),
                        }
                    };

                    let style = if is_editing {
                        Style::default()
                            .bg(Color::Blue)
                            .fg(color)
                            .add_modifier(modifier)
                    } else {
                        Style::default().fg(color).add_modifier(modifier)
                    };

                    let line = Line::from(Span::styled(display_content, style));
                    ListItem::new(line)
                }
            }
        })
        .collect();

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("Items"))
        .highlight_style(
            Style::default()
                .bg(Color::Yellow)
                .fg(Color::Black)
                .add_modifier(Modifier::BOLD),
        );

    let mut list_state = ListState::default();
    list_state.select(Some(app.selected_index));

    frame.render_stateful_widget(list, area, &mut list_state);
}

fn draw_footer(frame: &mut Frame, area: ratatui::layout::Rect, app: &App) {
    let footer_text = if app.edit_mode {
        "EDIT MODE | Enter: confirm | Esc: cancel | ←→: cursor | Backspace/Delete: edit".to_string()
    } else {
        format!(
            "Items: {} | Completed: {} | Selected: {} | ↑↓/j/k: navigate | Space: select | ?: help | q: quit",
            app.total_items(),
            app.completed_items(),
            app.selected_items.len()
        )
    };

    let footer = Paragraph::new(footer_text)
        .block(Block::default().borders(Borders::ALL))
        .style(Style::default().fg(Color::Yellow));

    frame.render_widget(footer, area);
}

fn draw_help_window(frame: &mut Frame, app: &mut App) {
    // First draw the normal interface
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Min(0),    // Main content
            Constraint::Length(3), // Footer
        ])
        .split(frame.size());

    draw_header(frame, chunks[0], app);
    draw_todo_list(frame, chunks[1], app);
    draw_footer(frame, chunks[2], app);

    // Then overlay the help window
    let help_text = vec![
        "Todo List - Keyboard Commands",
        "",
        "NAVIGATION:",
        "  ↑↓ / j/k          Navigate up/down",
        "  Enter             Toggle todo completion",
        "",
        "EDITING:",
        "  e                 Edit current item",
        "  a                 Add new todo below cursor",
        "  Shift+A           Add new todo at top/under heading",
        "",
        "MOVEMENT:",
        "  Shift+↑↓ / J/K    Move item up/down",
        "  Shift+←→ / H/L    Unindent/indent item",
        "",
        "BULK OPERATIONS:",
        "  Space             Select/deselect item for bulk operations",
        "  m                 Move selected items below cursor",
        "",
        "OTHER:",
        "  u                 Undo last operation",
        "  Esc               Clear selection",
        "  ?                 Show this help (press ? or Esc to close)",
        "  q / Ctrl+C        Quit application",
        "",
        "Press ? or Esc to close this help window",
    ];

    let help_paragraph = Paragraph::new(help_text.join("\n"))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Help - Keyboard Commands ")
                .style(Style::default().fg(Color::Yellow))
        )
        .style(Style::default().fg(Color::White))
        .wrap(ratatui::widgets::Wrap { trim: true });

    // Create a centered area for the help window
    let area = centered_rect(80, 70, frame.size());
    
    // Clear the area and render the help window
    frame.render_widget(Clear, area);
    frame.render_widget(help_paragraph, area);
}

fn centered_rect(percent_x: u16, percent_y: u16, r: ratatui::layout::Rect) -> ratatui::layout::Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
