use super::{
    structs::{Database},
    std::{
        fs::File,
        io::{BufReader, BufWriter, Read, Write, stdout},
    },
    aes_gcm_siv::{
        Aes256GcmSiv, Key, Nonce,
        aead::{Aead, KeyInit}
    },
    base64::{engine::general_purpose, Engine as _},
    rand::{RngCore, rng},
};


impl Database {
    /// Encrypt a string and return Base64 using AES-256-GCM-SIV
    pub fn encrypt_string(&self, plaintext: &str, key: &[u8]) -> Option<String> {
        let cipher = Aes256GcmSiv::new(Key::<Aes256GcmSiv>::from_slice(key));

        let mut nonce_bytes = [0u8; 12];
        rng().fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        let ciphertext = cipher.encrypt(nonce, plaintext.as_bytes()).ok()?;

        let mut result = nonce_bytes.to_vec();
        result.extend(ciphertext);

        Some(general_purpose::STANDARD.encode(&result))
    }

    /// Decrypt a Base64 string using AES-256-GCM-SIV
    pub fn decrypt_string(&self, encrypted_b64: &str, key: &[u8]) -> Option<String> {
        let data = general_purpose::STANDARD.decode(encrypted_b64).ok()?;
        if data.len() < 12 { return None; }

        let (nonce_bytes, ciphertext) = data.split_at(12);
        let nonce = Nonce::from_slice(nonce_bytes);

        let cipher = Aes256GcmSiv::new(Key::<Aes256GcmSiv>::from_slice(key));
        let plaintext = cipher.decrypt(nonce, ciphertext).ok()?;
        String::from_utf8(plaintext).ok()
    }

    /// Encrypt a file in chunks with AES-256-GCM-SIV
    pub fn encrypt_file_data(&self, input_path: &str, output_path: &str, key: &[u8]) -> bool {
        let cipher = Aes256GcmSiv::new(Key::<Aes256GcmSiv>::from_slice(key));

        let mut input_file = BufReader::new(File::open(input_path).unwrap());
        let mut output_file = BufWriter::new(File::create(output_path).unwrap());

        let file_size = input_file.get_ref().metadata().unwrap().len();
        let mut total_read = 0u64;
        let mut buffer = [0u8; 64 * 1024]; // 64 KB chunks

        while let Ok(n) = input_file.read(&mut buffer) {
            if n == 0 { break; }
            let chunk = &buffer[..n];

            let mut nonce_bytes = [0u8; 12];
            rng().fill_bytes(&mut nonce_bytes);
            let nonce = Nonce::from_slice(&nonce_bytes);

            let encrypted_chunk = cipher.encrypt(nonce, chunk).unwrap();

            output_file.write_all(&nonce_bytes).unwrap();
            output_file.write_all(&encrypted_chunk).unwrap();

            total_read += n as u64;
            print!("\r🔒 Encrypting: {:.2}%", (total_read as f64 / file_size as f64) * 100.0);
            stdout().flush().unwrap();
        }

        println!();
        true
    }

    /// Decrypt a file in chunks with AES-256-GCM-SIV
    pub fn decrypt_file_data(&self, input_path: &str, output_path: &str, key: &[u8]) -> bool {
        let cipher = Aes256GcmSiv::new(Key::<Aes256GcmSiv>::from_slice(key));

        let mut input_file = BufReader::new(File::open(input_path).unwrap());
        let mut output_file = BufWriter::new(File::create(output_path).unwrap());

        let file_size = input_file.get_ref().metadata().unwrap().len();
        let mut total_read = 0u64;

        let mut buffer = Vec::new();
        input_file.read_to_end(&mut buffer).unwrap();

        let mut cursor = 0;
        while cursor < buffer.len() {
            if cursor + 12 > buffer.len() { break; }
            let nonce_bytes = &buffer[cursor..cursor + 12];
            let nonce = Nonce::from_slice(nonce_bytes);
            cursor += 12;

            let next_nonce_pos = cursor + 64 * 1024 + 16;
            let chunk_end = std::cmp::min(next_nonce_pos, buffer.len());
            let encrypted_chunk = &buffer[cursor..chunk_end];
            cursor = chunk_end;

            let decrypted_chunk = match cipher.decrypt(nonce, encrypted_chunk) {
                Ok(p) => p,
                Err(_) => { eprintln!("❌ Wrong password or corrupted file"); return false; }
            };

            output_file.write_all(&decrypted_chunk).unwrap();
            total_read += decrypted_chunk.len() as u64;
            print!("\r📦 Decrypting: {:.2}%", (total_read as f64 / file_size as f64) * 100.0);
            stdout().flush().unwrap();
        }

        println!();
        true
    }

    /// Helper function to write raw bytes with progress
    pub fn write_data(&self, data: &[u8], dst_path: &str) -> bool {
        let total_size = data.len() as u64;
        let mut written_bytes = 0u64;

        let dst_file = match File::create(dst_path) {
            Ok(f) => f,
            Err(e) => { eprintln!("❌ Failed to create destination file: {}", e); return false; }
        };
        let mut writer = BufWriter::new(dst_file);

        for chunk in data.chunks(8192) {
            if let Err(e) = writer.write_all(chunk) {
                eprintln!("❌ Failed to write file: {}", e);
                return false;
            }
            written_bytes += chunk.len() as u64;
            print!("\r📦 Processing: {:.2}% complete", (written_bytes as f64 / total_size as f64) * 100.0);
            stdout().flush().unwrap();
        }

        println!("\n✅ Writing complete");
        true
    }
}

