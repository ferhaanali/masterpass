pub fn help_document() -> String {
    r#"
==========================
       DataSeal CLI       
==========================

A secure command-line vault for managing encrypted files and credentials.

USAGE:
    dataseal [OPTIONS]

OPTIONS:
    -h, --help        Show this help message
    -c, --cli         Launch CLI interface (default)
    -g, --gui         Launch GUI interface
    -v, --version     Show version of dataseal

SYSTEM COMMANDS:
    help                      Show this help document
    exit, quit                Exit the CLI
    clear                     Clear the screen

PATH COMMANDS:
    pwd                       Print current working directory
    ls                        List files and folders in current directory
    lsa                        List all files and folders in current directory
    cd <dir>                  Change current directory

NESTED DATABASE COMMANDS:
    list-nested-databases     listnesteddbs, lndb
    create-nested-database    createnesteddb, cndb
    delete-nested-database    delnesteddb, dndb
    logout-nested-database    logoutnesteddb, logondb
    login-nested-database     loginnesteddb, logndb

DATABASE COMMANDS:
    database-info             di

FILE COMMANDS:
    cut-add-file <name> <filename>      cutaddfile,caf
    add-file <name> <filename>          addfile, af
    list-files                          lsfiles, lf
    list-encrypted-files                lsencfiles, lencf
    list-decrypted-files                lsdecfiles, ldecf
    list-deleted-files                  lsdelfiles, ldf
    encrypt-file <filename>             encfile, encf
    decrypt-file <filename>             decfile, decf
    delete-file <name>                  delfile, df
    delete-all-files                    delallfiles, daf
    restore-file <name>                 resfile, rf
    restore-all-files                   resallfiles, raf
    search-files <query>                sf
    search-encrypted-files <query>      searchencfiles, sencf
    search-decrypted-files <query>      searchdecfiles, sdecf
    search-deleted-files <query>        searchdelfiles, sdf

PASSWORD COMMANDS:
    add-password <name> <password>      addpass, ap
    list-passwords                       lspass, lp
    list-encrypted-passwords             lsencpass, lencp
    list-decrypted-passwords             lsdecpass, ldecp
    list-deleted-passwords               lsdelpass, ldp
    encrypt-password <name>              encpass, encp
    decrypt-password <name>              decpass, decp
    delete-password <name>               delpass, dp
    delete-all-passwords                 delallpass, dap
    restore-password <name>              respass, rp
    restore-all-passwords                resallpass, rap
    change-password <name> <newpassword> chgpass, cp
    change-root-password <old> <new>     chgrootpass, crp

AUTH COMMANDS:
    change-root-password <old password> <new password>     chgrootpass, crp


NOTES:
    Use commands exactly as shown.
    Aliases can be used in place of full command names.
"#.to_string()
}





pub fn unknown_command_message(cmd: &str) -> String {
    format!(
        "Unknown command: {}\n\tTry 'help' for a list of available commands.",
        cmd
    )
}