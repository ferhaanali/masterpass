use super::{
    structs::{Database, FileEntry, PasswordEntry, FolderEntry},
};



impl Database {
    // Search passwords by query in active passwords only
    pub fn search_decrypted_files(&self, query: &str) -> Vec<&FileEntry> {
        self.meta.decrypted_meta.data.files
            .iter()
            .filter(|file| file.name.to_lowercase().contains(&query.to_lowercase()))
            .collect::<Vec<&FileEntry>>() // ✅ specify Vec type
    }
    pub fn search_decrypted_passwords(&self, query: &str) -> Vec<&PasswordEntry> {
        self.meta.decrypted_meta.data.passwords
            .iter()
            .filter(|p| p.name.to_lowercase().contains(&query.to_lowercase()))
            .collect()
    }

    pub fn list_decrypted_files(&self, status: Option<&str>) -> Vec<&FileEntry> {
        match status {
            Some("active") => self.meta.decrypted_meta.data.files.iter().collect(),
            Some("recycled") => self.meta.trash_meta.data.files.iter().collect(),
            _ => {
                // Combine both active and recycled files
                self.meta.decrypted_meta.data.files.iter().chain(self.meta.trash_meta.data.files.iter()).collect()
            }
        }
    }

    pub fn list_decrypted_passwords(&self, status: Option<&str>) -> Vec<&PasswordEntry> {
        match status {
            Some("recycled") => self.meta.trash_meta.data.passwords.iter().collect(),
            _ => self.meta.decrypted_meta.data.passwords.iter().collect(), // default active
        }
    }

    // Encrypted items
    pub fn list_encrypted_files(&self) -> Vec<&FileEntry> {
        self.meta.encrypted_meta.data.files.iter().collect()
    }

    pub fn list_encrypted_passwords(&self) -> Vec<&PasswordEntry> {
        self.meta.encrypted_meta.data.passwords.iter().collect()
    }

    pub fn search_encrypted_passwords(&self, query: &str) -> Vec<&PasswordEntry> {
        self.meta.encrypted_meta.data.passwords
            .iter()
            .filter(|p| p.name.to_lowercase().contains(&query.to_lowercase()))
            .collect()
    }

    pub fn search_encrypted_files(&self, query: &str) -> Vec<&FileEntry> {
        self.meta.encrypted_meta.data.files
            .iter()
            .filter(|file| file.name.to_lowercase().contains(&query.to_lowercase()))
            .collect::<Vec<&FileEntry>>() // ✅ specify Vec type
    }
 
    // Deleted / Recycled items
    pub fn list_deleted_files(&self) -> Vec<&FileEntry> {
        self.meta.trash_meta.data.files.iter().collect()
    }

    pub fn list_deleted_passwords(&self) -> Vec<&PasswordEntry> {
        self.meta.trash_meta.data.passwords.iter().collect()
    }

    pub fn search_deleted_files(&self, query: &str) -> Vec<&FileEntry> {
        self.meta.trash_meta.data.files
            .iter()
            .filter(|file| file.name.to_lowercase().contains(&query.to_lowercase()))
            .collect::<Vec<&FileEntry>>() // ✅ specify Vec type
    }

    pub fn search_deleted_passwords(&self, query: &str) -> Vec<&PasswordEntry> {
        self.meta.trash_meta.data.passwords
            .iter()
            .filter(|p| p.name.to_lowercase().contains(&query.to_lowercase()))
            .collect()
    }

    // All items (Files + Passwords)
    pub fn list_all_files(&self) -> Vec<&FileEntry> {
        self.meta.decrypted_meta.data.files
            .iter()
            .chain(self.meta.encrypted_meta.data.files.iter())
            .collect()
    }

    pub fn list_all_passwords(&self) -> Vec<&PasswordEntry> {
        self.meta.decrypted_meta.data.passwords
            .iter()
            .chain(self.meta.encrypted_meta.data.passwords.iter())
            .collect()
    }


    // Folders
    pub fn list_decrypted_folders(&self) -> Vec<&FolderEntry> {
        self.meta.decrypted_meta.data.folders.iter().collect()
    }

    pub fn list_encrypted_folders(&self) -> Vec<&FolderEntry> {
        self.meta.encrypted_meta.data.folders.iter().collect()
    }

    pub fn list_deleted_folders(&self) -> Vec<&FolderEntry> {
        self.meta.trash_meta.data.folders.iter().collect()
    }

    pub fn list_all_folders(&self) -> Vec<&FolderEntry> {
        self.meta.decrypted_meta.data.folders
            .iter()
            .chain(self.meta.encrypted_meta.data.folders.iter())
            .collect()
    }

    // Folder search functions
    pub fn search_decrypted_folders(&self, query: &str) -> Vec<&FolderEntry> {
        self.meta.decrypted_meta.data.folders
            .iter()
            .filter(|folder| folder.name.to_lowercase().contains(&query.to_lowercase()))
            .collect::<Vec<&FolderEntry>>()
    }

    pub fn search_encrypted_folders(&self, query: &str) -> Vec<&FolderEntry> {
        self.meta.encrypted_meta.data.folders
            .iter()
            .filter(|folder| folder.name.to_lowercase().contains(&query.to_lowercase()))
            .collect::<Vec<&FolderEntry>>()
    }

    pub fn search_deleted_folders(&self, query: &str) -> Vec<&FolderEntry> {
        self.meta.trash_meta.data.folders
            .iter()
            .filter(|folder| folder.name.to_lowercase().contains(&query.to_lowercase()))
            .collect::<Vec<&FolderEntry>>()
    }

    pub fn search_all_folders(&self, query: &str) -> Vec<&FolderEntry> {
        self.meta.decrypted_meta.data.folders
            .iter()
            .chain(self.meta.encrypted_meta.data.folders.iter())
            .filter(|folder| folder.name.to_lowercase().contains(&query.to_lowercase()))
            .collect::<Vec<&FolderEntry>>()
    }

    pub fn list_recycled_folders(&self) -> Vec<&FolderEntry> {
        self.meta.trash_meta.data.folders.iter().collect()
    }
}
