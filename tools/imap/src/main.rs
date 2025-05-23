mod utils;
mod visitor;

use std::{fs, path::PathBuf};

use clap::{Parser as ClapParser, Subcommand};
use serde::{Deserialize, Serialize};

use crate::utils::*;

#[derive(ClapParser)]
#[command(name = "IMap")]
#[command(about = "A tool to create source maps from deobfuscated JavaScript/TypeScript code.")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Copies files from code_dir to temp_dir
    Update {
        /// Directory containing the code to be copied
        #[arg(short, long, default_value = "PolyTrack")]
        code_dir: String,
        /// Directory to copy files to
        #[arg(short, long, default_value = "temp")]
        temp_dir: String,
    },
    /// Compares identifiers between two directories and creates the source map
    Create {
        /// Directory containing the modified code
        #[arg(short, long, default_value = "PolyTrack")]
        code_dir: String,
        /// Directory containing the original code
        #[arg(short, long, default_value = "temp")]
        temp_dir: String,
        /// Directory in which to create the source map
        #[arg(short, long, default_value = "source_maps")]
        source_map_dir: String,
    },
}

#[derive(Serialize, Deserialize)]
struct Mapping {
    original: String,
    modified: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Update { code_dir, temp_dir } => {
            // Copy files from code_dir to temp_dir
            update_temp_dir(&code_dir, &temp_dir)?;
            println!(
                "Files copied successfully from {} to {}",
                code_dir, temp_dir
            );
        }
        Commands::Create {
            code_dir,
            temp_dir,
            source_map_dir,
        } => {
            // Compare identifiers between code_dir and temp_dir
            let modified_dir = code_dir.clone();
            let original_dir = temp_dir.clone();

            let files = collect_js_files(&original_dir)?;
            println!("Found {} files to compare", files.len());

            for file in files {
                let original_path = PathBuf::from(&original_dir).join(&file);
                let modified_path = PathBuf::from(&modified_dir).join(&file);

                // Read source files
                let original_source = fs::read_to_string(&original_path)?;
                let modified_source = fs::read_to_string(&modified_path)?;

                // Extract identifiers
                let original_identifiers = extract_identifiers(&original_source)?;
                let modified_identifiers = extract_identifiers(&modified_source)?;

                // Compare identifiers
                let (only_in_original, only_in_modified) =
                    compare_identifiers(&original_identifiers, &modified_identifiers);

                match (only_in_original.len(), only_in_modified.len()) {
                    (0, 0) => println!("✓ No identifier differences in file: {:?}", file),
                    (1, 1) => {
                        println!(
                            "→ Single identifier change in {:?}: '{}' → '{}'",
                            file, only_in_original[0], only_in_modified[0]
                        );

                        // Create source map entry
                        let mapping = Mapping {
                            original: only_in_original[0].clone(),
                            modified: only_in_modified[0].clone(),
                        };

                        let source_map_path = PathBuf::from(&source_map_dir)
                            .join(&file)
                            .with_extension("json");

                        // If the path exists, load the existing mappings
                        let mut mappings: Vec<Mapping> = if source_map_path.exists() {
                            println!("Loading existing source map file: {:?}", source_map_path);
                            let data = fs::read_to_string(&source_map_path)?;
                            serde_json::from_str(&data)?
                        } else {
                            println!("Creating new source map file: {:?}", source_map_path);
                            fs::create_dir_all(source_map_path.parent().unwrap())?;
                            fs::File::create(&source_map_path)?;
                            vec![]
                        };

                        // Add the new mapping
                        mappings.push(mapping);

                        // Write the updated mappings to the file
                        let json_data = serde_json::to_string_pretty(&mappings)?;
                        fs::write(&source_map_path, json_data)?;
                        println!("Source map updated: {:?}", source_map_path);

                        // Update temp_dir
                        update_temp_dir(&code_dir, &temp_dir)?;
                        println!(
                            "Files copied successfully from {} to {}",
                            code_dir, temp_dir
                        );
                    }
                    (orig_count, mod_count) => {
                        eprintln!(
                            "⚠ Multiple identifier differences in {:?}: {} removed, {} added",
                            file, orig_count, mod_count
                        );
                    }
                }
            }
        }
    }

    Ok(())
}
