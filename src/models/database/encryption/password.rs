use super::{
    Database,
    entries::PasswordEntry,
    entries::{
        IsEncrypted,
        IsTrashed,
        ItemId,
        password_entry::{
            Password,
            PasswordName,
        }
    },
    crypto::{
        ItemKey,
    },
    core::Timestamp,
};

impl Database {
    // --- Password CRUD Operations ---

    pub fn add_password(&mut self, name: &str, password: &str) -> bool {
        // Check for duplicates in decrypted and encrypted passwords
        if self.meta.decrypted.data.passwords.0.iter().any(|e| e.display_name.0 == name) ||
           self.meta.encrypted.data.passwords.0.iter().any(|e| e.display_name.0 == name) 
        {
            println!("❌ Password with this name already exists: {}", name);
            return false;
        }

        let item_id = ItemId::new();
        
        // Generate item key and encrypt it using the Master Key
        let item_key = ItemKey::generate_vec_key();
        let encrypted_item_key = match self.crypto.wrap_item_key(&item_key) {
            Ok(eik) => ItemKey(eik),
            Err(e) => {
                println!("❌ Failed to generate encrypted item key for password: {} - {}", name, e);
                return false;
            }
        };

        // Create new password entry (NO encrypted_item_key field)
        let new_entry = PasswordEntry {
            id: item_id.clone(),
            display_name: PasswordName(name.to_string()),
            password: Password(password.to_string()),
            is_encrypted: IsEncrypted(false),
            is_trashed: IsTrashed(false),
            created_at: Timestamp::now(),
            updated_at: Timestamp::now(),
        };

        // Store the key in the key store
        self.crypto.item_key_store.set_key(item_id, encrypted_item_key);

        // Add to decrypted collection
        self.meta.decrypted.data.passwords.0.push(new_entry);
        
        // Save both metadata and key store
        if let Err(e) = self.meta.decrypted.save() {
            println!("❌ Failed to save decrypted metadata: {}", e);
            return false;
        }
        
        if let Err(e) = self.crypto.item_key_store.save() {
            println!("❌ Failed to save item key store: {}", e);
            return false;
        }
        
        println!("✅ Password added: {}", name);
        true
    }

    pub fn change_password(&mut self, name: &str, new_password: &str) -> bool {
        // Find password
        if let Some(entry) = self.meta.decrypted.data.passwords.0.iter_mut().find(|e| e.display_name.0 == name) {
            // Compare passwords
            if entry.password.0 == new_password {
                println!("❌ New password is the same as the old password for: {}", name);
                return false;
            }
            
            // Update the password
            entry.password = Password(new_password.to_string());
            entry.updated_at = Timestamp::now();
            
            if let Err(e) = self.meta.decrypted.save() {
                println!("❌ Failed to save decrypted metadata: {}", e);
                return false;
            }
            
            println!("🔄 Password updated for: {}", name);
            true
        } else {
            println!("❌ Password '{}' not found in decrypted list. If it's encrypted, decrypt it first before modifying.", name);
            false
        }
    }
    
    // --- Single Password Encryption/Decryption ---

    pub fn encrypt_password(&mut self, name: &str) -> bool {
        // Find the password
        if let Some(pos) = self.meta.decrypted.data.passwords.0.iter().position(|p| p.display_name.0 == name) {
            // Remove the entry to take ownership
            let mut entry = self.meta.decrypted.data.passwords.0.remove(pos);

            // Get the item key from the key store
            let key = match self.crypto.item_key_store.get_key(&entry.id) {
                Some(encrypted_key) => match self.crypto.unwrap_item_key(&encrypted_key.0) {
                    Ok(k) => k,
                    Err(e) => {
                        println!("❌ Wrong master password or corrupted item key for {}: {}", name, e);
                        self.meta.decrypted.data.passwords.0.insert(pos, entry); // Put it back
                        return false;
                    }
                },
                None => {
                    println!("❌ Item key not found in store for: {}", name);
                    self.meta.decrypted.data.passwords.0.insert(pos, entry); // Put it back
                    return false;
                }
            };

            // Encrypt the plaintext password string
            let encrypted_password = match self.crypto.encrypt_string(&entry.password.0, &key) {
                Ok(ep) => ep,
                Err(e) => {
                    println!("❌ Password encryption failed for '{}': {}", name, e);
                    self.meta.decrypted.data.passwords.0.insert(pos, entry); // Put it back
                    return false;
                }
            };

            // Update entry and move to encrypted meta
            entry.is_encrypted = IsEncrypted(true);
            entry.password = Password(encrypted_password);
            entry.updated_at = Timestamp::now();
            self.meta.encrypted.data.passwords.0.push(entry);

            // Save both metadata files
            if let Err(e) = self.meta.decrypted.save() {
                eprintln!("⚠️ Failed to save decrypted metadata after removal: {}", e);
            }
            if let Err(e) = self.meta.encrypted.save() {
                println!("❌ Failed to save encrypted metadata: {}", e);
                return false;
            }

            println!("🔒 Password '{}' encrypted successfully.", name);
            true
        } else {
            println!("❌ No decrypted password found with name: {}", name);
            false
        }
    }

    pub fn decrypt_password(&mut self, name: &str) -> bool {
        // Find the password
        if let Some(pos) = self.meta.encrypted.data.passwords.0.iter().position(|p| p.display_name.0 == name) {
            // Remove the entry to take ownership
            let mut entry = self.meta.encrypted.data.passwords.0.remove(pos);

            // Get the item key from the key store
            let key = match self.crypto.item_key_store.get_key(&entry.id) {
                Some(encrypted_key) => match self.crypto.unwrap_item_key(&encrypted_key.0) {
                    Ok(k) => k,
                    Err(e) => {
                        println!("❌ Wrong master password or corrupted item key for {}: {}", name, e);
                        self.meta.encrypted.data.passwords.0.insert(pos, entry); // Put it back
                        return false;
                    }
                },
                None => {
                    println!("❌ Item key not found in store for: {}", name);
                    self.meta.encrypted.data.passwords.0.insert(pos, entry); // Put it back
                    return false;
                }
            };

            // Decrypt the encrypted password string
            let decrypted_password = match self.crypto.decrypt_string(&entry.password.0, &key) {
                Ok(dp) => dp,
                Err(e) => {
                    println!("❌ Password decryption failed for '{}': {}", name, e);
                    self.meta.encrypted.data.passwords.0.insert(pos, entry); // Put it back
                    return false;
                }
            };

            // Update entry and move to decrypted meta
            entry.is_encrypted = IsEncrypted(false);
            entry.password = Password(decrypted_password);
            entry.updated_at = Timestamp::now();
            self.meta.decrypted.data.passwords.0.push(entry);

            // Save both metadata files
            if let Err(e) = self.meta.encrypted.save() {
                eprintln!("⚠️ Failed to save encrypted metadata after removal: {}", e);
            }
            if let Err(e) = self.meta.decrypted.save() {
                println!("❌ Failed to save decrypted metadata: {}", e);
                return false;
            }

            println!("🔓 Password '{}' decrypted successfully.", name);
            true
        } else {
            println!("❌ No encrypted password found with name: {}", name);
            false
        }
    }

    // --- Batch Encryption/Decryption ---

    pub fn encrypt_all_passwords(&mut self) -> bool {
        let mut success_count = 0;
        let mut failure_count = 0;
        
        // Collect IDs of unencrypted passwords to process
        let password_ids: Vec<ItemId> = self.meta.decrypted.data.passwords.0
            .iter()
            .filter(|p| !p.is_encrypted.0)
            .map(|p| p.id.clone())
            .collect();

        let total_to_process = password_ids.len();

        if total_to_process == 0 {
            println!("ℹ️ No unencrypted passwords found to encrypt.");
            return true;
        }

        println!("🔒 Encrypting {} passwords...", total_to_process);

        // Iterate by ID and process
        for (current, id) in password_ids.iter().enumerate() {
            // Find the entry by ID
            if let Some(pos) = self.meta.decrypted.data.passwords.0.iter().position(|p| &p.id == id) {
                let mut entry = self.meta.decrypted.data.passwords.0.remove(pos);
                let name = entry.display_name.0.clone();

                print!("[{}/{}] Encrypting '{}'... ", current + 1, total_to_process, name);

                // Get the item key from the key store
                let key = match self.crypto.item_key_store.get_key(&entry.id) {
                    Some(encrypted_key) => match self.crypto.unwrap_item_key(&encrypted_key.0) {
                        Ok(k) => k,
                        Err(e) => {
                            println!("❌ (key error: {})", e);
                            failure_count += 1;
                            self.meta.decrypted.data.passwords.0.insert(pos, entry);
                            continue;
                        }
                    },
                    None => {
                        println!("❌ (key not found)");
                        failure_count += 1;
                        self.meta.decrypted.data.passwords.0.insert(pos, entry);
                        continue;
                    }
                };

                // Encrypt the password
                match self.crypto.encrypt_string(&entry.password.0, &key) {
                    Ok(encrypted_password) => {
                        entry.is_encrypted = IsEncrypted(true);
                        entry.password = Password(encrypted_password);
                        entry.updated_at = Timestamp::now();
                        self.meta.encrypted.data.passwords.0.push(entry);
                        success_count += 1;
                        println!("✅");
                    }
                    Err(e) => {
                        println!("❌ (encryption failed: {})", e);
                        failure_count += 1;
                        self.meta.decrypted.data.passwords.0.insert(pos, entry);
                    }
                }
            } else {    
                println!("❌ (password not found/already encrypted)");
                failure_count += 1;
            }   
        }   

        // Save metadata files if we had successful operations
        if success_count > 0 {
            let _ = self.meta.decrypted.save();
            let _ = self.meta.encrypted.save();
        }   

        println!("\n📊 Encryption Summary: Success: {}, Failed: {}", success_count, failure_count);
        failure_count == 0
    }

    pub fn decrypt_all_passwords(&mut self) -> bool {
        let mut success_count = 0;
        let mut failure_count = 0;
        
        // Collect IDs of encrypted passwords to process
        let password_ids: Vec<ItemId> = self.meta.encrypted.data.passwords.0
            .iter()
            .filter(|p| p.is_encrypted.0)
            .map(|p| p.id.clone())
            .collect();

        let total_to_process = password_ids.len();

        if total_to_process == 0 {
            println!("ℹ️ No encrypted passwords found to decrypt.");
            return true;
        }

        println!("🔓 Decrypting {} passwords...", total_to_process);

        // Iterate by ID and process
        for (current, id) in password_ids.iter().enumerate() {
            // Find the entry by ID
            if let Some(pos) = self.meta.encrypted.data.passwords.0.iter().position(|p| &p.id == id) {
                let mut entry = self.meta.encrypted.data.passwords.0.remove(pos);
                let name = entry.display_name.0.clone();

                print!("[{}/{}] Decrypting '{}'... ", current + 1, total_to_process, name);

                // Get the item key from the key store
                let key = match self.crypto.item_key_store.get_key(&entry.id) {
                    Some(encrypted_key) => match self.crypto.unwrap_item_key(&encrypted_key.0) {
                        Ok(k) => k,
                        Err(e) => {
                            println!("❌ (key error: {})", e);
                            failure_count += 1;
                            self.meta.encrypted.data.passwords.0.insert(pos, entry);
                            continue;
                        }
                    },
                    None => {
                        println!("❌ (key not found)");
                        failure_count += 1;
                        self.meta.encrypted.data.passwords.0.insert(pos, entry);
                        continue;
                    }
                };

                // Decrypt the password
                match self.crypto.decrypt_string(&entry.password.0, &key) {
                    Ok(decrypted_password) => {
                        entry.is_encrypted = IsEncrypted(false);
                        entry.password = Password(decrypted_password);
                        entry.updated_at = Timestamp::now();
                        self.meta.decrypted.data.passwords.0.push(entry);
                        success_count += 1;
                        println!("✅");
                    }
                    Err(e) => {
                        println!("❌ (decryption failed: {})", e);
                        failure_count += 1;
                        self.meta.encrypted.data.passwords.0.insert(pos, entry);
                    }
                }
            } else {    
                println!("❌ (password not found/already decrypted)");
                failure_count += 1;
            }   
        }   

        // Save metadata files if we had successful operations
        if success_count > 0 {
            let _ = self.meta.decrypted.save();
            let _ = self.meta.encrypted.save();
        }   

        println!("\n📊 Decryption Summary: Success: {}, Failed: {}", success_count, failure_count);
        failure_count == 0
    }

}