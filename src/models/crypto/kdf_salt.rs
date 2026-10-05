use serde::{Serialize, Deserialize};
use rand::rngs::ThreadRng;
use base64::{engine::general_purpose, Engine};
use rand::RngCore;



#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct KdfSaltB64(pub String);



impl KdfSaltB64 {
    pub fn new() -> Self {
        let mut salt = [0u8; 16];
        ThreadRng::default().fill_bytes(&mut salt);
        Self(general_purpose::STANDARD.encode(&salt))
    }
}