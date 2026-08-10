pub mod metadata;
pub mod pdf;
pub mod policy;
pub mod replace;
pub mod unzip;

use std::io;
use std::io::IsTerminal;

fn print_success(message: impl std::fmt::Display) {
    if io::stdout().is_terminal() {
        println!("\x1b[32m✓ {message}\x1b[0m");
    } else {
        println!("✓ {message}");
    }
}

fn print_warning(message: impl std::fmt::Display) {
    if io::stdout().is_terminal() {
        println!("\x1b[33m⚠ {message}\x1b[0m");
    } else {
        println!("⚠ {message}");
    }
}
