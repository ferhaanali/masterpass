use super::{
    structs::{Database,PasswordEntry},
    time,
    enc_keys::{unwrap_item_key, wrap_item_key, generate_item_key}
};



impl Database {
    // Add password
    pub fn add_password(&mut self, name: &str, password: &str) -> bool {
        for password in &self.meta.decrypted_meta.data.passwords {
            if name == password.name {
                println!("❌ Password with this name already exists: {}",name);
                return false;
            }
        }
        for password in &self.meta.encrypted_meta.data.passwords {
            if name == password.name {
                println!("❌ Password with this name already exists in Encrypted: {}",name);
                return false;
            }
        }

        let item_key = generate_item_key();
        let encrypted_item_key = match wrap_item_key(&item_key, &self.master.key) {
            Some(eik) => eik,
            None => {
                println!("❌ Failed to generate encrypted item key for password: {}", name);
                return false;
            }
        };

        self.meta.decrypted_meta.data.passwords.push(PasswordEntry {
            name: name.to_string(),
            password: password.to_string(),
            encrypted_item_key,
            is_encrypted: false,
            is_recycled: false,
            created_at: time::now(),
            updated_at: 0.to_string(),
        });
        self.meta.decrypted_meta.save();
        println!("✅ Password added: {}", name);
        true
    }

    pub fn change_password(&mut self,name: &str, new_password: &str) -> bool {
        if let Some(entry) = self.meta.decrypted_meta.data.passwords.iter_mut().find(|e| e.name == name) {
            if entry.password == new_password {
                println!("❌ New password is the same as the old password for: {}", name);
                return false; // No change needed
            }
           // Update the password
            entry.password = new_password.to_string();
            entry.updated_at = time::now(); // bump updated_at
            self.meta.decrypted_meta.save();      // or self.save_meta() depending on your design
            println!("🔄 Password updated for: {}", name);
            true
        } else {
            println!("❌ Password '{}' not found. If it’s encrypted, decrypt it first before modifying.", name);
            false // password not found
        }
    }

    pub fn encrypt_password(&mut self, name: &str) -> bool {
        // Find the index of the password in meta
        if let Some(pos) = self.meta.decrypted_meta.data.passwords.iter().position(|p| p.name == name) {
            let mut entry = self.meta.decrypted_meta.data.passwords.remove(pos);

            if entry.is_encrypted {
                println!("❌ Password '{}' is already encrypted.", name);
                // Put it back
                self.meta.decrypted_meta.data.passwords.insert(pos, entry);
                return false;
            }

            // Encrypt the password
            let key = match unwrap_item_key(&entry.encrypted_item_key, &self.master.key) {
                Some(k) => k,
                None => {
                    println!("❌ Wrong password or corrupted String: {}", name);
                    // Put it back
                    self.meta.decrypted_meta.data.passwords.insert(pos, entry);
                    return false;
                }
            };
            let encrypted_password = match self.encrypt_string(&entry.password, &key) {
                Some(ep) => ep,
                None => {
                    println!("❌ Password encryption failed for '{}'", name);
                    // Put it back
                    self.meta.decrypted_meta.data.passwords.insert(pos, entry);
                    return false;
                }
            };

            // Update entry
            entry.is_encrypted = true;
            entry.password = encrypted_password;
            entry.updated_at = time::now();

            // Move to encrypted meta
            self.meta.encrypted_meta.data.passwords.push(entry);

            self.meta.decrypted_meta.save();
            self.meta.encrypted_meta.save();

            println!("🔓 Password '{}' encrypted successfully.", name);
            true
        } else {
            println!("❌ No password found with name: {}", name);
            false
        }
    }

    pub fn decrypt_password(&mut self, name: &str) -> bool {
        // Find the index of the password in encrypted_meta
        if let Some(pos) = self.meta.encrypted_meta.data.passwords.iter().position(|p| p.name == name) {
            let mut entry = self.meta.encrypted_meta.data.passwords.remove(pos);

            if !entry.is_encrypted {
                println!("❌ Password '{}' is not encrypted.", name);
                // Put it back
                self.meta.encrypted_meta.data.passwords.insert(pos, entry);
                return false;
            }

            // Decrypt the password
            let key = match unwrap_item_key(&entry.encrypted_item_key, &self.master.key) {
                Some(k) => k,
                None => {
                    println!("❌ Wrong password or corrupted String: {}", name);
                    // Put it back
                    self.meta.encrypted_meta.data.passwords.insert(pos, entry);
                    return false;
                }
            };
            let decrypted_password = match self.decrypt_string(&entry.password, &key) {
                Some(dp) => dp,
                None => {
                    println!("❌ Wrong password or corrupted saved password '{}'", name);
                    // Put it back
                    self.meta.encrypted_meta.data.passwords.insert(pos, entry);
                    return false;
                }
            };

            // Update entry
            entry.is_encrypted = false;
            entry.password = decrypted_password;
            entry.updated_at = time::now();

            // Move to main meta
            self.meta.decrypted_meta.data.passwords.push(entry);

            self.meta.decrypted_meta.save();
            self.meta.encrypted_meta.save();

            println!("🔓 Password '{}' decrypted successfully.", name);
            true
        } else {
            println!("❌ No encrypted password found with name: {}", name);
            false
        }
    }

    pub fn encrypt_all_passwords(&mut self) -> bool {
        let mut success_count = 0;
        let mut failure_count = 0;
        
            // Collect names instead of indices to avoid index shifting issues
            let password_names: Vec<String> = self.meta.decrypted_meta.data.passwords
                .iter()
                .filter(|p| !p.is_encrypted)
                .map(|p| p.name.clone())
                .collect();
        
            let total_to_process = password_names.len();
        
            if total_to_process == 0 {
                println!("ℹ️  No unencrypted passwords found to encrypt.");
                return true;
            }
    
            println!("🔒 Encrypting {} passwords...", total_to_process);
    
            // Process each password by name (which is stable)
            for (current, name) in password_names.iter().enumerate() {
                print!("[{}/{}] Encrypting '{}'... ", current + 1, total_to_process, name);
        
                // Find the password by name instead of index
                if let Some(pos) = self.meta.decrypted_meta.data.passwords.iter().position(|p| &p.name == name && !p.is_encrypted) {
                    let mut entry = self.meta.decrypted_meta.data.passwords.remove(pos);
            
                        let key = match unwrap_item_key(&entry.encrypted_item_key, &self.master.key) {
                            Some(k) => k,
                            None => {
                                println!("❌ (key error)");
                                failure_count += 1;
                                // Put it back since we failed
                                self.meta.decrypted_meta.data.passwords.insert(pos, entry);
                                continue;
                            }
                        };
            
                        match self.encrypt_string(&entry.password, &key) {
                            Some(encrypted_pass) => {
                                entry.password = encrypted_pass;
                                entry.updated_at = time::now();
                                entry.is_encrypted = true;
                    
                                    // Move to encrypted_meta
                                    self.meta.encrypted_meta.data.passwords.push(entry);
                                success_count += 1;
                                println!("✅");
                            }
                            None => {
                                println!("❌ (encryption failed)");
                                failure_count += 1;
                                // Put it back since we failed
                                self.meta.decrypted_meta.data.passwords.insert(pos, entry);
                            }
                    }   
                } else {    
                    println!("❌ (password not found)");
                    failure_count += 1;
                }   
        }   
    
        // Save both meta files if we had any changes
        if success_count > 0 {
            self.meta.decrypted_meta.save();
            self.meta.encrypted_meta.save();
        }   
    
        // Log results
        println!("\n📊 Encryption Summary:");
        println!("   Total:    {}", total_to_process);
        println!("   Success:  {}", success_count);
        println!("   Failed:   {}", failure_count);
    
        if success_count > 0 && failure_count == 0 {
            println!("✅ Successfully encrypted all {} passwords.", success_count);
        } else if success_count > 0 {
            println!("⚠️  Encrypted {}/{} passwords. {} failed.", success_count, total_to_process, failure_count);
        } else {
            println!("❌ Failed to encrypt any passwords. {}/{} failed.", failure_count, total_to_process);
        }
    
        failure_count == 0
    }

    pub fn decrypt_all_passwords(&mut self) -> bool {
        let mut success_count = 0;
        let mut failure_count = 0;
        let mut i = 0;

        // Collect encrypted passwords first to get count and names
        let encrypted_passwords: Vec<(usize, String)> = self.meta.encrypted_meta.data.passwords
            .iter()
            .enumerate()
            .filter(|(_, p)| p.is_encrypted)
            .map(|(i, p)| (i, p.name.clone()))
            .collect();

        let total_to_process = encrypted_passwords.len();

        if total_to_process == 0 {
            println!("ℹ️  No encrypted passwords found to decrypt.");
            return true;
        }

        println!("🔓 Decrypting {} passwords...", total_to_process);

        let mut current = 0;
        while i < self.meta.encrypted_meta.data.passwords.len() {
            let pass_entry = &self.meta.encrypted_meta.data.passwords[i];

            if !pass_entry.is_encrypted {
                i += 1;
                continue;
            }

            current += 1;
            print!("[{}/{}] Decrypting '{}'... ", current, total_to_process, pass_entry.name);

            let key = match unwrap_item_key(&pass_entry.encrypted_item_key, &self.master.key) {
                Some(k) => k,
                None => {
                    println!("❌ (key error)");
                    failure_count += 1;
                    i += 1;
                    continue;
                }
            };

            let decrypted_password = match self.decrypt_string(&pass_entry.password, &key) {
                Some(dp) => dp,
                None => {
                    println!("❌ (decryption failed)");
                    failure_count += 1;
                    i += 1;
                    continue;
                }
            };

            // Update and move to main meta
            let mut decrypted_entry = self.meta.encrypted_meta.data.passwords.remove(i);
            decrypted_entry.password = decrypted_password;
            decrypted_entry.updated_at = time::now();
            decrypted_entry.is_encrypted = false;

            self.meta.decrypted_meta.data.passwords.push(decrypted_entry);
            success_count += 1;
            println!("✅");
            // Don't increment i since we removed the current element
        }

        if success_count > 0 || failure_count > 0 {
            self.meta.decrypted_meta.save();
            self.meta.encrypted_meta.save();
        }

        // Log results
        println!("\n📊 Decryption Summary:");
        println!("   Total:    {}", total_to_process);
        println!("   Success:  {}", success_count);
        println!("   Failed:   {}", failure_count);

        if success_count > 0 && failure_count == 0 {
            println!("✅ Successfully decrypted all {} passwords.", success_count);
        } else if success_count > 0 {
            println!("⚠️  Decrypted {}/{} passwords. {} failed.", success_count, total_to_process, failure_count);
        } else {
            println!("❌ Failed to decrypt any passwords. {}/{} failed.", failure_count, total_to_process);
        }

        failure_count == 0
    }
}