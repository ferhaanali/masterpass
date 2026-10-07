use serde::{Serialize, Deserialize};


#[derive(Serialize, Deserialize)]
pub struct TimeStamp {
    inner: String,
}


impl TimeStamp {
    fn new () -> Self {

    }

    fn now () -> Self {
        TimeStamp::new()
    }
}