use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc, Local};
use std::fmt;




#[derive(Serialize, Deserialize, Debug, Clone, Hash, PartialEq, Eq)]
pub struct Timestamp(pub DateTime<Utc>);


impl Timestamp {
    pub fn now() -> Self {
        Self(Utc::now())
    }

    pub fn update(&mut self) {
        self.0 = Utc::now(); 
    } 

    pub fn as_string(&self) -> String {
        let local_time = self.0.with_timezone(&Local);
        local_time.format("%d %B, %Y, %I:%M %p").to_string()
    }
    
    pub fn len(&self) -> usize {
        self.as_string().len()
    }
}

impl fmt::Display for Timestamp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let local_time = self.0.with_timezone(&Local);
        write!(f, "{}", local_time.format("%d %B, %Y, %I:%M %p"))
    }
}
