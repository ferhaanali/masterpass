use serde::{Serialize, Deserialize};
use super::{
    response::{AppError},
    core::{
        StringPath,
    },
    nested_database::NestedDatabaseMetadata,
    data_collections::{
        FileCollection,
        PasswordCollection,
    },
};
use std::{
    path::{Path},
    fs::{create_dir_all, read_to_string, OpenOptions},
    io::{BufWriter, Write},
};




#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct DataCollection {
    pub passwords: PasswordCollection, 
    pub files: FileCollection,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Metadata {
    pub data: DataCollection,
    pub file_path: StringPath,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseMetadata {
    pub decrypted: Metadata,
    pub encrypted: Metadata, 
    pub trash: Metadata,
    pub nested_databases: NestedDatabaseMetadata,
}




impl Metadata {
    pub fn save(&self) -> Result<(), AppError> {
        if let Some(parent) = Path::new(&self.file_path.0).parent() {
            if let Err(e) = create_dir_all(parent) {
                eprintln!("Failed to create parent directory: {}", e);
                return Err(
                    AppError::runtime_error(format!("Failed to create parent directory: {}", e))
                        .with_context("Failed to save metadata")
                );
            }
        }

        let file = match OpenOptions::new()
            .write(true)
            .truncate(true)
            .create(true)
            .open(&self.file_path.0)
        {
            Ok(f) => f,
            Err(e) => {
                eprintln!("Failed to open file: {}", e);
                return Err(
                    AppError::runtime_error(format!("Failed to create parent directory: {}", e))
                        .with_context("Failed to save metadata")
                );
            }
        };

        let mut writer = BufWriter::new(file);
        match serde_json::to_string_pretty(&self.data) {
            Ok(serialized) => {
                if let Err(e) = writeln!(writer, "{}", serialized) {
                    eprintln!("Failed to write data: {}", e);
                    return Err(
                    AppError::runtime_error(format!("Failed to create parent directory: {}", e))
                        .with_context("Failed to save metadata")
                );
                }
            }
            Err(e) => {
                eprintln!("Failed to serialize data: {}", e);
                return Err(
                    AppError::runtime_error(format!("Failed to serialize data: {}", e))
                        .with_context("Failed to save metadata")
                );
            }
        }

        Ok(())
    }

    pub fn load(&mut self) -> Result<(), AppError> {
        if Path::new(&self.file_path.0).exists() {
            let data = read_to_string(&self.file_path.0).unwrap_or_default();
            if !data.trim().is_empty() {
                self.data = serde_json::from_str(&data).unwrap_or_default();
            }
            Ok(())
        }else {
            return Err(
                AppError::runtime_error("Meta data file don't exists")
                    .with_context("Failed to load metadata")
            );
        }
    }
}
