use serde::{Serialize, Deserialize};
use zeroize::Zeroize;


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MasterPassword(pub String);

impl MasterPassword {
    pub fn new(password: impl Into<String>) -> Self {
        let p = password.into();
        if p.trim().is_empty() {
            panic!("Master password cannot be empty");
        }

        Self(p)
    }

    pub fn from_str(password: &str) -> Self {
        if password.trim().is_empty() {
            panic!("Master password cannot be empty");
        }

        Self(password.to_string())
    }
}

impl Zeroize for MasterPassword {
    fn zeroize(&mut self) {
        unsafe {
            std::ptr::write_volatile(self.0.as_mut_ptr(), 0);
        }
        self.0.clear();
    }
}

impl Drop for MasterPassword {
    fn drop(&mut self) {
        self.zeroize();
    }
}