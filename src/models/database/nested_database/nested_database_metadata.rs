use serde::{Serialize, Deserialize};
use std::{
    path::{PathBuf},
    fs::{
        create_dir_all,
    },
    io::{Write},
};
use super::{
    core::{
        StringPath
    },
    response::AppError,
    operations::{DatabaseArguments,},
    Database,
};
use colored::*;
use std::io;

#[derive(Serialize, Deserialize, Debug, Clone, Hash, PartialEq, Eq)]
pub struct DatabaseName(pub String);

#[derive(Serialize, Deserialize, Debug, Clone, Hash, PartialEq, Eq)]
pub struct DatabasePath(pub PathBuf);

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct NestedDatabaseRecord {
    pub database_name: DatabaseName,
    pub database_path: DatabasePath,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct NestedDatabaseMetadata {
    pub data: Vec<NestedDatabaseRecord>,
    pub file_path: StringPath,
}

impl NestedDatabaseMetadata {
    pub fn login_nested_database(&self, db_name: &str, db_password: &str) -> Option<Database> {
        // Find the nested database record
        let record = match self.data.iter()
            .find(|db: &&NestedDatabaseRecord| db.database_name.0 == db_name) {
            Some(r) => r,
            None => {
                println!("{}", "❌ Nested database not found.".red());
                return None;
            }
        };

        // Try to open the nested database with the provided password directly
        let nested_db_args = DatabaseArguments {
            database_name: db_name.to_string(),
            owner: String::new(), // These fields won't be used for verification
            description: String::new(),
            master_password: db_password.to_string(), // Use the password directly
            is_nested: true,
            root_directory: record.database_path.0.clone(),
        };

        match Database::with_dir(&nested_db_args) {
            Ok(db) => Some(db),
            Err(_) => {
                println!("{}", "❌ Invalid password for nested database.".red());
                None
            }
        }
    }
    
    /// Adds a new nested database inside this database
    pub fn create_nested_database(&mut self, args: &DatabaseArguments) -> Result<bool, AppError> {
        // Create nested database arguments with the provided password directly
        let nested_args = DatabaseArguments {
            database_name: args.database_name.clone(),
            owner: args.owner.clone(),
            description: args.description.clone(),
            master_password: args.master_password.clone(), // Use the password directly
            is_nested: true,
            root_directory: PathBuf::from(&args.root_directory)
                .join("nested")
                .join(&args.database_name),
        };
        
        // Create the nested directory if it doesn't exist
        if !nested_args.root_directory.exists() {
            create_dir_all(&nested_args.root_directory).map_err(|e| {
                AppError::io_error(format!("Failed to create nested DB directory: {}", e))
            })?;
        }
        
        // Create the nested database
        let _nested_db: Database = match Database::with_dir(&nested_args) {
            Ok(d) => d,
            Err(e) => {
                println!("Failed to initialize nested database: {}", e);
                return Ok(false);
            }
        };
        
        self.data.push(NestedDatabaseRecord {
            database_name: DatabaseName(args.database_name.clone()),
            database_path: DatabasePath(nested_args.root_directory.clone()),
        });

        self.save()?;
        Ok(true)
    }

    /// Removes a nested database by name
    pub fn delete_nested_database(&mut self, db_name: &str, db_password: &str) -> Result<bool, AppError> {
        // Find the nested database record
        let index = match self.data.iter()
            .position(|db| db.database_name.0 == db_name) {
            Some(i) => i,
            None => {
                println!("{}", "❌ Nested database not found.".red());
                return Ok(false);
            }
        };

        // Get the database path before removing the record
        let db_path = self.data[index].database_path.0.clone();

        // First, try to login to the nested database to verify the password
        println!("{}", "🔐 Verifying nested database password...".yellow());

        // Try to open the nested database with the provided password directly
        let nested_db_args = DatabaseArguments {
            database_name: db_name.to_string(),
            owner: String::new(), // These fields won't be used for verification
            description: String::new(),
            master_password: db_password.to_string(), // Use the password directly
            is_nested: true,
            root_directory: db_path.clone(),
        };

        let _nested_db = match Database::with_dir(&nested_db_args) {
            Ok(db) => db,
            Err(_) => {
                println!("{}", "❌ Invalid password for nested database.".red());
                return Ok(false);
            }
        };

        // If we reach here, password verification was successful
        println!("{}", "✅ Password verified successfully.".green());

        // Ask for confirmation
        let mut input = String::new();
        println!("{}", "⚠️  This action cannot be undone!".yellow().bold());
        print!("{}", format!("🗑️  Delete nested database '{}' and all its data? (y/n): ", db_name).red());

        io::stdout().flush().map_err(|e| {
            AppError::io_error(format!("Failed to flush stdout: {}", e))
        })?;

        io::stdin().read_line(&mut input).map_err(|e| {
            AppError::io_error(format!("Failed to read input: {}", e))
        })?;

        match input.trim().to_lowercase().as_str() {
            "y" | "yes" => {
                // Proceed with deletion
            }
            "n" | "no" => {
                println!("{}", "✅ Deletion cancelled.".green());
                return Ok(false);
            }
            _ => {
                println!("{}", "❌ Invalid input. Deletion cancelled.".red());
                return Ok(false);
            }
        }

        // Remove the directory and all its contents
        if db_path.exists() {
            if let Err(e) = std::fs::remove_dir_all(&db_path) {
                eprintln!("{}: {}", "❌ Failed to remove database directory".red(), e);
                return Ok(false);
            }
            println!("{}", "✅ Database directory removed successfully.".green());
        } else {
            println!("{}", "⚠️  Database directory not found, but removing from records.".yellow());
        }

        // Remove from nested databases list
        self.data.remove(index);

        // Save the updated metadata
        self.save()?;

        println!("{}", "✅ Nested database removed successfully.".green());
        Ok(true)
    }

    /// Interactive function to remove a nested database with selection and password input
    pub fn delete_nested_database_interactive(&mut self) -> Result<bool, AppError> {
        if self.data.is_empty() {
            println!("{}", "No nested databases found to remove.".yellow());
            return Ok(false);
        }

        // Display available nested databases
        self.print_nested_list();

        let mut input = String::new();
        print!("{}", "Enter the number of the database to remove: ".yellow());

        io::stdout().flush().map_err(|e| {
            AppError::io_error(format!("Failed to flush stdout: {}", e))
        })?;

        io::stdin().read_line(&mut input).map_err(|e| {
            AppError::io_error(format!("Failed to read input: {}", e))
        })?;

        let index: usize = match input.trim().parse::<usize>() {
            Ok(i) if i > 0 && i <= self.data.len() => i - 1,
            _ => {
                println!("{}", "❌ Invalid selection.".red());
                return Ok(false);
            }
        };

        let db_name = self.data[index].database_name.0.clone();

        // Get password for the nested database
        let db_password = loop {
            print!("{}", format!("🔑 Enter master password for nested database '{}': ", db_name).yellow());
            io::stdout().flush().map_err(|e| {
                AppError::io_error(format!("Failed to flush stdout: {}", e))
            })?;

            let password = rpassword::prompt_password("").map_err(|e| {
                AppError::io_error(format!("Failed to read password: {}", e))
            })?;

            if password.is_empty() {
                println!("{}", "⚠️  Password cannot be empty.".red());
                continue;
            }

            break password;
        };

        self.delete_nested_database(&db_name, &db_password)
    }

    pub fn print_nested_list(&self) {
        if self.data.is_empty() {
            println!("{}", "No nested databases found.".yellow());
            return;
        }
        println!("{}", "Nested Databases:".green().bold());
        for (i, db) in self.data.iter().enumerate() {
            println!("{}: {}", i + 1, db.database_name.0.blue());
        }
    }  
}