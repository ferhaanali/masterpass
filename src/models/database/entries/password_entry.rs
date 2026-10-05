use serde::{Serialize, Deserialize};
use super::{
    // node::{Node},
    core::{Timestamp},
    ItemId,
    IsTrashed,
    IsEncrypted,
};


#[derive(Serialize, Deserialize, Debug, Clone, Hash, PartialEq, Eq)]
pub struct PasswordName(pub String);

#[derive(Serialize, Deserialize, Debug, Clone, Hash, PartialEq, Eq)]
pub struct Password(pub String);  


#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PasswordEntry {
    pub id: ItemId,
    pub display_name: PasswordName,
    pub password: Password, 
    pub is_encrypted: IsEncrypted,
    pub is_trashed: IsTrashed,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,   
}

