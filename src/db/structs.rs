use super::{
    // Config,
    serde::{Serialize, Deserialize},
    std::path::{PathBuf}
};


#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FileEntry {
    pub name: String,
    pub file_name: String,
    pub encrypted_item_key: String,
    pub file_path: String,
    pub size: String,
    pub extension: String,
    pub is_encrypted: bool,
    pub is_recycled: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FolderEntry {
    pub name: String,
    pub encrypted_item_key: String,
    pub folder_path: String,
    pub sub_files: Vec<FileEntry>,
    pub sub_folders: Vec<FolderEntry>,
    pub size: String,
    pub is_empty: bool,
    pub is_encrypted: bool,
    pub is_recycled: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PasswordEntry {
    pub name: String,
    pub password: String, // plain text for now
    pub encrypted_item_key: String,
    pub is_encrypted: bool,
    pub is_recycled: bool,
    pub created_at: String,
    pub updated_at: String,
}  

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct DatabaseMeta {
    pub folders: Vec<FolderEntry>, 
    pub passwords: Vec<PasswordEntry>,
    pub files: Vec<FileEntry>,
}
#[derive(Serialize, Deserialize,Clone, Debug)]
pub struct Directories {
    pub root_directory: String,
    pub decrypted_dir: String,
    pub decrypted_files_dir: String,
    pub decrypted_folders_dir : String,
    pub encrypted_dir: String,
    pub encrypted_files_dir: String,
    pub encrypted_folders_dir: String,
    pub recycle_dir: String,
    pub recycle_files_dir: String,
    pub recycle_folders_dir: String,
    pub nested_db_dir: String,
}

#[derive(Serialize, Deserialize,Clone, Debug)]
pub struct MetaType {
    pub data: DatabaseMeta,
    pub file_path: String,
}

#[derive(Serialize, Deserialize,Clone, Debug)]
pub struct NestedDatabaseRecord {
    pub db_name: String,
    pub db_path: PathBuf,
}

#[derive(Serialize, Deserialize,Clone, Debug)]
pub struct NestedDatabaseMetaType {
    pub data: Vec<NestedDatabaseRecord>,
    pub file_path: String,
}

#[derive(Serialize, Deserialize,Clone, Debug)]
pub struct Meta {
    pub decrypted_meta: MetaType,
    pub encrypted_meta: MetaType,
    pub trash_meta: MetaType,
    pub nested_db_meta: NestedDatabaseMetaType,
}
#[derive(Serialize, Deserialize,Clone, Debug)]
pub struct Master {
    pub key: [u8; 32],
    pub password: Option<String>,
}
#[derive(Serialize, Deserialize,Clone, Debug)]
pub struct Database {
    pub master: Master,
    pub directories: Directories,
    pub meta: Meta,
    pub config: Config,
}

pub struct DatabaseArguments {
    pub db_name: String,
    pub owner: String,
    pub description: String,
    pub master_password: String,
    pub is_nested: bool,
    pub root_directory: PathBuf
}





// Other
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

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Config {
    pub db_version: String,
    pub kdf_salt_b64: String,
    pub verifier_b64: String,
    pub max_file_size_mb: u64,
    pub file_path: Option<String>,
    pub is_nested: bool,
    pub db_info: DBInfo 
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DBInfo {
    pub name: String,
    pub created_at: String,
    pub last_login: String,
    pub owner: String,
    pub description: String,
    pub stats: DatabaseStats
}

pub struct PreparedRootChanges {
    pub new_master_key: [u8; 32],
    pub new_decrypted_meta: DatabaseMeta,
    pub new_encrypted_meta: DatabaseMeta,
    pub new_trash_meta: DatabaseMeta,
    pub new_config: Config,
}