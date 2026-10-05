use serde::{Serialize, Deserialize};
use uuid::Uuid;



#[derive(Serialize, Deserialize, Debug, Clone, Hash, PartialEq, Eq)]
pub struct ItemId(pub Uuid);


#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct IsTrashed(pub bool);

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct IsEncrypted(pub bool);


impl ItemId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}