use super::{
    structs::{Database, DatabaseArguments, NestedDatabaseRecord},
    rpassword::read_password,
    std::{
        path::PathBuf,
        io, io::{Write},
        fs::{create_dir_all, remove_dir_all},
    },
    colored::*
};



impl Database {
    pub fn login_nested_database(&self, db_name: &str, db_password: &str) -> Option<Database> {
        // Find the nested database record
        let record = match self.meta.nested_db_meta.data.iter()
            .find(|db: &&NestedDatabaseRecord| db.db_name == db_name) {
            Some(r) => r,
            None => {
                println!("{}", "❌ Nested database not found.".red());
                return None;
            }
        };

        // Try to open the nested database with the provided password directly
        let nested_db_args = DatabaseArguments {
            db_name: db_name.to_string(),
            owner: String::new(), // These fields won't be used for verification
            description: String::new(),
            master_password: db_password.to_string(), // Use the password directly
            is_nested: true,
            root_directory: record.db_path.clone(),
        };

        match Database::with_dir(&nested_db_args) {
            Some(db) => Some(db),
            None => {
                println!("{}", "❌ Invalid password for nested database.".red());
                None
            }
        }
    }
    
    /// Adds a new nested database inside this database
    pub fn create_nested_database(&mut self, args: &DatabaseArguments) -> bool {
        // Create nested database arguments with the provided password directly
        let nested_args = DatabaseArguments {
            db_name: args.db_name.clone(),
            owner: args.owner.clone(),
            description: args.description.clone(),
            master_password: args.master_password.clone(), // Use the password directly
            is_nested: true,
            root_directory: PathBuf::from(&self.directories.root_directory)
                .join("nested")
                .join(&args.db_name),
        };
        
        // Create the nested directory if it doesn't exist
        if !nested_args.root_directory.exists() {
            if create_dir_all(&nested_args.root_directory).is_err() {
                eprintln!("⚠️ Failed to create nested DB directory");
                return false;
            }
        }
        
        // Create the nested database
        let _nested_db: Database = match Database::with_dir(&nested_args) {
            Some(d) => d,
            None => {
                println!("Failed to initialize nested database. Exiting...");
                return false;
            }
        };
        
        self.meta.nested_db_meta.data.push( NestedDatabaseRecord {
            db_name: args.db_name.clone(),
            db_path: nested_args.root_directory.clone(),
        });

        let _ = self.meta.nested_db_meta.save();
        true
    }

    /// Removes a nested database by name
    pub fn delete_nested_database(&mut self, db_name: &str, db_password: &str) -> bool {
        // Find the nested database record
        let index = match self.meta.nested_db_meta.data.iter()
            .position(|db| db.db_name == db_name) {
            Some(i) => i,
            None => {
                println!("{}", "❌ Nested database not found.".red());
                return false;
            }
        };

        // Get the database path before removing the record
        let db_path = self.meta.nested_db_meta.data[index].db_path.clone();

        // First, try to login to the nested database to verify the password
        println!("{}", "🔐 Verifying nested database password...".yellow());

        // Try to open the nested database with the provided password directly
        let nested_db_args = DatabaseArguments {
            db_name: db_name.to_string(),
            owner: String::new(), // These fields won't be used for verification
            description: String::new(),
            master_password: db_password.to_string(), // Use the password directly
            is_nested: true,
            root_directory: db_path.clone(),
        };

        let _nested_db = match Database::with_dir(&nested_db_args) {
            Some(db) => db,
            None => {
                println!("{}", "❌ Invalid password for nested database.".red());
                return false;
            }
        };

        // If we reach here, password verification was successful
        println!("{}", "✅ Password verified successfully.".green());

        // Ask for confirmation
        let mut input = String::new();
        println!("{}", "⚠️  This action cannot be undone!".yellow().bold());
        print!("{}", format!("🗑️  Delete nested database '{}' and all its data? (y/n): ", db_name).red());

        if let Err(_) = std::io::stdout().flush() {
            eprintln!("Failed to flush stdout");
            return false;
        }

        if let Err(_) = std::io::stdin().read_line(&mut input) {
            eprintln!("Failed to read input");
            return false;
        }

        match input.trim().to_lowercase().as_str() {
            "y" | "yes" => {
                // Proceed with deletion
            }
            "n" | "no" => {
                println!("{}", "✅ Deletion cancelled.".green());
                return false;
            }
            _ => {
                println!("{}", "❌ Invalid input. Deletion cancelled.".red());
                return false;
            }
        }

        // Remove the directory and all its contents
        if db_path.exists() {
            if let Err(e) = remove_dir_all(&db_path) {
                eprintln!("{}: {}", "❌ Failed to remove database directory".red(), e);
                return false;
            }
            println!("{}", "✅ Database directory removed successfully.".green());
        } else {
            println!("{}", "⚠️  Database directory not found, but removing from records.".yellow());
        }

        // Remove from nested databases list
        self.meta.nested_db_meta.data.remove(index);

        // Save the updated metadata
        if !self.meta.nested_db_meta.save() {
            eprintln!("{}", "⚠️  Failed to save metadata updates.".yellow());
            return false;
        }

        println!("{}", "✅ Nested database removed successfully.".green());
        true
    }

    /// Interactive function to remove a nested database with selection and password input
    pub fn delete_nested_database_interactive(&mut self) -> bool {
        if self.meta.nested_db_meta.data.is_empty() {
            println!("{}", "No nested databases found to remove.".yellow());
            return false;
        }

        // Display available nested databases
        self.print_nested_list();

        let mut input = String::new();
        print!("{}", "Enter the number of the database to remove: ".yellow());

        if let Err(_) = std::io::stdout().flush() {
            eprintln!("Failed to flush stdout");
            return false;
        }

        if let Err(_) = std::io::stdin().read_line(&mut input) {
            eprintln!("Failed to read input");
            return false;
        }

        let index: usize = match input.trim().parse::<usize>() {
            Ok(i) if i > 0 && i <= self.meta.nested_db_meta.data.len() => i - 1,
            _ => {
                println!("{}", "❌ Invalid selection.".red());
                return false;
            }
        };

        let db_name = self.meta.nested_db_meta.data[index].db_name.clone();

        // Get password for the nested database
        let db_password = loop {
            print!("{}", format!("🔑 Enter master password for nested database '{}': ", db_name).yellow());
            io::stdout().flush().map_err(|e| e.to_string()).unwrap_or(());

            let password = read_password().unwrap_or_default();

            if password.is_empty() {
                println!("{}", "⚠️  Password cannot be empty.".red());
                continue;
            }

            break password;
        };

        self.delete_nested_database(&db_name, &db_password)
    }

    pub fn print_nested_list(&self) {
        if self.meta.nested_db_meta.data.is_empty() {
            println!("{}", "No nested databases found.".yellow());
            return;
        }
        println!("{}", "Nested Databases:".green().bold());
        for (i, db) in self.meta.nested_db_meta.data.iter().enumerate() {
            println!("{}: {}", i + 1, db.db_name.blue());
        }
    }
}

