mod metadata;
mod unzip;

use std::io;
use std::io::IsTerminal;
pub use metadata::{set_metadata, show_metadata};
pub use unzip::unzip_document;

fn print_success(filepath: &str, message: &str) {
    let rendered = format!("{message}: {filepath}");
    if io::stdout().is_terminal() {
        println!("\x1b[32m✓ {rendered}\x1b[0m");
    } else {
        println!("✓ {rendered}");
    }
}