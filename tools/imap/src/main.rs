mod types;
mod utils;
mod visitor;

use std::{fs, path::PathBuf};

use clap::{Parser as ClapParser, Subcommand};

use crate::types::*;
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
    /// Compares identifiers between two directories and creates the source map
    Create {
        /// Directory containing the modified code
        #[arg(short, long, default_value = "PolyTrack")]
        code_dir: String,
        /// Directory containing the original code
        #[arg(short, long, default_value = "original")]
        original_dir: String,
        /// Directory in which to create the source map
        #[arg(short, long, default_value = "source_maps")]
        source_map_dir: String,
    },
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Create {
            code_dir,
            original_dir,
            source_map_dir,
        } => {
            fs::create_dir_all(&source_map_dir).unwrap_or_else(|_| {
                eprintln!("Error creating source map directory: {:?}", source_map_dir);
                std::process::exit(1);
            });

            let modified_dir = code_dir.clone();
            let original_dir = original_dir.clone();

            let files = collect_js_files(&original_dir).unwrap_or_else(|_| {
                eprintln!("Error collecting files from directory: {:?}", original_dir);
                std::process::exit(1);
            });
            println!("Found {} files to compare", files.len());

            for file in files.iter() {
                let original_path = PathBuf::from(&original_dir).join(file);
                let modified_path = PathBuf::from(&modified_dir).join(file);

                let original_source = fs::read_to_string(&original_path).unwrap_or_else(|_| {
                    eprintln!("Error reading original file: {:?}", original_path);
                    std::process::exit(1);
                });
                let modified_source = fs::read_to_string(&modified_path).unwrap_or_else(|_| {
                    eprintln!("Error reading modified file: {:?}", modified_path);
                    std::process::exit(1);
                });

                let original_identifiers =
                    extract_identifiers(&original_source).unwrap_or_else(|_| {
                        eprintln!(
                            "Error extracting identifiers from original file: {:?}",
                            original_path
                        );
                        std::process::exit(1);
                    });
                let modified_identifiers =
                    extract_identifiers(&modified_source).unwrap_or_else(|_| {
                        eprintln!(
                            "Error extracting identifiers from modified file: {:?}",
                            modified_path
                        );
                        std::process::exit(1);
                    });

                let (only_in_original, only_in_modified) =
                    compare_identifiers(&original_identifiers, &modified_identifiers);

                let orig_count = only_in_original.len();
                let mod_count = only_in_modified.len();

                if orig_count != mod_count {
                    eprintln!(
                        "⚠ Identifier count mismatch in {:?}: {} original, {} modified",
                        file, orig_count, mod_count
                    );

                    std::process::exit(1);
                }

                let matches = check_identifier_matches(&only_in_original, &only_in_modified);
                if !matches {
                    eprintln!(
                        "⚠ Identifier mismatch in {:?}: {} original, {} modified",
                        file,
                        only_in_original.len(),
                        only_in_modified.len()
                    );

                    std::process::exit(1);
                }

                let source_map_path = PathBuf::from(&source_map_dir)
                    .join(file)
                    .with_extension("json");

                let mut mappings: Vec<Mapping> = if source_map_path.exists() {
                    println!("Loading existing source map file: {:?}", source_map_path);
                    let data = fs::read_to_string(&source_map_path).unwrap_or_else(|_| {
                        eprintln!("Error reading source map file: {:?}", source_map_path);
                        std::process::exit(1);
                    });
                    serde_json::from_str(&data).unwrap_or_else(|_| {
                        eprintln!("Error parsing source map file: {:?}", source_map_path);
                        std::process::exit(1);
                    })
                } else {
                    println!("Creating new source map file: {:?}", source_map_path);
                    fs::File::create(&source_map_path).unwrap_or_else(|_| {
                        eprintln!("Error creating source map file: {:?}", source_map_path);
                        std::process::exit(1);
                    });
                    vec![]
                };

                let mut mappings_new = vec![];
                for (orig, modif) in only_in_original.iter().zip(only_in_modified.iter()) {
                    if mappings.iter().any(|m| m.id == orig.2 && m.id == modif.2) {
                        continue;
                    }

                    println!(
                        "→ Identifier change in {:?}: '{}' → '{}'",
                        file, orig.0, modif.0
                    );

                    mappings_new.push(Mapping {
                        original: orig.0.clone(),
                        modified: modif.0.clone(),
                        scope_id: orig.1,
                        id: orig.2,
                        declaration_type: orig.3.clone(),
                    });
                }

                mappings.extend(mappings_new);

                let json_data = serde_json::to_string_pretty(&mappings).unwrap_or_else(|_| {
                    eprintln!(
                        "Error serializing mappings to JSON for file: {:?}",
                        source_map_path
                    );
                    std::process::exit(1);
                });
                fs::write(&source_map_path, json_data).unwrap_or_else(|_| {
                    eprintln!("Error writing source map file: {:?}", source_map_path);
                    std::process::exit(1);
                });
                println!("Source map updated: {:?}", source_map_path);
            }
        }
    }

    Ok(())
}
