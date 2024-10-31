mod extractor;

use clap::{Parser, Subcommand};
use std::env;
use std::fs;
use std::path::PathBuf;
use thiserror::Error;
use extractor::extract_uiua_definitions;

#[derive(Error, Debug)]
enum AppError {
    #[error("Directory does not exist: {0}")]
    DirectoryNotFound(PathBuf),
    
    #[error("Not a directory: {0}")]
    NotADirectory(PathBuf),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Permission denied: {0}")]
    PermissionDenied(PathBuf),
}

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
    
    #[arg(short, long)]
    dir: Option<PathBuf>,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate static site
    Build,
    /// Run development server
    Serve,
}

fn validate_directory(dir: Option<PathBuf>) -> Result<PathBuf, AppError> {
    let working_dir = if let Some(dir) = dir {
        if dir.is_absolute() {
            dir
        } else {
            let current_dir: PathBuf = env::current_dir()?;
            current_dir.join(dir)
        }
    } else {
        env::current_dir()?
    };

    // Check if path exists
    if !working_dir.exists() {
        return Err(AppError::DirectoryNotFound(working_dir));
    }

    // Check if it's a directory
    if !working_dir.is_dir() {
        return Err(AppError::NotADirectory(working_dir));
    }

    // Check if we have write permissions by attempting to create and remove a test file
    let test_file = working_dir.join(".write_test");
    match fs::write(&test_file, "") {
        Ok(_) => {
            let _ = fs::remove_file(test_file);
        }
        Err(_) => {
            return Err(AppError::PermissionDenied(working_dir));
        }
    }

    Ok(working_dir)
}

fn main() {
    let cli = Cli::parse();
    
    let working_dir = match validate_directory(cli.dir) {
        Ok(dir) => dir,
        Err(err) => {
            eprintln!("Error: {}", err);
            std::process::exit(1);
        }
    };

    match cli.command {
        Commands::Build => {
            let output = match extract_uiua_definitions(&working_dir) {
                Ok(output) => output,
                Err(err) => {
                    eprintln!("Error: {}", err);
                    std::process::exit(1);
                }
            };

            println!("TODO: Generate site in {:?}", working_dir);
            println!("{:#?}", output);
        }
        Commands::Serve => {
            println!("TODO: Serve site");
        }
    }
}