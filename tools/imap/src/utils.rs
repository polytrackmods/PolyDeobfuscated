use std::{collections::HashSet, fs, path::PathBuf};

use oxc_allocator::Allocator;
use oxc_ast_visit::Visit;
use oxc_parser::Parser;
use oxc_span::SourceType;
use walkdir::WalkDir;

use crate::visitor::IdentifierCollector;

/// Collects all JavaScript/TypeScript files from a directory
pub fn collect_js_files(dir: &str) -> Result<Vec<PathBuf>, Box<dyn std::error::Error>> {
    let files: Result<Vec<_>, _> = WalkDir::new(dir)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|entry| {
            entry.file_type().is_file()
                && entry
                    .path()
                    .extension()
                    .is_some_and(|ext| matches!(ext.to_str(), Some("js" | "ts" | "jsx" | "tsx")))
        })
        .map(|entry| {
            entry
                .path()
                .strip_prefix(dir)
                .map(|p| p.to_path_buf())
                .map_err(|e| e.into())
        })
        .collect();

    files
}

/// Extracts identifiers from JavaScript source code
pub fn extract_identifiers(source_code: &str) -> Result<Vec<(String, u32, usize)>, String> {
    let allocator = Allocator::default();
    let source_type = SourceType::default();
    let parser = Parser::new(&allocator, source_code, source_type);
    let parse_result = parser.parse();

    if !parse_result.errors.is_empty() {
        return Err(format!("Parse errors: {:?}", parse_result.errors));
    }

    let mut collector = IdentifierCollector::default();
    collector.visit_program(&parse_result.program);

    Ok(collector.identifiers)
}

/// Compares identifiers between two files and returns the differences
pub fn compare_identifiers(
    identifiers1: &[(String, u32, usize)],
    identifiers2: &[(String, u32, usize)],
) -> (Vec<(String, u32, usize)>, Vec<(String, u32, usize)>) {
    let set1: HashSet<_> = identifiers1.iter().collect();
    let set2: HashSet<_> = identifiers2.iter().collect();

    let only_in_first = identifiers1
        .iter()
        .filter(|s| !set2.contains(s))
        .cloned()
        .collect();

    let only_in_second = identifiers2
        .iter()
        .filter(|s| !set1.contains(s))
        .cloned()
        .collect();

    (only_in_first, only_in_second)
}

/// Updates the temporary directory with files from the code directory
pub fn update_temp_dir(code_dir: &str, temp_dir: &str) -> Result<(), Box<dyn std::error::Error>> {
    // Copy files from code_dir to temp_dir (which are js files)
    let files = collect_js_files(code_dir)?;
    for file in files {
        let original_path = PathBuf::from(code_dir).join(&file);
        let temp_path = PathBuf::from(temp_dir).join(&file);

        // Create the destination directory if it doesn't exist
        if let Some(parent) = temp_path.parent() {
            fs::create_dir_all(parent)?;
        }

        // Copy the file
        fs::copy(original_path, temp_path)?;
    }

    Ok(())
}

/// Checks that the changed identifiers are located in the same scope with the same unique ID
pub fn check_identifier_matches(
    original_identifiers: &[(String, u32, usize)],
    modified_identifiers: &[(String, u32, usize)],
) -> bool {
    let original: Vec<_> = original_identifiers
        .iter()
        .map(|ident| (ident.1, ident.2))
        .collect();
    let modified: Vec<_> = modified_identifiers
        .iter()
        .map(|ident| (ident.1, ident.2))
        .collect();

    original == modified
}
