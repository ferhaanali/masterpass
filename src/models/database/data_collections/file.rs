use serde::{Serialize, Deserialize};
use super::entries::FileEntry;


#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct FileCollection(pub Vec<FileEntry>);

impl FileCollection {
    pub fn len(&self) -> usize {
        self.0.len()
    }
    
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
    
    pub fn inner(&self) -> &Vec<FileEntry> {
        &self.0
    }
}

