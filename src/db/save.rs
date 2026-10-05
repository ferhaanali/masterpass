use super::{
    structs::{MetaType, NestedDatabaseMetaType},
    std::{
        fs,
        fs::{OpenOptions, read_to_string, create_dir_all},
        io::{BufWriter, Write},
        path::Path,
    },
};



impl MetaType {
    pub fn save(&self) -> bool {
        if let Some(parent) = Path::new(&self.file_path).parent() {
            if let Err(e) = create_dir_all(parent) {
                eprintln!("Failed to create parent directory: {}", e);
                return false;
            }
        }

        let file = match OpenOptions::new()
            .write(true)
            .truncate(true)
            .create(true)
            .open(&self.file_path)
        {
            Ok(f) => f,
            Err(e) => {
                eprintln!("Failed to open file: {}", e);
                return false;
            }
        };

        let mut writer = BufWriter::new(file);
        match serde_json::to_string_pretty(&self.data) {
            Ok(serialized) => {
                if let Err(e) = writeln!(writer, "{}", serialized) {
                    eprintln!("Failed to write data: {}", e);
                    return false;
                }
            }
            Err(e) => {
                eprintln!("Failed to serialize data: {}", e);
                return false;
            }
        }

        true
    }

    // Keep load unchanged
    pub fn load(&mut self) {
        if Path::new(&self.file_path).exists() {
            let data = read_to_string(&self.file_path).unwrap_or_default();
            if !data.trim().is_empty() {
                self.data = serde_json::from_str(&data).unwrap_or_default();
            }
        }
    }
}

impl NestedDatabaseMetaType {
    pub fn save(&self) -> bool {
        if let Some(parent) = Path::new(&self.file_path).parent() {
            if let Err(e) = create_dir_all(parent) {
                eprintln!("Failed to create directory: {}", e);
                return false;
            }
        }

        let file = match OpenOptions::new()
            .write(true)
            .truncate(true)
            .create(true)
            .open(&self.file_path)
        {
            Ok(f) => f,
            Err(e) => {
                eprintln!("Failed to open file: {}", e);
                return false;
            }
        };

        let serialized = match serde_json::to_string_pretty(&self.data) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("Failed to serialize data: {}", e);
                return false;
            }
        };

        let mut writer = BufWriter::new(file);
        if let Err(e) = writeln!(writer, "{}", serialized) {
            eprintln!("Failed to write data: {}", e);
            return false;
        }

        true
    }

    // Keep load unchanged
    pub fn load(&mut self) -> Result<(), String> {
        if !Path::new(&self.file_path).exists() {
            return Ok(());
        }

        let data = fs::read_to_string(&self.file_path)
            .map_err(|e| format!("Failed to read file: {}", e))?;

        if data.trim().is_empty() {
            return Ok(());
        }

        self.data = serde_json::from_str(&data)
            .map_err(|e| format!("Failed to deserialize data: {}", e))?;

        Ok(())
    }
}

