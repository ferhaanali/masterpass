use serde::{Serialize, Deserialize};
use super::{
    entries::{PasswordEntry},
};




#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct PasswordCollection(pub Vec<PasswordEntry>);





impl PasswordCollection {
    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn inner(&self) -> &Vec<PasswordEntry> {
        &self.0
    }
}


