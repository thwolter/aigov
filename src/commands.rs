mod metadata;
mod unzip;
mod pdf;

use std::io;
use std::io::IsTerminal;
pub use metadata::{set_metadata, show_metadata};
pub use unzip::unzip_document;
pub use pdf::convert_to_pdf;

fn print_success(message: impl std::fmt::Display) {
    if io::stdout().is_terminal() {
        println!("\x1b[32m✓ {message}\x1b[0m");
    } else {
        println!("✓ {message}");
    }
}