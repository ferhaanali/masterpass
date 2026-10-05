use super::{
    Database,
    core::{StringPath},
    entries::{
        IsTrashed,
        file_entry::{FileEntry, FileName,},
        password_entry::{PasswordName,},
    },
};

use std::{
        path::Path,
        fs::{remove_file},
};


impl Database {
 
    
    pub fn delete_file(&mut self, name: &str) -> bool {
        // Find the file entry in decrypted.data.files.0
        let entry_index = match self.meta.decrypted.data.files.0.iter().position(|f| f.display_name.0 == name) {
            Some(idx) => idx,
            None => {
                eprintln!("⚠️ File not found in database: {}", name);
                return false;
            }
        };

        let entry = &self.meta.decrypted.data.files.0[entry_index]; // borrow first (using .0)
        let path = Path::new(&entry.system_path.0); // using .0
        let dest_path = format!("{}/{}", self.directories.trash.files_dir.0, entry.system_name.0); // using .0 on dir and name

        if !path.exists() {
            eprintln!("⚠️ File does not exist on disk: {}", entry.system_path.0); // using .0
            return false;
        }

        if let Some(p) = path.to_str() {
            if !self.copy_file(p, &dest_path) {
                eprintln!("⚠️ Failed to move file to recycle bin");
                return false;
            }
        } else {
            eprintln!("⚠️ Invalid UTF-8 path: {}", entry.system_path.0); // using .0
            return false;
        }

        if let Err(e) = remove_file(path) {
            eprintln!("⚠️ Failed to delete original file: {}: {}", entry.system_path.0, e); // using .0
            return false;
        }

        // Remove from decrypted after successful copy
        let mut entry = self.meta.decrypted.data.files.0.remove(entry_index);
        entry.system_path = StringPath(dest_path); // Assuming StringPath wrapper
        entry.is_trashed = IsTrashed(true); // Assuming IsTrashed wrapper
        self.meta.trash.data.files.0.push(entry); // using .0

        let _ = (
            self.meta.trash.save(),
            self.meta.decrypted.save()
        );

        println!("♻️ File '{}' moved to recycle bin", name);
        true
    }

    pub fn delete_password(&mut self, name: &str) -> bool {
         let entry_index = match self.meta.decrypted.data.passwords.0.iter().position(|p| p.display_name.0 == name) { // using .0 and .0
            Some(idx) => idx,
            None => {
                eprintln!("⚠️ Password not found in database: {}", name);
                return false;
            }
        };

        let mut entry = self.meta.decrypted.data.passwords.0.remove(entry_index); // using .0
        entry.is_trashed = IsTrashed(true); // Assuming IsTrashed wrapper
        self.meta.trash.data.passwords.0.push(entry); // using .0
        
        // Save both metas
        let _ = (
            self.meta.trash.save(),
            self.meta.decrypted.save(),
        );
        println!("🗑️ Password '{}' moved to temp password file", name);
        true
    }

    pub fn delete_all_passwords(&mut self) -> bool {
        let mut passwords = std::mem::take(&mut self.meta.decrypted.data.passwords.0); // using .0
        for entry in &mut passwords {
            entry.is_trashed = IsTrashed(true); // Assuming IsTrashed wrapper
        }
        self.meta.trash.data.passwords.0.extend(passwords); // using .0
        let _ = (
            self.meta.decrypted.save(),
            self.meta.trash.save(),
        );
        println!("🗑️ All passwords moved to trash");
        true
    }
    
    pub fn delete_all_files(&mut self) -> bool {
        let drained: Vec<FileEntry> = self.meta.decrypted.data.files.0.drain(..).collect(); // using .0
        let mut still_meta: Vec<FileEntry> = vec![];
        
        for entry in drained {
            let file_path = Path::new(&entry.system_path.0); // using .0
        
            if !file_path.exists() {
                eprintln!("⚠️ File does not exist on disk: {}", entry.system_path.0); // using .0
                still_meta.push(entry);
                continue; // don't return early, just skip
            }
        
            // Ensure unique filename in recycle bin
            let file_name = match file_path.file_name().and_then(|n| n.to_str()) {
                Some(n) => n.to_string(),
                None => {
                    eprintln!("⚠️ Invalid file path: {}", entry.system_path.0); // using .0
                    still_meta.push(entry);
                    continue;
                }
            };
        
            let mut dest_path = Path::new(&self.directories.trash.files_dir.0).join(&file_name); // using .0
            if dest_path.exists() {
                let timestamp = chrono::Local::now().timestamp();
                let new_name = format!("{}_{}", timestamp, file_name);
                dest_path = Path::new(&self.directories.trash.files_dir.0).join(new_name); // using .0
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
                eprintln!("⚠️ Failed to delete original file: {}: {}", entry.system_path.0, e); // using .0
            }
        
            let mut new_entry = entry;
            new_entry.system_path = StringPath(dest_path.to_string_lossy().to_string()); // Assuming StringPath
            new_entry.is_trashed = IsTrashed(true); // Assuming IsTrashed
            self.meta.trash.data.files.0.push(new_entry); // using .0
        }
    
        // Restore skipped entries
        self.meta.decrypted.data.files.0.extend(still_meta); // using .0
    
        let _ = (
            self.meta.decrypted.save(),
            self.meta.trash.save()
        );
    
        println!("🗑️ All files moved to recycle bin");
        true
    }

    pub fn empty_recycle_bin(&mut self) -> (usize, usize) {
        let password_count = self.meta.trash.data.passwords.0.len(); // using .0
        let file_count = self.meta.trash.data.files.0.len(); // using .0

        // Delete files on disk
        for file_entry in &self.meta.trash.data.files.0 { // using .0
            let path = std::path::Path::new(&file_entry.system_path.0); // using .0
            if path.exists() {
                if let Err(e) = std::fs::remove_file(path) {
                    eprintln!("⚠️ Failed to delete file {}: {}", file_entry.system_path.0, e); // using .0
                }
            }
        }


        // Clear metadata
        self.meta.trash.data.passwords.0.clear(); // using .0
        self.meta.trash.data.files.0.clear(); // using .0

        // Save updated trash
        if self.meta.trash.save().is_err() {
            eprintln!("⚠️ Failed to save trash metadata after emptying recycle bin");
        }

        println!("🗑️ Emptied recycle bin: {} passwords, {} files", 
                 password_count, file_count);

        (password_count, file_count) // return deleted counts
    }

    pub fn restore_password(&mut self, name: &str) -> bool {
        if let Some(pos) = self.meta.trash.data.passwords.0.iter().position(|p| p.display_name.0 == name) { // using .0 and .0
            let mut entry = self.meta.trash.data.passwords.0.remove(pos); // using .0
            entry.is_trashed = IsTrashed(false); // Assuming IsTrashed
            entry.display_name = self.get_unique_password_name(&entry.display_name.0); // using .0
            self.meta.decrypted.data.passwords.0.push(entry); // using .0
            let _ = (
                self.meta.trash.save(),
                self.meta.decrypted.save(),
            );
            true
        } else {
            false // password not found
        }
    }

    pub fn restore_file(&mut self, name: &str) -> bool {
        // Find the file in trash
        let pos = match self.meta.trash.data.files.0.iter().position(|f| f.display_name.0 == name) { // using .0 and .0
            Some(p) => p,
            None => {
                eprintln!("⚠️ File '{}' not found in recycle bin.", name);
                return false;
            }
        };

        let mut entry = self.meta.trash.data.files.0.remove(pos); // remove from trash (using .0)

        // Determine subfolder based on extension
        let subfolder = self.get_sub_folder(&entry.extension.0); // using .0

        let src_path = Path::new(&entry.system_path.0); // using .0

        // Only generate a unique file name if conflict exists
        entry.system_name = if self.meta.decrypted.data.files.0.iter().any(|f| f.system_name.0 == entry.system_name.0) { // using .0 on collection and name fields
            FileName(self.get_unique_file_name(&entry.system_name.0)) // Assuming FileName wrapper, requires inner value of get_unique_file_name
        } else {
            entry.system_name.clone()
        };

        let dst_str = format!("{}/{}/{}", self.directories.decrypted.files_dir.0, subfolder, entry.system_name.0); // using .0 on dir and name
        let dst_path = Path::new(&dst_str);

        if let Some(parent) = dst_path.parent() {
            std::fs::create_dir_all(parent).expect("⚠️ Failed to create subfolder");
        }

        // Only generate a unique logical name if conflict exists
        entry.display_name = if self.meta.decrypted.data.files.0.iter().any(|f| f.display_name.0 == entry.display_name.0)
            || self.meta.encrypted.data.files.0.iter().any(|f| f.display_name.0 == entry.display_name.0) { // using .0 on collection and name fields
            FileName(self.get_unique_name_for_file(&entry.display_name.0).0) // Assuming PasswordName wrapper, requires inner value of get_unique_name_for_file
        } else {
            entry.display_name.clone()
        };

        // Copy file from recycle bin to decrypted folder
        let restored = if let (Some(src), Some(dst)) = (src_path.to_str(), dst_path.to_str()) {
            self.copy_file(src, dst)
        } else {
            false
        };

        if !restored {
            eprintln!("⚠️ Failed to restore file '{}' (file may not exist)", entry.system_name.0); // using .0
            return false;
        }

        entry.system_path = StringPath(dst_str); // Assuming StringPath
        entry.is_trashed = IsTrashed(false); // Assuming IsTrashed

        self.meta.decrypted.data.files.0.push(entry); // using .0

        // Save both metas
        let _ = ( 
            self.meta.trash.save(),
            self.meta.decrypted.save(),
        );

        println!("✅ Restored file: {}", name);
        true
    }
    
    pub fn restore_all_files(&mut self) -> bool {
        if self.meta.trash.data.files.0.is_empty() { // using .0
            println!("♻️ No files in recycle bin to restore.");
            return false;
        }

        let files_to_restore: Vec<FileEntry> = self.meta.trash.data.files.0.drain(..).collect(); // using .0
        let mut restored_count = 0;

        for mut entry in files_to_restore {
            let subfolder = self.get_sub_folder(&entry.extension.0); // using .0
            let src_path = Path::new(&entry.system_path.0); // using .0

            // Generate unique file_name if conflict exists
            entry.system_name = if self.meta.decrypted.data.files.0.iter().any(|f| f.system_name.0 == entry.system_name.0) { // using .0 on collection and name fields
                FileName(self.get_unique_file_name(&entry.system_name.0)) // Assuming FileName
            } else {
                entry.system_name.clone()
            };

            let dst_str = format!("{}/{}/{}", self.directories.decrypted.files_dir.0, subfolder, entry.system_name.0); // using .0 on dir and name
            let dst_path = Path::new(&dst_str);

            if let Some(parent) = dst_path.parent() {
                std::fs::create_dir_all(parent).expect("⚠️ Failed to create subfolder");
            }

            // Generate unique logical name
            entry.display_name = if self.meta.decrypted.data.files.0.iter().any(|f| f.display_name.0 == entry.display_name.0)
                || self.meta.encrypted.data.files.0.iter().any(|f| f.display_name.0 == entry.display_name.0) { // using .0 on collection and name fields
                FileName(self.get_unique_name_for_file(&entry.display_name.0).0) // Assuming PasswordName
            } else {
                entry.display_name.clone()
            };

            // Copy file from recycle bin to decrypted folder
            if let (Some(src), Some(dst)) = (src_path.to_str(), dst_path.to_str()) {
                if self.copy_file(src, dst) {
                    entry.system_path = StringPath(dst_str); // Assuming StringPath
                    entry.is_trashed = IsTrashed(false); // Assuming IsTrashed
                    self.meta.decrypted.data.files.0.push(entry); // using .0
                    restored_count += 1;
                } else {
                    eprintln!("⚠️ Failed to restore file: {}", entry.system_name.0); // using .0
                }
            }
        }

        let _ = (
            self.meta.decrypted.save(),
            self.meta.trash.save(),
        );

        println!("✅ Restored {} file(s) from recycle bin.", restored_count);
        true
    }

    pub fn restore_all_passwords(&mut self) -> bool {
        if self.meta.trash.data.passwords.0.is_empty() { // using .0
            println!("♻️ No passwords in recycle bin to restore.");
            return false;
        }

        let passwords_to_restore = self.meta.trash.data.passwords.0.drain(..).collect::<Vec<_>>(); // using .0
        let mut restored_count = 0;

        for mut entry in passwords_to_restore {
            entry.is_trashed = IsTrashed(false); // Assuming IsTrashed
            entry.display_name = self.get_unique_password_name(&entry.display_name.0); // using .0
            self.meta.decrypted.data.passwords.0.push(entry); // using .0
            restored_count += 1;
        }

        let _ = (
            self.meta.decrypted.save(),
            self.meta.trash.save(),
        );

        println!("✅ Restored {} password(s) from recycle bin.", restored_count);
        true
    }

    // Helper function
    pub fn get_unique_password_name(&self, name: &str) -> PasswordName { // returning PasswordName wrapper
        let mut counter = 0;
    
        loop {
            let current_name = if counter == 0 {
                name.to_string()
            } else {
                format!("{}{}", name, counter)
            };
        
            let exists_in_decrypted = self.meta.decrypted.data.passwords.0.iter().any(|entry| entry.display_name.0 == current_name); // using .0
            let exists_in_encrypted = self.meta.encrypted.data.passwords.0.iter().any(|entry| entry.display_name.0 == current_name); // using .0
        
            if !exists_in_decrypted && !exists_in_encrypted {
                return PasswordName(current_name); // returning wrapper
            }
        
            counter += 1;
        }
    }

    pub fn get_unique_name_for_file(&self, name: &str) -> PasswordName { // returning PasswordName wrapper
        let mut counter = 0;

        loop {
            let current_name = if counter == 0 {
                name.to_string()
            } else {
                format!("{}{}", name, counter)
            };

            let exists_in_files = self.meta.decrypted.data.files.0.iter().any(|entry| entry.display_name.0 == current_name); // using .0
            let exists_in_encrypted_files = self.meta.encrypted.data.files.0.iter().any(|entry| entry.display_name.0 == current_name); // using .0

            if !exists_in_files && !exists_in_encrypted_files {
                return PasswordName(current_name); // returning wrapper
            }

            counter += 1;
        }
    }

    // ... (rest of the methods, like the helper functions, have been adjusted above)
}