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
- Tests are embedded in modules using `#[cfg(test)]` and `#[test]` attributes

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
   - `parser.rs` - Markdown parsing logic for TODO.md files
   - `writer.rs` - File writing capabilities (future feature)
   - Supports both TODO items and markdown headings with indentation levels

3. **`tui`** - Terminal User Interface
   - `app.rs` - Application state and event handling
   - `ui.rs` - Ratatui-based rendering logic
   - Built with `ratatui` and `crossterm` for cross-platform terminal UI

### Application Flow

The application follows this execution pattern:
1. CLI argument parsing with `clap` (config commands, completion generation, or main TUI)
2. Configuration loading from TOML file
3. TODO.md file parsing into structured data
4. TUI initialization and event loop for interactive browsing

### Key Dependencies

- **CLI**: `clap` with derive features for command-line interface
- **TUI**: `ratatui` + `crossterm` for terminal user interface
- **Parsing**: Custom markdown parser for TODO items and headings
- **Config**: `serde` + `toml` for configuration serialization
- **Error Handling**: `anyhow` for error propagation

### Data Model

The core `ListItem` enum represents either:
- `Todo` items with content, completion status, indentation level, and line number
- `Heading` items with content, heading level (1-6), and line number

## Testing Strategy

Tests are embedded within modules using Rust's built-in testing framework. Each module with tests includes a `#[cfg(test)]` section containing unit tests for its functionality.