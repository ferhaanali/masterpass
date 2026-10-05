use super::{
    Database,
    entries::{
        FileEntry,
        PasswordEntry,
    },
};

impl Database {
    // --- Public Functions: Listing & Searching ---

    // File search: Uses display_name
    pub fn search_decrypted_files(&self, query: &str) -> Vec<&FileEntry> {
        self.meta.decrypted.data.files.0
            .iter()
            // Using file.display_name.0 for file search
            .filter(|file| file.display_name.0.to_lowercase().contains(&query.to_lowercase()))
            .collect::<Vec<&FileEntry>>() 
    }
    // Password search: Assumes display_name (since 'name' was used previously)
    pub fn search_decrypted_passwords(&self, query: &str) -> Vec<&PasswordEntry> {
        self.meta.decrypted.data.passwords.0
            .iter()
            // Assuming entry.name corresponds to entry.display_name.0
            .filter(|p| p.display_name.0.to_lowercase().contains(&query.to_lowercase()))
            .collect()
    }

    pub fn list_decrypted_files(&self, status: Option<&str>) -> Vec<&FileEntry> {
        match status {
            Some("active") => self.meta.decrypted.data.files.0.iter().collect(), 
            Some("recycled") => self.meta.trash.data.files.0.iter().collect(),   
            _ => {
                self.meta.decrypted.data.files.0.iter().chain(self.meta.trash.data.files.0.iter()).collect()
            }
        }
    }

    pub fn list_decrypted_passwords(&self, status: Option<&str>) -> Vec<&PasswordEntry> {
        match status {
            Some("recycled") => self.meta.trash.data.passwords.0.iter().collect(), 
            _ => self.meta.decrypted.data.passwords.0.iter().collect(),
        }
    }

    // Encrypted items
    pub fn list_encrypted_files(&self) -> Vec<&FileEntry> {
        self.meta.encrypted.data.files.0.iter().collect()
    }

    pub fn list_encrypted_passwords(&self) -> Vec<&PasswordEntry> {
        self.meta.encrypted.data.passwords.0.iter().collect()
    }

    // Encrypted Password Search
    pub fn search_encrypted_passwords(&self, query: &str) -> Vec<&PasswordEntry> {
        self.meta.encrypted.data.passwords.0 
            // Assuming entry.name corresponds to entry.display_name.0
            .iter()
            .filter(|p| p.display_name.0.to_lowercase().contains(&query.to_lowercase()))
            .collect()
    }

    // Encrypted File Search: Uses display_name
    pub fn search_encrypted_files(&self, query: &str) -> Vec<&FileEntry> {
        self.meta.encrypted.data.files.0 
            .iter()
            // Using file.display_name.0 for file search
            .filter(|file| file.display_name.0.to_lowercase().contains(&query.to_lowercase()))
            .collect::<Vec<&FileEntry>>() 
    }
 
    // Deleted / Recycled items
    pub fn list_deleted_files(&self) -> Vec<&FileEntry> {
        self.meta.trash.data.files.0.iter().collect()
    }

    pub fn list_deleted_passwords(&self) -> Vec<&PasswordEntry> {
        self.meta.trash.data.passwords.0.iter().collect()
    }

    // Deleted File Search: Uses display_name
    pub fn search_deleted_files(&self, query: &str) -> Vec<&FileEntry> {
        self.meta.trash.data.files.0 
            .iter()
            // Using file.display_name.0 for file search
            .filter(|file| file.display_name.0.to_lowercase().contains(&query.to_lowercase()))
            .collect::<Vec<&FileEntry>>() 
    }

    // Deleted Password Search
    pub fn search_deleted_passwords(&self, query: &str) -> Vec<&PasswordEntry> {
        self.meta.trash.data.passwords.0 
            // Assuming entry.name corresponds to entry.display_name.0
            .iter()
            .filter(|p| p.display_name.0.to_lowercase().contains(&query.to_lowercase()))
            .collect()
    }

    // All items (Files + Passwords)
    pub fn list_all_files(&self) -> Vec<&FileEntry> {
        self.meta.decrypted.data.files.0
            .iter()
            .chain(self.meta.encrypted.data.files.0.iter())
            .collect()
    }

    pub fn list_all_passwords(&self) -> Vec<&PasswordEntry> {
        self.meta.decrypted.data.passwords.0
            .iter()
            .chain(self.meta.encrypted.data.passwords.0.iter())
            .collect()
    }


}