use std::{
    path::{PathBuf},
    fmt,
    io,
    io::{Write},
};
use rpassword::{read_password};
use colored::*;
use super::{
    response::AppError,
};


#[derive(Clone, Debug, Default)]
pub struct DatabaseArguments {
    pub database_name: String,
    pub owner: String,
    pub description: String,
    pub master_password: String,
    pub is_nested: bool,
    pub root_directory: PathBuf
}

fn prompt_user(prompt: &str) -> String {
    print!("{}", prompt.bright_blue().bold());
    io::stdout().flush().unwrap();
    
    let mut input = String::new();
    io::stdin().read_line(&mut input).expect("❌ Failed to read input");
    input.trim().to_string()
}
fn prompt_password(prompt: &str) -> String {
    print!("{}", prompt.bright_blue().bold());
    io::stdout().flush().unwrap();
    
    read_password().expect("❌ Failed to read password")
}
fn confirm_operation(question: &str) -> bool {
    print!("{} {} ", question.bright_yellow().bold(), "(y/N)".bright_black());
    io::stdout().flush().unwrap();
    
    let mut input = String::new();
    io::stdin().read_line(&mut input).expect("❌ Failed to read input");
    
    matches!(input.trim().to_lowercase().as_str(), "y" | "yes")
}
fn print_dataseal_banner() {
    println!("{}", "🔐 DataSeal - Secure Database Manager".bright_green().bold());
    println!("{}", "=".repeat(50).bright_black());
}
fn print_arguments_summary(args: &DatabaseArguments) {
    println!();
    println!("{}", "📋 Configuration Summary:".bright_cyan());
    println!("{}", "―".repeat(30).bright_black());
    println!("{} {}", "Database:".bright_white(), args.database_name.bright_green());
    println!("{} {}", "Owner:".bright_white(), args.owner.bright_green());
    println!("{} {}", "Description:".bright_white(), args.description.bright_green());
    println!("{} {}", "Storage:".bright_white(), args.root_directory.display().to_string().bright_green());
    println!();
}


impl DatabaseArguments {
    pub fn collect_signup_arguments(root_directory: impl Into<PathBuf>) -> Result<Self, AppError> {
        print_dataseal_banner();
        let mut database_arguments: DatabaseArguments = DatabaseArguments::default();
        database_arguments.root_directory = root_directory.into();

        println!("{}", "🆕 Welcome to DataSeal! Let's set up your database.".bright_green());
        println!("{}", "―".repeat(50).bright_black());
        
        database_arguments.database_name = prompt_user("📂 Enter database name: ");
        database_arguments.owner = prompt_user("👤 Enter your name: ");
        database_arguments.description = prompt_user("📝 Enter database description: ");
        
        let master_password = prompt_password("🔑 Create master password: ");
        let confirm_password = prompt_password("🔑 Confirm master password: ");
        
        if master_password != confirm_password {
            eprintln!("{}", "❌ Passwords do not match!".bright_red());
            return Err(AppError::validation_error("Passwords do not match!")
                .with_context("While Signing up")
                .with_suggestion("Enter same password when sign in dataseal")
        );
        }
        
        if master_password.len() < 8 {
            eprintln!("{}", "❌ Password must be at least 8 characters long!".bright_red());
            return Err(AppError::validation_error("Password must be at least 8 characters long!")
                .with_context("While Signing up")
                .with_suggestion("Enter password at least8 or more then 8 characters")
        );
        }

        database_arguments.master_password = master_password;

        if !confirm_operation("Proceed with this configuration?") {
            println!("{}", "👋 Setup cancelled.".bright_yellow());
            return Err(AppError::validation_error("Setup cancelled."));
        }

        print_arguments_summary(&database_arguments);

        Ok(database_arguments)
    }

    pub fn collect_login_arguments(root_directory: impl Into<PathBuf>) -> Result<Self, AppError> {
        let root_directory = root_directory.into();
        let config_path = root_directory.join(".config/.config.json");

        if config_path.exists() {
            println!(
                "{} {}",
                "📁 Config path:".bright_cyan(),
                config_path.display().to_string().bright_white()
            );

            let master_password = prompt_password("🔑 Enter master password: ");

            if master_password.len() < 8 {
                return Err(
                    AppError::validation_error("Password must be at least 8 characters long!")
                        .with_context("While logging in")
                        .with_suggestion("Enter a password with 8 or more characters"),
                );
            }

            Ok(Self {
                database_name: "default".to_string(),
                owner: "user".to_string(),
                description: "Existing database".to_string(),
                master_password,
                is_nested: false,
                root_directory,
            })
        } else {
            Err(AppError::not_found(".config.json not found in the root directory")
                .with_context("While logging in")
                .with_suggestion("Run setup first to create a new database"))
        }
    }
}


impl fmt::Display for DatabaseArguments {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:#?}", self)
    }
}







