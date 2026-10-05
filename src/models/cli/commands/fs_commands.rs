use std::{
    io::{stdout, Write},
    env,
    fs,
    process::Command
};
use terminal_size::{Width, terminal_size};
use colored::*;



pub fn cd(path: &str) -> bool {
    if let Err(e) = env::set_current_dir(path) {
        eprintln!("Error: {}", e);
        return false;
    }
    match env::current_dir() {
        Ok(_dir) => {
            true
        }
        Err(e) => {
            eprintln!("cd: No such file or directory: {}", e);
            false
        }
    }
}

pub fn lsa() {
    // Collect entries
    let mut entries = Vec::new();
    if let Ok(read_dir) = fs::read_dir(".") {
        for entry in read_dir.flatten() {
            let path = entry.path();
            let name = entry.file_name().to_string_lossy().to_string();

            // Color coding like bash
            let colored_name = if path.is_dir() {
                name.blue()
            } else if path.is_file() {
                name.green()
            } else {
                name.white()
            };
            entries.push(colored_name);
        }
    }

    print_in_columns(&entries);
}

pub fn ls() {
    // Like lsa but skip hidden files/folders
    let mut entries = Vec::new();
    if let Ok(read_dir) = fs::read_dir(".") {
        for entry in read_dir.flatten() {
            let path = entry.path();
            let name = entry.file_name().to_string_lossy().to_string();

            if name.starts_with('.') {
                continue;
            }

            let colored_name = if path.is_dir() {
                name.blue()
            } else if path.is_file() {
                name.green()
            } else {
                name.white()
            };
            entries.push(colored_name);
        }
    }

    print_in_columns(&entries);
}

pub fn pwd() -> String {
    match env::current_dir() {
        Ok(dir) => dir.display().to_string(),
        Err(e) => format!("Error getting current directory: {}", e),
    }
}

pub fn clear() {
    if cfg!(target_os = "windows") {
        // On Windows, use the `cls` command
        let _ = Command::new("cmd")
            .args(&["/C", "cls"])
            .status();
    } else {
        // On Unix/Linux/macOS, use ANSI escape sequences
        print!("\x1B[2J\x1B[3J\x1B[H");
        // \x1B[2J -> clear screen
        // \x1B[3J -> clear scrollback buffer
        // \x1B[H  -> move cursor to top-left
        stdout().flush().unwrap();
    }
}

// Helper function to print vector of strings in terminal-style columns
fn print_in_columns(entries: &[colored::ColoredString]) {
    if entries.is_empty() {
        return;
    }

    let term_width = match terminal_size() {
        Some((Width(w), _)) => w as usize,
        None => 80, // default width if unknown
    };

    let max_len = entries.iter().map(|s| s.len()).max().unwrap_or(0) + 2; // +2 for spacing
    let cols = term_width / max_len.max(1);

    for (i, entry) in entries.iter().enumerate() {
        print!("{:width$}", entry, width = max_len);
        if (i + 1) % cols == 0 {
            println!();
        }
    }
    println!();
}