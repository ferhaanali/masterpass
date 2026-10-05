use super::{
    NodeManager,
    response::AppError,
    core::{StringPath}
};
use std::{
    path::{Path},
    fs::{create_dir_all, OpenOptions, read_to_string},
    io::{BufWriter, Write},
};
use serde_json;



impl NodeManager {
    pub fn with_file_path(mut self, file_path: &str) -> Self {
        self.file_path = file_path.to_string();
        self
    }

    pub fn save(&self) -> Result<(), AppError> {

        // Ensure parent directories exist
        if let Some(parent) = Path::new(self.file_path.as_ref()).parent() {
            create_dir_all(parent).map_err(|e| {
                AppError::io_error(format!("Failed to create parent directories: {}", e))
                    .with_context("Error while saving Node Manager data")
            })?;
        }

        // Open file for writing
        let file = OpenOptions::new()
            .write(true)
            .truncate(true)
            .create(true)
            .open(self.file_path.as_ref())
            .map_err(|e| {
                AppError::io_error(format!("Failed to open file: {}", e))
                    .with_context("Error while saving Node Manager data")
            })?;

        let mut writer = BufWriter::new(file);

        // Serialize `self` directly
        let serialized = serde_json::to_string_pretty(self).map_err(|e| {
            AppError::serialization_error(format!("Failed to serialize data: {}", e))
                .with_context("Error while saving Node Manager data")
        })?;

        // Write data
        writeln!(writer, "{}", serialized).map_err(|e| {
            AppError::io_error(format!("Failed to write data: {}", e))
                .with_context("Error while saving Node Manager data")
        })?;

        writer.flush().ok();

        Ok(())
    }


    pub fn load(&mut self) -> Result<(), AppError> {
        let file_path = self.file_path.as_ref();

        if !Path::new(file_path).exists() {
            *self = NodeManager::new().with_file_path(file_path);
            return self.save(); // Save the default empty manager
        }

        let data = read_to_string(file_path)
            .map_err(|e| {
                AppError::io_error(format!("Failed to read file: {}", e))
                    .with_context("Error while loading Node Manager data")
            })?;

        if data.trim().is_empty() {
            *self = NodeManager::new().with_file_path(file_path);
            return self.save(); // Save the default empty manager
        }

        let saved_data: serde_json::Value = serde_json::from_str(&data)
            .map_err(|e| {
                AppError::deserialization_error(format!("Failed to parse NodeManager data: {}", e))
                    .with_context("Error while loading Node Manager data")
            })?;

        let current_file_path = self.file_path.clone();
        *self = NodeManager::new();
        self.file_path = current_file_path;

        if let (Some(nodes_array), Some(refs_array)) = (
            saved_data.get("nodes").and_then(|v| v.as_array()),
            saved_data.get("item_references").and_then(|v| v.as_array())
        ) {
            for node_value in nodes_array {
                if let Ok(node) = serde_json::from_value::<Node>(node_value.clone()) {
                    if let Some(ref_value) = refs_array.iter().find(|ref_val| {
                        ref_val.get(0).and_then(|id| id.as_str()) == Some(&node.id.0)
                    }) {
                        if let (Some(_), Some(item_ref_value)) = (ref_value.get(0), ref_value.get(1)) {
                            if let Ok(item_ref) = serde_json::from_value::<NodeItemReference>(item_ref_value.clone()) {
                                let _ = self.add_node(node, item_ref);
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    }

    pub fn set_file_path(&mut self, file_path: &str) {
        self.file_path = file_path.to_string()
    }
}