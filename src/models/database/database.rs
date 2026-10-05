use serde::{Serialize, Deserialize};
use super::{
    crypto::{
        CryptoEngine,
        MasterPassword,
    },
    directories::DatabaseDirectories,
    metadata::{DatabaseMetadata, DataCollection, Metadata},
    config::DatabaseConfig,
    response::AppError,
    operations::DatabaseArguments,
    nested_database::{NestedDatabaseMetadata},
    directories::{DirectoriesType},
    core::{StringPath},
};
use std::{
    path::{Path, PathBuf},
    fs::{create_dir_all, write},
    env,
};



#[derive(Serialize, Deserialize,Clone, Debug)]
pub struct Database {
    pub directories: DatabaseDirectories,
    pub meta: DatabaseMetadata,
    pub config: DatabaseConfig,
    pub crypto: CryptoEngine,
}


impl Database {
    pub fn new(args: &mut DatabaseArguments) -> Result<Self, AppError> {
        let exe_path = env::current_exe().map_err(|e| AppError::io_error(format!("Failed to get current exe path: {}", e)))?;
        let exe_dir = exe_path.parent()
            .ok_or_else(|| AppError::runtime_error("Failed to get executable directory"))?
            .to_path_buf();

        let base_dir: PathBuf = if is_system_path(&exe_dir) {
            dirs::home_dir()
                .ok_or_else(|| AppError::runtime_error("Failed to get home directory"))?
                .join(".dataseal")
        } else {
            exe_dir.join("data")
        };

        let root_dir = if args.is_nested {
            base_dir.join("nested").join(&args.database_name)
        } else {
            base_dir.join(&args.database_name)
        };

        if !root_dir.exists() {
            create_dir_all(&root_dir)
                .map_err(|e| AppError::io_error(format!("Failed to create directory: {}", e)))?;
        }

        args.root_directory = root_dir.clone();
        
        Self::with_dir(args)
    }

    pub fn with_dir(args: &DatabaseArguments) -> Result<Self, AppError> {
        fn path_to_string(path: &Path) -> Result<String, AppError> {
            path.to_str()
                .map(|s| s.trim_end_matches('/').to_string())
                .ok_or_else(|| {
                    AppError::initialization_error("Invalid path")
                        .with_context("Path contains invalid UTF-8")
                        .with_details(format!("Path: {:?}", path))
                })
        }

        fn ensure_dir(path: &Path) -> Result<(), AppError> {
            if !path.exists() {
                create_dir_all(path).map_err(|e| {
                    AppError::initialization_error("Failed to create directory")
                        .with_details(format!("{}: {}", path.display(), e))
                })?;
            }
            Ok(())
        }

        fn ensure_json(path: &Path) -> Result<(), AppError> {
            if !path.exists() {
                write(path, "{}").map_err(|e| {
                    AppError::initialization_error("Failed to create JSON file")
                        .with_details(format!("{}: {}", path.display(), e))
                })?;
            }
            Ok(())
        }

        // Base paths
        let root = &args.root_directory;
        let to_s = |p: &Path| -> Result<String, AppError> { path_to_string(p) };

        let sub = |name: &str| root.join(name);
        let dir = |base: &Path, name: &str| base.join(name);

        let decrypted_dir = sub(".decrypted");
        let encrypted_dir = sub(".encrypted");
        let recycle_dir = sub(".recycle_bin");
        let nested_db_dir = sub(".nested");
        let config = sub(".config");

        let decrypted_files = dir(&decrypted_dir, ".files");
        let decrypted_folders = dir(&decrypted_dir, ".folders");
        let encrypted_files = dir(&encrypted_dir, ".files");
        let encrypted_folders = dir(&encrypted_dir, ".folders");
        let recycle_files = dir(&recycle_dir, ".files");
        let recycle_folders = dir(&recycle_dir, ".folders");

        let decrypted_meta = dir(&decrypted_dir, ".meta.json");
        let encrypted_meta = dir(&encrypted_dir, ".meta.json");
        let trash_meta = dir(&recycle_dir, ".trash_meta.json");
        let nested_meta = dir(&nested_db_dir, ".nested_db_record.json");
        let config_file = dir(&config, ".config.json");
        let crypto = dir(&config, ".crypto");

        // Config
        let config_file_str = path_to_string(&config_file)?;
        let database_config = DatabaseConfig::load_or_create(args, &config_file_str).map_err(|e| {
            AppError::configuration_error("Failed to load or create database configuration")
                .with_details(e.to_string())
                .with_suggestion("Check file permissions and disk space")
        })?;

        let crypto_folder = path_to_string(&crypto)?;
        let crypto = CryptoEngine::new(
            &crypto_folder,
            MasterPassword::from_str(&args.master_password),
        ).map_err(|e| AppError::crypto_error("Failed to initialize crypto engine")
            .with_details(e.to_string()))?;


        // Ensure directories
        for d in [
            &nested_db_dir,
            root,
            &decrypted_dir,
            &encrypted_dir,
            &recycle_dir,
            &decrypted_files,
            &encrypted_files,
            &encrypted_folders,
            &decrypted_folders,
        ] {
            ensure_dir(d)?;
        }

        // Subdirectories for file types
        for ext in ["photos", "videos", "documents", "other"] {
            ensure_dir(&decrypted_files.join(ext))?;
        }

        // Create meta files
        for f in [&nested_meta, &decrypted_meta, &trash_meta, &encrypted_meta] {
            ensure_json(f)?;
        }

        // Construct database with pre-converted string paths
        let mut db = Database {
            directories: DatabaseDirectories {
                root_directory: path_to_string(root)?,
                decrypted: DirectoriesType {
                    root_dir: StringPath(to_s(&decrypted_dir)?),
                    files_dir: StringPath(to_s(&decrypted_files)?),
                    folders_dir: StringPath(to_s(&decrypted_folders)?),
                },
                encrypted: DirectoriesType {
                    root_dir: StringPath(to_s(&encrypted_dir)?),
                    files_dir: StringPath(to_s(&encrypted_files)?),
                    folders_dir: StringPath(to_s(&encrypted_folders)?),
                },
                trash: DirectoriesType {
                    root_dir: StringPath(to_s(&recycle_dir)?),
                    files_dir: StringPath(to_s(&recycle_files)?),
                    folders_dir: StringPath(to_s(&recycle_folders)?),
                },
                nested_db_dir: StringPath(to_s(&nested_db_dir)?),
            },

            meta: DatabaseMetadata {
                decrypted: Metadata {
                    data: DataCollection::default(),
                    file_path: StringPath(to_s(&decrypted_meta)?),
                },
                encrypted: Metadata {
                    data: DataCollection::default(),
                    file_path: StringPath(to_s(&encrypted_meta)?),
                },
                trash: Metadata {
                    data: DataCollection::default(),
                    file_path: StringPath(to_s(&trash_meta)?),
                },
                nested_databases: NestedDatabaseMetadata {
                    data: Vec::new(),
                    file_path: StringPath(to_s(&nested_meta)?),
                },
            },

            config: database_config.clone(),
            crypto,
        };

        let _ = ( 
            db.config.save(),
            db.meta.encrypted.load(),
            db.meta.decrypted.load(),
            db.meta.trash.load(),
            db.meta.nested_databases.load(),
        );

        Ok(db)
    }

}

fn is_system_path(path: &Path) -> bool {
    let path_str = path.to_string_lossy().to_lowercase();

    // -----------------
    // Linux / Unix
    // -----------------
    path_str.starts_with("/bin")   ||
    path_str.starts_with("/sbin")  ||
    path_str.starts_with("/lib")   ||
    path_str.starts_with("/lib64") ||
    path_str.starts_with("/usr")   ||
    path_str.starts_with("/etc")   ||
    path_str.starts_with("/var")   ||
    path_str.starts_with("/opt")   ||
    path_str.starts_with("/snap")  ||

    // -----------------
    // Windows
    // -----------------
    path_str.contains("\\windows\\system32")        ||
    path_str.contains("\\windows")                  ||
    path_str.contains("\\program files")            ||
    path_str.contains("\\program files (x86)")      ||
    path_str.contains("\\programdata")              ||
    path_str.contains("\\appdata\\local\\programs") || 
    path_str.contains("\\users\\default")           ||           
    path_str.contains("\\users\\public")            ||

    // -----------------
    // macOS
    // -----------------
    path_str.starts_with("/applications") ||
    path_str.starts_with("/system")       ||
    path_str.starts_with("/library")      ||
    path_str.starts_with("/usr")          ||        
    path_str.starts_with("/bin")          ||
    path_str.starts_with("/sbin")         ||
    path_str.starts_with("/opt")          ||
    path_str.starts_with("/var")
}
