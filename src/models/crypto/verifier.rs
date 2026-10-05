use serde::{Serialize, Deserialize};
use super::{
    MasterKey,
    KdfSaltB64,
    MasterPassword,
};
use aes_gcm_siv::{
    Aes256GcmSiv, Key, Nonce,
    aead::{Aead, KeyInit},
};
use base64::{ engine::general_purpose, Engine as _ };










#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct VerifierB64(pub String);


impl VerifierB64 {
    pub fn new(nonce: [u8; 12], kdf_salt_b64: KdfSaltB64, master_password: MasterPassword) -> Self {
        let key = MasterKey::new(kdf_salt_b64 ,master_password);
        let cipher = Aes256GcmSiv::new(Key::<Aes256GcmSiv>::from_slice(&key.0));

        let ciphertext = cipher.encrypt(Nonce::from_slice(&nonce), b"verify" as &[u8])
            .expect("Encryption failed");

        let mut result = nonce.to_vec();
        result.extend(ciphertext);
        Self(general_purpose::STANDARD.encode(&result))
    }

    pub fn verify_password(&self, kdf_salt_b64: KdfSaltB64, master_password: MasterPassword) -> bool {
        let key = MasterKey::new( kdf_salt_b64 ,master_password);
        let data = match general_purpose::STANDARD.decode(&self.0) {
            Ok(d) => d,
            Err(_) => return false,
        };
        if data.len() < 12 { return false; }

        let (nonce_bytes, ciphertext) = data.split_at(12);
        let cipher = Aes256GcmSiv::new(Key::<Aes256GcmSiv>::from_slice(&key.0));
        match cipher.decrypt(Nonce::from_slice(nonce_bytes), ciphertext) {
            Ok(pt) => pt == b"verify",
            Err(_) => false,
        }
    }
}