// TODO: unit tests
// TODO: clean up code

mod types;
mod utils;
mod visitors;

use std::{fs, path};

use clap::{Parser, Subcommand};
use yansi::Paint;

use crate::{types::*, utils::*};

#[derive(Parser)]
#[command(name = "imap", version = "1.0", author = "PolyTrackMods Team", about = "A tool to create source maps from deobfuscated JavaScript code.", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Creates a source map from differences between identifiers in two JS files.
    Create {
        /// The name of the source map.
        name: String,
        /// Directory containing the modified JS files.
        #[arg(short, long, default_value = "polytrack-deobfuscated")]
        modified: String,
        /// Directory containing the original JS files.
        #[arg(short, long, default_value = "polytrack")]
        original: String,
        /// Directory to save the source map.
        #[arg(short, long, default_value = "polytrack-sourcemaps")]
        sourcemap: String,
    },
    /// Checks that no source maps conflict with each other.
    Check {
        /// Directory containing the names of the code files (either obfuscated or deobfuscated).
        #[arg(short, long, default_value = "polytrack-deobfuscated")]
        files: String,
        /// Directory containing the source maps to check.
        #[arg(short, long, default_value = "polytrack-sourcemaps")]
        sourcemap: String,
    },
    /// Generates the modified JS files from the source maps.
    Generate {
        /// Directory to save the modified JS files.
        #[arg(short, long, default_value = "polytrack-generated")]
        modified: String,
        /// Directory containing the original JS files.
        #[arg(short, long, default_value = "polytrack")]
        original: String,
        /// Directory containing the source maps to generate from.
        #[arg(short, long, default_value = "polytrack-sourcemaps")]
        sourcemap: String,
    },
    // TODO: generate docs command
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Create {
            name,
            modified,
            original,
            sourcemap,
        } => {
            let source_map_dir = path::Path::new(&sourcemap).join(&name);

            if source_map_dir.exists() {
                error(format!(
                    "{} {} {}",
                    "The source map directory".red(),
                    source_map_dir.display().blue(),
                    "already exists. Please choose a different name or remove the existing directory (especially if your last run produced an error).".red()
                ));
            }

            fs::create_dir_all(&source_map_dir).unwrap_or_else(|_| {
                error(format!(
                    "{} {}.",
                    "Failed to create directory:".red(),
                    source_map_dir.display().blue()
                ));
            });

            let modified_files = get_js_files(&modified);
            let original_files = get_js_files(&original);

            if modified_files.len() != original_files.len() {
                error(format!(
                    "{} {} ≠ {}. {}",
                    "The number of modified files does not match the number of original files."
                        .red(),
                    modified_files.len().yellow(),
                    original_files.len().yellow(),
                    "Please ensure both directories contain the same number of JS files.".red()
                ));
            }

            println!(
                "{} {} {}",
                "Found".blue(),
                modified_files.len().yellow(),
                "JS files to compare.".blue()
            );

            for (i, modified_file) in modified_files.iter().enumerate() {
                let modified_file_path = path::Path::new(modified_file);

                let original_file = &original_files[i];
                let original_file_path = path::Path::new(original_file);

                if modified_file_path.file_name() != original_file_path.file_name() {
                    error(format!(
                        "{} {} {} {}.",
                        "The modified file".red(),
                        modified_file_path.display().blue(),
                        "does not match the original file".red(),
                        original_file_path.display().blue()
                    ));
                }

                let modified_content =
                    fs::read_to_string(modified_file_path).unwrap_or_else(|_| {
                        error(format!(
                            "{} {}. {}",
                            "Failed to read modified file:".red(),
                            modified_file_path.display().blue(),
                            "Please check the file path and permissions.".red()
                        ));
                    });
                let original_content =
                    fs::read_to_string(original_file_path).unwrap_or_else(|_| {
                        error(format!(
                            "{} {}. {}",
                            "Failed to read original file:".red(),
                            original_file_path.display().blue(),
                            "Please check the file path and permissions.".red()
                        ));
                    });

                let modified_identifiers = get_identifiers(&modified_content);
                let mut original_identifiers = get_identifiers(&original_content);

                let source_map_file_path = source_map_dir.join(format!(
                    "{}.map",
                    modified_file_path.file_stem().unwrap().to_str().unwrap()
                ));
                let source_maps = get_map_files(
                    &sourcemap,
                    modified_file_path.file_stem().unwrap().to_str().unwrap(),
                );

                let mut mappings: Vec<(String, Mapping)> = Vec::new();
                for source_map in source_maps {
                    let data = fs::read_to_string(&source_map).unwrap_or_else(|_| {
                        error(format!(
                            "{} {}. {}",
                            "Failed to read source map file:".red(),
                            source_map.blue(),
                            "Please check the file path and permissions.".red()
                        ));
                    });
                    let map: Vec<Mapping> = serde_json::from_str(&data).unwrap_or_else(|_| {
                        error(format!(
                            "{} {}. {}",
                            "Failed to parse source map file:".red(),
                            source_map.blue(),
                            "Please ensure the file is a valid JSON source map.".red()
                        ));
                    });

                    for mapping in map {
                        // find if the identifier already exists in the original identifiers and if it does, rename it
                        if let Some(original_identifier) = original_identifiers
                            .iter_mut()
                            .find(|id| id.scope_id == mapping.original.scope_id)
                        {
                            original_identifier.name = mapping.modified.name.clone();
                        }
                        mappings.push((source_map.clone(), mapping));
                    }
                }

                let (only_in_modified, only_in_original) =
                    compare_identifiers(&modified_identifiers, &original_identifiers);

                check_identifiers(&only_in_modified, &only_in_original);

                let mut new_mappings: Vec<Mapping> = Vec::new();
                for (original, modified) in only_in_original.iter().zip(only_in_modified.iter()) {
                    // basically we are finding mappings that conflict with one another
                    if let Some(conflict_mapping) = mappings.iter().find(|m| {
                        m.1.original.scope_id == original.scope_id
                            && m.1.modified.name != modified.name
                    }) {
                        error(format!(
                            "{} {} {} {}: '{}:{}' ≠ '{}:{}'. {}",
                            "Conflict found in".red(),
                            modified_file_path.display().blue(),
                            "with the existing source map".red(),
                            conflict_mapping.0.blue(),
                            modified.name.yellow(),
                            modified.scope_id.yellow(),
                            conflict_mapping.1.modified.name.yellow(),
                            conflict_mapping.1.modified.scope_id.yellow(),
                            "Please resolve the conflict before proceeding.".red()
                        ));
                    }

                    println!(
                        "{} {}: '{}:{}' → '{}:{}'.",
                        "Identifier change in".green(),
                        modified_file_path.display().blue(),
                        original.name.yellow(),
                        original.scope_id.yellow(),
                        modified.name.yellow(),
                        modified.scope_id.yellow()
                    );

                    new_mappings.push(Mapping {
                        original: (*original).clone(),
                        modified: (*modified).clone(),
                    });
                }

                if !new_mappings.is_empty() {
                    let json_data = serde_json::to_string(&new_mappings).unwrap_or_else(|_| {
                        error(format!(
                            "{} {}. {}",
                            "Failed to serialize mappings to JSON:".red(),
                            source_map_file_path.display().blue(),
                            "Please check the mappings data.".red()
                        ));
                    });

                    fs::write(&source_map_file_path, json_data).unwrap_or_else(|_| {
                        error(format!(
                            "{} {}. {}",
                            "Failed to write source map file:".red(),
                            source_map_file_path.display().blue(),
                            "Please check the file path and permissions.".red()
                        ));
                    });

                    println!(
                        "{} {}",
                        "✓ Source map updated:".green(),
                        source_map_file_path.display().blue()
                    )
                } else {
                    println!(
                        "{} {}",
                        "No changes found for".yellow(),
                        modified_file_path.display().blue()
                    );
                }
            }
        }
        Commands::Check { files, sourcemap } => {
            let source_map_dir = path::Path::new(&sourcemap);

            if !source_map_dir.exists() {
                error(format!(
                    "{} {} {}",
                    "The source map directory".red(),
                    source_map_dir.display().blue(),
                    "does not exist. Please create it first.".red()
                ));
            }

            let js_files = get_js_files(&files);

            for file in js_files {
                let file_path = path::Path::new(&file);
                let source_map_files =
                    get_map_files(&sourcemap, file_path.file_stem().unwrap().to_str().unwrap());

                if source_map_files.is_empty() {
                    error(format!(
                        "{} {} {}",
                        "No source maps found for".red(),
                        file_path.display().blue(),
                        "Please ensure the source maps are correctly generated.".red()
                    ));
                }

                let mut mappings: Vec<(String, Mapping)> = Vec::new();
                for source_map_file in source_map_files {
                    let data = fs::read_to_string(&source_map_file).unwrap_or_else(|_| {
                        error(format!(
                            "{} {}. {}",
                            "Failed to read source map file:".red(),
                            source_map_file.blue(),
                            "Please check the file path and permissions.".red()
                        ));
                    });
                    let map: Vec<Mapping> = serde_json::from_str(&data).unwrap_or_else(|_| {
                        error(format!(
                            "{} {}. {}",
                            "Failed to parse source map file:".red(),
                            source_map_file.blue(),
                            "Please ensure the file is a valid JSON source map.".red()
                        ));
                    });

                    for mapping in map {
                        mappings.push((source_map_file.clone(), mapping));
                    }
                }

                let mut compared = Vec::new(); // to reduce duplicate warnings
                for mapping in mappings.iter() {
                    let mut found_mapping = None;
                    if mappings.iter().filter(|m| m.0 != mapping.0).any(|m| {
                        found_mapping = Some(m);
                        m.1.original.scope_id == mapping.1.original.scope_id
                            && m.1.modified.name != mapping.1.modified.name
                    }) {
                        let found_mapping = found_mapping.unwrap();
                        error(format!(
                            "{} {} {} {}: '{}:{}' ≠ '{}:{}'. {}",
                            "Conflict found in".red(),
                            found_mapping.0.blue(),
                            "with the source map".red(),
                            mapping.0.blue(),
                            mapping.1.modified.name.yellow(),
                            mapping.1.modified.scope_id.yellow(),
                            mapping.1.original.name.yellow(),
                            mapping.1.original.scope_id.yellow(),
                            "Please resolve the conflict before proceeding.".red()
                        ));
                    }
                    let mut found_mapping = None;
                    if mappings.iter().filter(|m| m.0 != mapping.0).any(|m| {
                        found_mapping = Some(m);
                        m.1.original.scope_id == mapping.1.original.scope_id
                    }) {
                        if compared.contains(&mapping.1.modified.scope_id) {
                            continue; // skip if already warned about this mapping
                        }
                        compared.push(mapping.1.modified.scope_id);

                        let found_mapping = found_mapping.unwrap();
                        println!(
                            "{} {} {} {} {} {}:{}.",
                            "WARNING: Source maps".yellow(),
                            found_mapping.0.blue(),
                            "and".yellow(),
                            mapping.0.blue(),
                            "contain an identical identifer mapping for".yellow(),
                            mapping.1.modified.name.yellow(),
                            mapping.1.modified.scope_id.yellow()
                        );
                    }
                }
            }

            println!("{}", "✓ All source maps checked successfully.".green());
        }
        Commands::Generate {
            modified,
            original,
            sourcemap,
        } => {
            let source_map_dir = path::Path::new(&sourcemap);

            if !source_map_dir.exists() {
                error(format!(
                    "{} {} {}",
                    "The source map directory".red(),
                    source_map_dir.display().blue(),
                    "does not exist. Please create it first.".red()
                ));
            }

            let original_files = get_js_files(&original);

            println!(
                "{} {} {}",
                "Found".blue(),
                original_files.len().yellow(),
                "JS files to generate.".blue()
            );

            for original_file in original_files {
                let original_file_path = path::Path::new(&original_file);
                let original_content =
                    fs::read_to_string(original_file_path).unwrap_or_else(|_| {
                        error(format!(
                            "{} {}. {}",
                            "Failed to read original file:".red(),
                            original_file_path.display().blue(),
                            "Please check the file path and permissions.".red()
                        ));
                    });
                let source_map_files = get_map_files(
                    &sourcemap,
                    original_file_path.file_stem().unwrap().to_str().unwrap(),
                );

                if source_map_files.is_empty() {
                    error(format!(
                        "{} {} {}",
                        "No source maps found for".red(),
                        original_file_path.display().blue(),
                        "Please ensure the source maps are correctly generated.".red()
                    ));
                }

                let mut mappings: Vec<Mapping> = Vec::new();
                for source_map_file in source_map_files {
                    let data = fs::read_to_string(&source_map_file).unwrap_or_else(|_| {
                        error(format!(
                            "{} {}. {}",
                            "Failed to read source map file:".red(),
                            source_map_file.blue(),
                            "Please check the file path and permissions.".red()
                        ));
                    });
                    let map: Vec<Mapping> = serde_json::from_str(&data).unwrap_or_else(|_| {
                        error(format!(
                            "{} {}. {}",
                            "Failed to parse source map file:".red(),
                            source_map_file.blue(),
                            "Please ensure the file is a valid JSON source map.".red()
                        ));
                    });

                    mappings.extend(map);
                }

                let changes = get_changes(&original_content, mappings);
                let new_content = apply_changes(&original_content, changes);

                let modified_file_path =
                    path::Path::new(&modified).join(original_file_path.file_name().unwrap());
                fs::create_dir_all(modified_file_path.parent().unwrap()).unwrap_or_else(|_| {
                    error(format!(
                        "{} {}. {}",
                        "Failed to create directory for modified file:".red(),
                        modified_file_path.display().blue(),
                        "Please check the file path and permissions.".red()
                    ));
                });
                fs::write(&modified_file_path, new_content).unwrap_or_else(|_| {
                    error(format!(
                        "{} {}. {}",
                        "Failed to write modified file:".red(),
                        modified_file_path.display().blue(),
                        "Please check the file path and permissions.".red()
                    ));
                });
                println!(
                    "{} {}.",
                    "✓ Modified file generated:".green(),
                    modified_file_path.display().blue()
                );
            }

            println!("{}", "✓ Modified JS files generated successfully.".green(),);
        }
    }
}
