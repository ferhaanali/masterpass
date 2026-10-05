use serde::{Serialize, Deserialize};
use super::{
    KdfSaltB64,
    MasterPassword,
};
use base64::{ engine::general_purpose, Engine as _ };
use pbkdf2::pbkdf2_hmac;
use sha2::Sha256;


#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct MasterKey(pub [u8; 32]);



impl MasterKey {
    pub fn new(kdf_salt_b64: KdfSaltB64, master_password: MasterPassword) -> Self {
        let salt_bytes = general_purpose::STANDARD.decode(kdf_salt_b64.0)
            .expect("Failed to decode salt");
        let mut key = [0u8; 32];
        pbkdf2_hmac::<Sha256>(master_password.0.as_bytes(), &salt_bytes, 100_000, &mut key);
        Self(key)
    }
}