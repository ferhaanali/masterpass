use super::{
    structs::{Database, FileEntry},
    std::{
        path::Path,
        fs::{ remove_file, remove_dir_all, create_dir_all },
        io::{stdout, Write},
    },
};



impl Database {

    pub fn delete_file(&mut self, name: &str) -> bool {
        // Find the file entry in meta
        let entry_index = match self.meta.decrypted_meta.data.files.iter().position(|f| f.name == name) {
            Some(idx) => idx,
            None => {
                eprintln!("⚠️ File not found in database: {}", name);
                return false;
            }
        };

        let entry = &self.meta.decrypted_meta.data.files[entry_index]; // borrow first
        let path = Path::new(&entry.file_path);
        let dest_path = format!("{}/{}", self.directories.recycle_files_dir, entry.file_name);

        if !path.exists() {
            eprintln!("⚠️ File does not exist on disk: {}", entry.file_path);
            return false;
        }

        if let Some(p) = path.to_str() {
            if !self.copy_file(p, &dest_path) {
                eprintln!("⚠️ Failed to move file to recycle bin");
                return false;
            }
        } else {
            eprintln!("⚠️ Invalid UTF-8 path: {}", entry.file_path);
            return false;
        }

        if let Err(e) = remove_file(path) {
            eprintln!("⚠️ Failed to delete original file: {}: {}", entry.file_path, e);
            return false;
        }

        // Remove from meta after successful copy
        let mut entry = self.meta.decrypted_meta.data.files.remove(entry_index);
        entry.file_path = dest_path;
        entry.is_recycled = true;
        self.meta.trash_meta.data.files.push(entry);

        self.meta.trash_meta.save();
        self.meta.decrypted_meta.save();

        println!("♻️ File '{}' moved to recycle bin", name);
        true
    }

    pub fn delete_password(&mut self, name: &str) -> bool {
         let entry_index = match self.meta.decrypted_meta.data.passwords.iter().position(|p| p.name == name) {
            Some(idx) => idx,
            None => {
                eprintln!("⚠️ Password not found in database: {}", name);
                return false;
            }
        };

        let entry = self.meta.decrypted_meta.data.passwords.remove(entry_index);
        self.meta.trash_meta.data.passwords.push(entry);
        // Save both metas
        self.meta.trash_meta.save();
        self.meta.decrypted_meta.save();
        println!("🗑️ Password '{}' moved to temp password file", name);
        true
    }

    pub fn delete_all_passwords(&mut self) -> bool {
        let mut passwords = std::mem::take(&mut self.meta.decrypted_meta.data.passwords);
        for entry in &mut passwords {
            entry.is_recycled = true;
        }
        self.meta.trash_meta.data.passwords.extend(passwords);
        self.meta.decrypted_meta.save();
        self.meta.trash_meta.save();
        println!("🗑️ All passwords moved to trash");
        true
    }
    
    pub fn delete_all_files(&mut self) -> bool {
        let drained: Vec<FileEntry> = self.meta.decrypted_meta.data.files.drain(..).collect();
        let mut still_meta: Vec<FileEntry> = vec![];
        
        for entry in drained {
            let file_path = Path::new(&entry.file_path);
        
            if !file_path.exists() {
                eprintln!("⚠️ File does not exist on disk: {}", entry.file_path);
                still_meta.push(entry);
                continue; // don't return early, just skip
            }
        
            // Ensure unique filename in recycle bin
            let file_name = match file_path.file_name().and_then(|n| n.to_str()) {
                Some(n) => n.to_string(),
                None => {
                    eprintln!("⚠️ Invalid file path: {}", entry.file_path);
                    still_meta.push(entry);
                    continue;
                }
            };
        
            let mut dest_path = Path::new(&self.directories.recycle_files_dir).join(&file_name);
            if dest_path.exists() {
                let timestamp = chrono::Local::now().timestamp();
                let new_name = format!("{}_{}", timestamp, file_name);
                dest_path = Path::new(&self.directories.recycle_files_dir).join(new_name);
            }
        
            if let (Some(src), Some(dst)) = (file_path.to_str(), dest_path.to_str()) {
                if !self.copy_file(src, dst) {
                    eprintln!("⚠️ Failed to move file to recycle bin");
                    still_meta.push(entry);
                    continue;
                }
            } else {
                eprintln!("⚠️ Path contains invalid UTF-8");
                still_meta.push(entry);
                continue;
            }
        
            if let Err(e) = remove_file(&file_path) {
                eprintln!("⚠️ Failed to delete original file: {}: {}", entry.file_path, e);
            }
        
            let mut new_entry = entry;
            new_entry.file_path = dest_path.to_string_lossy().to_string();
            new_entry.is_recycled = true;
            self.meta.trash_meta.data.files.push(new_entry);
        }
    
        // Restore skipped entries
        self.meta.decrypted_meta.data.files.extend(still_meta);
    
        self.meta.decrypted_meta.save();
        self.meta.trash_meta.save();
    
        println!("🗑️ All files moved to recycle bin");
        true
    }

    pub fn empty_recycle_bin(&mut self) -> (usize, usize, usize) {
        let password_count = self.meta.trash_meta.data.passwords.len();
        let file_count = self.meta.trash_meta.data.files.len();
        let folder_count = self.meta.trash_meta.data.folders.len();

        // Delete files on disk
        for file_entry in &self.meta.trash_meta.data.files {
            let path = std::path::Path::new(&file_entry.file_path);
            if path.exists() {
                if let Err(e) = std::fs::remove_file(path) {
                    eprintln!("⚠️ Failed to delete file {}: {}", file_entry.file_path, e);
                }
            }
        }

        // Delete folders on disk
        for folder_entry in &self.meta.trash_meta.data.folders {
            let path = std::path::Path::new(&folder_entry.folder_path);
            if path.exists() && path.is_dir() {
                if let Err(e) = remove_dir_all(path) {
                    eprintln!("⚠️ Failed to delete folder {}: {}", folder_entry.folder_path, e);
                }
            }
        }

        // Clear metadata
        self.meta.trash_meta.data.passwords.clear();
        self.meta.trash_meta.data.files.clear();
        self.meta.trash_meta.data.folders.clear();

        // Save updated trash_meta
        if !self.meta.trash_meta.save() {
            eprintln!("⚠️ Failed to save trash metadata after emptying recycle bin");
        }

        println!("🗑️ Emptied recycle bin: {} passwords, {} files, {} folders deleted", 
                 password_count, file_count, folder_count);

        (password_count, file_count, folder_count) // return deleted counts
    }

    pub fn restore_password(&mut self, name: &str) -> bool {
        if let Some(pos) = self.meta.trash_meta.data.passwords.iter().position(|p| p.name == name) {
            let mut entry = self.meta.trash_meta.data.passwords.remove(pos);
            entry.is_recycled = false;
            entry.name = self.get_unique_password_name(&entry.name);
            self.meta.decrypted_meta.data.passwords.push(entry); // move back to active passwords
            self.meta.trash_meta.save();
            self.meta.decrypted_meta.save();
            true
        } else {
            false // password not found
        }
    }

    pub fn restore_file(&mut self, name: &str) -> bool {
        // Find the file in trash
        let pos = match self.meta.trash_meta.data.files.iter().position(|f| f.name == name) {
            Some(p) => p,
            None => {
                eprintln!("⚠️ File '{}' not found in recycle bin.", name);
                return false;
            }
        };

        let mut entry = self.meta.trash_meta.data.files.remove(pos); // remove from trash_meta

        // Determine subfolder based on extension
        let subfolder = self.get_sub_folder(&entry.extension.as_str());

        let src_path = Path::new(&entry.file_path);

        // Only generate a unique file name if conflict exists
        entry.file_name = if self.meta.decrypted_meta.data.files.iter().any(|f| f.file_name == entry.file_name) {
            self.get_unique_file_name(&entry.file_name)
        } else {
            entry.file_name.clone()
        };

        let dst_str = format!("{}/{}/{}", self.directories.decrypted_files_dir, subfolder, entry.file_name);
        let dst_path = Path::new(&dst_str);

        if let Some(parent) = dst_path.parent() {
            std::fs::create_dir_all(parent).expect("⚠️ Failed to create subfolder");
        }

        // Only generate a unique logical name if conflict exists
        entry.name = if self.meta.decrypted_meta.data.files.iter().any(|f| f.name == entry.name)
            || self.meta.encrypted_meta.data.files.iter().any(|f| f.name == entry.name) {
            self.get_unique_name_for_file(&entry.name)
        } else {
            entry.name.clone()
        };

        // Copy file from recycle bin to decrypted folder
        let restored = if let (Some(src), Some(dst)) = (src_path.to_str(), dst_path.to_str()) {
            self.copy_file(src, dst)
        } else {
            false
        };

        if !restored {
            eprintln!("⚠️ Failed to restore file '{}' (file may not exist)", entry.file_name);
            return false;
        }

        entry.file_path = dst_str;
        entry.is_recycled = false;

        self.meta.decrypted_meta.data.files.push(entry);

        // Save both metas
        self.meta.trash_meta.save();
        self.meta.decrypted_meta.save();

        println!("✅ Restored file: {}", name);
        true
    }
    
    pub fn restore_all_files(&mut self) -> bool {
        if self.meta.trash_meta.data.files.is_empty() {
            println!("♻️ No files in recycle bin to restore.");
            return false;
        }

        let files_to_restore: Vec<FileEntry> = self.meta.trash_meta.data.files.drain(..).collect();
        let mut restored_count = 0;

        for mut entry in files_to_restore {
            let subfolder = self.get_sub_folder(&entry.extension);
            let src_path = Path::new(&entry.file_path);

            // Generate unique file_name if conflict exists
            entry.file_name = if self.meta.decrypted_meta.data.files.iter().any(|f| f.file_name == entry.file_name) {
                self.get_unique_file_name(&entry.file_name)
            } else {
                entry.file_name.clone()
            };

            let dst_str = format!("{}/{}/{}", self.directories.decrypted_files_dir, subfolder, entry.file_name);
            let dst_path = Path::new(&dst_str);

            if let Some(parent) = dst_path.parent() {
                std::fs::create_dir_all(parent).expect("⚠️ Failed to create subfolder");
            }

            // Generate unique logical name
            entry.name = if self.meta.decrypted_meta.data.files.iter().any(|f| f.name == entry.name)
                || self.meta.encrypted_meta.data.files.iter().any(|f| f.name == entry.name) {
                self.get_unique_name_for_file(&entry.name)
            } else {
                entry.name.clone()
            };

            // Copy file from recycle bin to decrypted folder
            if let (Some(src), Some(dst)) = (src_path.to_str(), dst_path.to_str()) {
                if self.copy_file(src, dst) {
                    entry.file_path = dst_str;
                    entry.is_recycled = false;
                    self.meta.decrypted_meta.data.files.push(entry);
                    restored_count += 1;
                } else {
                    eprintln!("⚠️ Failed to restore file: {}", entry.file_name);
                }
            }
        }

        self.meta.decrypted_meta.save();
        self.meta.trash_meta.save();

        println!("✅ Restored {} file(s) from recycle bin.", restored_count);
        true
    }

    pub fn restore_all_passwords(&mut self) -> bool {
        if self.meta.trash_meta.data.passwords.is_empty() {
            println!("♻️ No passwords in recycle bin to restore.");
            return false;
        }

        let passwords_to_restore = self.meta.trash_meta.data.passwords.drain(..).collect::<Vec<_>>();
        let mut restored_count = 0;

        for mut entry in passwords_to_restore {
            entry.is_recycled = false;
            entry.name = self.get_unique_password_name(&entry.name);
            self.meta.decrypted_meta.data.passwords.push(entry);
            restored_count += 1;
        }

        self.meta.decrypted_meta.save();
        self.meta.trash_meta.save();

        println!("✅ Restored {} password(s) from recycle bin.", restored_count);
        true
    }



    // Folders
    pub fn delete_folder(&mut self, name: &str, force: bool) -> bool {
        
        let folder_index = self.meta.decrypted_meta.data.folders.iter()
        .position(|n| n.name == name);

        let folder = match folder_index {
            Some(index) => self.meta.decrypted_meta.data.folders[index].clone(),
            None => {
                eprintln!("❌ Folder '{}' not found!", name);
                return false;
            }
        };

        // Check if folder is empty (unless force delete)
        if !folder.is_empty && !force {
            eprintln!("❌ Folder '{}' is not empty! Use delete_folder_force() or delete contents first.", name);
            return false;
        }

        let source_path = &folder.folder_path;
        let trash_path = format!("{}/{}", self.directories.recycle_folders_dir, folder.name);

        // Create trash directory if it doesn't exist
        if let Err(e) = create_dir_all(&self.directories.recycle_folders_dir) {
            eprintln!("❌ Failed to create trash directory: {}", e);
            return false;
        }

        // Use the copy_folder function to move (copy + delete original)
        if !self.copy_folder(source_path, &trash_path) {
            eprintln!("❌ Failed to copy folder to trash: {}", name);
            return false;
        }

        // After successful copy, delete the original
        if let Err(e) = remove_dir_all(source_path) {
            eprintln!("❌ Failed to remove original folder: {}", e);
            // Clean up the copied version if original deletion fails
            let _ = remove_dir_all(&trash_path);
            return false;
        }

        self.meta.decrypted_meta.data.folders.remove(folder_index.unwrap());
        self.meta.trash_meta.data.folders.push(folder.clone());

        if !self.meta.decrypted_meta.save() || !self.meta.trash_meta.save() {
            eprintln!("❌ Failed to save metadata after moving folder to trash: {}", name);
            return false;
        }

        println!("🗑️ Moved folder '{}' to recycle bin", name);
        true
    }

    // Alternative version that shows progress with folder names
    pub fn delete_all_folders_with_progress(&mut self, force: bool) -> bool {
        let folders = self.meta.decrypted_meta.data.folders.clone(); // Clone to avoid borrowing issues
        let total_folders = folders.len();

        if total_folders == 0 {
            println!("ℹ️ No folders to delete.");
            return true;
        }

        println!("🗑️ Deleting {} folders...", total_folders);

        let mut success_count = 0;
        let mut fail_count = 0;

        for (index, folder) in folders.iter().enumerate() {
            let progress = (index + 1) as f64 / total_folders as f64 * 100.0;
            print!("\r🗑️ Progress: {:.1}% | Deleting: '{}'...", progress, folder.name);
            stdout().flush().unwrap();

            if self.delete_folder(&folder.name, force) {
                success_count += 1;
            } else {
                fail_count += 1;
                eprintln!("\r❌ Failed to delete folder '{}'", folder.name);
            }
        }

        println!("\r✅ Completed: {}/{} folders deleted successfully", success_count, total_folders);

        if fail_count > 0 {
            eprintln!("❌ {} folders failed to delete", fail_count);
        }

        // Save metadata
        if !self.meta.decrypted_meta.save() {
            eprintln!("❌ Failed to save metadata after deleting all folders");
            return false;
        }

        fail_count == 0
    }

    // Delete only empty folders
    pub fn delete_all_empty_folders(&mut self) -> bool {
        let empty_folders: Vec<String> = self.meta.decrypted_meta.data.folders
            .iter()
            .filter(|folder| folder.is_empty)
            .map(|folder| folder.name.clone())
            .collect();

        if empty_folders.is_empty() {
            println!("ℹ️ No empty folders to delete.");
            return true;
        }

        println!("🗑️ Deleting {} empty folders...", empty_folders.len());

        let mut success_count = 0;
        for folder_name in empty_folders {
            if self.delete_folder(&folder_name, false) {
                success_count += 1;
            } else {
                eprintln!("❌ Failed to delete empty folder '{}'", folder_name);
            }
        }

        println!("✅ Deleted {} empty folders", success_count);
        
        if !self.meta.decrypted_meta.save() {
            eprintln!("❌ Failed to save metadata after deleting empty folders");
            return false;
        }

        true
    }

    












    // Helper function
    pub fn get_unique_password_name(&self, name: &str) -> String {
        let mut counter = 0;
    
        loop {
            let current_name = if counter == 0 {
                name.to_string()
            } else {
                format!("{}{}", name, counter)
            };
        
            let exists_in_decrypted = self.meta.decrypted_meta.data.passwords.iter().any(|entry| entry.name == current_name);
            let exists_in_encrypted = self.meta.encrypted_meta.data.passwords.iter().any(|entry| entry.name == current_name);
        
            if !exists_in_decrypted && !exists_in_encrypted {
                return current_name;
            }
        
            counter += 1;
        }
    }

    pub fn get_unique_name_for_file(&self, name: &str) -> String {
        let mut counter = 0;

        loop {
            let current_name = if counter == 0 {
                name.to_string()
            } else {
                format!("{}{}", name, counter)
            };

            let exists_in_files = self.meta.decrypted_meta.data.files.iter().any(|entry| entry.name == current_name);
            let exists_in_encrypted_files = self.meta.encrypted_meta.data.files.iter().any(|entry| entry.name == current_name);

            if !exists_in_files && !exists_in_encrypted_files {
                return current_name;
            }

            counter += 1;
        }
    }

    
}
