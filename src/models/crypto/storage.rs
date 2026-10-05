use super::{
    CryptoEngine,
    ItemKeyStore,
    response::{
        AppError,
    },
    
};
use std::{
    fs::{create_dir_all, OpenOptions, read_to_string},
    io::{BufWriter, Write,},
    path::Path,
};




impl CryptoEngine {
      /// Save only the serializable CryptoEngine fields
    pub fn save(&self) -> Result<(), AppError> {
        let path = Path::new(&self.system_path.0);

        if let Some(parent) = path.parent() {
            create_dir_all(parent).map_err(|e| AppError::io_error(e.to_string()))?;
        }

        let file = OpenOptions::new()
            .write(true)
            .truncate(true)
            .create(true)
            .open(path)
            .map_err(|e| {
                AppError::runtime_error(format!("Failed to create file: {}", e))
                    .with_context("Error while saving crypto data")
            })?;

        // serialize only the allowed fields (serde will skip `master_key` & `item_key_store`)
        let writer = BufWriter::new(file);
        serde_json::to_writer_pretty(writer, &self)
            .map_err(|e| {
                AppError::runtime_error(format!("Failed to serialize crypto data: {}", e))
                    .with_context("Error while saving crypto data")
            })?;

        // save item key store separately
        self.item_key_store.save()?;

        Ok(())
    }

    /// Load CryptoEngine data (without master key) and load item key store
    pub fn load(&mut self) -> Result<(), AppError> {
        let path = Path::new(&self.system_path.0);

        if !path.exists() {
            return Err(
                AppError::runtime_error("Crypto storage file doesn't exist")
                    .with_context("Failed to load crypto data"),
            );
        }

        let data = read_to_string(path)
            .map_err(|e| AppError::io_error(format!("Failed to read file: {}", e)))?;

        if data.trim().is_empty() {
            return Err(AppError::runtime_error("Crypto file is empty"));
        }

        let loaded: CryptoEngine = serde_json::from_str(&data)
            .map_err(|e| AppError::runtime_error(format!("Failed to parse crypto JSON: {}", e)))?;

        // Only copy safe fields — skip master key (should remain runtime only)
        self.verifier_b64 = loaded.verifier_b64;
        self.kdf_salt_b64 = loaded.kdf_salt_b64;
        self.system_path = loaded.system_path.clone();

        // load item key store separately
        self.item_key_store.load()?;

        Ok(())
    }
}

impl ItemKeyStore {
    pub fn save(&self) -> Result<(), AppError> {
        if let Some(parent) = Path::new(&self.system_path.0).parent() {
            create_dir_all(parent).map_err(|e| AppError::io_error(e.to_string()))?;
        }

        let file = OpenOptions::new()
            .write(true)
            .truncate(true)
            .create(true)
            .open(&self.system_path.0)
            .map_err(|e| {
                AppError::runtime_error(format!("Failed to create file: {}", e))
                    .with_context("Failed to save metadata")
            })?;

        let mut writer = BufWriter::new(file);
        let serialized = serde_json::to_string_pretty(&self.collection)
            .map_err(|e| {
                AppError::runtime_error(format!("Failed to serialize data: {}", e))
                    .with_context("Error while serializing item keys data")
            })?;

        writer.write_all(serialized.as_bytes())
            .map_err(|e| {
                AppError::io_error(format!("Failed to write data: {}", e))
                    .with_context("Error while saving item keys data")
            })?;

        writer.flush()
            .map_err(|e| {
                AppError::io_error(format!("Failed to flush data: {}", e))
                    .with_context("Error while saving item keys data")
            })?;

        Ok(())
    }

    pub fn load(&mut self) -> Result<(), AppError> {
        if Path::new(&self.system_path.0).exists() {
            let data = read_to_string(&self.system_path.0)
                .map_err(|e| {
                    AppError::io_error(format!("Failed to read file: {}", e))
                        .with_context("Error while loading item keys data")
                })?;
            
            if !data.trim().is_empty() {
                self.collection = serde_json::from_str(&data)
                    .map_err(|e| {
                        AppError::runtime_error(format!("Failed to deserialize data: {}", e))
                            .with_context("Error while parsing item keys data")
                    })?;
            }
            Ok(())
        } else {
            Err(
                AppError::runtime_error("Crypto data file doesn't exist")
                    .with_context("Failed to load metadata")
            )
        }
    }
}