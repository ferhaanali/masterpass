use super::{
    MasterKey,
    KdfSaltB64,
    MasterPassword,
    VerifierB64,
    ItemKeyStore,
    nonce::generate_nonce,
    response::AppError,
    core::StringPath,
};
use serde::{Serialize, Deserialize};
use std::{
    path::Path,
};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CryptoEngine {
    #[serde(skip)]
    pub master_key: MasterKey,

    pub verifier_b64: VerifierB64,
    pub kdf_salt_b64: KdfSaltB64,

    #[serde(skip)]
    pub item_key_store: ItemKeyStore,

    pub system_path: StringPath,
}
impl CryptoEngine {
    pub fn new(folder_path: &str, master_password: MasterPassword) -> Result<Self, AppError> {
        if master_password.0.is_empty() {
            return Err(AppError::crypto_error("Master Password can't be empty"));
        }

        // construct paths
        let system_path = StringPath(format!("{}/{}", folder_path, ".crypto.enc"));
        let item_path = StringPath(format!("{}/{}", folder_path, ".crypto.item_keys.enc"));

        // Check if crypto data already exists
        let path = Path::new(&system_path.0);
        
        if path.exists() {
            // Load existing crypto data and verify password
            Self::load_existing(system_path, item_path, master_password)
        } else {
            // Create new crypto data
            Self::create_new(system_path, item_path, master_password)
        }
    }

    /// Create new crypto data (first time setup)
    fn create_new(
        system_path: StringPath,
        item_path: StringPath,
        master_password: MasterPassword
    ) -> Result<Self, AppError> {
        // generate salts + keys
        let kdf_salt_b64 = KdfSaltB64::new();
        let master_key = MasterKey::new(kdf_salt_b64.clone(), master_password.clone());
        let verifier_b64 = VerifierB64::new(
            generate_nonce(), 
            kdf_salt_b64.clone(), 
            master_password
        );

        let item_key_store = ItemKeyStore::new(item_path);

        let crypto_engine = Self {
            master_key,
            verifier_b64,
            kdf_salt_b64,
            item_key_store,
            system_path: system_path.clone(),
        };

        // Save the newly created crypto data
        crypto_engine.save()?;

        Ok(crypto_engine)
    }

    /// Load existing crypto data and verify password
    fn load_existing(
        system_path: StringPath,
        item_path: StringPath,
        master_password: MasterPassword
    ) -> Result<Self, AppError> {
        // Create a temporary instance to load data
        let mut temp_engine = Self {
            master_key: MasterKey::default(), // Will be set after verification
            verifier_b64: VerifierB64::default(),
            kdf_salt_b64: KdfSaltB64::default(),
            item_key_store: ItemKeyStore::new(item_path.clone()),
            system_path: system_path.clone(),
        };

        // Load the stored crypto data
        temp_engine.load().map_err(|e| {
            AppError::crypto_error(format!("Failed to load crypto data: {}", e))
        })?;

        // Verify the provided password against stored verifier
        if !temp_engine.verifier_b64.verify_password(
            temp_engine.kdf_salt_b64.clone(), 
            master_password.clone()
        ) {
            return Err(AppError::crypto_error("Wrong master password"));
        }

        // Password is correct, create the final instance with proper master key
        let master_key = MasterKey::new(
            temp_engine.kdf_salt_b64.clone(), 
            master_password
        );

        Ok(Self {
            master_key,
            verifier_b64: temp_engine.verifier_b64,
            kdf_salt_b64: temp_engine.kdf_salt_b64,
            item_key_store: temp_engine.item_key_store,
            system_path,
        })
    }

}
