use serde::{Serialize, Deserialize};
use super::{
    core::{StringPath, DataSize, Timestamp},
    ItemId,
    IsTrashed,
    IsEncrypted,
};


#[derive(Serialize, Deserialize, Debug, Clone, Hash, PartialEq, Eq)]
pub struct FileName(pub String);

#[derive(Serialize, Deserialize, Debug, Clone, Hash, PartialEq, Eq)]
pub struct FileExtension(pub String);


#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FileEntry {
    pub id: ItemId,
    pub display_name: FileName,           
    pub system_name: FileName,            
    pub system_path: StringPath,
    pub size: DataSize,
    pub extension: FileExtension,
    pub is_encrypted: IsEncrypted,
    pub is_trashed: IsTrashed,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,   
}




