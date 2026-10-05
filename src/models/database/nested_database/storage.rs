use super::{
    NestedDatabaseMetadata,
    response::AppError,
};
use std::{
    path::{Path},
    fs::{
        create_dir_all,
        OpenOptions,
        read_to_string
    },
    io::{BufWriter, Write},
};















impl NestedDatabaseMetadata {
      pub fn save(&self) -> Result<(), AppError> {
        if let Some(parent) = Path::new(&self.file_path.0).parent() {
            create_dir_all(parent).map_err(|e| {
                AppError::io_error(format!("Failed to create parent directory: {}", e))
            })?;
        }

        let file = OpenOptions::new()
            .write(true)
            .truncate(true)
            .create(true)
            .open(&self.file_path.0)
            .map_err(|e| {
                AppError::io_error(format!("Failed to open file: {}", e))
            })?;

        let mut writer = BufWriter::new(file);
        let serialized = serde_json::to_string_pretty(&self.data).map_err(|e| {
            AppError::initialization_error(format!("Failed to serialize data: {}", e))
        })?;

        writeln!(writer, "{}", serialized).map_err(|e| {
            AppError::io_error(format!("Failed to write data: {}", e))
        })?;

        Ok(())
    }

    pub fn load(&mut self) -> Result<(), AppError> {
        if Path::new(&self.file_path.0).exists() {
            let data = read_to_string(&self.file_path.0).map_err(|e| {
                AppError::io_error(format!("Failed to read file: {}", e))
            })?;
            if !data.trim().is_empty() {
                self.data = serde_json::from_str(&data).map_err(|e| {
                    AppError::initialization_error(format!("Failed to deserialize data: {}", e))
                })?;
            }
            Ok(())
        } else {
            Err(AppError::runtime_error("Meta data file doesn't exist")
                .with_context("Failed to load metadata"))
        }
    }
}
