use super::{
    Database,
    entries::{
        file_entry::{
            FileEntry,
            FileName,           
            FileExtension,                 
        },
    },
    core::{
        Timestamp,
        StringPath,
        DataSize,
    },
    entries::{
        ItemId,
        IsTrashed,
        IsEncrypted,
    },
    crypto::{
        ItemKey,
    },
};
use std::{
    path::{Path},
    fs::{remove_file, create_dir_all, self, metadata, File},
    io::{BufWriter, Write, Read, stdout},
};

impl Database {

    // --- Add File Operations ---

    pub fn cut_add_file(&mut self, name: &str, file_path: &str) -> bool {
        let path = Path::new(file_path);

        // ✅ Ensure source path exists
        if !path.exists() {
            println!("❌ Source path does not exist: {}", file_path);
            return false;
        }

        // ✅ Reject folders
        if !path.is_file() {
            println!("❌ '{}' is a directory, only files can be added", file_path);
            return false;
        }

        // Detect extension
        let extension_str = path
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("unknown")
            .to_lowercase();
        let extension = FileExtension(extension_str.clone());

        // Select subfolder based on extension
        let subfolder = self.get_sub_folder(&extension.0);

        // Ensure subfolder exists
        let target_dir = format!("{}/{}", self.directories.decrypted.files_dir.0, subfolder);
        if !Path::new(&target_dir).exists() {
            fs::create_dir_all(&target_dir).expect("⚠️ Failed to create subfolder");
        }

        // Destination path
        let temp_file_name = path.file_name().unwrap().to_str().unwrap();
        let file_name = self.get_unique_file_name(temp_file_name);
        let dest_path = format!("{}/{}", target_dir, file_name);

        // Check size limit
        let metadata = fs::metadata(path).expect("⚠️ Failed to read file metadata");
        let file_size_bytes = metadata.len();
        let file_size_mb = file_size_bytes as f64 / (1024.0 * 1024.0);
        let max_size_bytes = (self.config.max_file_size_mb * 1024 * 1024) as u64;

        if file_size_bytes > max_size_bytes {
            println!(
                "❌ File '{}' exceeds max size of {} MB",
                file_name, self.config.max_file_size_mb
            );
            return false;
        }

        // Prevent duplicates
        if self.meta.decrypted.data.files.0.iter().any(|f| f.display_name.0 == name) {
            println!("❌ File with name '{}' already exists in meta!", name);
            return false;
        }
        if self.meta.encrypted.data.files.0.iter().any(|f| f.display_name.0 == name) {
            println!("❌ File with name '{}' already exists in encrypted files!", name);
            return false;
        }

        let item_id = ItemId::new();
        let encrypted_item_key = match self.crypto.wrap_item_key(&ItemKey::generate_vec_key()) {
            Ok(eik_str) => ItemKey(eik_str),
            Err(e) => {
                println!("❌ Failed to generate encrypted item key for file: {}: {}", name, e);
                return false;
            }
        };

        // Try restore if missing
        if let Some(index) = self.meta.decrypted.data.files.0.iter().position(|f| f.display_name.0 == name) {
            let file_missing = !Path::new(&self.meta.decrypted.data.files.0[index].system_path.0).exists();
                
            if file_missing {
                // Now safely borrow mutably after copy is done
                let entry = &mut self.meta.decrypted.data.files.0[index];
                entry.id = item_id.clone();
                entry.system_path = StringPath(dest_path.clone());
                entry.system_name = FileName(file_name.to_string());
                entry.size = DataSize(file_size_bytes as f64);
                entry.extension = extension.clone();
                entry.is_trashed = IsTrashed(false);
                entry.created_at = Timestamp::now();
                entry.updated_at = Timestamp::now();

                // Store the key
                self.crypto.item_key_store.set_key(item_id.clone(), encrypted_item_key);

                if let Some(src) = path.to_str() {
                    if !self.copy_file(src, dest_path.as_str()) {
                        println!("Failed to copy file!");
                        return false;
                    } else {
                        let _ = (
                            self.meta.decrypted.save(),
                            self.crypto.item_key_store.save(),
                        );
                    } 
                } else {
                    println!("Invalid path (not UTF-8)!");
                    return false;
                }
            
                println!("♻️ Restored missing file for '{}'", name);
                return true;
            }
        }

        self.meta.decrypted.data.files.0.push(FileEntry {
            id: item_id.clone(),
            display_name: FileName(name.to_string()),
            system_name: FileName(file_name.to_string()),
            system_path: StringPath(dest_path.clone()),
            size: DataSize(file_size_mb as f64),
            extension: extension.clone(),
            is_trashed: IsTrashed(false),
            is_encrypted: IsEncrypted(false),
            created_at: Timestamp::now(),
            updated_at: Timestamp::now(),
        });

        // Store the key using item_key_store
        self.crypto.item_key_store.set_key(item_id, encrypted_item_key);

        if let Some(src) = path.to_str() {
            if !self.copy_file(src, dest_path.as_str()) {
                println!("Failed to copy file!");
                return false;
            } else {
                let _ = (
                    self.meta.decrypted.save(),
                    self.crypto.item_key_store.save(),
                );
            }
            if let Err(e) = remove_file(src) {
                println!("❌ Failed to delete original file: {}", e);
                return false;
            }
        } else {
            println!("❌ Invalid path (not UTF-8)!");
            return false;
        }

        println!(
            "✅ File added: {} (.{}) size <{} MB> (..cutted..)",
            name, extension.0, file_size_mb
        );
        true
    }

    pub fn add_file(&mut self, name: &str, file_path: &str) -> bool {
        let path = Path::new(file_path);

        // ✅ Ensure source path exists
        if !path.exists() {
            println!("❌ Source path does not exist: {}", file_path);
            return false;
        }

        // ✅ Reject folders
        if !path.is_file() {
            println!("❌ '{}' is a directory, only files can be added", file_path);
            return false;
        }

        // Detect extension
        let extension_str = path
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("unknown")
            .to_lowercase();
        let extension = FileExtension(extension_str.clone());

        // Select subfolder based on extension
        let subfolder = self.get_sub_folder(&extension.0);

        // Ensure subfolder exists
        let target_dir = format!("{}/{}", self.directories.decrypted.files_dir.0, subfolder);
        if !Path::new(&target_dir).exists() {
            fs::create_dir_all(&target_dir).expect("⚠️ Failed to create subfolder");
        }

        // Destination path
        let temp_file_name = path.file_name().unwrap().to_str().unwrap();
        let file_name = self.get_unique_file_name(temp_file_name);
        let dest_path = format!("{}/{}", target_dir, file_name);

        // Check size limit
        let metadata = fs::metadata(path).expect("⚠️ Failed to read file metadata");
        let file_size_bytes = metadata.len();
        let file_size_mb = file_size_bytes as f64 / (1024.0 * 1024.0);
        let max_size_bytes = (self.config.max_file_size_mb * 1024 * 1024) as u64;

        if file_size_bytes > max_size_bytes {
            println!(
                "❌ File '{}' exceeds max size of {} MB",
                file_name, self.config.max_file_size_mb
            );
            return false;
        }

        // Prevent duplicates
        if self.meta.decrypted.data.files.0.iter().any(|f| f.display_name.0 == name) {
            println!("❌ File with name '{}' already exists in meta!", name);
            return false;
        }
        if self.meta.encrypted.data.files.0.iter().any(|f| f.display_name.0 == name) {
            println!("❌ File with name '{}' already exists in encrypted files!", name);
            return false;
        }

        let item_id = ItemId::new();
        let encrypted_item_key = match self.crypto.wrap_item_key(&ItemKey::generate_vec_key()) {
            Ok(eik_str) => ItemKey(eik_str),
            Err(e) => {
                println!("❌ Failed to generate encrypted item key for file: {}: {}", name, e);
                return false;
            }
        };

        // Try restore if missing
        if let Some(index) = self.meta.decrypted.data.files.0.iter().position(|f| f.display_name.0 == name) {
            let file_missing = !Path::new(&self.meta.decrypted.data.files.0[index].system_path.0).exists();
                
            if file_missing {
                // Now safely borrow mutably after copy is done
                let entry = &mut self.meta.decrypted.data.files.0[index];
                entry.id = item_id.clone();
                entry.system_path = StringPath(dest_path.clone());
                entry.system_name = FileName(file_name.to_string());
                entry.size = DataSize(file_size_mb as f64);
                entry.extension = extension.clone();
                entry.is_trashed = IsTrashed(false);
                entry.created_at = Timestamp::now();
                entry.updated_at = Timestamp::now();

                // Store the key
                self.crypto.item_key_store.set_key(item_id.clone(), encrypted_item_key);

                if let Some(src) = path.to_str() {
                    if !self.copy_file(src, dest_path.as_str()) {
                        println!("Failed to copy file!");
                        return false;
                    } else {
                        let _ = (
                            self.meta.decrypted.save(),
                            self.crypto.item_key_store.save(),
                        );
                    } 
                } else {
                    println!("Invalid path (not UTF-8)!");
                    return false;
                }
            
                println!("♻️ Restored missing file for '{}'", name);
                return true;
            }
        }

        self.meta.decrypted.data.files.0.push(FileEntry {
            id: item_id.clone(),
            display_name: FileName(name.to_string()),
            system_name: FileName(file_name.to_string()),
            system_path: StringPath(dest_path.clone()),
            size: DataSize(file_size_mb as f64),
            extension: extension.clone(),
            is_trashed: IsTrashed(false),
            is_encrypted: IsEncrypted(false),
            created_at: Timestamp::now(),
            updated_at: Timestamp::now(),
        });

        // Store the key using item_key_store
        self.crypto.item_key_store.set_key(item_id, encrypted_item_key);

        if let Some(src) = path.to_str() {
            if !self.copy_file(src, dest_path.as_str()) {
                println!("Failed to copy file!");
                return false;
            } else {
                let _ = ( 
                    self.meta.decrypted.save(),
                    self.crypto.item_key_store.save(),
                );
            }
        } else {
            println!("Invalid path (not UTF-8)!");
            return false;
        }

        println!(
            "✅ File added: {} (.{}) size <{} MB> (..copied..)",
            name, extension.0, file_size_mb
        );
        true
    }

    // --- Updated encrypt/decrypt operations ---

    pub fn encrypt_all_files(&mut self) -> bool {
        let mut success_count = 0;
        let mut failure_count = 0;

        // Collect files that need encryption
        let files_to_encrypt: Vec<FileEntry> = self.meta.decrypted.data.files.0
            .iter()
            .filter(|f| !f.is_encrypted.0)
            .cloned()
            .collect();

        let total_to_process = files_to_encrypt.len();
        
        if total_to_process == 0 {
            println!("ℹ️  No unencrypted files found to encrypt.");
            return true;
        }

        println!("🔒 Encrypting {} files:", total_to_process);

        for (current, entry) in files_to_encrypt.iter().enumerate() {
            print!("  {}/{}: {}... ", current + 1, total_to_process, entry.display_name.0);
            
            // Check if file exists
            if !Path::new(&entry.system_path.0).exists() {
                println!("❌ (file missing)");
                failure_count += 1;
                continue;
            }

            // Encrypted file path
            let encrypted_path = format!("{}/{}.enc", self.directories.encrypted.files_dir.0, entry.system_name.0);

            // Get item key from store
            let key = match self.crypto.item_key_store.get_key(&entry.id) {
                Some(item_key) => match self.crypto.unwrap_item_key(&item_key.0) {
                    Ok(k) => k,
                    Err(e) => {
                        println!("❌ (key error: {})", e);
                        failure_count += 1;
                        continue;
                    }
                },
                None => {
                    println!("❌ (key not found in store)");
                    failure_count += 1;
                    continue;
                }
            };

            // Encrypt the file
            let encrypt_result = self.crypto.encrypt_file_data(&entry.system_path.0, &encrypted_path, &key);
            
            if let Err(e) = encrypt_result {
                if Path::new(&encrypted_path).exists() {
                    let _ = remove_file(&encrypted_path); // rollback
                }
                println!("❌ (encryption failed: {})", e);
                failure_count += 1;
                continue;
            }

            // Add entry to encrypted_meta
            self.meta.encrypted.data.files.0.push(FileEntry {
                id: entry.id.clone(), // Keep the same ID
                display_name: entry.display_name.clone(),
                system_name: entry.system_name.clone(),
                system_path: StringPath(encrypted_path.clone()),
                size: entry.size.clone(),
                extension: entry.extension.clone(),
                is_encrypted: IsEncrypted(true),
                is_trashed: entry.is_trashed.clone(), 
                created_at: entry.created_at.clone(),
                updated_at: Timestamp::now(),
            });

            // Remove from meta
            self.meta.decrypted.data.files.0.retain(|f| f.id != entry.id);

            // Delete original file
            if let Err(e) = remove_file(&entry.system_path.0) {
                eprintln!("⚠️ Failed to delete original file: {}", e);
            }

            success_count += 1;
        }

        // Save metadata if we had successful operations
        if success_count > 0 {
            let _ = ( 
                self.meta.encrypted.save(),
                self.meta.decrypted.save()
            );
        }

        // Log results
        println!("\n📊 Encryption Summary:");
        println!("   Total:    {}", total_to_process);
        println!("   Success:  {}", success_count);
        println!("   Failed:   {}", failure_count);
        
        if success_count > 0 && failure_count == 0 {
            println!("✅ Successfully encrypted all {} files.", success_count);
        } else if success_count > 0 {
            println!("⚠️  Encrypted {}/{} files. {} failed.", success_count, total_to_process, failure_count);
        } else {
            println!("❌ Failed to encrypt any files. {}/{} failed.", failure_count, total_to_process);
        }

        failure_count == 0
    }

    pub fn decrypt_all_files(&mut self) -> bool {
        let mut success_count = 0;
        let mut failure_count = 0;

        // Collect files that need decryption
        let files_to_decrypt: Vec<FileEntry> = self.meta.encrypted.data.files.0
            .iter()
            .filter(|f| f.is_encrypted.0)
            .cloned()
            .collect();

        let total_to_process = files_to_decrypt.len();
        
        if total_to_process == 0 {
            println!("ℹ️  No encrypted files found to decrypt.");
            return true;
        }

        println!("🔓 Decrypting {} files:", total_to_process);

        for (current, entry) in files_to_decrypt.iter().enumerate() {
            print!("  {}/{}: {}... ", current + 1, total_to_process, entry.display_name.0);
            
            // Check if encrypted file exists
            if !Path::new(&entry.system_path.0).exists() {
                println!("❌ (encrypted file missing)");
                failure_count += 1;
                continue;
            }

            let subfolder = self.get_sub_folder(&entry.extension.0);
            let decrypted_path = format!("{}/{}/{}", self.directories.decrypted.files_dir.0, subfolder, entry.system_name.0);

            // Ensure target directory exists
            let target_dir = format!("{}/{}", self.directories.decrypted.files_dir.0, subfolder);
            if !Path::new(&target_dir).exists() {
                if let Err(e) = create_dir_all(&target_dir) {
                    println!("❌ Failed to create directory: {}", e);
                    failure_count += 1;
                    continue;
                }
            }

            // Get item key from store
            let key = match self.crypto.item_key_store.get_key(&entry.id) {
                Some(item_key) => match self.crypto.unwrap_item_key(&item_key.0) {
                    Ok(k) => k,
                    Err(e) => {
                        println!("❌ (key error: {})", e);
                        failure_count += 1;
                        continue;
                    }
                },
                None => {
                    println!("❌ (key not found in store)");
                    failure_count += 1;
                    continue;
                }
            };

            let decrypt_result = self.crypto.decrypt_file_data(&entry.system_path.0, &decrypted_path, &key);
            
            if let Err(e) = decrypt_result {
                if Path::new(&decrypted_path).exists() {
                    let _ = remove_file(&decrypted_path); // rollback
                }
                println!("❌ (decryption failed: {})", e);
                failure_count += 1;
                continue;
            }

            // Add entry back to meta
            self.meta.decrypted.data.files.0.push(FileEntry {
                id: entry.id.clone(),
                display_name: entry.display_name.clone(),
                system_name: entry.system_name.clone(),
                system_path: StringPath(decrypted_path.clone()),
                size: entry.size.clone(),
                extension: entry.extension.clone(),
                is_encrypted: IsEncrypted(false),
                is_trashed: entry.is_trashed.clone(),
                created_at: entry.created_at.clone(),
                updated_at: Timestamp::now(),
            });

            // Remove from encrypted_meta
            self.meta.encrypted.data.files.0.retain(|f| f.id != entry.id);

            // Delete encrypted file
            if let Err(e) = remove_file(&entry.system_path.0) {
                eprintln!("⚠️ Failed to delete encrypted file: {}", e);
            }

            success_count += 1;
        }

        // Save metadata if we had successful operations
        if success_count > 0 {
            let _ = ( 
                self.meta.encrypted.save(),
                self.meta.decrypted.save()
            );
        }

        // Log results
        println!("\n📊 Decryption Summary:");
        println!("   Total:    {}", total_to_process);
        println!("   Success:  {}", success_count);
        println!("   Failed:   {}", failure_count);
        
        if success_count > 0 && failure_count == 0 {
            println!("✅ Successfully decrypted all {} files.", success_count);
        } else if success_count > 0 {
            println!("⚠️  Decrypted {}/{} files. {} failed.", success_count, total_to_process, failure_count);
        } else {
            println!("❌ Failed to decrypt any files. {}/{} failed.", failure_count, total_to_process);
        }

        failure_count == 0
    }

    // --- Updated single file encrypt/decrypt ---

    pub fn encrypt_file(&mut self, file_name: &str) -> bool {
        if let Some(entry) = self.meta.decrypted.data.files.0.iter().find(|f| f.display_name.0 == file_name).cloned() {
            if !Path::new(&entry.system_path.0).exists() {
                println!("❌ File does not exist: {}", entry.system_path.0);
                return false;
            }

            // Encrypted file path
            let encrypted_path = format!("{}/{}.enc", self.directories.encrypted.files_dir.0, entry.system_name.0);

            // Get item key from store
            let key = match self.crypto.item_key_store.get_key(&entry.id) {
                Some(item_key) => match self.crypto.unwrap_item_key(&item_key.0) {
                    Ok(k) => k,
                    Err(e) => {
                        println!("❌ Wrong password or corrupted file: {}: {}", file_name, e);
                        return false;
                    }
                },
                None => {
                    println!("❌ Key not found in store for file: {}", file_name);
                    return false;
                }
            };
            
            // Encrypt the file
            let encrypt_result = self.crypto.encrypt_file_data(&entry.system_path.0, &encrypted_path, &key);

            if let Err(e) = encrypt_result {
                if Path::new(&encrypted_path).exists() {
                    let _ = remove_file(&encrypted_path); // rollback
                }
                println!("❌ Encryption failed: {}", e);
                return false;
            }

            // Add entry to encrypted_meta
            self.meta.encrypted.data.files.0.push(FileEntry {
                id: entry.id.clone(), // Keep same ID
                display_name: entry.display_name.clone(),
                system_name: entry.system_name.clone(),
                system_path: StringPath(encrypted_path.clone()),
                size: entry.size.clone(),
                extension: entry.extension.clone(),
                is_encrypted: IsEncrypted(true),
                is_trashed: entry.is_trashed.clone(),
                created_at: entry.created_at.clone(),
                updated_at: Timestamp::now(),
            });

            // Remove from meta
            self.meta.decrypted.data.files.0.retain(|f| f.id != entry.id);

            // Delete original file
            if let Err(e) = remove_file(&entry.system_path.0) {
                eprintln!("⚠️ Failed to delete original file: {}: {}", entry.system_path.0, e);
            }

            let _ = ( 
                self.meta.encrypted.save(),
                self.meta.decrypted.save()
            );

            println!("🔒 File encrypted to: {}", encrypted_path);
            true
        } else {
            println!("❌ No file found with name: {}", file_name);
            false
        }
    }

    pub fn decrypt_file(&mut self, file_name: &str) -> bool {
        if let Some(entry) = self.meta.encrypted.data.files.0.iter().find(|f| f.display_name.0 == file_name).cloned() {
            // Check file exists
            if !Path::new(&entry.system_path.0).exists() {
                println!("❌ Encrypted file does not exist: {}", entry.system_path.0);
                return false;
            }

            let subfolder = self.get_sub_folder(&entry.extension.0);

            // Decrypted file path
            let decrypted_path = format!("{}/{}/{}", self.directories.decrypted.files_dir.0, subfolder, entry.system_name.0);

            // Get item key from store
            let key = match self.crypto.item_key_store.get_key(&entry.id) {
                Some(item_key) => match self.crypto.unwrap_item_key(&item_key.0) {
                    Ok(k) => k,
                    Err(e) => {
                        println!("❌ Wrong password or corrupted file: {}: {}", file_name, e);
                        return false;
                    }
                },
                None => {
                    println!("❌ Key not found in store for file: {}", file_name);
                    return false;
                }
            };
            
            let decrypt_result = self.crypto.decrypt_file_data(&entry.system_path.0, &decrypted_path, &key);

            if let Err(e) = decrypt_result { 
                if Path::new(&decrypted_path).exists() {
                    let _ = remove_file(&decrypted_path); // rollback
                }
                println!("❌ Decryption failed: {}", e);
                return false; 
            }

            // Add entry back to meta
            self.meta.decrypted.data.files.0.push(FileEntry {
                id: entry.id.clone(),
                display_name: entry.display_name.clone(),
                system_name: entry.system_name.clone(),
                system_path: StringPath(decrypted_path.clone()),
                size: entry.size.clone(),
                extension: entry.extension.clone(),
                is_encrypted: IsEncrypted(false),
                is_trashed: entry.is_trashed.clone(),
                created_at: entry.created_at.clone(),
                updated_at: Timestamp::now(),
            });

            // Remove from encrypted_meta
            self.meta.encrypted.data.files.0.retain(|f| f.id != entry.id);

            // Delete encrypted file
            if let Err(e) = remove_file(&entry.system_path.0) {
                eprintln!("⚠️ Failed to delete encrypted file: {}: {}", entry.system_path.0, e);
            }

            let _ = (
                self.meta.encrypted.save(),
                self.meta.decrypted.save()
            );

            println!("🔓 File decrypted to: {}", decrypted_path);
            true
        } else {
            println!("❌ No encrypted file found with name: {}", file_name);
            false
        }
    }

    // --- Updated cut_paste_file to remove keys ---

    pub fn paste_file(&mut self, name: &str, dst_path: &str) -> bool {
        if let Some(file) = self.meta.decrypted.data.files.0.iter().find(|f| f.display_name.0 == name).cloned() {

            if !Path::new(&file.system_path.0).exists() {
                println!("❌ File doesn't exist or it is corrupted");
                return false;
            }

            let target_path = format!("{}/{}", dst_path, file.system_name.0);
        
            if !self.copy_file(&file.system_path.0, &target_path) {
                println!("❌ Failed to copy file");
                return false;
            }

            println!("✅ File pasted Successfully! (..copied..)");
            true 
        } else {
            println!("❌ No files found in Database (If it is encrypted, decrypt it first)");
            false
        } 
    }

    pub fn cut_paste_file(&mut self, name: &str, dst_path: &str) -> bool {
        if let Some(file) = self.meta.decrypted.data.files.0.iter().find(|f| f.display_name.0 == name).cloned() {
            if !Path::new(&file.system_path.0).exists() {
                println!("❌ File doesn't exist or it is corrupted");
                return false;
            }

            let target_path = format!("{}/{}", dst_path, file.system_name.0);
        
            if !self.copy_file(&file.system_path.0, &target_path) {
                println!("❌ Failed to copy file");
                return false;
            }

            // Remove from database and key store
            self.meta.decrypted.data.files.0.retain(|f| f.display_name.0 != file.display_name.0);
            self.crypto.item_key_store.remove_key(&file.id);

            if let Err(e) = remove_file(&file.system_path.0) {
                println!("❌ Failed to remove original file {}", e);
                return false;
            }

            println!("✅ File pasted successfully! (..cutted..)");
            let _ = (
                self.meta.decrypted.save(),
                self.crypto.item_key_store.save()
            );

            true 
        } else {
            println!("❌ No files found in Database (If it is encrypted, decrypt it first)");
            false
        } 
    }

    // --- Helper Functions ---

    pub fn get_unique_file_name(&self, file_name: &str) -> String {
        let path = std::path::Path::new(file_name);
        let stem = path.file_stem().unwrap().to_string_lossy();
        let ext = path.extension().map(|e| e.to_string_lossy()).unwrap_or_default();

        let mut new_name = file_name.to_string();
        let mut counter = 1;

        loop {
            // Check if the name exists in meta or encrypted_meta
            let exists_in_meta = self.meta.decrypted.data.files.0.iter().any(|f| f.system_name.0 == new_name); // Use .0
            let exists_in_encrypted = self.meta.encrypted.data.files.0.iter().any(|f| f.system_name.0 == new_name); // Use .0

            if !exists_in_meta && !exists_in_encrypted {
                break;
            }

            // Generate new name with counter
            new_name = if ext.is_empty() {
                format!("{}{}", stem, counter)
            } else {
                format!("{}{}.{}", stem, counter, ext)
            };

            counter += 1;
        }

        new_name
    }
    
    pub fn copy_file(&self, src_path: &str, dst_path: &str) -> bool {
        let metadata = metadata(src_path).expect("Failed to read file metadata");
        let file_size_bytes = metadata.len();

        let mut src_file = File::open(src_path).expect("Failed to open source file!");
        let dst_file = File::create(dst_path).expect("Failed to create destination file!");
        let mut dst_writer = BufWriter::new(dst_file);

        let mut buffer = [0u8; 8192];
        let mut copied_bytes: u64 = 0;

        let file_name = Path::new(src_path)
            .file_name()
            .unwrap()
            .to_string_lossy();

        loop {
            let n = src_file.read(&mut buffer).expect("File read error");
            if n == 0 { break; }

            dst_writer.write_all(&buffer[..n]).expect("File write error");
            copied_bytes += n as u64;

            let progress = copied_bytes as f64 / file_size_bytes as f64 * 100.0;
            print!("\r Copying '{}' : {:.2}%", file_name, progress);
            stdout().flush().unwrap();
        }

        // ✅ Ensure everything is written to disk
        dst_writer.flush().expect("Failed to flush data");
        println!();
        true
    }

    pub fn get_sub_folder(&self, extension: &str) -> &str {
        let folder = match extension.to_lowercase().as_str() {
            // Images
            "jpg" | "jpeg" | "png" | "gif" | "bmp" | "webp" | "svg" | "ico" => "media/images",
            
            // Videos
            "mp4" | "mkv" | "avi" | "mov" | "wmv" | "webm" => "media/videos",
            
            // Audio
            "mp3" | "wav" | "flac" | "aac" | "ogg" => "media/audio",
            
            // Documents
            "pdf" => "documents/pdf",
            "doc" | "docx" => "documents/word",
            "xls" | "xlsx" => "documents/excel", 
            "ppt" | "pptx" => "documents/powerpoint",
            "txt" => "documents/text",
            
            // Code
            "html" | "htm" => "code/web",
            "css" => "code/web",
            "js" | "jsx" => "code/javascript",
            "ts" | "tsx" => "code/typescript",
            "py" => "code/python",
            "java" => "code/java",
            "cpp" | "c" | "h" => "code/cpp",
            "cs" => "code/csharp",
            "php" => "code/php",
            "rb" => "code/ruby",
            "go" => "code/go",
            "rs" => "code/rust",
            "json" | "xml" | "yaml" | "yml" => "code/config",
            
            // Archives
            "zip" | "rar" | "7z" | "tar" | "gz" => "archives",
            
            // Executables
            "exe" | "msi" => "executables/windows",
            "dmg" | "pkg" => "executables/macos",
            "deb" | "rpm" => "executables/linux",
            
            // Databases
            "db" | "sqlite" => "databases",
            "sql" => "code/database",
            
            _ => "other",
        };
        folder
    }
}