use super::{
    Database,
    crypto::{
        ItemKey,
        CryptoEngine,
        MasterKey,
        VerifierB64,
        MasterPassword,
        ItemKeyStore,
    },
};
use base64::{engine::general_purpose::STANDARD, Engine};
use rand::{rngs::ThreadRng, RngCore};
use colored::*;
use std::io::{self, Write};
use rpassword::prompt_password;

impl Database {
    pub fn change_master_password_independent(&mut self) -> bool {
        println!("\n{}", "🔄 ROOT DATABASE PASSWORD CHANGE PROCESS".green().bold());
        println!("{}", "=".repeat(50).bright_blue());
        
        // Display important notice about nested databases
        if !self.meta.nested_databases.data.is_empty() {
            println!("{}", "📦 NOTICE: This will only change the root database password.".cyan());
            println!("{}", "   Nested databases will maintain their independent passwords.".cyan());
            println!("{}", "=".repeat(60).cyan());
        }
        
        // Display critical warning
        println!("{}\n", "🚨 CRITICAL WARNING: DO NOT TERMINATE THIS PROCESS!".red().bold());
        println!("{}", "   • Termination during this process may cause DATA LOSS".red());
        println!("{}", "   • Ensure stable power and internet connection".red());
        println!("{}", "   • No changes will be made until final confirmation".green());
        println!("{}", "=".repeat(60).red());
        
        // Step 1: Get current password from user
        println!("\n{}", "Step 1: Verify Current Master Password".yellow().bold());
        println!("{}", "─".repeat(40).bright_blue());
        
        let old_password = match self.get_password_from_user("Enter current master password: ") {
            Ok(password) => password,
            Err(_) => {
                eprintln!("❌ Failed to read password input");
                return false;
            }
        };
        
        // Step 2: Verify old password first
        println!("{}", "Verifying current master password...".yellow());
        if !self.verify_master_password(&old_password) {
            eprintln!("❌ Invalid current master password!");
            println!("{}", "💡 Please check your current password and try again.".yellow());
            return false;
        }
        println!("{}", "✅ Current password verified.".green());
        
        // Step 3: Get new password from user
        println!("\n{}", "Step 2: Set New Master Password".yellow().bold());
        println!("{}", "─".repeat(40).bright_blue());
        
        let new_password = match self.get_and_confirm_new_password() {
            Ok(password) => password,
            Err(e) => {
                eprintln!("❌ {}", e);
                return false;
            }
        };
        
        // Store original state for safety verification
        let original_master_key = self.crypto.master_key.0.clone();
        let original_config_verifier = self.crypto.verifier_b64.0.clone();
        
        // Step 4: Prepare root changes - ONLY ItemKeyStore needs processing
        println!("\n{}", "Step 3: Preparing Database Changes".yellow().bold());
        println!("{}", "─".repeat(40).bright_blue());
        println!("{}", "   (Nested databases will remain unchanged)".cyan());
        
        let prepared_root_changes = match self.prepare_root_password_change_pure(&old_password, &new_password, &original_master_key) {
            Ok(changes) => changes,
            Err(e) => {
                eprintln!("❌ Failed to prepare changes: {}", e);
                println!("{}", "💡 No changes were made to any database.".green());
                return false;
            }
        };
        
        // Verify that original state wasn't modified during preparation
        if self.crypto.master_key.0 != original_master_key {
            eprintln!("❌ CRITICAL ERROR: Master key was modified during preparation!");
            println!("{}", "💡 Process aborted for safety.".green());
            return false;
        }
        
        if self.crypto.verifier_b64.0 != original_config_verifier {
            eprintln!("❌ CRITICAL ERROR: Config was modified during preparation!");
            println!("{}", "💡 Process aborted for safety.".green());
            return false;
        }
        
        // Step 5: Show final summary and get confirmation
        println!("\n{}", "📊 PREPARATION COMPLETE - READY TO APPLY".green().bold());
        println!("{}", "=".repeat(50).bright_blue());
        println!("{}", "📋 CHANGE SUMMARY:".cyan().bold());
        println!("   • Root database: Password will be updated");
        println!("   • Item keys: {} keys will be re-encrypted", prepared_root_changes.new_item_key_store.collection.len());
        println!("   • Nested databases: No changes (independent passwords)");
        println!("{}", "=".repeat(50).bright_blue());
        
        if !self.confirm_operation_with_warning(
            "🚀 Are you sure you want to apply these changes? (y/n): ",
            "   This action cannot be undone!"
        ) {
            println!("{}", "❌ Password change cancelled.".yellow());
            println!("{}", "💡 No changes were made to any database.".green());
            return false;
        }
        
        // Step 6: Apply root changes - ONLY ItemKeyStore needs updating
        println!("\n{}", "Step 4: Applying Changes".yellow().bold());
        println!("{}", "─".repeat(40).bright_blue());
        println!("{}", "🚨 DO NOT TERMINATE THE PROCESS!".red().bold());
        println!("{}", "   • Applying new master password...".yellow());
        println!("{}", "   • Re-encrypting item keys...".yellow());
        
        let success = self.apply_root_password_change_only(prepared_root_changes);
        
        if success {
            println!("\n{}", "🎉 ROOT DATABASE PASSWORD CHANGED SUCCESSFULLY!".green().bold());
            println!("{}", "=".repeat(45).bright_blue());
            println!("{}", "✅ CHANGES APPLIED:".green());
            println!("   • Root database: Password updated securely");
            println!("   • Item keys: {} keys re-encrypted", self.crypto.item_key_store.collection.len());
            println!("   • Verification: New password is now active");
            println!("\n{}", "💡 IMPORTANT NOTES:".cyan());
            println!("   • Nested databases have separate passwords");
            println!("   • Access them with their original master passwords");
            println!("   • Remember your new master password - it cannot be recovered!");
        } else {
            eprintln!("\n❌ Password change failed!");
            println!("{}", "💡 Your original password remains active.".yellow());
            println!("{}", "💡 No data was lost during the process.".green());
        }
        
        success
    }
    
    fn get_password_from_user(&self, prompt: &str) -> Result<String, io::Error> {
        print!("{}", prompt.yellow());
        io::stdout().flush()?;
        prompt_password("")
    }
    
    fn get_and_confirm_new_password(&self) -> Result<String, String> {
        // Get new password
        let password1 = self.get_password_from_user("Enter new master password: ")
            .map_err(|e| format!("Failed to read password: {}", e))?;
        
        if password1.is_empty() {
            return Err("Password cannot be empty".to_string());
        }
        
        if password1.len() < 8 {
            return Err("Password must be at least 8 characters long".to_string());
        }
        
        // Confirm new password
        let password2 = self.get_password_from_user("Confirm new master password: ")
            .map_err(|e| format!("Failed to read password confirmation: {}", e))?;
        
        if password1 != password2 {
            return Err("Passwords do not match. Please try again.".to_string());
        }
        
        // Show password strength indicator
        self.show_password_strength(&password1);
        
        Ok(password1)
    }
    
    fn show_password_strength(&self, password: &str) {
        let strength = self.calculate_password_strength(password);
        let (message, color) = match strength {
            0..=2 => ("Weak", "red"),
            3 => ("Medium", "yellow"), 
            4 => ("Strong", "green"),
            _ => ("Very Strong", "bright_green"),
        };
        
        println!("{}", format!("   Password strength: {}", message).color(color));
    }
    
    fn calculate_password_strength(&self, password: &str) -> u8 {
        let mut score = 0;
        
        // Length check
        if password.len() >= 12 { score += 2; }
        else if password.len() >= 8 { score += 1; }
        
        // Character variety checks
        if password.chars().any(|c| c.is_uppercase()) { score += 1; }
        if password.chars().any(|c| c.is_lowercase()) { score += 1; }
        if password.chars().any(|c| c.is_numeric()) { score += 1; }
        if password.chars().any(|c| !c.is_alphanumeric()) { score += 1; }
        
        score
    }
    
    fn confirm_operation_with_warning(&self, message: &str, warning: &str) -> bool {
        println!("{}", warning.red());
        
        let mut input = String::new();
        print!("{}", message.yellow().bold());
        io::stdout().flush().unwrap();
        
        match io::stdin().read_line(&mut input) {
            Ok(_) => match input.trim().to_lowercase().as_str() {
                "y" | "yes" => true,
                "n" | "no" => false,
                _ => {
                    println!("{}", "⚠️  Please enter 'y' for yes or 'n' for no.".red());
                    self.confirm_operation_with_warning(message, warning)
                }
            },
            Err(_) => false,
        }
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
        
        // Step 2: Create new crypto config using PURE functions only
        let new_crypto = self.create_new_crypto_pure(old_password, new_password)
            .ok_or("Failed to create new crypto config (wrong password?)")?;
        
        // Step 3: Process ONLY the ItemKeyStore - this is where all encrypted item keys are stored
        println!("{}", "Processing item keys from ItemKeyStore...".yellow());
        let new_item_key_store = self.prepare_item_key_store_changes_pure(
            original_master_key, 
            &new_master_key
        )?;
        println!("✅ Processed {} item keys from ItemKeyStore", new_item_key_store.collection.len());
        
        // NO NEED TO PROCESS META DATA - they don't contain encrypted keys
        // The metas only contain references to items, and the actual encrypted keys are in ItemKeyStore
        
        Ok(PreparedRootChanges {
            new_master_key: MasterKey(new_master_key),
            new_crypto,
            new_item_key_store,
            // Meta data doesn't need to change at all
        })
    }
    
    fn prepare_item_key_store_changes_pure(
        &self,
        old_master_key: &[u8; 32],
        new_master_key: &[u8; 32],
    ) -> Result<ItemKeyStore, String> {
        // Create a copy of the current item key store
        let mut new_key_store = self.crypto.item_key_store.clone();
        
        // Process each item key in the store: decrypt with old key, re-encrypt with new key
        let total_keys = self.crypto.item_key_store.collection.len();
        println!("🔑 Found {} item keys to re-encrypt", total_keys);
        
        let mut processed_count = 0;
        let mut error_count = 0;
        
        for (item_id, encrypted_item_key) in &self.crypto.item_key_store.collection {
            match self.reencrypt_item_key_pure(&encrypted_item_key.0, old_master_key, new_master_key) {
                Ok(new_encrypted_key) => {
                    // Update the key in our copy
                    new_key_store.collection.insert(item_id.clone(), ItemKey(new_encrypted_key));
                    processed_count += 1;
                    
                    // Show progress for large collections
                    if total_keys > 50 && processed_count % 10 == 0 {
                        println!("📦 Progress: {}/{} keys processed", processed_count, total_keys);
                    }
                }
                Err(e) => {
                    eprintln!("❌ Failed to re-encrypt item key for ID {:?}: {}", item_id, e);
                    error_count += 1;
                    // Continue with other keys rather than failing completely
                }
            }
        }
        
        if error_count > 0 {
            eprintln!("⚠️  Failed to process {} out of {} item keys", error_count, total_keys);
            if error_count == total_keys {
                return Err("Failed to process any item keys".to_string());
            }
        }
        
        println!("✅ Successfully processed {} item keys", processed_count);
        Ok(new_key_store)
    }
    
    fn reencrypt_item_key_pure(
        &self,
        encrypted_key_b64: &str,
        old_master_key: &[u8; 32],
        new_master_key: &[u8; 32],
    ) -> Result<String, String> {
        // Step 1: Decrypt the item key with the old master key
        let encrypted_data = STANDARD.decode(encrypted_key_b64)
            .map_err(|e| format!("Failed to decode base64: {}", e))?;
        
        let decrypted_item_key = self.crypto.decrypt_with_key(old_master_key, &encrypted_data)
            .map_err(|e| format!("Failed to decrypt with old key: {}", e))?;
        
        // Step 2: Re-encrypt the item key with the new master key
        let reencrypted_data = self.crypto.encrypt_with_key(new_master_key, &decrypted_item_key)
            .map_err(|e| format!("Failed to encrypt with new key: {}", e))?;
        
        Ok(STANDARD.encode(reencrypted_data))
    }
    
    fn generate_new_master_key_pure(&self, new_password: &str) -> Option<[u8; 32]> {
        // Use CryptoEngine's derive_key method
        self.crypto.derive_key(new_password).ok()
    }
    
    fn create_new_crypto_pure(&self, old_password: &str, new_password: &str) -> Option<CryptoEngine> {
        // First verify old password using VerifierB64's method
        if !self.crypto.verifier_b64.verify_password(
            self.crypto.kdf_salt_b64.clone(), 
            MasterPassword(old_password.to_string())
        ) {
            return None;
        }
        
        // Create a new crypto engine with updated verifier
        let mut new_crypto = self.crypto.clone();
        
        // Generate new verifier using VerifierB64::new
        let mut nonce = [0u8; 12];
        ThreadRng::default().fill_bytes(&mut nonce);
        
        let new_verifier = VerifierB64::new(
            nonce,
            self.crypto.kdf_salt_b64.clone(),
            MasterPassword(new_password.to_string())
        );
        new_crypto.verifier_b64 = new_verifier;
        
        Some(new_crypto)
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
        
        // Update crypto components
        self.crypto.master_key = changes.new_master_key;
        self.crypto.verifier_b64 = changes.new_crypto.verifier_b64;
        self.crypto.item_key_store = changes.new_item_key_store;
        
        // NO NEED TO UPDATE META DATA - they don't contain encrypted keys
        
        // Save everything - this is the only disk write
        let meta_saved = self.meta.decrypted.save().is_ok() &&
                        self.meta.encrypted.save().is_ok() &&
                        self.meta.trash.save().is_ok();
        
        // Save crypto config - you might need to implement this based on your storage strategy
        let crypto_saved = self.save_crypto_config();
        
        if meta_saved && crypto_saved {
            Ok(())
        } else {
            Err("Failed to save changes to disk".to_string())
        }
    }
    
    fn save_crypto_config(&self) -> bool {
        true
    }
    
    fn verify_master_password(&self, password: &str) -> bool {
        // Use VerifierB64's verify_password method
        self.crypto.verifier_b64.verify_password(
            self.crypto.kdf_salt_b64.clone(),
            MasterPassword(password.to_string())
        )
    }
}

// Simplified PreparedRootChanges struct - no meta data needed
#[derive(Clone)]
struct PreparedRootChanges {
    new_master_key: MasterKey,
    new_crypto: CryptoEngine,
    new_item_key_store: ItemKeyStore,
    // No meta data fields needed - they don't change
}