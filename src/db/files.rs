use super::{
    structs::{Database, FileEntry},
    std::{fs, 
        fs::{File,metadata,remove_file,create_dir_all},
        path::{Path},
        io::{Read, Write, BufWriter, stdout},
    },
    time,
    enc_keys::{unwrap_item_key, wrap_item_key, generate_item_key}
};


impl Database {
    pub fn encrypt_all_files(&mut self) -> bool {
        let mut success_count = 0;
        let mut failure_count = 0;

        // Collect files that need encryption
        let files_to_encrypt: Vec<FileEntry> = self.meta.decrypted_meta.data.files
            .iter()
            .filter(|f| !f.is_encrypted)
            .cloned()
            .collect();

        let total_to_process = files_to_encrypt.len();
        
        if total_to_process == 0 {
            println!("ℹ️  No unencrypted files found to encrypt.");
            return true;
        }

        println!("🔒 Encrypting {} files:", total_to_process);

        for (current, entry) in files_to_encrypt.iter().enumerate() {
            print!("  {}/{}: {}... ", current + 1, total_to_process, entry.name);
            
            // Check if file exists
            if !Path::new(&entry.file_path).exists() {
                println!("❌ (file missing)");
                failure_count += 1;
                continue;
            }

            // Encrypted file path
            let encrypted_path = format!("{}/{}.enc", self.directories.encrypted_files_dir, entry.name);

            // Unwrap item key
            let key = match unwrap_item_key(&entry.encrypted_item_key, &self.master.key) {
                Some(k) => k,
                None => {
                    println!("❌ (key error)");
                    failure_count += 1;
                    continue;
                }
            };

            // Encrypt the file
            let is_encrypted = self.encrypt_file_data(&entry.file_path, &encrypted_path, &key);
            if !is_encrypted {
                if Path::new(&encrypted_path).exists() {
                    let _ = remove_file(&encrypted_path); // rollback
                }
                println!("❌ (encryption failed)");
                failure_count += 1;
                continue;
            }

            // Add entry to encrypted_meta
            self.meta.encrypted_meta.data.files.push(FileEntry {
                name: entry.name.clone(),
                file_name: entry.file_name.clone(),
                file_path: encrypted_path.clone(),
                size: entry.size.clone(),
                extension: entry.extension.clone(),
                encrypted_item_key: entry.encrypted_item_key.clone(),
                is_encrypted: true,
                is_recycled: false,
                created_at: entry.created_at.clone(),
                updated_at: time::now(),
            });

            // Remove from meta
            self.meta.decrypted_meta.data.files.retain(|f| f.name != entry.name);

            // Delete original file
            if let Err(e) = remove_file(&entry.file_path) {
                eprintln!("⚠️ Failed to delete original file: {}", e);
            }

            success_count += 1;
        }

        // Save metadata if we had successful operations
        if success_count > 0 {
            self.meta.encrypted_meta.save();
            self.meta.decrypted_meta.save();
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
        let files_to_decrypt: Vec<FileEntry> = self.meta.encrypted_meta.data.files
            .iter()
            .filter(|f| f.is_encrypted)
            .cloned()
            .collect();

        let total_to_process = files_to_decrypt.len();
        
        if total_to_process == 0 {
            println!("ℹ️  No encrypted files found to decrypt.");
            return true;
        }

        println!("🔓 Decrypting {} files:", total_to_process);

        for (current, entry) in files_to_decrypt.iter().enumerate() {
            print!("  {}/{}: {}... ", current + 1, total_to_process, entry.name);
            
            // Check if encrypted file exists
            if !Path::new(&entry.file_path).exists() {
                println!("❌ (encrypted file missing)");
                failure_count += 1;
                continue;
            }

            let subfolder = self.get_sub_folder(&entry.extension);
            let decrypted_path = format!("{}/{}/{}", self.directories.decrypted_files_dir, subfolder, entry.file_name);


            // Ensure target directory exists
            let target_dir = format!("{}/{}", self.directories.decrypted_files_dir, subfolder);
            if !Path::new(&target_dir).exists() {
                if let Err(e) = create_dir_all(&target_dir) {
                    println!("❌ Failed to create directory: {}",e);
                    failure_count += 1;
                    continue;
                }
            }

            // Decrypt the file
            let key = match unwrap_item_key(&entry.encrypted_item_key, &self.master.key) {
                Some(k) => k,
                None => {
                    println!("❌ (key error)");
                    failure_count += 1;
                    continue;
                }
            };

            let is_decrypted = self.decrypt_file_data(&entry.file_path, &decrypted_path, &key);
            if !is_decrypted {
                if Path::new(&decrypted_path).exists() {
                    let _ = remove_file(&decrypted_path); // rollback
                }
                println!("❌ (decryption failed)");
                failure_count += 1;
                continue;
            }

            // Add entry back to meta
            self.meta.decrypted_meta.data.files.push(FileEntry {
                name: entry.name.clone(),
                file_name: entry.file_name.clone(),
                encrypted_item_key: entry.encrypted_item_key.clone(),
                file_path: decrypted_path.clone(),
                size: entry.size.clone(),
                extension: entry.extension.clone(),
                is_encrypted: false,
                is_recycled: false,
                created_at: entry.created_at.clone(),
                updated_at: time::now(),
            });

            // Remove from encrypted_meta
            self.meta.encrypted_meta.data.files.retain(|f| f.name != entry.name);

            // Delete encrypted file
            if let Err(e) = remove_file(&entry.file_path) {
                eprintln!("⚠️ Failed to delete encrypted file: {}", e);
            }

            success_count += 1;
        }

        // Save metadata if we had successful operations
        if success_count > 0 {
            self.meta.encrypted_meta.save();
            self.meta.decrypted_meta.save();
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
    // Add file
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
        let extension = path
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("unknown")
            .to_lowercase();

        // Select subfolder based on extension
        let subfolder = self.get_sub_folder(&extension);

        // Ensure subfolder exists
        let target_dir = format!("{}/{}", self.directories.decrypted_files_dir, subfolder);
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
        if self.meta.decrypted_meta.data.files.iter().any(|f| f.name == name) {
            println!("❌ File with name '{}' already exists in meta!", name);
            return false;
        }
        if self.meta.encrypted_meta.data.files.iter().any(|f| f.name == name) {
            println!("❌ File with name '{}' already exists in encrypted files!", name);
            return false;
        }

        let item_key = generate_item_key();
        let encrypted_item_key = match wrap_item_key(&item_key, &self.master.key) {
            Some(eik) => eik,
            None => {
                println!("❌ Failed to generate encrypted item key for file: {}", name);
                return false;
            }
        };

        // Try restore if missing
        if let Some(index) = self.meta.decrypted_meta.data.files.iter().position(|f| f.name == name) {
            let file_missing = !Path::new(&self.meta.decrypted_meta.data.files[index].file_path).exists();
                
            if file_missing {
        
                // Now safely borrow mutably after copy is done
                let entry = &mut self.meta.decrypted_meta.data.files[index];
                entry.file_path = dest_path.clone();
                entry.file_name = file_name.to_string();
                entry.encrypted_item_key = encrypted_item_key;
                entry.size = file_size_mb.to_string();
                entry.extension = extension.clone();
                entry.is_recycled = false;
                entry.created_at = time::now();
                entry.updated_at = 0.to_string();

                if let Some(src) = path.to_str() {
                    if !self.copy_file(src, dest_path.as_str()) {
                        println!("Failed to copy file!");
                        return false;
                    }else {
                        self.meta.decrypted_meta.save();
                    } 
                } else {
                    println!("Invalid path (not UTF-8)!");
                    return false;
                }
            
                println!("♻️ Restored missing file for '{}'", name);
                return true;
            }
        }

        self.meta.decrypted_meta.data.files.push(FileEntry {
            name: name.to_string(),
            file_name: file_name.to_string(),
            encrypted_item_key,
            file_path: dest_path.clone(),
            size: file_size_mb.to_string(),
            extension: extension.clone(),
            is_recycled: false,
            is_encrypted: false,
            created_at: time::now(),
            updated_at: 0.to_string(),
        });

        if let Some(src) = path.to_str() {
            if !self.copy_file(src, dest_path.as_str()) {
                println!("Failed to copy file!");
                return false;
            }else {
                self.meta.decrypted_meta.save();
            }
            if let Err(e) = remove_file(src) {
                println!("❌ Failed to delete original file: {}",e);
                return false;
            }
        } else {
            println!("❌ Invalid path (not UTF-8)!");
            return false;
        }

        println!(
            "✅ File added: {} (.{}) size <{} MB> (..cutted..)",
            name, extension, file_size_mb
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
        let extension = path
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("unknown")
            .to_lowercase();

        // Select subfolder based on extension
        let subfolder = self.get_sub_folder(&extension);

        // Ensure subfolder exists
        let target_dir = format!("{}/{}", self.directories.decrypted_files_dir, subfolder);
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
        if self.meta.decrypted_meta.data.files.iter().any(|f| f.name == name) {
            println!("❌ File with name '{}' already exists in meta!", name);
            return false;
        }
        if self.meta.encrypted_meta.data.files.iter().any(|f| f.name == name) {
            println!("❌ File with name '{}' already exists in encrypted files!", name);
            return false;
        }

        let item_key = generate_item_key();
        let encrypted_item_key = match wrap_item_key(&item_key, &self.master.key) {
            Some(eik) => eik,
            None => {
                println!("❌ Failed to generate encrypted item key for file: {}", name);
                return false;
            }
        };

        // Try restore if missing
        if let Some(index) = self.meta.decrypted_meta.data.files.iter().position(|f| f.name == name) {
            let file_missing = !Path::new(&self.meta.decrypted_meta.data.files[index].file_path).exists();
                
            if file_missing {
        
                // Now safely borrow mutably after copy is done
                let entry = &mut self.meta.decrypted_meta.data.files[index];
                entry.file_path = dest_path.clone();
                entry.file_name = file_name.to_string();
                entry.encrypted_item_key = encrypted_item_key;
                entry.size = file_size_mb.to_string();
                entry.extension = extension.clone();
                entry.is_recycled = false;
                entry.created_at = time::now();
                entry.updated_at = 0.to_string();

                if let Some(src) = path.to_str() {
                    if !self.copy_file(src, dest_path.as_str()) {
                        println!("Failed to copy file!");
                        return false;
                    }else {
                        self.meta.decrypted_meta.save();
                    } 
                } else {
                    println!("Invalid path (not UTF-8)!");
                    return false;
                }
            
                println!("♻️ Restored missing file for '{}'", name);
                return true;
            }
        }

        self.meta.decrypted_meta.data.files.push(FileEntry {
            name: name.to_string(),
            file_name: file_name.to_string(),
            encrypted_item_key,
            file_path: dest_path.clone(),
            size: file_size_mb.to_string(),
            extension: extension.clone(),
            is_recycled: false,
            is_encrypted: false,
            created_at: time::now(),
            updated_at: 0.to_string(),
        });

        if let Some(src) = path.to_str() {
            if !self.copy_file(src, dest_path.as_str()) {
                println!("Failed to copy file!");
                return false;
            }else {
                self.meta.decrypted_meta.save();
            }
        } else {
            println!("Invalid path (not UTF-8)!");
            return false;
        }

        println!(
            "✅ File added: {} (.{}) size <{} MB> (..copied..)",
            name, extension, file_size_mb
        );
        true
    }

    pub fn paste_file(&mut self, name: &str, dst_path: &str) -> bool {
            if let Some(file) = self.meta.decrypted_meta.data.files.iter().find(|f| f.name == name).cloned() {

            if !Path::new(&file.file_path).exists() {
                println!("❌ File don't exists or it is corrupted");
                return false;
            }

            let target_path = format!("{}/{}",dst_path,file.file_name);
        
            if !self.copy_file(&file.file_path, &target_path) {
                println!("❌ Failed to copy file");
                return false;
            }

            println!("✅ File pasted Successfully! (..copied..)");
            true 
        } else {
            println!("❌ No files found in Databse ( If it is encrypted, decrypt it first )");
            return false;
        } 
    }

    pub fn cut_paste_file(&mut self, name: &str, dst_path: &str) -> bool {
            if let Some(file) = self.meta.decrypted_meta.data.files.iter().find(|f| f.name == name).cloned() {

            if !Path::new(&file.file_path).exists() {
                println!("❌ File don't exists or it is corrupted");
                return false;
            }

            let target_path = format!("{}/{}",dst_path,file.file_name);
        
            if !self.copy_file(&file.file_path, &target_path) {
                println!("❌ Failed to copy file");
                return false;
            }

            self.meta.decrypted_meta.data.files.retain(|f| f.name != file.name);

            if let Err(e) = remove_file(&file.file_path) {
                println!("❌ Failed to remove original file {}",e);
                return false;
            }

            println!("✅ File pasted Successfully! (..cutted..)");
            self.meta.decrypted_meta.save();

            true 
        } else {
            println!("❌ No files found in Databse ( If it is encrypted, decrypt it first )");
            return false;
        } 
    }

    pub fn encrypt_file(&mut self, file_name: &str) -> bool {
        if let Some(entry) = self.meta.decrypted_meta.data.files.iter().find(|f| f.name == file_name).cloned() {
            if !Path::new(&entry.file_path).exists() {
                println!("❌ File does not exist: {}", entry.file_path);
                return false;
            }

            // Encrypted file path
            let encrypted_path = format!("{}/{}.enc", self.directories.encrypted_files_dir, file_name);

            // Unwrap item key
            let key = match unwrap_item_key(&entry.encrypted_item_key, &self.master.key) {
                Some(k) => k,
                None => {
                    println!("❌ Wrong password or corrupted file: {}", file_name);
                    return false;
                }
            };
            // Encrypt the file
            let is_encrypted = self.encrypt_file_data(&entry.file_path, &encrypted_path, &key);
            if !is_encrypted {
                if Path::new(&encrypted_path).exists() {
                    let _ = remove_file(&encrypted_path); // rollback
                }
                return false;
            }

            // Add entry to encrypted_meta
            self.meta.encrypted_meta.data.files.push(FileEntry {
                name: entry.name.clone(),
                file_name: entry.file_name.clone(),
                file_path: encrypted_path.clone(),
                size: entry.size.clone(),
                extension: entry.extension.clone(),
                encrypted_item_key: entry.encrypted_item_key.clone(), // keep key
                is_encrypted: true,
                is_recycled: false,
                created_at: entry.created_at.clone(),
                updated_at: time::now(),
            });

            // Remove from meta
            self.meta.decrypted_meta.data.files.retain(|f| f.name != entry.name);

            // Delete original file
            if let Err(e) = remove_file(&entry.file_path) {
                eprintln!("⚠️ Failed to delete original file: {}: {}", entry.file_path, e);
            }

            self.meta.encrypted_meta.save();
            self.meta.decrypted_meta.save();

            println!("🔒 File encrypted to: {}", encrypted_path);
            true
        } else {
            println!("❌ No file found with name: {}", file_name);
            false
        }
    }

    pub fn decrypt_file(&mut self, file_name: &str) -> bool {
        if let Some(entry) = self.meta.encrypted_meta.data.files.iter().find(|f| f.name == file_name).cloned() {
            // Check file exists
            if !Path::new(&entry.file_path).exists() {
                println!("❌ Encrypted file does not exist: {}", entry.file_path);
                return false;
            }

            let subfolder = self.get_sub_folder(entry.extension.as_str());

            // Decrypted file path
            let decrypted_path = format!("{}/{}/{}", self.directories.decrypted_files_dir, subfolder, entry.file_name);

            // Decrypt the file
            let key = match unwrap_item_key(&entry.encrypted_item_key, &self.master.key) {
                Some(k) => k,
                None => {
                    println!("❌ Wrong password or corrupted file: {}", file_name);
                    return false;
                }   
            };
            let is_decrypted = self.decrypt_file_data(&entry.file_path, &decrypted_path, &key);

            if !is_decrypted { return false; }

            // Add entry back to meta
            self.meta.decrypted_meta.data.files.push(FileEntry {
                name: entry.name.clone(),
                file_name: entry.file_name.clone(),
                encrypted_item_key: entry.encrypted_item_key.clone(), // keep key
                file_path: decrypted_path.clone(),
                size: entry.size.clone(),
                extension: entry.extension.clone(),
                is_encrypted: false,
                is_recycled: false,
                created_at: entry.created_at.clone(),
                updated_at: time::now(),
            });

            // Remove from encrypted_meta
            self.meta.encrypted_meta.data.files.retain(|f| f.name != entry.name);

            // Delete encrypted file
            if let Err(e) = remove_file(&entry.file_path) {
                eprintln!("⚠️ Failed to delete encrypted file: {}: {}", entry.file_path, e);
            }

            self.meta.encrypted_meta.save();
            self.meta.decrypted_meta.save();

            println!("🔓 File decrypted to: {}", decrypted_path);
            true
        } else {
            println!("❌ No encrypted file found with name: {}", file_name);
            false
        }
    }

    // Helper function
    pub fn get_unique_file_name(&self, file_name: &str) -> String {
        let path = std::path::Path::new(file_name);
        let stem = path.file_stem().unwrap().to_string_lossy();
        let ext = path.extension().map(|e| e.to_string_lossy()).unwrap_or_default();

        let mut new_name = file_name.to_string();
        let mut counter = 1;

        loop {
            // Check if the name exists in meta or encrypted_meta
            let exists_in_meta = self.meta.decrypted_meta.data.files.iter().any(|f| f.file_name == new_name);
            let exists_in_encrypted = self.meta.encrypted_meta.data.files.iter().any(|f| f.file_name == new_name);

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
            // ========== CODING LANGUAGES - DETAILED ==========
            // Web Development - these must come before general video formats
            "tsx" | "cts" => "code/web/typescript",
            "mts" => "code/web/typescript", // Only mts for TypeScript, general mts handled in videos
            
            // Documentation - tex must come before documents/text
            "bib" => "code/docs/latex",
            
            // ========== IMAGE FORMATS ==========
            "jpg" | "jpeg" | "png" | "gif" | "bmp" | "tiff" | "tif" | "webp" | "svg" | "ico" | 
            "raw" | "cr2" | "nef" | "arw" | "psd" | "ai" | "eps" | "heic" | "heif" | "icns" | 
            "ppm" | "pgm" | "pbm" | "pnm" | "hdr" | "exr" | "dds" | "xcf" | "kra" => "media/images",
        
            // ========== VIDEO FORMATS ==========
            "mp4" | "mkv" | "avi" | "mov" | "wmv" | "flv" | "webm" | "m4v" | "mpg" | "mpeg" | 
            "3gp" | "vob" | "ogv" | "m2ts" | "divx" | "f4v" | "rm" | "rmvb" |
            "asf" | "mxf" | "ogm" | "m2v" | "m4p" | "m4b" | "qt" | "yuv" => "media/videos",
        
            // ========== AUDIO FORMATS ==========
            "mp3" | "wav" | "flac" | "aac" | "ogg" | "wma" | "m4a" | "aiff" | "ape" | "opus" |
            "mid" | "midi" | "amr" | "aif" | "aifc" | "cda" | "ac3" | "dts" | "mka" | "ra" => "media/audio",
        
            // ========== DOCUMENT FORMATS ==========
            // Microsoft Office
            "doc" | "docx" | "dot" | "dotx" | "docm" | "dotm" => "documents/microsoft/word",
            "xls" | "xlsx" | "xlsm" | "xlsb" | "xlt" | "xltx" => "documents/microsoft/excel",
            "ppt" | "pptx" | "pps" | "ppsx" | "pot" | "potx" | "pptm" | "potm" | "ppsm" | "sldx" | "sldm" => "documents/microsoft/powerpoint",
        
            // PDF and universal documents
            "pdf" => "documents/pdf",
        
            // Text documents
            "txt" | "rtf" | "odt" | "pages" | "wpd" | "wps" | "abw" | "zabw" | "lwp" | "mcw" | "uot" | "uof" => "documents/text",
        
            // ========== SPREADSHEETS ==========
            "csv" | "tsv" | "ods" | "numbers" | "dif" | "slk" | "prn" => "documents/spreadsheets",
        
            // ========== PRESENTATIONS ==========
            "key" | "odp" => "documents/presentations",
        
            // ========== ARCHIVE/COMPRESSION FORMATS ==========
            "zip" | "rar" | "7z" | "tar" | "gz" | "bz2" | "xz" | "iso" | "cab" | "arj" | "lzh" | "z" | "lz" | "lzma" | "tlz" | "txz" => "archives/compressed",
        
            // ========== CODING LANGUAGES - DETAILED ==========
            // Web Development
            "html" | "htm" | "xhtml" | "shtml" => "code/web/markup",
            "css" | "scss" | "sass" | "less" | "styl" => "code/web/styles",
            "js" | "jsx" | "mjs" | "cjs" => "code/web/javascript",
            "ts" => "code/web/typescript", // Only ts for TypeScript, general ts handled in videos
            "vue" => "code/web/vue",
            "svelte" => "code/web/svelte",
            "astro" => "code/web/astro",
        
            // Backend Web
            "php" | "phtml" => "code/web/php",
            "jsp" | "jspx" => "code/web/jsp",
            "asp" | "aspx" => "code/web/asp",
        
            // Python Ecosystem
            "py" | "pyw" => "code/python/source",
            "pyc" | "pyo" | "pyd" => "code/python/compiled",
            "pyi" => "code/python/stubs",
            "pyz" | "pyzw" => "code/python/archives",
        
            // Java Ecosystem
            "java" => "code/java/source",
            "class" => "code/java/compiled",
        
            // C/C++ Ecosystem
            "c" | "cpp" | "cc" | "cxx" | "c++" => "code/c_cpp/source",
            "h" | "hpp" | "hh" | "hxx" | "h++" => "code/c_cpp/headers",
            "obj" => "code/c_cpp/objects",
            "lib" | "exp" => "code/c_cpp/libraries",
        
            // C# Ecosystem
            "cs" | "csx" => "code/csharp/source",
        
            // Rust Ecosystem
            "rs" => "code/rust/source",
            "rlib" => "code/rust/libraries",
        
            // Go Ecosystem
            "go" => "code/go/source",
            "mod" | "sum" => "code/go/modules",
            "test" => "code/go/tests",
        
            // Ruby Ecosystem
            "rb" | "rbw" => "code/ruby/source",
            "rake" | "gemspec" => "code/ruby/tools",
            "erb" | "rhtml" => "code/ruby/templates",
        
            // Swift Ecosystem
            "swift" => "code/swift/source",
            "swiftmodule" | "swiftdoc" => "code/swift/modules",
        
            // Kotlin Ecosystem
            "kt" | "kts" | "ktm" => "code/kotlin/source",
        
            // Scala Ecosystem
            "scala" | "sc" => "code/scala/source",
        
            // Perl Ecosystem
            "pl" | "pm" | "t" | "pod" => "code/perl/source",
        
            // Haskell Ecosystem
            "hs" | "lhs" => "code/haskell/source",
            "hi" => "code/haskell/interface",
        
            // Lua Ecosystem
            "lua" => "code/lua/source",
            "luac" => "code/lua/compiled",
        
            // R Ecosystem
            "r" => "code/r/source",
            "rdata" | "rds" | "rda" => "code/r/data",
        
            // MATLAB/Octave
            "m" => "code/matlab/source",
            "mat" => "code/matlab/data",
            "fig" => "code/matlab/figures",
        
            // Shell Scripting
            "sh" | "bash" | "zsh" | "fish" | "csh" | "tcsh" | "ksh" => "code/shell/scripts",
        
            // PowerShell
            "ps1" | "psm1" | "psd1" | "ps1xml" => "code/powershell/scripts",
        
            // Windows Batch
            "bat" | "cmd" => "code/batch/scripts",
        
            // Configuration/Data Formats
            "json" | "jsonl" | "json5" | "jsonc" => "code/config/json",
            "xml" => "code/config/xml",
            "yaml" | "yml" => "code/config/yaml",
            "toml" => "code/config/toml",
            "ini" | "cfg" | "conf" | "config" | "properties" | "prop" => "code/config/ini",
            "env" | "env.example" => "code/config/environment",
        
            // Database/SQL
            "sql" | "ddl" | "dml" | "pks" | "pkb" | "pck" => "code/database/sql",
        
            // Documentation
            "md" | "markdown" => "code/docs/markdown",
            "rst" => "code/docs/restructured",
            "adoc" | "asciidoc" => "code/docs/asciidoc",
            "tex" => "code/docs/latex",
        
            // Docker/Container
            "dockerfile" => "code/docker/files",
            "dockerignore" => "code/docker/ignore",
        
            // Git
            "gitignore" | "gitattributes" | "gitmodules" | "gitkeep" => "code/git/config",
        
            // Build Tools
            "makefile" | "mk" => "code/build/make",
            "cmake" | "cmakelists.txt" => "code/build/cmake",
            "gradle" => "code/build/gradle",
            "pom.xml" => "code/build/maven",
            "build.xml" => "code/build/ant",
            "meson.build" => "code/build/meson",
            "buck" | "bazel" | "bazelrc" | "buckconfig" => "code/build/buck_bazel",
        
            // IDE/Editor Specific
            "sln" => "code/ide/visualstudio/solutions",
            "csproj" | "vbproj" => "code/ide/visualstudio/projects",
            "vcxproj" | "vcproj" | "dsp" | "dsw" => "code/ide/visualstudio/cpp",
            "xcodeproj" | "pbxproj" | "xcworkspace" => "code/ide/xcode/projects",
            "project" | "workspace" => "code/ide/ide_general",
            "vsix" => "code/ide/visualstudio/extensions",
            "vscodeignore" | "code-workspace" => "code/ide/vscode",
        
            // Other Programming Languages
            "dart" => "code/dart/source",
            "elm" => "code/elm/source",
            "clj" | "cljc" | "cljs" | "edn" => "code/clojure/source",
            "ex" | "exs" => "code/elixir/source",
            "gleam" => "code/gleam/source",
            "fs" | "fsx" | "fsi" | "fsscript" => "code/fsharp/source",
            "ml" | "mli" => "code/ocaml/source",
            "zig" => "code/zig/source",
            "v" => "code/v/source",
            "nim" => "code/nim/source",
            "cr" => "code/crystal/source",
            "d" => "code/d/source",
            "pas" | "pp" | "lpr" => "code/pascal/source",
            "ada" | "adb" | "ads" => "code/ada/source",
            "pro" => "code/prolog/source",
            "plist" => "code/macos/plist",
            "proto" => "code/protobuf/source",
            "thrift" | "avdl" => "code/thrift/source",
            "graphql" | "gql" => "code/graphql/schema",
            "prisma" => "code/prisma/schema",
        
            // Template Files
            "ejs" => "code/templates/ejs",
            "hbs" | "handlebars" => "code/templates/handlebars",
            "mustache" => "code/templates/mustache",
            "njk" | "nunjucks" => "code/templates/nunjucks",
            "twig" => "code/templates/twig",
            "j2" | "jinja2" => "code/templates/jinja",
        
            // ========== EXECUTABLES/BINARIES - OS SPECIFIC ==========
            // Windows Executables
            "exe" | "com" | "scr" | "pif" => "executables/windows/binaries",
            "cpl" => "executables/windows/control_panels",
            "mui" => "executables/windows/multilingual",
            "acm" | "ax" | "drv" => "executables/windows/drivers",
            "ocx" | "rbz" | "tsp" | "vbx" => "executables/windows/activex",
        
            // macOS Applications and Binaries
            "app" => "executables/macos/applications",
            "bundle" | "framework" => "executables/macos/bundles",
            "kext" => "executables/macos/extensions",
            "plugin" | "component" | "prefpan" | "qtz" | "saver" | "service" | "wdgt" | "xpc" => "executables/macos/components",
        
            // Linux/Unix Binaries and Packages
            "appimage" => "executables/linux/appimage",
            "snap" => "executables/linux/snap",
            "flatpak" => "executables/linux/flatpak",
            "run" | "out" => "executables/linux/binaries",
            "ko" => "executables/linux/libraries",
            "elf" => "executables/linux/elf",
        
            // Cross-platform/Generic Binaries
            "nexe" => "executables/cross_platform/node",
            "nw" | "electron" => "executables/cross_platform/electron",
        
            // Game and Application Data
            "pak" | "dat" | "data" | "assets" | "resource" | "res" => "executables/games/data",
        
            // System Files
            "sys" | "vxd" | "386" => "executables/system/drivers",
            "rom" => "executables/system/firmware",
            "msp" | "msu" | "mst" => "executables/system/updates",
            "pat" | "qvm" => "executables/system/patches",
            "wlx" | "wpx" => "executables/system/extensions",
        
            // ========== ARCHIVE/COMPRESSION FORMATS (packages) ==========
            // These come after specific code/executable categories
            "dmg" | "pkg" | "msi" | "apk" | "deb" | "rpm" | "crx" | "egg" | "whl" | "war" | "jar" | "ear" | "sar" | "xpi" => "archives/packages",
        
            // ========== DATABASE FILES ==========
            "db" | "sqlite" | "sqlite3" => "databases/sqlite",
            "mdb" | "accdb" => "databases/access",
            "frm" | "myd" | "myi" | "ibd" => "databases/mysql",
            "mdf" | "ldf" | "ndf" => "databases/sqlserver",
            "dmp" => "databases/backups",
            "ora" => "databases/oracle",
        
            // ========== E-BOOK FORMATS ==========
            "epub" => "ebooks/epub",
            "mobi" | "azw" | "azw3" => "ebooks/kindle",
            "fb2" => "ebooks/fictionbook",
            "lit" => "ebooks/microsoft",
            "lrf" => "ebooks/sony",
            "pml" => "ebooks/palm",
            "snb" => "ebooks/samsung",
        
            // ========== FONT FILES ==========
            "ttf" => "fonts/truetype",
            "otf" => "fonts/opentype",
            "woff" | "woff2" => "fonts/web",
            "eot" => "fonts/embedded",
            "pfb" | "pfm" | "afm" => "fonts/postscript",
            "dfont" => "fonts/macos",
        
            // ========== VIRTUAL MACHINE/DISK IMAGES ==========
            "vdi" => "virtual_machines/virtualbox",
            "vmdk" => "virtual_machines/vmware",
            "vhd" | "vhdx" => "virtual_machines/hyperv",
            "ova" | "ovf" => "virtual_machines/ovf",
            "qcow2" | "qed" => "virtual_machines/qemu",
            "hdd" => "virtual_machines/hard_disks",
        
            // ========== BACKUP FILES ==========
            "backup" | "old" | "bk" | "bkp" => "backups/automatic",
            "tmp" | "temp" => "backups/temporary",
            "swp" | "swo" => "backups/editor",
            "sav" | "save" => "backups/manual",
        
            // ========== CERTIFICATE FILES ==========
            "pem" | "crt" | "cer" | "der" => "certificates/public",
            "pfx" | "p12" => "certificates/pkcs12",
            "csr" => "certificates/requests",
            "jks" | "keystore" | "truststore" => "certificates/java",
        
            // ========== 3D/CAD FILES ==========
            "stl" => "3d_models/stereolithography",
            "fbx" => "3d_models/autodesk",
            "dwg" | "dxf" => "3d_models/cad",
            "blend" => "3d_models/blender",
            "3ds" => "3d_models/3ds_max",
            "max" | "ma" | "mb" => "3d_models/autodesk",
            "c4d" => "3d_models/cinema4d",
            "ztl" => "3d_models/zbrush",
            "skp" => "3d_models/sketchup",
            "lwo" | "lws" => "3d_models/lightwave",
            "x3d" | "vrml" | "wrl" => "3d_models/vrml",
        
            // ========== PROJECT FILES ==========
            "bower.json" => "projects/javascript/bower",
            "package.json" => "projects/javascript/npm",
            "composer.json" => "projects/php/composer",
            "cargo.toml" => "projects/rust/cargo",
            "go.mod" => "projects/go/modules",
            "mix.exs" => "projects/elixir/mix",
            "project.clj" => "projects/clojure/leiningen",
        
            // ========== GENERAL LIBRARIES AND OBJECTS ==========
            // These come after specific categories
            "so" | "dll" | "dylib" => "code/c_cpp/libraries",
            "o" | "a" => "code/c_cpp/objects",
        
            // ========== FALLBACK CATEGORIES ==========
            // These handle extensions that could belong to multiple categories
            "dbf" => "databases/dbase",
            "pdb" => "databases/oracle",
            "bak" => "databases/backups",
            "bin" => "executables/system/firmware",
            "efi" => "executables/system/efi",
            "fon" => "fonts/windows",
        
            _ => "other/unknown",
        };
        folder
    }
}