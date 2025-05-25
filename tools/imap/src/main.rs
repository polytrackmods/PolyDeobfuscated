mod types;
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
    scope_id: u32,
    id: usize,
    declaration_type: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Update { code_dir, temp_dir } => {
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
            let modified_dir = code_dir.clone();
            let original_dir = temp_dir.clone();

            let files = collect_js_files(&original_dir)?;
            println!("Found {} files to compare", files.len());

            let mut all_mappings: Vec<Vec<Mapping>> = vec![];

            for file in files.iter() {
                let original_path = PathBuf::from(&original_dir).join(&file);
                let modified_path = PathBuf::from(&modified_dir).join(&file);

                let original_source = fs::read_to_string(&original_path)?;
                let modified_source = fs::read_to_string(&modified_path)?;

                let original_identifiers = extract_identifiers(&original_source)?;
                let modified_identifiers = extract_identifiers(&modified_source)?;

                let (only_in_original, only_in_modified) =
                    compare_identifiers(&original_identifiers, &modified_identifiers);

                match (only_in_original.len(), only_in_modified.len()) {
                    (0, 0) => println!("✓ No identifier differences in file: {:?}", file),
                    (orig_count, mod_count) => {
                        if orig_count != mod_count {
                            eprintln!(
                                "⚠ Identifier count mismatch in {:?}: {} original, {} modified",
                                file, orig_count, mod_count
                            );

                            std::process::exit(1);
                        }

                        let matches =
                            check_identifier_matches(&only_in_original, &only_in_modified);
                        if !matches {
                            eprintln!(
                                "⚠ Identifier mismatch in {:?}: {} original, {} modified",
                                file,
                                only_in_original.len(),
                                only_in_modified.len()
                            );

                            std::process::exit(1);
                        }

                        let mut mappings_new = vec![];
                        for (orig, modif) in only_in_original.iter().zip(only_in_modified.iter()) {
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

                        let source_map_path = PathBuf::from(&source_map_dir)
                            .join(&file)
                            .with_extension("json");

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

                        mappings.extend(mappings_new);
                        all_mappings.push(mappings);
                    }
                }
            }

            for (i, mappings) in all_mappings.iter().enumerate() {
                let source_map_path = PathBuf::from(&source_map_dir).join(files[i].clone()).with_extension("json");

                let json_data = serde_json::to_string_pretty(&mappings)?;
                fs::write(&source_map_path, json_data)?;
                println!("Source map updated: {:?}", source_map_path);
            }

            update_temp_dir(&code_dir, &temp_dir)?;
            println!(
                "Files copied successfully from {} to {}",
                code_dir, temp_dir
            );
        }
    }

    Ok(())
}
