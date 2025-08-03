mod config;
mod todo;
mod tui;

use clap::{Parser, Subcommand, ValueHint, Command, CommandFactory};
use clap_complete::{generate, Generator, Shell};
use config::{Config, ConfigError};
use std::io;
use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    Terminal,
};
use todo::parser::parse_todo_file;
use tui::{app::App, ui};

#[derive(Parser)]
#[command(name = "todo")]
#[command(about = "A TUI for managing markdown-based TODO lists")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    #[command(about = "Configuration management")]
    Config {
        #[command(subcommand)]
        action: ConfigAction,
    },
    #[command(about = "Generate shell completion scripts")]
    Completion {
        #[arg(help = "Shell to generate completions for")]
        shell: Shell,
    },
}

#[derive(Subcommand)]
enum ConfigAction {
    #[command(about = "Set a configuration value")]
    Set {
        #[arg(help = "Configuration key (currently only 'file_path' is supported)")]
        key: String,
        #[arg(help = "Configuration value", value_hint = ValueHint::FilePath)]
        value: String,
    },
    #[command(about = "Get a configuration value")]
    Get {
        #[arg(help = "Configuration key")]
        key: String,
    },
    #[command(about = "List all configuration values")]
    List,
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Config { action }) => {
            if let Err(e) = handle_config_command(action) {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        }
        Some(Commands::Completion { shell }) => {
            let mut cmd = Cli::command();
            print_completions(shell, &mut cmd);
        }
        None => {
            if let Err(e) = run_main_app() {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        }
    }
}

fn handle_config_command(action: ConfigAction) -> Result<(), ConfigError> {
    match action {
        ConfigAction::Set { key, value } => {
            if key != "file_path" {
                eprintln!("Error: Unknown configuration key '{}'. Only 'file_path' is supported.", key);
                std::process::exit(1);
            }
            
            let mut config = match Config::load() {
                Ok(config) => config,
                Err(ConfigError::ConfigNotFound) => Config {
                    file_path: String::new(),
                },
                Err(e) => return Err(e),
            };
            
            config.set_file_path(value);
            config.save()?;
            println!("Configuration saved successfully.");
        }
        ConfigAction::Get { key } => {
            if key != "file_path" {
                eprintln!("Error: Unknown configuration key '{}'. Only 'file_path' is supported.", key);
                std::process::exit(1);
            }
            
            let config = Config::load()?;
            println!("{}", config.file_path);
        }
        ConfigAction::List => {
            let config = Config::load()?;
            println!("file_path = {}", config.file_path);
        }
    }
    Ok(())
}

fn run_main_app() -> Result<()> {
    let config = Config::load()
        .map_err(|e| anyhow::anyhow!("Configuration error: {}", e))?;
    
    let todo_list = parse_todo_file(&config.file_path)?;
    let mut app = App::new(todo_list);
    
    run_tui(&mut app)?;
    
    Ok(())
}

fn run_tui(app: &mut App) -> Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let result = run_app(&mut terminal, app);

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    result
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> Result<()> {
    loop {
        terminal.draw(|f| ui::draw(f, app))?;

        if let Event::Key(key) = event::read()? {
            app.handle_key_event(key)?;
            if app.should_quit {
                break;
            }
        }
    }
    Ok(())
}

fn print_completions<G: Generator>(generator: G, cmd: &mut Command) {
    generate(generator, cmd, cmd.get_name().to_string(), &mut io::stdout());
}
