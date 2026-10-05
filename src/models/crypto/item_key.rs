use super::{
    CryptoEngine,
    response::AppError,
};
use aes_gcm_siv::{
    Aes256GcmSiv, Key, Nonce,
    aead::{Aead, KeyInit},
};
use rand::{RngCore, rngs::ThreadRng};
use base64::{engine::general_purpose::STANDARD, Engine};
use pbkdf2::pbkdf2_hmac;
use sha2::Sha256;
use serde::{Serialize, Deserialize};
use rand::thread_rng;

#[derive(Serialize, Deserialize, Debug, Clone, Hash, PartialEq, Eq)]
pub struct ItemKey(pub String);

impl ItemKey {
    pub fn generate_vec_key() -> Vec<u8> {
        let mut key = [0u8; 32];
        thread_rng().fill_bytes(&mut key);
        key.to_vec()
    }

    pub fn from_str(key: impl Into<String>) -> Self {
        let key_string = key.into();
        if key_string.is_empty() {
            panic!("Item key can't be empty");
        }
        ItemKey(key_string)
    }
}

impl CryptoEngine {

    pub fn wrap_item_key(&self, item_key: &[u8]) -> Result<String, AppError> {
        let encrypted = self.encrypt_with_key(&self.master_key.0, item_key)
            .map_err(|e| e.with_context("Failed to wrap item key"))?;
        Ok(STANDARD.encode(encrypted))
    }

    pub fn unwrap_item_key(&self, encrypted_item_key: &str) -> Result<Vec<u8>, AppError> {
        let decoded = STANDARD.decode(encrypted_item_key)
            .map_err(|e| AppError::crypto_error(format!("Failed to decode base64 item key: {}", e))
                .with_context("Invalid base64 encoding for encrypted item key"))?;
        
        self.decrypt_with_key(&self.master_key.0, &decoded)
            .map_err(|e| e.with_context("Failed to unwrap item key"))
    }

    pub fn encrypt_with_key(&self, key: &[u8], plaintext: &[u8]) -> Result<Vec<u8>, AppError> {
        if key.len() != 32 {
            return Err(AppError::invalid_key(format!("Key must be 32 bytes, got {} bytes", key.len()))
                .with_context("Invalid key length for encryption"));
        }

        let cipher = Aes256GcmSiv::new(Key::<Aes256GcmSiv>::from_slice(key));
        let nonce = self.generate_nonce();
        
        let ciphertext = cipher.encrypt(Nonce::from_slice(&nonce), plaintext)
            .map_err(|e| AppError::encryption_failed(format!("AES-GCM-SIV encryption failed: {}", e))
                .with_context("Failed to encrypt data with provided key"))?;
        
        let mut result = nonce.to_vec();
        result.extend(ciphertext);
        Ok(result)
    }

    pub fn decrypt_with_key(&self, key: &[u8], data: &[u8]) -> Result<Vec<u8>, AppError> {
        if key.len() != 32 {
            return Err(AppError::invalid_key(format!("Key must be 32 bytes, got {} bytes", key.len()))
                .with_context("Invalid key length for decryption"));
        }

        if data.len() < 12 {
            return Err(AppError::decryption_failed("Encrypted data too short - missing nonce")
                .with_context("Invalid encrypted data format"));
        }

        let (nonce_bytes, ciphertext) = data.split_at(12);
        let cipher = Aes256GcmSiv::new(Key::<Aes256GcmSiv>::from_slice(key));
        
        cipher.decrypt(Nonce::from_slice(nonce_bytes), ciphertext)
            .map_err(|e| AppError::decryption_failed(format!("AES-GCM-SIV decryption failed: {}", e))
                .with_context("Failed to decrypt data with provided key")
                .with_suggestion("Check if the key is correct and data is not corrupted"))
    }

    pub fn generate_nonce(&self) -> [u8; 12] {
        let mut nonce = [0u8; 12];
        ThreadRng::default().fill_bytes(&mut nonce);
        nonce
    }

    pub fn derive_key(&self, password: &str) -> Result<[u8; 32], AppError> {
        let salt_bytes = STANDARD.decode(&self.kdf_salt_b64.0)
            .map_err(|e| AppError::crypto_error(format!("Failed to decode KDF salt: {}", e))
                .with_context("Invalid base64 encoding for KDF salt"))?;

        let mut key = [0u8; 32];
        pbkdf2_hmac::<Sha256>(password.as_bytes(), &salt_bytes, 100_000, &mut key);
        Ok(key)
    }

    pub fn derive_key_with_secret_and_salt(kdf_salt_b64: &str, secret: &str) -> Result<[u8; 32], AppError> {
        let salt_bytes = STANDARD.decode(kdf_salt_b64)
            .map_err(|e| AppError::crypto_error(format!("Failed to decode KDF salt: {}", e))
                    .with_context("Invalid base64 encoding for KDF salt"))?;
        let mut key = [0u8; 32];
        pbkdf2_hmac::<Sha256>(secret.as_bytes(), &salt_bytes, 100_000, &mut key);
        Ok(key)
    }

}
