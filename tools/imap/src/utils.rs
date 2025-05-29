use std::collections::HashSet;

use oxc::{
    allocator::Allocator,
    ast_visit::Visit,
    parser::Parser,
    span::{SourceType, Span},
};
use walkdir::WalkDir;
use yansi::Paint;

use crate::{
    types::*,
    visitors::{IdentifierCollector, Renamer},
};

pub fn error<T: std::fmt::Display>(message: T) -> ! {
    eprintln!("{} {}", "Error:".red(), message);
    std::process::exit(1);
}

pub fn get_js_files(path: &str) -> Vec<String> {
    let mut js_files = Vec::new();
    for entry in WalkDir::new(path) {
        let entry = entry.unwrap_or_else(|_| {
            error(format!("{} {}.", "Failed to read entry:".red(), path));
        });

        if entry.file_type().is_file() && entry.path().extension().is_some_and(|ext| ext == "js") {
            js_files.push(entry.path().to_string_lossy().to_string());
        }
    }
    js_files
}

pub fn get_map_files(path: &str, file_name: &str) -> Vec<String> {
    let mut map_files = Vec::new();
    for entry in WalkDir::new(path) {
        let entry = entry.unwrap();

        if entry.file_type().is_file()
            && entry.path().extension().is_some_and(|ext| ext == "map")
            && entry
                .path()
                .file_stem()
                .is_some_and(|stem| stem == file_name)
        {
            map_files.push(entry.path().to_string_lossy().to_string());
        }
    }
    map_files
}

pub fn get_identifiers(source_text: &str) -> Vec<Identifier> {
    let allocator = Allocator::default();
    let source_type = SourceType::default();
    let parser = Parser::new(&allocator, source_text, source_type);
    let parse_result = parser.parse();

    if parse_result.errors.is_empty() {
        let mut visitor = IdentifierCollector::default();
        visitor.visit_program(&parse_result.program);
        visitor.identifiers
    } else {
        error("Parsing errors occurred...".red());
    }
}

pub fn compare_identifiers<'a>(
    identifiers1: &'a [Identifier],
    identifiers2: &'a [Identifier],
) -> (Vec<&'a Identifier>, Vec<&'a Identifier>) {
    let set1: HashSet<_> = identifiers1.iter().collect();
    let set2: HashSet<_> = identifiers2.iter().collect();

    let only_in_1: Vec<_> = set1.difference(&set2).cloned().collect();
    let only_in_2: Vec<_> = set2.difference(&set1).cloned().collect();

    let mut only_in_1 = only_in_1;
    let mut only_in_2 = only_in_2;
    only_in_1.sort_by_key(|id| id.id);
    only_in_2.sort_by_key(|id| id.id);

    (only_in_1, only_in_2)
}

pub fn check_identifiers(identifiers1: &[&Identifier], identifiers2: &[&Identifier]) {
    if identifiers1.len() != identifiers2.len() {
        error(format!(
            "{} {} ≠ {}. {}",
            "Identifiers count mismatch:".red(),
            identifiers1.len().to_string().yellow(),
            identifiers2.len().to_string().yellow(),
            "Please ensure both sets have the same number of identifiers.".red()
        ));
    }

    // used to verify that for each id, there is an identifier with the same scope_id in the second set
    let id_set2: HashSet<_> = identifiers2.iter().map(|id| id.scope_id).collect();

    for id in identifiers1 {
        if !id_set2.contains(&id.scope_id) {
            error(format!(
                "{} {}:{} {}",
                "Identifier".red(),
                id.name.yellow(),
                id.scope_id.yellow(),
                "was not found in the second set.".red()
            ));
        }
    }
}

pub fn get_changes(source_text: &str, mappings: Vec<Mapping>) -> Vec<(Span, String)> {
    let allocator = Allocator::default();
    let source_type = SourceType::default();
    let parser = Parser::new(&allocator, source_text, source_type);
    let parse_result = parser.parse();

    if parse_result.errors.is_empty() {
        let mut visitor = Renamer::new(mappings);
        visitor.visit_program(&parse_result.program);
        visitor.changes
    } else {
        error("Parsing errors occurred...".red());
    }
}

pub fn apply_changes(source_text: &str, changes: Vec<(Span, String)>) -> String {
    // we have to use some special shenanigans to apply the changes
    // because if you insert a string that is longer than the original string,
    // the spans will be off, so we have to adjust them accordingly
    let mut result = String::new();
    let mut last_end = 0;
    for (span, new_name) in changes {
        let start = span.start as usize;
        let end = span.end as usize;

        result.push_str(&source_text[last_end..start]);
        result.push_str(&new_name);
        last_end = end;
    }
    result.push_str(&source_text[last_end..]);
    result
}
