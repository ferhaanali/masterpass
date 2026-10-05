use rand::{ rngs::ThreadRng, RngCore };






pub fn generate_nonce() -> [u8; 12] {
    let mut nonce = [0u8; 12];
    ThreadRng::default().fill_bytes(&mut nonce);
    nonce
}