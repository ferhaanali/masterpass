use serde::{Serialize, Deserialize};
use uuid::Uuid;
use std::fmt;
use super::{
    entries::{FileId, FolderId, PasswordId},
    core::{Timestamp,},
};

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq, Hash )]
pub struct NodeId(pub Uuid);

#[derive(Serialize, Deserialize, Debug, Clone )]
pub enum NodeType {
    File,
    Folder,
    Password,
    // SecureNote,
}

#[derive(Serialize, Deserialize, Debug, Clone )]
pub struct ItemId(pub Uuid);

#[derive(Serialize, Deserialize, Debug, Clone )]
pub struct NodeName(pub String);

impl NodeName {
    pub fn from_str(name: &str) -> Self {
        Self(name.to_string())
    }
}

impl NodeId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    pub fn root() -> Self {
        Self(Uuid::nil())
    }

    pub fn is_root(&self) -> bool {
        self.0 == Uuid::nil()
    }
}

impl ItemId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl fmt::Display for NodeId{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl fmt::Display for ItemId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}


#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Node {
    pub id: NodeId,
    pub name: NodeName,
    pub node_type: NodeType,
    pub parent_id: Option<NodeId>,
    pub virtual_path: String,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum NodeItemReference {
    File(FileId),
    Folder(FolderId),
    Password(PasswordId),
    // SecureNote(SecureNoteEntry),
}