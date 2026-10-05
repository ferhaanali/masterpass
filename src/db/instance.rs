use super::{
    structs::{
        DatabaseMeta, Database, Directories, Master, Meta, MetaType, NestedDatabaseMetaType, DatabaseArguments,
        Config,
    },
    enc_keys::{derive_key},
    std::{
        fs::{create_dir_all, write},
        path::{Path, PathBuf},
        env,
    },
};




impl Database {
    pub fn new(args: &mut DatabaseArguments) -> Option<Self> {
        // Detect where the binary is running from
        let exe_path = env::current_exe().ok()?;
        let exe_dir = exe_path.parent()?.to_path_buf();

        // Base directory for all databases
        let base_dir: PathBuf = if is_system_path(&exe_dir) {
            dirs::home_dir()?.join(".dataseal")
        } else {
            exe_dir.join("data")
        };

        // Root directory for this specific DB
        let root_dir = if args.is_nested {
            base_dir.join("nested").join(args.db_name.clone())
        } else {
            base_dir.join(args.db_name.clone())
        };

        // Create root directory if it doesn't exist
        if !root_dir.exists() {
            create_dir_all(&root_dir).ok()?;
        }

        let root_dir_static: &'static Path = Box::leak(Box::new(root_dir));
        args.root_directory = root_dir_static.to_path_buf();

        Self::with_dir(args)
    }

    pub fn with_dir( args: &DatabaseArguments ) -> Option<Self> {
        let root_directory_str = args.root_directory.to_str()?.trim_end_matches('/').to_string();

        // Main folders
        let decrypted_dir = args.root_directory.join(".decrypted");
        let decrypted_files_dir = decrypted_dir.join(".files");
        let decrypted_folders_dir = decrypted_dir.join(".folders");

        let encrypted_dir = args.root_directory.join(".encrypted");
        let encrypted_files_dir = encrypted_dir.join(".files");
        let encrypted_folders_dir = encrypted_dir.join(".folders");

        let recycle_dir = args.root_directory.join(".recycle_bin");
        let recycle_files_dir = recycle_dir.join(".files");
        let recycle_folders_dir = recycle_dir.join(".folders");

        let nested_db_dir = args.root_directory.join(".nested");

        // Meta and config files
        let decrypted_meta_file = decrypted_dir.join(".meta.json");
        let trash_meta_file = recycle_dir.join(".trash_meta.json");
        let encrypted_meta_file = encrypted_dir.join(".meta.json");
        let nested_db_meta_file = nested_db_dir.join(".nested_db_record.json");
        let config_file = args.root_directory.join(".config.json");


        // Load or create config
        let config = match Config::load_or_create(args, &config_file.to_str().unwrap()) {
            Ok(cfg) => cfg,
            Err(e) => {
                eprintln!("{}", e);
                return None;
            }
        };

        let master_key = derive_key(&config.kdf_salt_b64, &args.master_password);

        // Ensure directories exist
        for dir in &[
            nested_db_dir.as_path(), &args.root_directory, decrypted_dir.as_path(),
            encrypted_dir.as_path(), recycle_dir.as_path(), decrypted_files_dir.as_path(),
            encrypted_files_dir.as_path(), encrypted_folders_dir.as_path(), decrypted_folders_dir.as_path()
        ] {
            if !dir.exists() {
                if let Err(e) = create_dir_all(dir) {
                    eprintln!("⚠️ Failed to create directory {}: {}", dir.display(), e);
                    return None;
                }
            }
        }

        // Create extensions inside decrypted/files
        for ext in ["photos", "videos", "documents", "other"] {
            let ext_dir = decrypted_files_dir.join(ext);
            if !ext_dir.exists() {
                if let Err(e) = create_dir_all(&ext_dir) {
                    eprintln!("⚠️ Failed to create folder {}: {}", ext_dir.display(), e);
                    return None;
                }
            }
        }

        // Helper to create empty JSON file
        fn create_empty_json_file(path: &Path) -> Result<(), std::io::Error> {
            if !path.exists() {
                write(path, "{}")?;
            }
            Ok(())
        }

        // Create meta files
        for file in &[&nested_db_meta_file ,&decrypted_meta_file, &trash_meta_file, &encrypted_meta_file] {
            if let Err(e) = create_empty_json_file(file) {
                eprintln!("⚠️ Failed to create {}: {}", file.display(), e);
                return None;
            }
        }


        let mut db = Database {

            directories: Directories {
                root_directory: root_directory_str,

                decrypted_dir: decrypted_dir.to_str()?.to_string(),
                decrypted_files_dir: decrypted_files_dir.to_str()?.to_string(),
                decrypted_folders_dir: decrypted_folders_dir.to_str()?.to_string(),

                encrypted_dir: encrypted_dir.to_str()?.to_string(),
                encrypted_files_dir: encrypted_files_dir.to_str()?.to_string(),
                encrypted_folders_dir: encrypted_folders_dir.to_str()?.to_string(),

                recycle_dir: recycle_dir.to_str()?.to_string(),
                recycle_files_dir: recycle_files_dir.to_str()?.to_string(), 
                recycle_folders_dir: recycle_folders_dir.to_str()?.to_string(),

                nested_db_dir: nested_db_dir.to_str()?.to_string(),
            },

            meta: Meta {
                decrypted_meta: MetaType {
                    data: DatabaseMeta::default(),
                    file_path: decrypted_meta_file.to_str()?.to_string(),
                },
                encrypted_meta: MetaType {
                    data: DatabaseMeta::default(),
                    file_path: encrypted_meta_file.to_str()?.to_string(),
                },
                trash_meta: MetaType {
                    data: DatabaseMeta::default(),
                    file_path: trash_meta_file.to_str()?.to_string(),
                },
                nested_db_meta: NestedDatabaseMetaType {
                    data: Vec::new(),
                    file_path: nested_db_meta_file.to_str()?.to_string(),
                },
            },

            config: config.clone(),

            master: Master {
                key: master_key,
                password: Some(args.master_password.to_string()),
            }

        };

        config.save();
        db.meta.encrypted_meta.load();
        db.meta.decrypted_meta.load();
        db.meta.trash_meta.load();
        db.config.db_info.stats.update(&db.get_database_stats());
        let _ = db.meta.nested_db_meta.load();

        Some(db)
    }
}

fn is_system_path(path: &Path) -> bool {
    let path_str = path.to_string_lossy().to_lowercase();

    // -----------------
    // Linux / Unix
    // -----------------
    path_str.starts_with("/bin") ||
    path_str.starts_with("/sbin") ||
    path_str.starts_with("/lib") ||
    path_str.starts_with("/lib64") ||
    path_str.starts_with("/usr") ||     // /usr, /usr/bin, /usr/local, /usr/share...
    path_str.starts_with("/etc") ||
    path_str.starts_with("/var") ||
    path_str.starts_with("/opt") ||
    path_str.starts_with("/snap") ||

    // -----------------
    // Windows
    // -----------------
    path_str.contains("\\windows\\system32") ||
    path_str.contains("\\windows") ||
    path_str.contains("\\program files") ||
    path_str.contains("\\program files (x86)") ||
    path_str.contains("\\programdata") ||
    path_str.contains("\\appdata\\local\\programs") || // e.g. VSCode install dir
    path_str.contains("\\users\\default") ||           // default profile
    path_str.contains("\\users\\public") ||

    // -----------------
    // macOS
    // -----------------
    path_str.starts_with("/applications") ||
    path_str.starts_with("/system") ||
    path_str.starts_with("/library") ||
    path_str.starts_with("/usr") ||        // /usr/bin, /usr/local
    path_str.starts_with("/bin") ||
    path_str.starts_with("/sbin") ||
    path_str.starts_with("/opt") ||
    path_str.starts_with("/var")
}
