use super::{
    metadata::{
        Metadata,
    },
    config::{
        DatabaseConfig
    }
};













pub struct PreparedRootChanges {
    pub new_master_key: [u8; 32],
    pub new_decrypted_meta: Metadata,
    pub new_encrypted_meta: Metadata,
    pub new_trash_meta: Metadata,
    pub new_config: DatabaseConfig,
}