use super::{
    Database,
    entries::{
        PasswordEntry,
    },
};
use colored::*;
use terminal_size::{Width, terminal_size};



pub fn decrypt_all_passwords(db: &mut Database) -> bool {
    db.decrypt_all_passwords()
}

pub fn encrypt_all_passwords(db: &mut Database) -> bool {
    db.encrypt_all_passwords()
}

pub fn restore_all_passwords(db: &mut Database) -> bool {
    db.restore_all_passwords()
}

pub fn add_password(db: &mut Database, parts: &[&str]) -> bool {
    // Skip the command and get the rest
    let args = &parts[1..];

    if args.len() % 2 != 0 {
        eprintln!("❌ Mismatched arguments: you must provide name/password pairs.");
        return false;
    }

    let mut all_ok = true;
    for pair in args.chunks(2) {
        let name = pair[0];
        let password = pair[1];

        let ok = db.add_password(name, password);
        if !ok {
            eprintln!("Failed to add password for: {}", name);
            all_ok = false;
        }
    }

    all_ok
}


pub fn change_password(db: &mut Database, parts: &[&str]) -> bool {
    let args = &parts[1..];

    if args.len() % 2 != 0 {
        eprintln!("❌ Mismatched arguments: you must provide name/new_password pairs.");
        return false;
    }

    let mut all_ok = true;
    for pair in args.chunks(2) {
        let name = pair[0];
        let new_password = pair[1];

        let ok = db.change_password(name, new_password);
        if !ok {
            eprintln!("Failed to change password for: {}", name);
            all_ok = false;
        }
    }

    all_ok
}


pub fn encrypt_password(db: &mut Database, parts: &[&str]) -> bool {
    let names = &parts[1..];

    let mut all_ok = true;
    for name in names {
        let ok = db.encrypt_password(name);
        if !ok {
            eprintln!("Failed to restore password for: {}", name);
            all_ok = false;
        }
    }

    all_ok
}

pub fn decrypt_password(db: &mut Database, parts: &[&str]) -> bool {
    let names = &parts[1..];

    let mut all_ok = true;
    for name in names {
        let ok = db.decrypt_password(name);
        if !ok {
            eprintln!("Failed to restore password for: {}", name);
            all_ok = false;
        }
    }

    all_ok
}

pub fn delete_password(db: &mut Database, parts: &[&str]) -> bool {
    let names = &parts[1..];

    let mut all_ok = true;
    for name in names {
        let ok = db.delete_password(name);
        if !ok {
            eprintln!("Failed to restore password for: {}", name);
            all_ok = false;
        }
    }

    all_ok
}

pub fn delete_all_passwords(db: &mut Database) -> bool {
    db.delete_all_passwords()
}

pub fn restore_password(db: &mut Database, parts: &[&str]) -> bool {
    let names = &parts[1..];

    let mut all_ok = true;
    for name in names {
        let ok = db.restore_password(name);
        if !ok {
            eprintln!("Failed to restore password for: {}", name);
            all_ok = false;
        }
    }

    all_ok
}


pub fn list_decrypted_passwords(db: &mut Database) -> bool {
    let passwords = db.list_decrypted_passwords(Some("active"));
    print_passwords_pretty(&passwords);
    true
}

pub fn list_encrypted_passwords(db: &mut Database) -> bool {
    let passwords = db.list_encrypted_passwords();
    print_passwords_pretty(&passwords);
    true
}

pub fn list_deleted_passwords(db: &mut Database) -> bool {
    let passwords = db.list_deleted_passwords();
    print_passwords_pretty(&passwords);
    true
} 

pub fn search_encrypted_passwords(db: &mut Database, parts: &[&str])-> bool {
    let query = parts[1];
    let passwords = db.search_encrypted_passwords(query);
    print_passwords_pretty(&passwords);
    true
}

pub fn search_decrypted_passwords(db: &mut Database, parts: &[&str]) -> bool {
    let query = parts[1];
    let passwords = db.search_decrypted_passwords(query);
    print_passwords_pretty(&passwords);
    true
}

pub fn search_deleted_passwords(db: &mut Database, parts: &[&str]) -> bool {
    let query = parts[1];
    let passwords = db.search_deleted_passwords(query);
    print_passwords_pretty(&passwords);
    true
}

pub fn list_all_passwords(db: &mut Database) -> bool {
    let mut passwords = db.list_decrypted_passwords(Some("active"));
    passwords.extend(db.list_encrypted_passwords());
    print_passwords_pretty(&passwords);
    true
}

pub fn search_all_passwords(db: &mut Database, parts: &[&str]) -> bool {
    let query = parts[1];
    let mut passwords = db.search_decrypted_passwords(query);
    passwords.extend(db.search_encrypted_passwords(query));

    print_passwords_pretty(&passwords);
    true
}

// Helper functions
pub fn print_passwords_pretty(passwords: &[&PasswordEntry]) {
    if passwords.is_empty() {
        println!("{}", "No passwords found.".yellow());
        return;
    }

    println!(); // blank line on top

    // Get terminal width
    let term_width = match terminal_size() {
        Some((Width(w), _)) => w as usize,
        None => 100, // fallback width
    };

    // Base column widths with minimums
    let mut name_width = 25;
    let mut password_width = 20;
    let encrypted_width = 10; // Fixed for "Encrypted"/"Yes"/"No"
    let recycled_width = 10;  // Fixed for "Deleted"/"Yes"/"No"
    let mut created_width = 22;
    let mut updated_width = 22;

    // Find maximum content lengths
    for p in passwords {
        name_width = name_width.max(p.display_name.0.len() + 2);
        // For encrypted passwords, we only need space for "********"
        password_width = password_width.max(if p.is_encrypted.0 { 
            8 
        } else { 
            p.password.0.len() + 2 
        });
        created_width = created_width.max(p.created_at.len() + 2);
        updated_width = updated_width.max(p.updated_at.len() + 2);
    }

    // Apply minimum widths
    name_width = name_width.max(10);
    password_width = password_width.max(15);
    created_width = created_width.max(15);
    updated_width = updated_width.max(15);

    // Calculate total width
    let total_width = name_width + password_width + encrypted_width + recycled_width + created_width + updated_width + 5;

    // If total width exceeds terminal, scale proportionally but preserve fixed columns
    if total_width > term_width {
        // Calculate available width for variable columns (excluding fixed ones)
        let fixed_columns_width = encrypted_width + recycled_width + 5;
        let available_width = term_width - fixed_columns_width;
        
        // Distribute available width proportionally to variable columns
        let variable_columns_total = name_width + password_width + created_width + updated_width;
        let scale = available_width as f64 / variable_columns_total as f64;
        
        name_width = (name_width as f64 * scale).max(10.0) as usize;
        password_width = (password_width as f64 * scale).max(8.0) as usize; // Min 8 for "********"
        created_width = (created_width as f64 * scale).max(12.0) as usize;
        updated_width = (updated_width as f64 * scale).max(12.0) as usize;
    }

    // Header
    println!(
        "{:<name_width$} {:<password_width$} {:<encrypted_width$} {:<recycled_width$} {:<created_width$} {:<updated_width$}",
        "Name".bright_yellow(),
        "Password".bright_yellow(),
        "Encrypted".bright_yellow(),
        "Deleted".bright_yellow(),
        "Created".bright_yellow(),
        "Updated".bright_yellow(),
        name_width = name_width,
        password_width = password_width,
        encrypted_width = encrypted_width,
        recycled_width = recycled_width,
        created_width = created_width,
        updated_width = updated_width
    );

    // Separator
    println!("{}", "-".repeat(term_width.min(total_width)).blue());

    // Rows
    for p in passwords {
        let encrypted_str = if p.is_encrypted.0 { 
            "Yes".bright_green().bold() 
        } else { 
            "No".red() 
        };
        
        let recycled_str = if p.is_trashed.0 { 
            "Yes".red().bold()
        } else { 
            "No".bright_green() 
        };

        // Handle password display with truncation for long passwords
        let password_display = if p.is_encrypted.0 {
            "********".to_string()
        } else {
            // Truncate long passwords with ellipsis
            if p.password.0.len() > password_width {
                format!("{}...", &p.password.0[..password_width.saturating_sub(3)])
            } else {
                p.password.0.clone()
            }
        };

        let password_colored = if p.is_encrypted.0 {
            password_display.yellow()
        } else {
            password_display.bright_red()
        };

        // Handle name truncation for long names
        let name_display = if p.display_name.0.len() > name_width {
            format!("{}...", &p.display_name.0[..name_width.saturating_sub(3)])
        } else {
            p.display_name.0.clone()
        };

        // Handle date truncation if needed
        let created_display = if p.created_at.len() > created_width {
            format!("{}...", &p.created_at.as_string()[..created_width.saturating_sub(3)])
        } else {
            p.created_at.as_string()
        };

        let updated_display = if p.updated_at.len() > updated_width {
            format!("{}...", &p.updated_at.as_string()[..updated_width.saturating_sub(3)])
        } else {
            p.updated_at.as_string()
        };

        println!(
            "{:<name_width$} {:<password_width$} {:<encrypted_width$} {:<recycled_width$} {:<created_width$} {:<updated_width$}",
            name_display.bright_cyan(),
            password_colored,
            encrypted_str,
            recycled_str,
            created_display.dimmed(),
            updated_display.dimmed(),
            name_width = name_width,
            password_width = password_width,
            encrypted_width = encrypted_width,
            recycled_width = recycled_width,
            created_width = created_width,
            updated_width = updated_width
        );
    }

    println!(); // blank line at bottom
}