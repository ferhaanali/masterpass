





#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SecretEntry {
    pub id: Option<i64>,
    pub domain: String,
    pub identity: String,
    pub purpose: String,
    pub version: u32,
    pub is_derived: bool,
    pub encrypted_value: Option<String>,
    pub created_at: String,
}

impl SecretEntry {
    pub fn new_derived(
        domain: &str,
        identity: &str,
        purpose: &str,
        version: u32,
        created_at: &str,
    ) -> Self {
        Self {
            id: None,
            domain: domain.trim().to_lowercase(),
            identity: identity.trim().to_lowercase(),
            purpose: purpose.trim().to_lowercase(),
            version,
            is_derived: true,
            encrypted_value: None,
            created_at: created_at.to_string(),
        }
    }

    pub fn new_stored(
        domain: &str,
        identity: &str,
        purpose: &str,
        encrypted_value: String,
        created_at: &str,
    ) -> Self {
        Self {
            id: None,
            domain: domain.trim().to_lowercase(),
            identity: identity.trim().to_lowercase(),
            purpose: purpose.trim().to_lowercase(),
            version: 1,
            created_at: created_at.to_string(),
        }
    }

    pub fn from_row(row: &Row) -> rusqlite::Result<Self> {
        Ok(Self {
            id: Some(row.get(0)?),
            domain: row.get(1)?,
            identity: row.get(2)?,
            purpose: row.get(3)?,
            version: row.get::<_, u32>(4)?,
            created_at: row.get(7)?,
        })
    }
}