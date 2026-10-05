pub mod models;
pub use crate::{
    models::{
        cli::start,
        database::{
            operations::{
                DatabaseArguments,
            },
            response::AppError,
        },
    },
};

fn main() {
    let path: &str = ".cache/local";
    
    let mut database_args: DatabaseArguments = match DatabaseArguments::collect_login_arguments(&path) {
        Ok(args) => args,
        Err(e) => {
            if e.code == 401 {
                println!("{}", e);
                return;
            }

            match DatabaseArguments::collect_signup_arguments(&path) {
                Ok(args) => args,
                Err(e) => { 
                    println!("{}", e);
                    return;
                },
            }
        },
    };

    start(&mut database_args);
}