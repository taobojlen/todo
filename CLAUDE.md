# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a personal CLI tool for managing a simple `TODO.md` markdown-file-based task system. The project is written in Rust and implements the author's productivity system for managing TODO lists as described in their blog post at https://btao.org/posts/2025-03-15-productivity/.

## Development Commands

### Building and Running

- `cargo build` - Compile the project
- `cargo run` - Build and run the application
- `cargo build --release` - Build optimized release version
- `cargo check` - Check code for errors without building

### Testing

- `cargo test` - Run all tests
- `cargo test <test_name>` - Run a specific test
- `cargo test --lib` - Run only library tests
- `cargo test -- --nocapture` - Show println! output during tests

### Development Environment

- Uses Rust 1.88.0 (managed via mise.toml)
- `mise install` - Install the correct Rust version

## Architecture Overview

The application is structured into three main modules:

### Core Modules

1. **`config`** - Configuration management system
   - Handles TOML-based configuration stored in `~/.config/todo/config.toml`
   - Currently manages only `file_path` setting for TODO.md location
   - Uses `dirs` crate for cross-platform config directory detection

2. **`todo`** - TODO list data modeling and file operations
   - `models.rs` - Core data structures (`ListItem`, `TodoList`)
   - `parser.rs` - Markdown parsing logic for TODO.md files (includes comprehensive tests)
   - `writer.rs` - Serialization logic for writing TODO lists back to markdown
   - Supports TODO items (checkboxes), notes (bullet points), and markdown headings

3. **`tui`** - Terminal User Interface with multiple submodules:
   - `app.rs` - Main application state and coordination
   - `ui.rs` - Ratatui-based rendering logic
   - `handlers.rs` - Keyboard event handling and mode-specific actions
   - `navigation.rs` - Navigation state and item creation
   - `edit.rs` - In-place editing functionality
   - `search.rs` - Search/filter functionality
   - `undo.rs` - Undo/redo operations
   - `actions.rs` - Item manipulation actions (toggle, delete)
   - `persistence.rs` - File saving operations
   - `state.rs` - Shared state definitions

### Application Flow

1. CLI argument parsing with `clap` (config commands, completion generation, or main TUI)
2. Configuration loading from TOML file
3. TODO.md file parsing into structured data
4. TUI initialization and event loop for interactive browsing

### Key Dependencies

- **CLI**: `clap` with derive features for command-line interface
- **TUI**: `ratatui` + `crossterm` for terminal user interface  
- **Parsing**: Custom markdown parser for TODO items, notes, and headings
- **Config**: `serde` + `toml` for configuration serialization
- **Error Handling**: `anyhow` for error propagation

### Data Model

The core `ListItem` enum represents three types of content:
- `Todo` - Checkbox items with completion status and indentation level
- `Note` - Bullet point items without checkboxes, with indentation level
- `Heading` - Markdown headings with level (1-6)

### TUI Keyboard Shortcuts

The application supports vim-style navigation and multiple modes:
- Normal mode: `j/k` (up/down), `g/G` (top/bottom), `Space` (toggle), `d` (delete), etc.
- Search mode: `/` to enter, type to filter, `Esc` to exit
- Edit mode: `e` to enter, edit text, `Enter` to save, `Esc` to cancel
- Help mode: `?` to toggle

## Testing Strategy

Tests are embedded within modules using Rust's built-in testing framework. Key test files:
- `src/todo/parser.rs` - Comprehensive parsing tests including roundtrip serialization
- Tests use `#[cfg(test)]` modules with `#[test]` attributes