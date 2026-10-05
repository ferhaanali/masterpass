use super::{
    utils::run_in_dir,
    Database,
    entries::{
        FileEntry,
    }
};
use colored::*;
use std::path::{Path, PathBuf};
use terminal_size::{Width, terminal_size};




pub fn decrypt_all_files(db: &mut Database, initial_dir: &Path) -> bool {
    run_in_dir(initial_dir,|| {
        db.decrypt_all_files()
    })
}

pub fn encrypt_all_files(db: &mut Database, initial_dir: &Path) -> bool {
    run_in_dir(initial_dir, || {
        db.encrypt_all_files()
    })
}

pub fn paste_file(db: &mut Database, parts: &[&str],current_dir: &Path, initial_dir: &Path) -> bool {
    let file_name = parts[1];

    run_in_dir(initial_dir, || {
        db.paste_file(file_name, current_dir.to_string_lossy().as_ref())
    })
    
}

pub fn cut_paste_file(db: &mut Database, parts: &[&str],current_dir: &Path, initial_dir: &Path) -> bool {
    let file_name = parts[1];

    run_in_dir(initial_dir, || {
        db.cut_paste_file(file_name, current_dir.to_string_lossy().as_ref())
    })
    
}

pub fn restore_all_files(db: &mut Database) -> bool {
    db.restore_all_files()
}

pub fn empty_recycle_bin_verbose(db: &mut Database) -> bool {
    let (deleted_passwords, deleted_files) = db.empty_recycle_bin();

    if deleted_passwords == 0 && deleted_files == 0 {
        println!("🗑️ Recycle bin is already empty.");
    } else {
        println!("🗑️ Recycle Bin Emptied!");
        println!("Deleted passwords: {}", deleted_passwords);
        println!("Deleted files: {}", deleted_files);
    }

    true
}

pub fn add_file(
    db: &mut Database,
    parts: &[&str],
    current_dir: &Path,
    initial_dir: &Path
) -> bool {
    let args = &parts[1..];

    if args.is_empty() {
        eprintln!("No name/file pairs provided.");
        return false;
    }

    if args.len() % 2 != 0 {
        eprintln!("Mismatched arguments: you must provide name/file pairs.");
        return false;
    }

    let mut all_ok = true;

    for pair in args.chunks(2) {
        let name = pair[0];
        let filename = pair[1];

        let full_path = if Path::new(filename).is_absolute() {
            PathBuf::from(filename)
        } else {
            current_dir.join(filename)
        };

        if !full_path.exists() {
            eprintln!("{}: file not found", full_path.display().to_string().red());
            all_ok = false;
            continue;
        }

        let ok = run_in_dir(initial_dir, || {
            db.add_file(name, full_path.to_string_lossy().as_ref())
        });

        if !ok {
            eprintln!("Failed to add file for: {}", name);
            all_ok = false;
        }
    }

    all_ok
}

pub fn cut_add_file(
    db: &mut Database,
    parts: &[&str],
    current_dir: &Path,
    initial_dir: &Path
) -> bool {
    let args = &parts[1..];

    if args.is_empty() {
        eprintln!("No name/file pairs provided.");
        return false;
    }

    if args.len() % 2 != 0 {
        eprintln!("Mismatched arguments: you must provide name/file pairs.");
        return false;
    }

    let mut all_ok = true;

    for pair in args.chunks(2) {
        let name = pair[0];
        let filename = pair[1];

        let full_path = if Path::new(filename).is_absolute() {
            PathBuf::from(filename)
        } else {
            current_dir.join(filename)
        };

        if !full_path.exists() {
            eprintln!("{}: file not found", full_path.display().to_string().red());
            all_ok = false;
            continue;
        }

        let ok = run_in_dir(initial_dir, || {
            db.cut_add_file(name, full_path.to_string_lossy().as_ref())
        });

        if !ok {
            eprintln!("Failed to cut-add file for: {}", name);
            all_ok = false;
        }
    }

    all_ok
}

pub fn encrypt_file(db: &mut Database, parts: &[&str], initial_dir: &Path) -> bool {
    let filenames = &parts[1..];

    run_in_dir(initial_dir, || {
        let mut all_ok = true;
        for filename in filenames {
            let ok = db.encrypt_file(filename);
            if !ok {
                all_ok = false;
            }
        }
        all_ok
    })
}

pub fn decrypt_file(db: &mut Database, parts: &[&str], initial_dir: &Path) -> bool {
    let filenames = &parts[1..];

    run_in_dir(initial_dir, || {
        let mut all_ok = true;
        for filename in filenames {
            let ok = db.decrypt_file(filename);
            if !ok {
                all_ok = false;
            }
        }
        all_ok
    })
}


pub fn delete_file(db: &mut Database, parts: &[&str]) -> bool {
    let filenames = &parts[1..];

    let mut all_ok = true;
    for filename in filenames {
        let ok = db.delete_file(filename);
        if !ok {
            all_ok = false;
        }
    }
    all_ok
}

pub fn delete_all_files(db: &mut Database) -> bool {
    db.delete_all_files()
}

pub fn restore_file(db: &mut Database, parts: &[&str]) -> bool {
    let filenames = &parts[1..];

    let mut all_ok = true;
    for filename in filenames {
        let ok = db.restore_file(filename);
        if !ok {
            all_ok = false;
        }
    }
    all_ok
}

pub fn list_decrypted_files(db: &mut Database) -> bool {
    let files = db.list_decrypted_files(Some("active"));
    print_files_pretty(&files);
    true
}

pub fn list_encrypted_files(db: &mut Database) -> bool {
    let files = db.list_encrypted_files();
    print_files_pretty(&files);
    true
}

pub fn list_deleted_files(db: &mut Database) -> bool {
    let files = db.list_deleted_files();
    print_files_pretty(&files);
    true
}

pub fn search_decrypted_files(db: &mut Database, parts: &[&str]) -> bool {
    let query = parts[1];
    let files = db.search_decrypted_files(query);
    print_files_pretty(&files);
    true
}

pub fn search_encrypted_files(db: &mut Database, parts: &[&str]) -> bool {
    let query = parts[1];
    let files = db.search_encrypted_files(query);
    print_files_pretty(&files);
    true
}

pub fn search_deleted_files(db: &mut Database, parts: &[&str]) -> bool {
    let query = parts[1];
    let files = db.search_deleted_files(query);
    print_files_pretty(&files);
    true
}

pub fn list_all_files(db: &mut Database) -> bool {
    let mut files = db.list_decrypted_files(Some("active"));
    files.extend(db.list_encrypted_files());

    print_files_pretty(&files);
    true
}

pub fn search_all_files(db: &mut Database, parts: &[&str]) -> bool {
    let query = parts[1];
    let mut files = db.search_decrypted_files(query);
    files.extend(db.search_encrypted_files(query));

    print_files_pretty(&files);
    true
}

// Helper function
pub fn print_files_pretty(files: &[&FileEntry]) {
    if files.is_empty() {
        println!("{}", "No files found.".yellow());
        return;
    }

    println!(); // blank line on top

    // Get terminal width (fallback to 100 if unknown)
    let term_width = match terminal_size() {
        Some((Width(w), _)) => w as usize,
        None => 100,
    };

    // Calculate minimum required widths based on content
    let mut name_width = 25;
    let mut ext_width = 8;
    let mut size_width = 10;
    let encrypted_width = 10; // Fixed width for "Encrypted"/"Yes"/"No"
    let recycled_width = 10;  // Fixed width for "Deleted"/"Yes"/"No"
    let mut created_width = 20;
    let mut updated_width = 20;

    // Adjust widths according to content
    for f in files {
        name_width = name_width.max(f.display_name.0.len() + 2);
        ext_width = ext_width.max(f.extension.0.len() + 2);
        size_width = size_width.max(f.size.len() + 2);
        created_width = created_width.max(f.created_at.len() + 2);
        updated_width = updated_width.max(f.updated_at.len() + 2);
    }

    // Compute total width
    let total_width = name_width + ext_width + size_width + encrypted_width + recycled_width + created_width + updated_width + 6;

    // If total width exceeds terminal, scale proportionally but preserve name column
    if total_width > term_width {
        // Calculate available width for variable columns (excluding fixed ones)
        let fixed_columns_width = encrypted_width + recycled_width + 6;
        let available_width = term_width - fixed_columns_width;
        
        // Distribute available width proportionally to variable columns
        let variable_columns_total = name_width + ext_width + size_width + created_width + updated_width;
        let scale = available_width as f64 / variable_columns_total as f64;
        
        name_width = (name_width as f64 * scale).max(10.0) as usize; // Minimum 10 chars for name
        ext_width = (ext_width as f64 * scale).max(4.0) as usize;
        size_width = (size_width as f64 * scale).max(8.0) as usize;
        created_width = (created_width as f64 * scale).max(12.0) as usize;
        updated_width = (updated_width as f64 * scale).max(12.0) as usize;
    }

    // Header
    println!(
        "{:<name_width$} {:<ext_width$} {:<size_width$} {:<encrypted_width$} {:<recycled_width$} {:<created_width$} {:<updated_width$}",
        "Name".bright_yellow(),
        "Ext".bright_yellow(),
        "Size".bright_yellow(),
        "Encrypted".bright_yellow(),
        "Deleted".bright_yellow(),
        "Created".bright_yellow(),
        "Updated".bright_yellow(),
        name_width = name_width,
        ext_width = ext_width,
        size_width = size_width,
        encrypted_width = encrypted_width,
        recycled_width = recycled_width,
        created_width = created_width,
        updated_width = updated_width
    );

    println!("{}", "-".repeat(term_width.min(total_width)).blue());

    // Rows - handle long names with truncation and ellipsis
    for f in files {
        let encrypted_str = if f.is_encrypted.0 {
            "Yes".bright_green().bold()
        } else {
            "No".red()
        };
        let recycled_str = if f.is_trashed.0 {
            "Yes".red().bold()
        } else {
            "No".bright_green()
        };

        // Truncate long names with ellipsis
        let name_display = if f.display_name.0.len() > name_width {
            format!("{}...", &f.display_name.0[..name_width.saturating_sub(3)])
        } else {
            f.display_name.0.clone()
        };

        // Truncate other long fields if needed
        let ext_display = if f.extension.0.len() > ext_width {
            format!("{}...", &f.extension.0[..ext_width.saturating_sub(3)])
        } else {
            f.extension.0.clone()
        };

        let created_display = if f.created_at.len() > created_width {
            format!("{}...", &f.created_at.as_string()[..created_width.saturating_sub(3)])
        } else {
            f.created_at.as_string()
        };

        let updated_display = if f.updated_at.len() > updated_width {
            format!("{}...", &f.updated_at.as_string()[..updated_width.saturating_sub(3)])
        } else {
            f.updated_at.as_string()
        };

        println!(
            "{:<name_width$} {:<ext_width$} {:<size_width$} {:<encrypted_width$} {:<recycled_width$} {:<created_width$} {:<updated_width$}",
            name_display.bright_cyan(),
            ext_display.normal(),
            f.size.to_string(),
            encrypted_str,
            recycled_str,
            created_display.dimmed(),
            updated_display.dimmed(),
            name_width = name_width,
            ext_width = ext_width,
            size_width = size_width,
            encrypted_width = encrypted_width,
            recycled_width = recycled_width,
            created_width = created_width,
            updated_width = updated_width
        );
    }

    println!(); // blank line at bottom
}