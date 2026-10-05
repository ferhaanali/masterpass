use super::{
    time,
    structs::{ DatabaseStats, DatabaseArguments, Config, DBInfo },
    serde_json,
    std::{ fs, path::Path },
    rand::{ rngs::ThreadRng, RngCore },
    aes_gcm_siv::{
        Aes256GcmSiv, Key, Nonce,
        aead::{Aead, KeyInit}
    },
    pbkdf2::pbkdf2_hmac,
    sha2::Sha256,
    base64::{ engine::general_purpose, Engine as _ }
};



impl DBInfo {
    pub fn default() -> Self {
        Self {
            name: "".to_string(),
            created_at: "".to_string(),
            last_login: "".to_string(),
            owner: "".to_string(),
            description: "".to_string(),
            stats: DatabaseStats::default()
        }
    }
}

impl Config {
    /// Generate a random 128-bit salt, base64 encoded
    pub fn generate_salt() -> String {
        let mut salt = [0u8; 16];
        ThreadRng::default().fill_bytes(&mut salt);
        general_purpose::STANDARD.encode(&salt)
    }

    /// Default configuration
    pub fn default() -> Self {
        Self {
            db_version: "0.1.0".to_string(),
            kdf_salt_b64: Self::generate_salt(),
            verifier_b64: "".to_string(),
            max_file_size_mb: 100,
            file_path: None,
            is_nested: false,
            db_info: DBInfo::default(),
        }
    }

    /// Generate a random 96-bit nonce
    pub fn generate_nonce() -> [u8; 12] {
        let mut nonce = [0u8; 12];
        ThreadRng::default().fill_bytes(&mut nonce);
        nonce
    }

    /// Derive a 32-byte master key from password + salt using PBKDF2
    pub fn derive_master_key(&self, master_password: &str) -> [u8; 32] {
        let salt_bytes = general_purpose::STANDARD.decode(&self.kdf_salt_b64)
            .expect("Failed to decode salt");
        let mut key = [0u8; 32];
        pbkdf2_hmac::<Sha256>(master_password.as_bytes(), &salt_bytes, 100_000, &mut key);
        key
    }

    /// Encrypt the verifier string ("verify") with master password
    pub fn encrypt_verifier(&self, master_password: &str) -> (String, [u8; 32]) {
        let key = self.derive_master_key(master_password);
        let cipher = Aes256GcmSiv::new(Key::<Aes256GcmSiv>::from_slice(&key));
        let nonce = Self::generate_nonce();

        let ciphertext = cipher.encrypt(Nonce::from_slice(&nonce), b"verify" as &[u8])
            .expect("Encryption failed");

        let mut result = nonce.to_vec();
        result.extend(ciphertext);
        (general_purpose::STANDARD.encode(&result), key)
    }

    /// Check if provided password can decrypt verifier
    pub fn check_verifier(&self, master_password: &str) -> bool {
        let key = self.derive_master_key(master_password);
        let data = match general_purpose::STANDARD.decode(&self.verifier_b64) {
            Ok(d) => d,
            Err(_) => return false,
        };
        if data.len() < 12 { return false; }

        let (nonce_bytes, ciphertext) = data.split_at(12);
        let cipher = Aes256GcmSiv::new(Key::<Aes256GcmSiv>::from_slice(&key));
        match cipher.decrypt(Nonce::from_slice(nonce_bytes), ciphertext) {
            Ok(pt) => pt == b"verify",
            Err(_) => false,
        }
    }

    // Change password - PURE VERSION (no disk writes)
    pub fn change_master_password_pure(&self, old_password: &str, new_password: &str) -> Option<([u8; 32], Self)> {
        // Step 1: Check old password
        if !self.check_verifier(old_password) {
            return None;
        }

        // Step 2: Generate new verifier with new password
        let (new_verifier, master_key) = self.encrypt_verifier(new_password);
        
        // Create a new config with the updated verifier (no disk writes)
        let mut new_config = self.clone();
        new_config.verifier_b64 = new_verifier;

        Some((master_key, new_config))
    }

    // Original version - only for final application
    pub fn change_master_password(&mut self, old_password: &str, new_password: &str) -> Option<[u8; 32]> {
        // Step 1: Check old password
        if !self.check_verifier(old_password) {
            return None;
        }

        // Step 2: Generate new verifier with new password
        let (new_verifier, master_key) = self.encrypt_verifier(new_password);
        self.verifier_b64 = new_verifier;

        // Save to disk (this should only happen in the final apply step)
        if let Some(path) = &self.file_path {
            let _ = fs::write(path, serde_json::to_string_pretty(&self).unwrap())
                .map_err(|e| format!("Failed to write updated config: {e}"));
            Some(master_key)
        } else {
            None
        }
    }

    /// Load existing config or create new one; verify master password
    pub fn load_or_create(
        args: &DatabaseArguments,
        path: &str,
    ) -> Result<Self, String> {
        let path_ref = Path::new(path);

        // If file does not exist OR exists but is empty → create new config
        let should_create = !path_ref.exists()
            || fs::metadata(path_ref)
                .map(|m| m.len() == 0)
                .unwrap_or(true);

        if should_create {
            if let Some(parent) = path_ref.parent() {
                fs::create_dir_all(parent)
                    .map_err(|e| format!("Failed to create parent directory: {e}"))?;
            }

            let mut cfg = Self::default();
            cfg.is_nested = args.is_nested;

            let (verifier_b64, _) = cfg.encrypt_verifier(&args.master_password);
            cfg.verifier_b64 = verifier_b64;
            cfg.file_path = Some(path.to_string());
            cfg.db_info.name = args.db_name.clone();
            cfg.db_info.owner = args.owner.clone();
            cfg.db_info.description = args.description.clone();
            cfg.db_info.created_at = time::now();
            cfg.db_info.last_login = time::now();

            fs::write(path_ref, serde_json::to_string_pretty(&cfg).unwrap())
                .map_err(|e| format!("Failed to write config: {e}"))?;

            Ok(cfg)
        } else {
            // Load existing config
            let data = fs::read_to_string(path_ref)
                .map_err(|e| format!("Failed to read config: {e}"))?;

            // Extra guard: if file accidentally contains only whitespace → recreate
            if data.trim().is_empty() {
                return Self::load_or_create(args, path);
            }

            let mut cfg: Self = serde_json::from_str(&data)
                .map_err(|e| format!("Failed to parse config: {e}"))?;

            cfg.file_path = Some(path.to_string());
            cfg.db_info.last_login = time::now();

            if cfg.check_verifier(&args.master_password) {
                Ok(cfg)
            } else {
                Err("❌ Wrong master password! Exiting...".to_string())
            }
        }
    }

    pub fn save(&self) -> bool {
        let path = match self.file_path.as_ref() {
            Some(p) => p,
            None => {
                eprintln!("Config file path is not set.");
                return false;
            }
        };

        // Ensure the directory exists
        if let Some(parent) = Path::new(path).parent() {
            if let Err(e) = fs::create_dir_all(parent) {
                eprintln!("Failed to create config directory: {}", e);
                return false;
            }
        }

        let serialized = match serde_json::to_string_pretty(&self) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("Failed to serialize config: {}", e);
                return false;
            }
        };

        if let Err(e) = fs::write(path, serialized) {
            eprintln!("Failed to write config to file: {}", e);
            return false;
        }

        true
    }


}