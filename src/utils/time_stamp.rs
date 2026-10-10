use chrono::{DateTime, ParseError, Utc};
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TimeStamp(pub DateTime<Utc>);

impl TimeStamp {
    /// Captures current UTC timestamp
    pub fn now() -> Self {
        Self(Utc::now())
    }

    /// Formats timestamp as RFC3339 string for SQLite storage
    pub fn to_str(&self) -> String {
        self.0.to_rfc3339()
    }

    /// Parses SQLite text back into a TimeStamp struct
    pub fn from_str(s: &str) -> Result<Self, ParseError> {
        let dt = DateTime::parse_from_rfc3339(s)?;
        Ok(Self(dt.with_timezone(&Utc)))
    }
}

impl fmt::Display for TimeStamp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0.format("%Y-%m-%d %H:%M:%S UTC"))
    }
}