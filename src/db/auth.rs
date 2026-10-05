use super::{
    structs::{Database, Config, PreparedRootChanges},
    enc_keys::{unwrap_item_key, wrap_item_key},
    colored::*,
    std::io::{self, Write},
};




impl Database {
    pub fn change_master_password_independent(
        &mut self, 
        old_password: &str, 
        new_password: &str
    ) -> bool {
        println!("\n{}", "🔄 ROOT DATABASE PASSWORD CHANGE PROCESS".green().bold());
        println!("{}", "=".repeat(50).bright_blue());
        
        // Display important notice about nested databases
        if !self.meta.nested_db_meta.data.is_empty() {
            println!("{}", "📦 NOTICE: This will only change the root database password.".cyan());
            println!("{}", "   Nested databases will maintain their independent passwords.".cyan());
            println!("{}", "=".repeat(60).cyan());
        }
        
        // Store original state for safety verification
        let original_master_key = self.master.key.clone();
        let original_config_verifier = self.config.verifier_b64.clone();
        
        // Display critical warning
        println!("{}\n", "🚨 CRITICAL WARNING: DO NOT TERMINATE THIS PROCESS!".red().bold());
        println!("{}", "   • Termination during this process may cause DATA LOSS".red());
        println!("{}", "   • Ensure stable power and internet connection".red());
        println!("{}", "   • No changes will be made until final confirmation".green());
        println!("{}", "=".repeat(60).red());
        
        // Step 1: Verify old password first
        println!("\n{}", "Step 1: Verifying current master password...".yellow());
        if !self.verify_master_password(old_password) {
            eprintln!("❌ Invalid current master password!");
            return false;
        }
        println!("{}", "✅ Current password verified.".green());
        
        // Step 2: Prepare root changes only (no nested database modifications)
        println!("\n{}", "Step 2: Preparing root database changes...".yellow());
        println!("{}", "   (Nested databases will remain unchanged)".green());
        
        let prepared_root_changes = match self.prepare_root_password_change_pure(old_password, new_password, &original_master_key) {
            Ok(changes) => changes,
            Err(e) => {
                eprintln!("❌ Failed to prepare changes: {}", e);
                println!("{}", "💡 No changes were made to any database.".green());
                return false;
            }
        };
        
        // Verify that original state wasn't modified during preparation
        if self.master.key != original_master_key {
            eprintln!("❌ CRITICAL ERROR: Master key was modified during preparation!");
            println!("{}", "💡 Process aborted for safety.".green());
            return false;
        }
        
        if self.config.verifier_b64 != original_config_verifier {
            eprintln!("❌ CRITICAL ERROR: Config was modified during preparation!");
            println!("{}", "💡 Process aborted for safety.".green());
            return false;
        }
        
        // Step 3: Show final summary and get confirmation
        println!("\n{}", "📊 PREPARATION COMPLETE - READY TO APPLY".green().bold());
        println!("{}", "=".repeat(50).bright_blue());
        println!("✅ Root database: Changes prepared in memory");
        
        if !self.confirm_operation_with_warning("Apply root database password change now? (y/n): ") {
            println!("{}", "❌ Password change cancelled.".yellow());
            println!("{}", "💡 No changes were made to any database.".green());
            return false;
        }
        
        // Step 4: Apply root changes only
        println!("\n{}", "Step 3: Applying root database changes...".yellow());
        println!("{}", "🚨 DO NOT TERMINATE THE PROCESS!".red().bold());
        
        let success = self.apply_root_password_change_only(prepared_root_changes);
        
        if success {
            println!("\n{}", "🎉 ROOT DATABASE PASSWORD CHANGED SUCCESSFULLY!".green().bold());
            println!("{}", "=".repeat(45).bright_blue());
            println!("✅ Root database: Password changed securely");
            println!("\n{}", "💡 Remember: Nested databases have separate passwords.".cyan());
            println!("{}", "💡 Access them with their original master passwords.".cyan());
        } else {
            eprintln!("\n❌ Password change failed!");
            println!("{}", "💡 Your original password remains active.".yellow());
        }
        
        success
    }
    
    fn prepare_root_password_change_pure(
        &self,
        old_password: &str,
        new_password: &str,
        original_master_key: &[u8; 32],
    ) -> Result<PreparedRootChanges, String> {
        // CRITICAL: This method must be PURELY READ-ONLY
        // It should not modify ANY state and should not call any methods that modify state
        
        // Step 1: Generate new master key using PURE functions only
        let new_master_key = self.generate_new_master_key_pure(new_password)
            .ok_or("Failed to generate new master key")?;
        
        // Step 2: Create new config using PURE functions only
        let new_config = self.create_new_config_pure(old_password, new_password)
            .ok_or("Failed to create new config (wrong password?)")?;
        
        // Helper closure to rewrap item keys (pure function)
        let rewrap = |enc_b64: &str, old_key: &[u8], new_key: &[u8]| -> Option<String> {
            if enc_b64.is_empty() { return Some(String::new()); }
            let item_key = unwrap_item_key(enc_b64, old_key)?;
            wrap_item_key(&item_key, new_key)
        };
        
        // Create COPIES of all metas with rewrapped keys
        let mut new_decrypted_meta = self.meta.decrypted_meta.data.clone();
        let mut new_encrypted_meta = self.meta.encrypted_meta.data.clone();
        let mut new_trash_meta = self.meta.trash_meta.data.clone();
        
        // Rewrap passwords in the copies using the ORIGINAL master key
        for entry in &mut new_decrypted_meta.passwords {
            entry.encrypted_item_key = rewrap(&entry.encrypted_item_key, original_master_key, &new_master_key)
                .ok_or_else(|| format!("Failed to rewrap password '{}'", entry.name))?;
        }
        
        for entry in &mut new_encrypted_meta.passwords {
            entry.encrypted_item_key = rewrap(&entry.encrypted_item_key, original_master_key, &new_master_key)
                .ok_or_else(|| format!("Failed to rewrap encrypted password '{}'", entry.name))?;
        }
        
        for entry in &mut new_trash_meta.passwords {
            entry.encrypted_item_key = rewrap(&entry.encrypted_item_key, original_master_key, &new_master_key)
                .ok_or_else(|| format!("Failed to rewrap trashed password '{}'", entry.name))?;
        }
        
        // Rewrap files in the copies using the ORIGINAL master key
        for entry in &mut new_decrypted_meta.files {
            entry.encrypted_item_key = rewrap(&entry.encrypted_item_key, original_master_key, &new_master_key)
                .ok_or_else(|| format!("Failed to rewrap file '{}'", entry.name))?;
        }
        
        for entry in &mut new_encrypted_meta.files {
            entry.encrypted_item_key = rewrap(&entry.encrypted_item_key, original_master_key, &new_master_key)
                .ok_or_else(|| format!("Failed to rewrap encrypted file '{}'", entry.name))?;
        }
        
        for entry in &mut new_trash_meta.files {
            entry.encrypted_item_key = rewrap(&entry.encrypted_item_key, original_master_key, &new_master_key)
                .ok_or_else(|| format!("Failed to rewrap trashed file '{}'", entry.name))?;
        }
        
        Ok(PreparedRootChanges {
            new_master_key,
            new_decrypted_meta,
            new_encrypted_meta,
            new_trash_meta,
            new_config,
        })
    }
    
    fn generate_new_master_key_pure(&self, new_password: &str) -> Option<[u8; 32]> {
        // Pure function to generate master key without side effects
        use pbkdf2::pbkdf2_hmac;
        use sha2::Sha256;
        use base64::{engine::general_purpose, Engine as _};
        
        let salt_bytes = general_purpose::STANDARD.decode(&self.config.kdf_salt_b64).ok()?;
        let mut key = [0u8; 32];
        pbkdf2_hmac::<Sha256>(new_password.as_bytes(), &salt_bytes, 100_000, &mut key);
        Some(key)
    }
    
    fn create_new_config_pure(&self, old_password: &str, new_password: &str) -> Option<Config> {
        // Pure function to create new config without side effects
        
        // First verify old password
        if !self.config.check_verifier(old_password) {
            return None;
        }
        
        // Create a new config with updated verifier
        let mut new_config = self.config.clone();
        
        // Generate new verifier using pure functions
        let new_verifier = self.generate_verifier_pure(new_password)?;
        new_config.verifier_b64 = new_verifier;
        
        Some(new_config)
    }
    
    fn generate_verifier_pure(&self, password: &str) -> Option<String> {
        // Pure function to generate verifier without side effects
        use aes_gcm_siv::{Aes256GcmSiv, Key, Nonce};
        use aes_gcm_siv::aead::{Aead, KeyInit};
        use base64::{engine::general_purpose, Engine as _};
        use rand::rngs::ThreadRng;
        use rand::RngCore;
        
        // Generate master key for the new password
        let key_bytes = self.generate_new_master_key_pure(password)?;
        let key = Key::<Aes256GcmSiv>::from_slice(&key_bytes);
        let cipher = Aes256GcmSiv::new(key);
        
        // Generate nonce
        let mut nonce = [0u8; 12];
        ThreadRng::default().fill_bytes(&mut nonce);
        
        // Encrypt verifier
        let ciphertext = cipher.encrypt(Nonce::from_slice(&nonce), b"verify" as &[u8]).ok()?;
        
        let mut result = nonce.to_vec();
        result.extend(ciphertext);
        Some(general_purpose::STANDARD.encode(&result))
    }
    
    fn apply_root_password_change_only(&mut self, changes: PreparedRootChanges) -> bool {
        println!("{}", "Applying changes to root database only...".yellow());
        
        // Apply root database changes
        match self.apply_root_password_change(changes) {
            Ok(_) => {
                println!("{}", "✅ Root database updated successfully.".green());
                true
            }
            Err(e) => {
                eprintln!("❌ Failed to apply root database changes: {}", e);
                false
            }
        }
    }
    
    fn apply_root_password_change(&mut self, changes: PreparedRootChanges) -> Result<(), String> {
        // THIS IS THE ONLY POINT WHERE WE MODIFY SELF
        self.master.key = changes.new_master_key;
        self.config = changes.new_config;
        self.meta.decrypted_meta.data = changes.new_decrypted_meta;
        self.meta.encrypted_meta.data = changes.new_encrypted_meta;
        self.meta.trash_meta.data = changes.new_trash_meta;
        
        // Save everything - this is the only disk write
        let meta_saved = self.meta.decrypted_meta.save() &&
                        self.meta.encrypted_meta.save() &&
                        self.meta.trash_meta.save();
        
        let config_saved = self.config.save();
        
        if meta_saved && config_saved {
            Ok(())
        } else {
            Err("Failed to save changes to disk".to_string())
        }
    }
    
    fn confirm_operation_with_warning(&self, message: &str) -> bool {
        let mut input = String::new();
        
        print!("{}", message.yellow());
        io::stdout().flush().unwrap();
        
        match io::stdin().read_line(&mut input) {
            Ok(_) => match input.trim().to_lowercase().as_str() {
                "y" | "yes" => true,
                "n" | "no" => false,
                _ => {
                    println!("{}", "⚠️  Please enter 'y' for yes or 'n' for no.".red());
                    self.confirm_operation_with_warning(message)
                }
            },
            Err(_) => false,
        }
    }
    
    fn verify_master_password(&self, password: &str) -> bool {
        // Pure verification without side effects
        self.config.check_verifier(password)
    }
}

// Keep the apply_prepared_changes method for completeness
impl Database {
    pub fn apply_prepared_changes(&mut self) -> bool {
        // This should apply changes that were prepared earlier
        // For now, just save the current state
        let meta_saved = self.meta.decrypted_meta.save() &&
                        self.meta.encrypted_meta.save() &&
                        self.meta.trash_meta.save();
        
        let config_saved = self.config.save();
        
        meta_saved && config_saved
    }
}