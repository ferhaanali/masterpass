use std::env;
use std::path::Path;

/// Temporarily change working directory, run code, then restore original directory.
pub fn run_in_dir<F, T>(dir: &Path, f: F) -> T
where
    F: FnOnce() -> T,
{
    let old_cwd = env::current_dir().expect("Failed to get current dir");
    env::set_current_dir(dir).expect("Failed to switch directory");

    let result = f();

    // Restore previous directory
    env::set_current_dir(old_cwd).expect("Failed to restore directory");

    result
}
