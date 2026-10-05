use std::fmt;
use colored::*;
use chrono::{Utc, DateTime};
use super::{
    core::{Timestamp},
};


pub struct AppError {
    pub kind: ErrorKind,
    pub message: String,
    pub context: Option<String>,
    pub details: Option<String>,
    pub timestamp: Option<Timestamp>,
    pub suggestion: Option<String>,

    pub code: i32,
}

pub enum ErrorKind {
    // Security
    CryptoError,
    EncryptionFailed,
    DecryptionFailed,
    InvalidKey,
    
    // Storage
    IoError,
    FileNotFound,
    PermissionDenied,
    DiskFull,
    
    // Database  
    DatabaseError,
    CorruptedData,
    VersionMismatch,
    
    // Validation
    ValidationError,
    InvalidInput,
    DuplicateEntry,
    NotFound,
    
    // Performance
    Timeout,
    ResourceLimit,
    
    // System
    ConfigurationError,
    InitializationError,
    RuntimeError,

    // Operations
    DuplicatePath,
    InvalidOperation
}

impl AppError {
    pub fn new(kind: ErrorKind, msg: impl Into<String>) -> Self {
        Self {
            kind,
            message: msg.into(),
            context: None,
            details: None,
            timestamp: None,
            suggestion: None, 
            code: 0
        }
    }

    pub fn with_details(mut self, details: impl Into<String>) -> Self {
        self.details = Some(details.into());
        self
    }

    pub fn with_context(mut self, context: impl Into<String>) -> Self {
        self.context = Some(context.into());
        self
    }

    pub fn with_time(mut self, time: DateTime<Utc>) -> Self {
        self.timestamp = Some(Timestamp(time));
        self
    }

    pub fn with_time_now(mut self) -> Self { // Added missing `pub`
        self.timestamp = Some(Timestamp::now());
        self
    }

    pub fn with_suggestion(mut self, suggestion: impl Into<String>) -> Self { // Fixed parameter name
        self.suggestion = Some(suggestion.into());
        self
    }

    pub fn with_code(mut self, code: impl Into<i32>) -> Self {
        self.code = code.into();
        self
    }
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f)?;
        writeln!(f, "{}", "┌─ ERROR ─────────────────────────────────────────────────┐".bright_red().bold())?;
        
        let main_text = format!("{}: {}", self.kind, self.message);
        for (i, line) in main_text.lines().enumerate() {
            if i == 0 {
                write!(f, "{} {}", "│".bright_red().bold(), line.bright_white().bold())?;
            } else {
                write!(f, "\n{}   {}", "│".bright_red().bold(), line.bright_white())?;
            }
        }

        let sections = [
            ("Details", &self.details, colored::Color::Cyan),
            ("Context", &self.context, colored::Color::Magenta),
            ("Suggestion", &self.suggestion, colored::Color::Yellow),
            ("Time", &self.timestamp.as_ref().map(|t| t.to_string()), colored::Color::Green),
        ];

        for (title, content_opt, color) in sections.iter() {
            if let Some(content) = content_opt {
                write!(f, "\n{} {}", "│".bright_red().bold(), 
                    format!("{}:", title).color(*color).bold())?;
                
                for (i, line) in content.lines().enumerate() {
                    if i == 0 {
                        write!(f, "\n{}   {}", "│".bright_red().bold(), line.bright_white())?;
                    } else {
                        write!(f, "\n{}   {}", "│".bright_red().bold(), line.bright_white())?;
                    }
                }
            }
        }

        writeln!(f)?;
        write!(f, "{}", "└────────────────────────────────────────────────────────────┘".bright_red().bold())?;
        Ok(())
    }
}

impl fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let error_str = match self {
            // Security
            ErrorKind::CryptoError => "Crypto Error",
            ErrorKind::EncryptionFailed => "Encryption Failed",
            ErrorKind::DecryptionFailed => "Decryption Failed",
            ErrorKind::InvalidKey => "Invalid Key",
            
            // Storage
            ErrorKind::IoError => "I/O Error",
            ErrorKind::FileNotFound => "File Not Found",
            ErrorKind::PermissionDenied => "Permission Denied",
            ErrorKind::DiskFull => "Disk Full",
            
            // Database
            ErrorKind::DatabaseError => "Database Error",
            ErrorKind::CorruptedData => "Corrupted Data",
            ErrorKind::VersionMismatch => "Version Mismatch",
            
            // Validation
            ErrorKind::ValidationError => "Validation Error",
            ErrorKind::InvalidInput => "Invalid Input",
            ErrorKind::DuplicateEntry => "Duplicate Entry",
            ErrorKind::NotFound => "Not Found",
            
            // Performance
            ErrorKind::Timeout => "Timeout",
            ErrorKind::ResourceLimit => "Resource Limit Exceeded",
            
            // System
            ErrorKind::ConfigurationError => "Configuration Error",
            ErrorKind::InitializationError => "Initialization Error",
            ErrorKind::RuntimeError => "Runtime Error",

            // Operation
            ErrorKind::DuplicatePath => "Duplicate Path",
            ErrorKind::InvalidOperation => "Invalid Operation",
        };
        
        write!(f, "{}", error_str.red().bold())
    }
}


impl AppError {
   
    // Security constructors
    pub fn crypto_error(message: impl Into<String>) -> Self {
        Self::new(ErrorKind::CryptoError, message)
    }

    pub fn encryption_failed(message: impl Into<String>) -> Self {
        Self::new(ErrorKind::EncryptionFailed, message)
    }

    pub fn decryption_failed(message: impl Into<String>) -> Self {
        Self::new(ErrorKind::DecryptionFailed, message)
    }

    pub fn invalid_key(message: impl Into<String>) -> Self {
        Self::new(ErrorKind::InvalidKey, message)
    }

    // Storage constructors
    pub fn io_error(message: impl Into<String>) -> Self {
        Self::new(ErrorKind::IoError, message)
    }

    pub fn file_not_found(message: impl Into<String>) -> Self {
        Self::new(ErrorKind::FileNotFound, message)
    }

    pub fn permission_denied(message: impl Into<String>) -> Self {
        Self::new(ErrorKind::PermissionDenied, message)
    }

    pub fn disk_full(message: impl Into<String>) -> Self {
        Self::new(ErrorKind::DiskFull, message)
    }

    // Database constructors
    pub fn database_error(message: impl Into<String>) -> Self {
        Self::new(ErrorKind::DatabaseError, message)
    }

    pub fn corrupted_data(message: impl Into<String>) -> Self {
        Self::new(ErrorKind::CorruptedData, message)
    }

    pub fn version_mismatch(message: impl Into<String>) -> Self {
        Self::new(ErrorKind::VersionMismatch, message)
    }

    // Validation constructors
    pub fn validation_error(message: impl Into<String>) -> Self {
        Self::new(ErrorKind::ValidationError, message)
    }

    pub fn invalid_input(message: impl Into<String>) -> Self {
        Self::new(ErrorKind::InvalidInput, message)
    }

    pub fn duplicate_entry(message: impl Into<String>) -> Self {
        Self::new(ErrorKind::DuplicateEntry, message)
    }

    pub fn not_found(message: impl Into<String>) -> Self {
        Self::new(ErrorKind::NotFound, message)
    }

    // Performance constructors
    pub fn timeout(message: impl Into<String>) -> Self {
        Self::new(ErrorKind::Timeout, message)
    }

    pub fn resource_limit(message: impl Into<String>) -> Self {
        Self::new(ErrorKind::ResourceLimit, message)
    }

    // System constructors
    pub fn configuration_error(message: impl Into<String>) -> Self {
        Self::new(ErrorKind::ConfigurationError, message)
    }

    pub fn initialization_error(message: impl Into<String>) -> Self {
        Self::new(ErrorKind::InitializationError, message)
    }

    pub fn runtime_error(message: impl Into<String>) -> Self {
        Self::new(ErrorKind::RuntimeError, message)
    }
    
    pub fn duplicate_path(message: impl Into<String>) -> Self {
        Self::new(ErrorKind::DuplicatePath, message) 
    }

    pub fn invalid_operation(message: impl Into<String>) -> Self {
        Self::new(ErrorKind::InvalidOperation, message) 
    }
}