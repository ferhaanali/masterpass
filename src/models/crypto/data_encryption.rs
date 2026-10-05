use super::{
    CryptoEngine,
    response::AppError,
};
use std::{
    fs::File,
    io::{BufReader, BufWriter, Read, Write, stdout},
};
use aes_gcm_siv::{
    Aes256GcmSiv, Key, Nonce,
    aead::{Aead, KeyInit}
};
use base64::{engine::general_purpose::STANDARD, Engine as _};
use rand::{rngs::ThreadRng, RngCore};

impl CryptoEngine {
    /// Encrypt a string and return Base64 using AES-256-GCM-SIV
    pub fn encrypt_string(&self, plaintext: &str, key: &[u8]) -> Result<String, AppError> {
        if key.len() != 32 {
            return Err(AppError::invalid_key("Encryption key must be 32 bytes"));
        }

        let cipher = Aes256GcmSiv::new(Key::<Aes256GcmSiv>::from_slice(key));

        let mut nonce_bytes = [0u8; 12];
        ThreadRng::default().fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        let ciphertext = cipher.encrypt(nonce, plaintext.as_bytes())
            .map_err(|e| AppError::encryption_failed(format!("Failed to encrypt string: {}", e)))?;

        let mut result = nonce_bytes.to_vec();
        result.extend(ciphertext);

        Ok(STANDARD.encode(&result))
    }

    /// Decrypt a Base64 string using AES-256-GCM-SIV
    pub fn decrypt_string(&self, encrypted_b64: &str, key: &[u8]) -> Result<String, AppError> {
        if key.len() != 32 {
            return Err(AppError::invalid_key("Decryption key must be 32 bytes"));
        }

        let data = STANDARD.decode(encrypted_b64)
            .map_err(|e| AppError::crypto_error(format!("Invalid base64 data: {}", e)))?;
        
        if data.len() < 12 { 
            return Err(AppError::decryption_failed("Encrypted data too short"));
        }

        let (nonce_bytes, ciphertext) = data.split_at(12);
        let nonce = Nonce::from_slice(nonce_bytes);

        let cipher = Aes256GcmSiv::new(Key::<Aes256GcmSiv>::from_slice(key));
        let plaintext = cipher.decrypt(nonce, ciphertext)
            .map_err(|e| AppError::decryption_failed(format!("Failed to decrypt string: {}", e)))?;
        
        String::from_utf8(plaintext)
            .map_err(|e| AppError::decryption_failed(format!("Invalid UTF-8 in decrypted data: {}", e)))
    }

    /// Encrypt a file in chunks with AES-256-GCM-SIV
    pub fn encrypt_file_data(&self, input_path: &str, output_path: &str, key: &[u8]) -> Result<(), AppError> {
        if key.len() != 32 {
            return Err(AppError::invalid_key("Encryption key must be 32 bytes"));
        }

        let input_file = File::open(input_path)
            .map_err(|e| AppError::io_error(format!("Failed to open input file: {}", e)))?;
        
        let output_file = File::create(output_path)
            .map_err(|e| AppError::io_error(format!("Failed to create output file: {}", e)))?;

        let mut input_reader = BufReader::new(input_file);
        let mut output_writer = BufWriter::new(output_file);
        let cipher = Aes256GcmSiv::new(Key::<Aes256GcmSiv>::from_slice(key));

        let file_size = input_reader.get_ref().metadata()
            .map_err(|e| AppError::io_error(format!("Failed to get file metadata: {}", e)))?.len();

        let mut total_read = 0u64;
        let mut buffer = [0u8; 64 * 1024]; // 64 KB chunks

        while let Ok(n) = input_reader.read(&mut buffer) {
            if n == 0 { break; }
            let chunk = &buffer[..n];

            let mut nonce_bytes = [0u8; 12];
            ThreadRng::default().fill_bytes(&mut nonce_bytes);
            let nonce = Nonce::from_slice(&nonce_bytes);

            let encrypted_chunk = cipher.encrypt(nonce, chunk)
                .map_err(|e| AppError::encryption_failed(format!("Failed to encrypt file chunk: {}", e)))?;

            output_writer.write_all(&nonce_bytes)
                .map_err(|e| AppError::io_error(format!("Failed to write nonce: {}", e)))?;
            
            output_writer.write_all(&encrypted_chunk)
                .map_err(|e| AppError::io_error(format!("Failed to write encrypted data: {}", e)))?;

            total_read += n as u64;
            print!("\r🔒 Encrypting: {:.2}%", (total_read as f64 / file_size as f64) * 100.0);
            stdout().flush().unwrap();
        }

        output_writer.flush()
            .map_err(|e| AppError::io_error(format!("Failed to flush output: {}", e)))?;

        println!();
        Ok(())
    }

    /// Decrypt a file in chunks with AES-256-GCM-SIV
    pub fn decrypt_file_data(&self, input_path: &str, output_path: &str, key: &[u8]) -> Result<(), AppError> {
        if key.len() != 32 {
            return Err(AppError::invalid_key("Decryption key must be 32 bytes"));
        }

        let input_file = File::open(input_path)
            .map_err(|e| AppError::io_error(format!("Failed to open input file: {}", e)))?;
        
        let output_file = File::create(output_path)
            .map_err(|e| AppError::io_error(format!("Failed to create output file: {}", e)))?;

        let mut input_reader = BufReader::new(input_file);
        let mut output_writer = BufWriter::new(output_file);
        let cipher = Aes256GcmSiv::new(Key::<Aes256GcmSiv>::from_slice(key));

        let file_size = input_reader.get_ref().metadata()
            .map_err(|e| AppError::io_error(format!("Failed to get file metadata: {}", e)))?.len();

        let mut total_read = 0u64;
        let mut buffer = Vec::new();

        input_reader.read_to_end(&mut buffer)
            .map_err(|e| AppError::io_error(format!("Failed to read input file: {}", e)))?;

        let mut cursor = 0;
        while cursor < buffer.len() {
            if cursor + 12 > buffer.len() { 
                break; 
            }
            
            let nonce_bytes = &buffer[cursor..cursor + 12];
            let nonce = Nonce::from_slice(nonce_bytes);
            cursor += 12;

            let chunk_size = 64 * 1024 + 16; // 64KB + 16 bytes tag
            let chunk_end = std::cmp::min(cursor + chunk_size, buffer.len());
            let encrypted_chunk = &buffer[cursor..chunk_end];
            cursor = chunk_end;

            let decrypted_chunk = cipher.decrypt(nonce, encrypted_chunk)
                .map_err(|e| AppError::decryption_failed(format!("Failed to decrypt file chunk: {}", e)))?;

            output_writer.write_all(&decrypted_chunk)
                .map_err(|e| AppError::io_error(format!("Failed to write decrypted data: {}", e)))?;

            total_read += decrypted_chunk.len() as u64;
            print!("\r📦 Decrypting: {:.2}%", (total_read as f64 / file_size as f64) * 100.0);
            stdout().flush().unwrap();
        }

        output_writer.flush()
            .map_err(|e| AppError::io_error(format!("Failed to flush output: {}", e)))?;

        println!();
        Ok(())
    }

    /// Helper function to write raw bytes with progress
    pub fn write_data(&self, data: &[u8], dst_path: &str) -> Result<(), AppError> {
        let total_size = data.len() as u64;
        let mut written_bytes = 0u64;

        let dst_file = File::create(dst_path)
            .map_err(|e| AppError::io_error(format!("Failed to create destination file: {}", e)))?;
        
        let mut writer = BufWriter::new(dst_file);

        for chunk in data.chunks(8192) {
            writer.write_all(chunk)
                .map_err(|e| AppError::io_error(format!("Failed to write file: {}", e)))?;
            
            written_bytes += chunk.len() as u64;
            print!("\r📦 Processing: {:.2}% complete", (written_bytes as f64 / total_size as f64) * 100.0);
            stdout().flush().unwrap();
        }

        writer.flush()
            .map_err(|e| AppError::io_error(format!("Failed to flush writer: {}", e)))?;

        println!("\n✅ Writing complete");
        Ok(())
    }
}