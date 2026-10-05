use super::{
    database::{
        Database,
        operations::{
            DatabaseArguments,
        },
    },
    help_document::{help_document, unknown_command_message},
    commands::{fs_commands, pass_commands, file_commands, auth_commands, nested_db},
    validate_args::{validate_args,print_usage},
};
use colored::*;
use std::{
    path::{PathBuf},
    env::{current_dir},
    io::{self, Write},
};

pub fn start(args: &mut DatabaseArguments) {
    // Clear Screen
    fs_commands::clear();

    let mut db: Database = match Database::with_dir(args) {
        Ok(d) => d,
        Err(e) => {
            println!("Failed to initialize database. Exiting... \n {}", e);
            return;
        }
    };

    let mut current_db_location = db.config.name.clone();
    
    let mut current_directory: PathBuf = current_dir().unwrap();
    let initial_dir = current_directory.clone();
    println!("DataSeal CLI ready. Type 'help' for commands.");

    loop {
        print!(
            "{}: {}{}",
            current_db_location.bright_green(),
            current_directory.display().to_string().blue(),
            ">".bright_green()
        );
        io::stdout().flush().unwrap();

        let mut input = String::new();
        if io::stdin().read_line(&mut input).is_err() {
            continue;
        }

        let input = input.trim();
        if input.is_empty() {
            continue;
        }
        let commands: Vec<&str> = input.split("&&").map(|s| s.trim()).collect();

        for cmd in commands {
            if cmd.is_empty() {
                continue;
            }

            let parts: Vec<&str> = cmd.split_whitespace().collect();

            match parts[0] {
                // Auth 
                "change-root-password" | "chgrootpass" | "crp" => {
                    if validate_args(&["change-root-password","chgrootpass","crp"], &parts, 0) {
                        if auth_commands::change_root_password(&mut db) {
                            fs_commands::clear();
                            // println!("✅ Master password changed.");
                        }
                    }
                },
                // Recycle Bin
                "empty-recycle-bin" | "emp-rec-bin" | "erb" => {
                    if validate_args(&["empty-recycle-bin","emp-rec-bin","erb"], &parts, 0) {
                        file_commands::empty_recycle_bin_verbose(&mut db);
                    }
                },
                // Password Commands
                "decrypt-all-passwords" | "decallpass" | "decap" => {
                    if validate_args(&["decrypt-all-passwords","decallpass","decap"], &parts, 0) {
                        pass_commands::decrypt_all_passwords(&mut db);
                    }
                },
                "encrypt-all-passwords" | "encallpass" | "encap" => {
                    if validate_args(&["encrypt-all-passwords","encallpass","encap"], &parts, 0) {
                        pass_commands::encrypt_all_passwords(&mut db);
                    }
                },
                "restore-all-passwords" | "resallpass" | "rap" => {
                    if validate_args(&["restore-all-passwords","resallpass","rap"], &parts, 0) {
                        pass_commands::restore_all_passwords(&mut db);
                    }
                }
                "search-deleted-passwords" | "searchdelpass" | "sdp" => {
                    if validate_args(&["search-deleted-passwords","searchdelpass","sdp"], &parts, 1) {
                        pass_commands::search_deleted_passwords(&mut db, &parts);
                    }
                },
                "search-decrypted-passwords" | "searchdecpass" | "sdecp" => {
                    if validate_args(&["search-decrypted-passwords","searchdecpass","sdecp"], &parts, 1) {
                        pass_commands::search_decrypted_passwords(&mut db, &parts);
                    }
                },
                "search-passwords" | "searchpass" | "sp" => {
                    if validate_args(&["search-passwords","searchpass","sp"], &parts, 1) {
                        pass_commands::search_all_passwords(&mut db, &parts);
                    }
                },
                "search-encrypted-passwords" | "searchencpass" | "sencp" => {
                    if validate_args(&["search-encrypted-passwords","searchencpass","sencp"], &parts, 1) {
                        pass_commands::search_encrypted_passwords(&mut db, &parts);
                    }
                },
                "list-deleted-passwords" | "lsdelpass" | "ldp" => {
                    if validate_args(&["list-deleted-passwords","listdelpass","ldp"], &parts, 0) {
                        pass_commands::list_deleted_passwords(&mut db);
                    }
                },
                "list-encrypted-passwords" | "lsencpass" | "lencp" => {
                    if validate_args(&["list-encrypted-passwords","listencpass","lencp"], &parts, 0) {
                        pass_commands::list_encrypted_passwords(&mut db);
                    }
                },
                "list-decrypted-passwords" | "lsdecpass" | "ldecp" => {
                    if validate_args(&["list-decrypted-passwords","list-passwords","lspass","lp","ldecp", "lsdecpass"], &parts, 0) {
                        pass_commands::list_decrypted_passwords(&mut db);
                    }
                },
                "list-passwords" | "lspass" | "lp"  => {
                    if validate_args(&["list-decrypted-passwords","list-passwords","lspass","lp","ldecp", "lsdecpass"], &parts, 0) {
                        pass_commands::list_all_passwords(&mut db);
                    }
                },
                "restore-password" | "respass" | "rp" => {
                    if parts.len() - 1 == 0 {
                        print_usage(&["restore-password <name> <name> ...", "respass <name> <name> ...", "rp <name> <name> ..."]);
                        return;
                    }
                    pass_commands::restore_password(&mut db, &parts);
                },
                "delete-all-passwords" | "delallpass" | "dap" => {
                    if validate_args(&["delete-all-passwords", "delallpass", "dap"], &parts, 0) {
                        pass_commands::delete_all_passwords(&mut db);
                    }
                },
                "delete-password" | "delpass" | "dp" => {
                    if parts.len() - 1 == 0 {
                        print_usage(&["delete-password <name> <name> ...", "delpass <name> <name> ...", "dp <name> <name> ..."]);
                        return
                    }
                    pass_commands::delete_password(&mut db, &parts);
                },
                "decrypt-password" | "decpass" | "decp" => {
                    if parts.len() - 1 == 0 {
                        print_usage(&["decrypt-password <name> <name> ...", "decpass <name> <name> ...", "decp <name> <name> ..."]);
                        return;
                    }
                    pass_commands::decrypt_password(&mut db, &parts);
                },
                "encrypt-password" | "encpass" | "encp"=> {
                    if parts.len() - 1 == 0 {
                        print_usage(&["encrypt-password <name> <name> ...", "encpass <name> <name> ...", "encp <name> <name> ..."])
                    }
                    pass_commands::encrypt_password(&mut db, &parts);
                },
                "change-password" | "chgpass" | "cp" => {
                    if parts.len() == 0 || parts.len() == 1 {
                        print_usage(&["change-password <name> <newpassword> <name> <newpass> ...", "chgpass <name> <newpassword> <name> <newpass> ...",
                        "cp <name> <newpassword> <name> <newpass> ..."])
                    }
                    pass_commands::change_password(&mut db, &parts);
                },
                "add-password" | "addpass" | "ap" => {
                    if parts.len() - 1 == 0 || parts.len() - 1 == 1 {
                        print_usage(&["add-password <name> <password> <name> <password> ...", "addpass <name> <password> <name> <password> ...",
                        "ap <name> <password> <name> <password> ..."])
                    } 
                    pass_commands::add_password(&mut db, &parts);
                },
                // Default Commands
                "help" => {
                    if validate_args(&["help"], &parts, 0) {
                        println!("{}", help_document());
                    }
                },
                "clear" => {
                    if validate_args(&["clear"], &parts, 0) {
                        fs_commands::clear();
                    }
                },
                "version" | "--version" | "-v" => {
                    println!("Dataseal {}", env!("CARGO_PKG_VERSION"));
                }
                "exit" | "quit" => return,
                _ => println!("{}", unknown_command_message(cmd)),
            }       
        }
    }
}



pub fn push_path(path: &mut String, name: &str) {
    if let Some(pos) = path.find('/') {
        // Already has two segments -> replace second
        if path[pos + 1..].contains('/') {
            // truncate after first segment
            if let Some(first_slash) = path.find('/') {
                path.truncate(first_slash);
            }
        }
        // truncate everything after first slash
        if let Some(first_slash) = path.find('/') {
            path.truncate(first_slash);
        }
        path.push('/');
        path.push_str(name);
    } else {
        // Only root exists -> just append
        path.push('/');
        path.push_str(name);
    }
}
pub fn pop_path(path: &mut String) {
    if let Some(pos) = path.find('/') {
        // keep only root part
        path.truncate(pos);
    }
}