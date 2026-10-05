use serde::{Serialize, Deserialize};
use std::fmt;

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct DataSize(pub f64);

impl DataSize {
    
    pub fn from_str(s: &str) -> Option<Self> {
        let lower = s.trim().to_lowercase();
        
        // 1. Find the first token that could be the number part.
        if let Some(num_str) = lower.split_whitespace().next() {
            // 2. Parse the number part to f64
            if let Ok(value) = num_str.parse::<f64>() {
                
                // 3. Determine the multiplier based on the whole lower string
                let bytes = if lower.contains("gb") {
                    value * 1024.0 * 1024.0 * 1024.0 // 1024^3
                } else if lower.contains("mb") {
                    value * 1024.0 * 1024.0          // 1024^2
                } else if lower.contains("kb") {
                    value * 1024.0                   // 1024^1
                } else {
                    value // Assume raw bytes if no unit is found
                };
                
                return Some(DataSize(bytes));
            }
        }
        // If parsing the number fails or the input is empty/malformed
        None
    }

    pub fn as_bytes(&self) -> f64 { self.0 }
    pub fn as_kb(&self) -> f64 { self.0 / 1024.0 }
    pub fn as_mb(&self) -> f64 { self.0 / (1024.0 * 1024.0) }
    pub fn as_gb(&self) -> f64 { self.0 / (1024.0 * 1024.0 * 1024.0) }

    pub fn len(&self) -> usize {
        self.to_string().len()
    }
}

impl fmt::Display for DataSize {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.0 >= 1024.0 * 1024.0 * 1024.0 {
            write!(f, "{:.2} GB", self.as_gb())
        } else if self.0 >= 1024.0 * 1024.0 {
            write!(f, "{:.2} MB", self.as_mb())
        } else if self.0 >= 1024.0 {
            write!(f, "{:.2} KB", self.as_kb())
        } else {
            write!(f, "{:.0} bytes", self.0)
        }
    }
}
