use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use super::{
    core::StringPath,
    response::AppError,
    entries::ItemId,
    ItemKey,
};

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct ItemKeyStore {
    pub collection: HashMap<ItemId, ItemKey>,
    pub system_path: StringPath,
}

impl ItemKeyStore {
    pub fn new(system_path: StringPath) -> Self {
        Self {
            collection: HashMap::new(),
            system_path,
        }
    }

    pub fn set_key(&mut self, id: ItemId, key: ItemKey) {
        self.collection.insert(id, key);
    }

    pub fn set_key_if_absent(&mut self, id: ItemId, key: ItemKey) {
        self.collection.entry(id).or_insert(key);
    }

    pub fn update_key(&mut self, id: &ItemId, key: ItemKey) -> Result<(), AppError> {
        if let Some(existing) = self.collection.get_mut(id) {
            *existing = key;
            Ok(())
        } else {
            Err(AppError::not_found("Item ID not found in collection"))
        }
    }

    pub fn remove_key(&mut self, id: &ItemId) -> Option<ItemKey> {
        self.collection.remove(id)
    }

    pub fn get_key(&self, id: &ItemId) -> Option<&ItemKey> {
        self.collection.get(id)
    }

    pub fn has_key(&self, id: &ItemId) -> bool {
        self.collection.contains_key(id)
    }

    pub fn all_keys(&self) -> Vec<&ItemId> {
        self.collection.keys().collect()
    }

    pub fn all_values(&self) -> Vec<&ItemKey> {
        self.collection.values().collect()
    }

    pub fn all_entries(&self) -> Vec<(&ItemId, &ItemKey)> {
        self.collection.iter().collect()
    }
}
