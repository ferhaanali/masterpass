






#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TotpSeed {
    pub id: Option<i64>,
    pub domain: String,
    pub identity: String,
    pub encrypted_seed: String,
    pub issuer: Option<String>,
    pub digits: usize,
    pub perid: u64,
    pub algorithm: String,
    pub created_at: String,
}


impl TotpSeed {
    pub fn new(
        domain: &str,
        identity: &str,
        encrypted_seed: String,
        issuer: Option<&str>,
        digits: usize,
        period: u64,
        algorithm: &str,
        created_at: &str,
    ) -> Self {
        Self {
            id: None,
            domain: domain.trim().to_lowercase(),
            identity: identity.trim().to_lowercase(),
            encrypted_seed,
            issuer: issuer.map(|s| s.trim().to_string()),
            digits,
            period,
            algorithm: algorithm.to_uppercase(),
            created_at: created_at.to_string(),
        }
    }

    pub fn from_row(row: &Row) -> rusqlite::Result<Self> {
        Ok(Self {
            id: Some(row.get(0)?),
            domain: row.get(1)?,
            identity: row.get(2)?,
            encrypted_seed: row.get(3)?,
            issuer: row.get(4)?,
            digits: row.get::<_, i64>(5)? as usize,
            period: row.get::<_, i64>(6)? as u64,
            algorithm: row.get(7)?,
            created_at: row.get(8)?,
        })
    }
}