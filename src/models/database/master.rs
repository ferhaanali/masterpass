use serde::{Serialize, Deserialize};
use super::{
    crypto::{
        MasterPassword,
        MasterKey,
    },
};


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Master {
    pub key: MasterKey,
    pub password: Option<MasterPassword>,
}

