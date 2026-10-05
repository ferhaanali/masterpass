use serde::{Serialize, Deserialize};
use super::{
    Database,
};
use colored::Colorize;
use std::{
    fs,
};




#[derive(Serialize, Deserialize, Debug,Clone)]
pub struct DatabaseStats {
    pub total_size_bytes: u64,
    pub encrypted_files_size: u64,
    pub decrypted_files_size: u64,
    pub metadata_size: u64,
    pub file_count: usize,
    pub password_count: usize,
    pub encrypted_count: usize,
    pub decrypted_count: usize,
}




impl DatabaseStats {
    pub fn default() -> Self {
        Self {
            total_size_bytes: 0,
            encrypted_files_size: 0,
            decrypted_files_size: 0,
            metadata_size: 0,
            file_count: 0,
            password_count: 0,
            encrypted_count: 0,
            decrypted_count: 0,
        }
    }
    pub fn update(&mut self, stats: &DatabaseStats) {
        *self = stats.clone();
    }
}

impl Database {
    pub fn get_database_stats(&self) -> DatabaseStats {
        let mut stats = DatabaseStats {
            total_size_bytes: 0,
            encrypted_files_size: 0,
            decrypted_files_size: 0,
            metadata_size: 0,
            file_count: self.meta.decrypted.data.files.0.len() + self.meta.encrypted.data.files.0.len(),
            password_count: self.meta.decrypted.data.passwords.0.len() + self.meta.encrypted.data.passwords.0.len(),
            encrypted_count: self.meta.encrypted.data.files.0.len() + self.meta.encrypted.data.passwords.0.len(),
            decrypted_count: self.meta.decrypted.data.files.0.len() + self.meta.decrypted.data.passwords.0.len(),
        };

        // Calculate encrypted files size
        for file in &self.meta.encrypted.data.files.0 {
            if let Ok(metadata) = fs::metadata(&file.system_path) {
                stats.encrypted_files_size += metadata.len();
            }
        }

        // Calculate decrypted files size
        for file in &self.meta.decrypted.data.files.0 {
            if let Ok(metadata) = fs::metadata(&file.system_path) {
                stats.decrypted_files_size += metadata.len();
            }
        }

        // Calculate metadata size (approximate)
        let meta_json = serde_json::to_string(&self.meta.decrypted.data).unwrap_or_default();
        let encrypted_meta_json = serde_json::to_string(&self.meta.encrypted.data).unwrap_or_default();
        let config_json = serde_json::to_string(&self.config).unwrap_or_default();
        
        stats.metadata_size = (meta_json.len() + encrypted_meta_json.len() + config_json.len()) as u64;
        stats.total_size_bytes = stats.encrypted_files_size + stats.decrypted_files_size + stats.metadata_size;

        stats
    }

    pub fn show_database_info(&mut self) {
        let stats = self.get_database_stats();
        let _ = self.config.save();
        let total_mb = stats.total_size_bytes as f64 / (1024.0 * 1024.0);
        let encrypted_mb = stats.encrypted_files_size as f64 / (1024.0 * 1024.0);
        let decrypted_mb = stats.decrypted_files_size as f64 / (1024.0 * 1024.0);
        let metadata_mb = stats.metadata_size as f64 / (1024.0 * 1024.0);
        let efficiency = encrypted_mb / total_mb * 100.0;

        println!();
        println!("{}", "┌──────────────────────────────────────────────────┐".cyan());
        println!("{}", "│            💾 STORAGE OVERVIEW                 │".bold().cyan());
        println!("{}", "├──────────────────────────────────────────────────┤".cyan());
        
        // Storage sizes in a box
        println!("{} {:<15} {:>8.2} MB {}", 
            "│".cyan(), "Total:".bold().green(), total_mb, "│".cyan());
        println!("{} {:<15} {:>8.2} MB {}", 
            "│".cyan(), "Encrypted:".yellow(), encrypted_mb, "│".cyan());
        println!("{} {:<15} {:>8.2} MB {}", 
            "│".cyan(), "Decrypted:".yellow(), decrypted_mb, "│".cyan());
        println!("{} {:<15} {:>8.2} MB {}", 
            "│".cyan(), "Metadata:".yellow(), metadata_mb, "│".cyan());
        println!("{}", "├──────────────────────────────────────────────────┤".cyan());
        
        // Item counts
        println!("{} {:<15} {:>8} {}", 
            "│".cyan(), "Files:".bold().green(), stats.file_count, "│".cyan());
        println!("{} {:<15} {:>8} {}", 
            "│".cyan(), "Passwords:".bold().green(), stats.password_count, "│".cyan());
        println!("{} {:<15} {:>8} {}", 
            "│".cyan(), "Encrypted:".green(), stats.encrypted_count, "│".cyan());
        println!("{} {:<15} {:>8} {}", 
            "│".cyan(), "Decrypted:".green(), stats.decrypted_count, "│".cyan());
        println!("{}", "├──────────────────────────────────────────────────┤".cyan());
        
        // Efficiency with visual indicator
        let efficiency_str = format!("{:.1}%", efficiency);
        let efficiency_display = if efficiency > 80.0 {
            efficiency_str.bold().green()
        } else if efficiency > 50.0 {
            efficiency_str.bold().yellow()
        } else {
            efficiency_str.bold().red()
        };

        let status_icon = if efficiency > 80.0 {
            "🔒 Excellent".green()
        } else if efficiency > 50.0 {
            "🛡️ Good".yellow()
        } else {
            "⚠️ Low".red()
        };

        println!("{} {:<15} {:>8} {}", 
            "│".cyan(), "Efficiency:".bold().green(), efficiency_display, "│".cyan());
        println!("{} {:<15} {:>8} {}", 
            "│".cyan(), "Status:".bold().green(), status_icon, "│".cyan());

        // Progress bar
        let bars = (efficiency / 5.0) as usize;
        let progress = format!("{:.<20}", "█".repeat(bars)).green();
        println!("{} {:<15} {:<20} {}", 
            "│".cyan(), "Encryption:".dimmed(), progress, "│".cyan());

        println!("{}", "└──────────────────────────────────────────────────┘".cyan());
        println!();
    }
}


