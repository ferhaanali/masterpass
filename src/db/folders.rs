use super::{
    structs::{Database, FolderEntry},
    std::{
        io,
        io::{Write,Read},
        fs,
        fs::{File, create_dir_all, remove_dir_all, },
        path::{Path},
    },
    time,
    enc_keys::{generate_item_key, wrap_item_key, unwrap_item_key},
};





impl Database {
    pub fn create_folder(&mut self, name: &str) -> bool {
        // Validate folder name
        if !self.is_valid_folder_name(name) {
            eprintln!("❌ Invalid folder name '{}'!", name);
            return false;
        }

        // Early return if folder already exists in either location (by name)
        if self.meta.decrypted_meta.data.folders.iter().any(|n| n.name == name) 
            || self.meta.encrypted_meta.data.folders.iter().any(|n| n.name == name) {
            eprintln!("❌ Folder '{}' already exists!", name);
            return false;
        }

        let directory_path = format!("{}/{}", self.directories.decrypted_folders_dir, name);

        // Create directory with proper error handling
        if let Err(e) = create_dir_all(&directory_path) {
            eprintln!("❌ Failed to create directory '{}': {}", directory_path, e);
            return false;
        }

        // Generate and encrypt item key
        let encrypted_item_key = match wrap_item_key(&generate_item_key(), &self.master.key) {
            Some(k) => k,
            None => {
                eprintln!("❌ Failed to generate encrypted item key for folder: {}", name);
                return false;
            },
        };

        // Create folder entry
        let now = time::now();
        self.meta.decrypted_meta.data.folders.push(FolderEntry {
            name: name.to_string(),
            encrypted_item_key,
            folder_path: directory_path,
            sub_files: Vec::new(),
            sub_folders: Vec::new(),
            size: "0".to_string(),
            is_empty: true,
            is_encrypted: false,
            is_recycled: false,
            created_at: now,
            updated_at: "0".to_string(),
        });

        // Save metadata
        if !self.meta.decrypted_meta.save() {
            eprintln!("❌ Failed to save metadata for folder: {}", name);
            return false;
        }
        println!("✅ Created folder: {}", name);

        true
    }









    
    // Helper 
    fn is_valid_folder_name(&self, name: &str) -> bool {
        // Check for empty name or only whitespace
        if name.trim().is_empty() {
            eprintln!("❌ Folder name cannot be empty or whitespace only");
            return false;
        }

        // Check length limits (typical filesystem limits)
        if name.is_empty() || name.len() > 255 {
            eprintln!("❌ Folder name must be between 1 and 255 characters");
            return false;
        }

        // Check for reserved names (Windows and common systems)
        let reserved_names = [
            // "CON", "PRN", "AUX", "NUL",
            // "COM1", "COM2", "COM3", "COM4", "COM5", "COM6", "COM7", "COM8", "COM9",
            // "LPT1", "LPT2", "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8", "LPT9",
            "..", ".", ""
        ];

        let upper_name = name.to_uppercase();
        if reserved_names.contains(&upper_name.as_str()) {
            eprintln!("❌ '{}' is a reserved folder name", name);
            return false;
        }

        // Check for invalid characters
        let invalid_chars = ['/', '\\', ':', '*', '?', '"', '<', '>', '|'];
        if let Some(invalid_char) = name.chars().find(|c| invalid_chars.contains(c)) {
            eprintln!("❌ Folder name contains invalid character: '{}'", invalid_char);
            return false;
        }

        // Check for control characters (ASCII 0-31)
        if name.chars().any(|c| c.is_control()) {
            eprintln!("❌ Folder name cannot contain control characters");
            return false;
        }

        // Check for trailing spaces or periods (Windows restriction)
        if name.ends_with(' ') || name.ends_with('.') {
            eprintln!("❌ Folder name cannot end with a space or period");
            return false;
        }

        // Check for multiple consecutive spaces (can be confusing)
        if name.contains("  ") {
            eprintln!("❌ Folder name cannot contain multiple consecutive spaces");
            return false;
        }

        // Check that it's a single folder name (no path separators)
        if name.contains('/') || name.contains('\\') {
            eprintln!("❌ Folder name cannot contain path separators (/ or \\)");
            return false;
        }

        // Additional check: ensure it's not a path with multiple components
        if name.contains(std::path::MAIN_SEPARATOR) {
            eprintln!("❌ Folder name cannot be a path, must be a single name");
            return false;
        }

        true
    }

    pub fn copy_folder(&self, src_path: &str, dst_path: &str) -> bool {
        let src = Path::new(src_path);
        let dst = Path::new(dst_path);
        
        if !src.exists() {
            eprintln!("❌ Source folder '{}' does not exist!", src_path);
            return false;
        }

        if !src.is_dir() {
            eprintln!("❌ '{}' is not a directory!", src_path);
            return false;
        }

        // Create destination directory
        if let Err(e) = create_dir_all(dst) {
            eprintln!("❌ Failed to create destination directory: {}", e);
            return false;
        }

        let folder_name = src.file_name()
            .unwrap_or_default()
            .to_string_lossy();

        println!("📁 Copying folder '{}'...", folder_name);

        // Get total size for progress calculation
        let total_size = self.calculate_folder_size(src_path);
        let mut copied_size: u64 = 0;

        // Copy contents recursively
        if let Err(e) = self.copy_folder_contents(src, dst, total_size, &mut copied_size) {
            eprintln!("❌ Failed to copy folder: {}", e);
            return false;
        }

        println!("\r✅ Folder '{}' copied successfully! ({})", 
                 folder_name, 
                 self.format_file_size(total_size));
        true
    }

    fn calculate_folder_size(&self, path: &str) -> u64 {
        let mut total_size = 0;
        if let Ok(entries) = fs::read_dir(path) {
            for entry in entries.flatten() {
                let metadata = match entry.metadata() {
                    Ok(md) => md,
                    Err(_) => continue,
                };

                if metadata.is_dir() {
                    total_size += self.calculate_folder_size(entry.path().to_string_lossy().as_ref());
                } else {
                    total_size += metadata.len();
                }
            }
        }
        total_size
    }

    fn copy_folder_contents(&self, src: &Path, dst: &Path, total_size: u64, copied_size: &mut u64) -> io::Result<()> {
        let entries = fs::read_dir(src)?;

        for entry in entries {
            let entry = entry?;
            let file_type = entry.file_type()?;
            let file_name = entry.file_name();
            let dst_path = dst.join(&file_name);

            if file_type.is_dir() {
                // Create subdirectory
                fs::create_dir_all(&dst_path)?;
                // Recursively copy subdirectory contents
                self.copy_folder_contents(&entry.path(), &dst_path, total_size, copied_size)?;
            } else if file_type.is_file() {
                // Copy individual file with progress
                self.copy_file_with_progress(
                    entry.path().to_string_lossy().as_ref(),
                    dst_path.to_string_lossy().as_ref(),
                    total_size,
                    copied_size
                )?;
            }
        }

        Ok(())
    }

    fn copy_file_with_progress(&self, src_path: &str, dst_path: &str, total_size: u64, copied_size: &mut u64) -> io::Result<()> {
        let metadata = fs::metadata(src_path)?;
        let file_size = metadata.len();

        let mut src_file = File::open(src_path)?;
        let mut dst_file = File::create(dst_path)?;

        let mut buffer = [0u8; 8192];
        let mut file_copied_bytes: u64 = 0;

        let file_name = Path::new(src_path)
            .file_name()
            .unwrap_or_default()
            .to_string_lossy();

        loop {
            let n = src_file.read(&mut buffer)?;
            if n == 0 { break; }

            dst_file.write_all(&buffer[..n])?;
            file_copied_bytes += n as u64;
            *copied_size += n as u64;

            let progress = *copied_size as f64 / total_size as f64 * 100.0;
            print!("\r📁 Copying... {:.2}% | Current: {}", progress, file_name);
            io::stdout().flush()?;
        }

        dst_file.flush()?;
        Ok(())
    }

    fn format_file_size(&self, size: u64) -> String {
        const UNITS: [&str; 5] = ["B", "KB", "MB", "GB", "TB"];
        let mut size = size as f64;
        let mut unit_index = 0;

        while size >= 1024.0 && unit_index < UNITS.len() - 1 {
            size /= 1024.0;
            unit_index += 1;
        }

        format!("{:.2} {}", size, UNITS[unit_index])
    } 

    // fn find_unique_folder_name(&self, folder_name: &str) -> String {
    //     let mut counter = 1;
    //     let mut unique_folder_name = folder_name.to_string();

    //     // Extract file extension and base name
    //     let (base_name, extension) = if let Some(dot_pos) = folder_name.rfind('.') {
    //         (&folder_name[..dot_pos], &folder_name[dot_pos..])
    //     } else {
    //         (folder_name, "")
    //     };

    //     // Check if folder_name already exists
    //     while self.meta.decrypted_meta.data.folders.iter().any(|n| n.folder_name == unique_folder_name) 
    //         || self.meta.encrypted_meta.data.folders.iter().any(|n| n.folder_name == unique_folder_name) {
            
    //         unique_folder_name = if extension.is_empty() {
    //             format!("{}_{}", base_name, counter)
    //         } else {
    //             format!("{}_{}{}", base_name, counter, extension)
    //         };
    //         counter += 1;
    //     }

    //     unique_folder_name
    // }
}