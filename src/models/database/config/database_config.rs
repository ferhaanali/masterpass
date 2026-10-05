use super::{
    operations::DatabaseArguments,
    core::{StringPath, Timestamp},
    response::AppError,
};
use serde::{Serialize, Deserialize};
use serde_json;
use std::{fs, path::Path};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DatabaseConfig {
    pub database_version: String,
    pub max_file_size_mb: u64,
    pub is_nested: bool,

    pub name: String,
    pub created_at: Timestamp,
    pub last_login: Timestamp,
    pub owner: String,
    pub description: String,

    pub file_path: Option<StringPath>,
}

impl DatabaseConfig {
    /// Default 
    pub fn default() -> Self {
        Self {
            database_version: "0.1.0".to_string(),
            max_file_size_mb: 100,
            is_nested: false,

            name: "default_db".to_string(),
            created_at: Timestamp::now(),
            last_login: Timestamp::now(),
            owner: "unknown".to_string(),
            description: "Auto-generated database configuration".to_string(),

            file_path: None,
        }
    }

    pub fn load(&mut self) -> Result<(), AppError> {
        let path = match &self.file_path {
            Some(p) => p,
            None => {
                return Err(AppError::configuration_error("No file path set in configuration")
                    .with_context("Loading database configuration")
                    .with_suggestion("Use `load_or_create()` if configuration file is missing"));
            }
        };

        let path_ref = Path::new(&path.0);

        if !path_ref.exists() {
            return Err(AppError::not_found(format!(
                "Configuration file not found: {}",
                path.0
            ))
            .with_context("Loading database configuration"));
        }

        let data = fs::read_to_string(path_ref)
            .map_err(|e| AppError::io_error(format!("Failed to read config: {e}"))
                .with_context("Reading configuration file"))?;

        if data.trim().is_empty() {
            return Err(AppError::corrupted_data("Configuration file is empty")
                .with_context("Loading database configuration"));
        }

        // Deserialize into a temporary object, then assign values to self
        let mut loaded: Self = serde_json::from_str(&data)
            .map_err(|e| AppError::corrupted_data(format!("Failed to parse config: {e}"))
                .with_context("Parsing database configuration file"))?;

        loaded.last_login.update();
        *self = loaded;

        Ok(())
    }

    pub fn load_or_create(args: &DatabaseArguments, path: &str) -> Result<Self, AppError> {
        let path_ref = Path::new(path);

        let should_create = !path_ref.exists()
            || fs::metadata(path_ref).map(|m| m.len() == 0).unwrap_or(true);

        if should_create {
            if let Some(parent) = path_ref.parent() {
                fs::create_dir_all(parent)
                    .map_err(|e| AppError::io_error(format!("Failed to create parent directory: {e}"))
                        .with_context("Creating config directory structure"))?;
            }

            let mut cfg = Self::default();
            cfg.is_nested = args.is_nested;
            cfg.name = args.database_name.clone();
            cfg.owner = args.owner.clone();
            cfg.description = args.description.clone();
            cfg.file_path = Some(StringPath(path.to_string()));

            let config_data = serde_json::to_string_pretty(&cfg)
                .map_err(|e| AppError::configuration_error(format!("Failed to serialize config: {e}"))
                    .with_context("Creating new database configuration"))?;

            fs::write(path_ref, config_data)
                .map_err(|e| AppError::io_error(format!("Failed to write config: {e}"))
                    .with_context("Saving new database file"))?;

            Ok(cfg)
        } else {
            let mut cfg = Self::default();
            cfg.file_path = Some(StringPath::from_str(path));
            cfg.load()?; // load into existing instance
            Ok(cfg)
        }
    }

    pub fn save(&self) -> Result<(), AppError> {
        let path = match self.file_path.as_ref() {
            Some(p) => p,
            None => {
                return Err(AppError::configuration_error("Config file path is not set")
                    .with_context("Saving database configuration")
                    .with_suggestion("Set a valid file path before saving"));
            }
        };

        if let Some(parent) = Path::new(path).parent() {
            fs::create_dir_all(parent)
                .map_err(|e| AppError::io_error(format!("Failed to create config directory: {e}"))
                    .with_context("Creating directory structure for database file"))?;
        }

        let serialized = serde_json::to_string_pretty(&self)
            .map_err(|e| AppError::configuration_error(format!("Failed to serialize config: {e}"))
                .with_context("Serializing database configuration"))?;

        fs::write(path, serialized)
            .map_err(|e| AppError::io_error(format!("Failed to write config to file: {e}"))
                .with_context("Writing database file to disk"))?;

        Ok(())
    }
}
