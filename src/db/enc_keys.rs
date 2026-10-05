use super::{
    aes_gcm_siv::{
        Aes256GcmSiv, Key, Nonce,
        aead::{Aead, KeyInit},
    },
    rand::{RngCore, rngs::ThreadRng},
    base64::{engine::general_purpose,engine::general_purpose::STANDARD,Engine},
    pbkdf2::pbkdf2_hmac,
    sha2::Sha256,
};



pub fn generate_item_key() -> Vec<u8> {
    let mut key = [0u8; 32];
    ThreadRng::default().fill_bytes(&mut key);
    key.to_vec()
}

pub fn wrap_item_key(item_key: &[u8], master_key: &[u8]) -> Option<String> {
    let encryptted = match encrypt_with_key(master_key, item_key) {
        Some(e) => e,
        None => return None, // encryption failed
    };
    Some(STANDARD.encode(encryptted))
}

pub fn unwrap_item_key(encrypted_item_key: &str, master_key: &[u8]) -> Option<Vec<u8>> {
    let decoded = STANDARD.decode(encrypted_item_key).ok()?;
    match decrypt_with_key(master_key, &decoded) {
        Some(d) => Some(d),
        None => None, // decryption failed
    }
}

pub fn encrypt_with_key(key: &[u8], plaintext: &[u8]) -> Option<Vec<u8>> {
    let cipher = Aes256GcmSiv::new(Key::<Aes256GcmSiv>::from_slice(key));
    let nonce = generate_nonce();
    let ciphertext = match cipher.encrypt(Nonce::from_slice(&nonce), plaintext) {
        Ok(c) => c,
        Err(_) => return None, // encryption failed
    };
    let mut result = nonce.to_vec();
    result.extend(ciphertext);
    Some(result)
}

pub fn decrypt_with_key(key: &[u8], data: &[u8]) -> Option<Vec<u8>> {
    let (nonce_bytes, ciphertext) = data.split_at(12);
    let cipher = Aes256GcmSiv::new(Key::<Aes256GcmSiv>::from_slice(key));
    cipher.decrypt(Nonce::from_slice(nonce_bytes), ciphertext).ok()
}
// Generate Nonce
pub fn generate_nonce() -> [u8; 12] {
    let mut nonce = [0u8; 12];
    ThreadRng::default().fill_bytes(&mut nonce);
    nonce
}

pub fn derive_key(kdf_salt_b64: &str, password: &str) -> [u8; 32] {
    let salt_bytes = general_purpose::STANDARD
        .decode(kdf_salt_b64)
        .expect("Failed to decode salt");
    let mut key = [0u8; 32];
    pbkdf2_hmac::<Sha256>(password.as_bytes(), &salt_bytes, 100_000, &mut key);
    key
}