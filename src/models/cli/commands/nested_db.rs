use super::{
    Database, 
    operations::{
        DatabaseArguments,
    },
    utils::{run_in_dir},
    repl::{push_path, pop_path},
};
use std::io::{self, Write};
use colored::Colorize;
use std::path::{PathBuf, Path};
use rpassword::read_password;


pub fn login_nested_database(db: &mut Database, parts: &[&str], initial_dir: &Path, current_db_location: &mut String) -> bool {
    if parts.len() < 2 {
        eprintln!("❌ Usage: login_nested <database_name>");
        return false;
    }
    
    let db_name = parts[1];
    
    // First check if the nested database exists
    if !nested_database_exists(db, db_name) {
        eprintln!("❌ Nested database '{}' not found.", db_name);
        return false;
    }
    
    // Read password securely
    let db_password = match read_nested_db_password(db_name) {
        Ok(password) => password,
        Err(e) => {
            eprintln!("❌ {}", e);
            return false;
        }
    };
    
    run_in_dir(initial_dir, || {
        match db.meta.nested_databases.login_nested_database(db_name, &db_password) {
            Some(nested_db) => {
                println!("\n{}", "✅ Successfully logged into nested database!".green());
                push_path(current_db_location, &nested_db.config.name);
                *db = nested_db;
                true
            },
            None => {
                eprintln!("❌ Failed to log into nested database. Invalid password.");
                false
            },
        }
    })
}

pub fn logout_nested_database(db: &mut Database, args: &DatabaseArguments, initial_dir: &Path, current_db_location: &mut String) -> bool {
    run_in_dir(initial_dir, || {
        match Database::with_dir(args) {
            Ok(parent_db) => {
                pop_path(current_db_location);
                *db = parent_db; 
                println!("{}", "✅ Successfully logged out to parent database.".green());
                true
            },
            Err(e) => {
                eprintln!("❌ Failed to log out to parent database /n {}", e);
                false
            }
        }
    })
}

pub fn create_nested_database(db: &mut Database, _parts: &[&str]) {
    println!("\n{}", "🏗️  Create Nested Database".green().bold());
    println!("{}", "===================================".cyan());
    
    // Collect arguments interactively
    let args = match collect_nested_db_args(db) {
        Ok(args) => args,
        Err(e) => {
            eprintln!("❌ Failed to collect arguments: {}", e);
            return;
        }
    };
    
    // Create the nested database
    if match db.meta.nested_databases.create_nested_database(&args) {
        Ok(b) => b,
        Err(e) => {
            println!("{}",e);
            return;
        }
    } {
        println!("{}", "✅ Nested database created successfully!".green());
        println!("📁 Location: {}", args.root_directory.display());
    } else {
        eprintln!("❌ Failed to create nested database");
    }
}

pub fn delete_nested_database(db: &mut Database, parts: &[&str]) -> bool {
    if parts.len() < 2 {
        eprintln!("❌ Usage: delete_nested <database_name>");
        return false;
    }
    
    let db_name = parts[1];
    
    // First check if the nested database exists
    if !nested_database_exists(db, db_name) {
        eprintln!("❌ Nested database '{}' not found.", db_name);
        return false;
    }
    
    // Read password securely
    let db_password = match read_nested_db_password(db_name) {
        Ok(password) => password,
        Err(e) => {
            eprintln!("❌ {}", e);
            return false;
        }
    };
    
    match db.meta.nested_databases.delete_nested_database(db_name, &db_password) {
        Ok(_) => true,
        Err(_) => false, 
    }
}

pub fn list_nested_databases(db: &Database, _parts: &[&str]) {
    db.meta.nested_databases.print_nested_list();
}

// Helper function to check if nested database exists
fn nested_database_exists(db: &Database, db_name: &str) -> bool {
    db.meta.nested_databases.data.iter()
        .any(|nested_db| nested_db.database_name.0 == db_name)
}

// Helper function to read nested database password securely
fn read_nested_db_password(db_name: &str) -> Result<String, String> {
    print!("{}", format!("🔑 Enter master password for nested database '{}': ", db_name).yellow());
    io::stdout().flush().map_err(|e| e.to_string())?;
    
    let password = read_password().map_err(|e| e.to_string())?;
    
    if password.is_empty() {
        return Err("Password cannot be empty.".to_string());
    }
    
    Ok(password)
}

// Helper function to collect arguments interactively
fn collect_nested_db_args(parent_db: &Database) -> Result<DatabaseArguments, String> {
    let mut input = String::new();
    
    // Get nested database name with validation
    let db_name = loop {
        input.clear();
        print!("{}", "📛 Enter nested database name: ".yellow());
        io::stdout().flush().map_err(|e| e.to_string())?;
        io::stdin().read_line(&mut input).map_err(|e| e.to_string())?;
        let db_name = input.trim().to_string();
        
        if db_name.is_empty() {
            println!("{}", "⚠️  Database name cannot be empty.".red());
            continue;
        }
        
        // Check if database with this name already exists in nested databases
        let db_exists = parent_db.meta.nested_databases.data.iter()
            .any(|nested_db| nested_db.database_name.0 == db_name);
        
        if db_exists {
            println!("{}", "❌ Database with this name already exists.".red());
            continue;
        }
        
        break db_name;
    };
    
    // Get owner (default to parent database owner)
    let parent_owner = &parent_db.config.owner;
    print!("{}", format!("👤 Enter owner (default: {}): ", parent_owner).yellow());
    io::stdout().flush().map_err(|e| e.to_string())?;
    input.clear();
    io::stdin().read_line(&mut input).map_err(|e| e.to_string())?;
    let owner = match input.trim() {
        "" => parent_owner.clone(),
        o => o.to_string()
    };
    
    // Get description
    input.clear();
    print!("{}", "📝 Enter description: ".yellow());
    io::stdout().flush().map_err(|e| e.to_string())?;
    io::stdin().read_line(&mut input).map_err(|e| e.to_string())?;
    let description = input.trim().to_string();
    
    // Get master password with validation (silent input)
    let master_password = loop {
        print!("{}", "🔑 Enter master password for nested database: ".yellow());
        io::stdout().flush().map_err(|e| e.to_string())?;
        let password = read_password().map_err(|e| e.to_string())?;
        
        if password.is_empty() {
            println!("{}", "⚠️  Password cannot be empty.".red());
            continue;
        }
        
        break password;
    };
    
    // Confirm password with validation (silent input)
    let confirm_password = loop {
        print!("{}", "🔑 Confirm master password: ".yellow());
        io::stdout().flush().map_err(|e| e.to_string())?;
        let confirm = read_password().map_err(|e| e.to_string())?;
        
        if confirm.is_empty() {
            println!("{}", "⚠️  Confirmation password cannot be empty.".red());
            continue;
        }
        
        break confirm;
    };
    
    // Check if passwords match
    if master_password != confirm_password {
        return Err("❌ Passwords do not match.".to_string());
    }
    
    // Build root directory path
    let root_directory = PathBuf::from(&parent_db.directories.root_directory)
        .join("nested")
        .join(&db_name);
    
    // Create database arguments
    let db_args = DatabaseArguments {
        database_name: db_name.clone(),
        owner: owner.clone(),
        description: description.clone(),
        master_password,
        is_nested: true,
        root_directory,
    };
    
    // Display summary and ask for confirmation
    println!("\n{}", "📋 Nested Database Summary:".green().bold());
    println!("{} {}", "📛 Name:".cyan(), db_name);
    println!("{} {}", "👤 Owner:".cyan(), owner);
    println!("{} {}", "📝 Description:".cyan(), description);
    println!("{} {}", "📁 Root Directory:".cyan(), db_args.root_directory.display());
    
    // Ask for confirmation
    loop {
        input.clear();
        print!("{}", "✅ Continue with creation? (y/n): ".yellow());
        io::stdout().flush().map_err(|e| e.to_string())?;
        io::stdin().read_line(&mut input).map_err(|e| e.to_string())?;
        
        match input.trim().to_lowercase().as_str() {
            "y" | "yes" => {
                println!("{}", "🚀 Creating nested database...".green());
                break;
            }
            "n" | "no" => {
                return Err("❌ Database creation cancelled by user.".to_string());
            }
            _ => {
                println!("{}", "⚠️  Please enter 'y' for yes or 'n' for no.".red());
                continue;
            }
        }
    }
    
    Ok(db_args)
}