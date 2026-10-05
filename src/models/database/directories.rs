use serde::{Serialize, Deserialize};
use super::{
    core::{
        StringPath,
    },
};









#[derive(Serialize, Deserialize,Clone, Debug)]
pub struct DatabaseDirectories {
    // Root
    pub root_directory: String,
    // Main
    pub decrypted: DirectoriesType,
    pub encrypted: DirectoriesType,
    pub trash: DirectoriesType,
    // Nested
    pub nested_db_dir: StringPath,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirectoriesType {
    pub root_dir: StringPath,
    pub files_dir: StringPath,
    pub folders_dir: StringPath,
}