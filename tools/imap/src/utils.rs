use std::{collections::HashSet, path::PathBuf};

use oxc_allocator::Allocator;
use oxc_ast_visit::Visit;
use oxc_parser::Parser;
use oxc_span::SourceType;
use walkdir::WalkDir;

use crate::types::*;
use crate::visitor::IdentifierCollector;

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

pub fn extract_identifiers(source_code: &str) -> Result<Vec<IdentifierDeclarationType>, String> {
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

pub fn compare_identifiers(
    identifiers1: &[IdentifierDeclarationType],
    identifiers2: &[IdentifierDeclarationType],
) -> (
    Vec<IdentifierDeclarationType>,
    Vec<IdentifierDeclarationType>,
) {
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

pub fn check_identifier_matches(
    original_identifiers: &[IdentifierDeclarationType],
    modified_identifiers: &[IdentifierDeclarationType],
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
