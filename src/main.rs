use std::fs::File;
use std::io;
use clap::{Parser, Subcommand};
use zip::ZipArchive;

/// Doc comment
#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Inspect {

        filepath: String,
    }
}

fn main() {
    let args = Cli::parse();
    match args.command {
        Commands::Inspect { filepath } => {
            println!("Test command with file: {}", filepath);
            inspect_docx(&filepath).unwrap();
        }
    }
    println!();
}

fn inspect_docx(filepath: &str) ->io::Result<()> {
    let file = File::open(filepath)?;
    let mut archive = ZipArchive::new(file)?;

    for i in 0..archive.len() {
        let file = archive.by_index(i)?;
        println!("File: {}", file.name());
    }
    Ok(())
}