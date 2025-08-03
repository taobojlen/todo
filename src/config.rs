use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::fs;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub file_path: String,
}

impl Config {
    pub fn load() -> Result<Self, ConfigError> {
        let config_path = get_config_file_path()?;
        
        if !config_path.exists() {
            return Err(ConfigError::ConfigNotFound);
        }

        let content = fs::read_to_string(&config_path)
            .map_err(|e| ConfigError::ReadError(e.to_string()))?;
        
        let config: Config = toml::from_str(&content)
            .map_err(|e| ConfigError::ParseError(e.to_string()))?;
        
        Ok(config)
    }

    pub fn save(&self) -> Result<(), ConfigError> {
        let config_path = get_config_file_path()?;
        
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| ConfigError::WriteError(e.to_string()))?;
        }

        let content = toml::to_string(self)
            .map_err(|e| ConfigError::SerializeError(e.to_string()))?;
        
        fs::write(&config_path, content)
            .map_err(|e| ConfigError::WriteError(e.to_string()))?;
        
        Ok(())
    }

    pub fn set_file_path(&mut self, path: String) {
        self.file_path = path;
    }
}

fn get_config_file_path() -> Result<PathBuf, ConfigError> {
    let config_dir = dirs::config_dir()
        .ok_or(ConfigError::ConfigDirNotFound)?;
    
    Ok(config_dir.join("todo").join("config.toml"))
}

#[derive(Debug)]
pub enum ConfigError {
    ConfigNotFound,
    ConfigDirNotFound,
    ReadError(String),
    WriteError(String),
    ParseError(String),
    SerializeError(String),
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigError::ConfigNotFound => {
                write!(f, "Configuration not found. Run 'todo config set file_path <path>' to configure your TODO file location.")
            }
            ConfigError::ConfigDirNotFound => {
                write!(f, "Could not find config directory")
            }
            ConfigError::ReadError(msg) => {
                write!(f, "Failed to read config file: {}", msg)
            }
            ConfigError::WriteError(msg) => {
                write!(f, "Failed to write config file: {}", msg)
            }
            ConfigError::ParseError(msg) => {
                write!(f, "Failed to parse config file: {}", msg)
            }
            ConfigError::SerializeError(msg) => {
                write!(f, "Failed to serialize config: {}", msg)
            }
        }
    }
}

impl std::error::Error for ConfigError {}