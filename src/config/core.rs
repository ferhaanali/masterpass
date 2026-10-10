use crate::utils::TimeStamp;
use rusqlite::{params, Connection, Result};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Config {
    pub owner_name: String,
    pub vault_name: String,
    pub description: String,
    pub created_at: String,
    pub db_path: PathBuf,
}

impl Config {
    /// Resolves single-file database path 
    pub fn resolve_db_path(portable: bool) -> PathBuf {
            PathBuf::from("masterpass.db")
    }

    /// Opens DB connection, builds standard tables if absent, and loads Config into RAM
    pub fn load_or_init(
        owner: Option<&str>,
        vault_name: Option<&str>,
        description: Option<&str>,
    ) -> Result<(Self, Connection)> {
        let db_path = Self::resolve_db_path(portable);
        let conn = Connection::open(&db_path)?;

        // Store 1: Config & Vault Metadata Table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS config (
                 key TEXT PRIMARY KEY,
                 value TEXT NOT NULL
             );",
            [],
        )?;

        // Store 2: Derived Password Mappings Table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS secrets (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                domain TEXT NOT NULL,
                identity TEXT NOT NULL DEFAULT 'default',
                purpose TEXT NOT NULL DEFAULT 'password',
                version INTEGER NOT NULL DEFAULT 1,
                created_at TEXT NOT NULL,
              UNIQUE(domain, identity, purpose, version)
            );",
            [],
        )?;

        // Store 3: TOTP Seeds & Details Table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS totp_seeds (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            domain TEXT NOT NULL,
            identity TEXT NOT NULL DEFAULT 'default',
            encrypted_seed TEXT NOT NULL,
            issuer TEXT,
            digits INTEGER NOT NULL DEFAULT 6,
            period INTEGER NOT NULL DEFAULT 30,
            algorithm TEXT NOT NULL DEFAULT 'SHA1',
            created_at TEXT NOT NULL,
            UNIQUE(domain, identity)
        );",
            [],
        )?;

        // Check if vault configuration already exists in database
        let exists: bool = conn.query_row(
            "SELECT EXISTS(SELECT 1 FROM config WHERE key = 'owner_name')",
            [],
            |row| row.get(0),
        )?;

        if !exists {
            let default_owner = owner.unwrap_or("MasterPass User");
            let default_vault = vault_name.unwrap_or("Primary Vault");
            let default_desc = description.unwrap_or("Sovereign Vault");
            let now_str = TimeStamp::now().to_str();

            conn.execute(
                "INSERT INTO config (key, value) VALUES ('owner_name', ?1)",
                params![default_owner],
            )?;
            conn.execute(
                "INSERT INTO config (key, value) VALUES ('vault_name', ?1)",
                params![default_vault],
            )?;
            conn.execute(
                "INSERT INTO config (key, value) VALUES ('description', ?1)",
                params![default_desc],
            )?;
            conn.execute(
                "INSERT INTO config (key, value) VALUES ('created_at', ?1)",
                params![now_str],
            )?;
        }

        // Read configuration back from database into RAM struct
        let owner_name: String = conn.query_row(
            "SELECT value FROM config WHERE key = 'owner_name'",
            [],
            |row| row.get(0),
        )?;
        let vault_name: String = conn.query_row(
            "SELECT value FROM config WHERE key = 'vault_name'",
            [],
            |row| row.get(0),
        )?;
        let description: String = conn.query_row(
            "SELECT value FROM config WHERE key = 'description'",
            [],
            |row| row.get(0),
        )?;
        let created_at: String = conn.query_row(
            "SELECT value FROM config WHERE key = 'created_at'",
            [],
            |row| row.get(0),
        )?;

        let config = Config {
            owner_name,
            vault_name,
            description,
            created_at,
            db_path,
            is_portable: portable,
        };

        Ok((config, conn))
    }
}