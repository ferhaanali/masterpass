use super::{
    Node,
    NodeId,
    core::{StringPath, Timestamp},
    NodeItemReference, 
    response::AppError,
    NodeType,
    NodeName
    entries::{
        FolderEntry,
        FolderId,
    },
};
use std::collections::HashMap; 
use serde::{Serialize, Deserialize};


#[derive(Debug, Default, Serialize, Deserialize)]
pub struct NodeManager {
    nodes: HashMap<NodeId, Node>,
    path_index: HashMap<String, NodeId>,
    children_index: HashMap<Option<NodeId>, Vec<NodeId>>,
    item_references: HashMap<NodeId, NodeItemReference>,
    #[serde(skip)] 
    file_path: StringPath,
}

impl NodeManager {
    pub fn new() -> Self {
        let mut manager = NodeManager::default();
        manager.initialize_root();
        manager
    }

    fn initialize_root(&mut self) {
        let root_node = Node {
            id: NodeId::new(),
            name: NodeName::from_str("root"),
            node_type: NodeType::Folder,
            parent_id: None,
            virtual_path: "/".to_string(),
            created_at: Timestamp::now(),
            updated_at: Timestamp::now(),
        };

        let roo_folder = FolderEntry {
            id: FolderId::new(),
            display_name: "root",
            system_path: StringPath::from_str("/"),
            
        }
    }
}