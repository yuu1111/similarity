use crate::language_util::make_parser;
use rayon::prelude::*;
use similarity_core::{
    cli_parallel::{FileData, SimilarityResult},
    generic_parser_config::GenericParserConfig,
    language_parser::{GenericFunctionDef, LanguageParser},
    tsed::{TSEDOptions, calculate_tsed},
};
use std::fs;
use std::path::PathBuf;
use std::rc::Rc;

pub type GenericFileData = FileData<GenericFunctionDef>;

/// Load and parse files in parallel, creating a fresh parser per file
pub fn load_files_parallel(
    files: &[PathBuf],
    language: &str,
    config: Option<&GenericParserConfig>,
) -> Vec<GenericFileData> {
    files
        .par_iter()
        .filter_map(|file| {
            let content = match fs::read_to_string(file) {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("Error reading {}: {}", file.display(), e);
                    return None;
                }
            };
            let filename = file.to_string_lossy();

            let mut parser = make_parser(language, config).ok()?;

            match parser.extract_functions(&content, &filename) {
                Ok(functions) => Some(FileData { path: file.clone(), content, functions }),
                Err(e) => {
                    eprintln!("Error parsing {}: {}", file.display(), e);
                    None
                }
            }
        })
        .collect()
}

/// Check for duplicates within each file, using already-loaded file data
pub fn check_within_file_duplicates_parallel(
    file_data: &[GenericFileData],
    threshold: f64,
    options: &TSEDOptions,
    language: &str,
    config: Option<&GenericParserConfig>,
) -> Vec<(PathBuf, Vec<SimilarityResult<GenericFunctionDef>>)> {
    file_data
        .par_iter()
        .filter_map(|data| {
            let file_str = data.path.to_string_lossy();
            let mut parser = make_parser(language, config).ok()?;
            let lines: Vec<&str> = data.content.lines().collect();
            let mut similar_pairs = Vec::new();

            for i in 0..data.functions.len() {
                for j in (i + 1)..data.functions.len() {
                    let func1 = &data.functions[i];
                    let func2 = &data.functions[j];

                    if func1.end_line - func1.start_line + 1 < options.min_lines
                        || func2.end_line - func2.start_line + 1 < options.min_lines
                    {
                        continue;
                    }

                    let body1 = extract_function_body(&lines, func1);
                    let body2 = extract_function_body(&lines, func2);

                    let similarity = match (
                        parser.parse(&body1, &format!("{}:{}", file_str, func1.name)),
                        parser.parse(&body2, &format!("{}:{}", file_str, func2.name)),
                    ) {
                        (Ok(tree1), Ok(tree2)) => calculate_tsed(&tree1, &tree2, options),
                        _ => 0.0,
                    };

                    if similarity >= threshold {
                        similar_pairs.push(SimilarityResult::new(
                            func1.clone(),
                            func2.clone(),
                            similarity,
                        ));
                    }
                }
            }

            if similar_pairs.is_empty() {
                None
            } else {
                Some((data.path.clone(), similar_pairs))
            }
        })
        .collect()
}

/// Pre-parsed function with its AST tree, ready for cross-file comparison
struct ParsedFunc {
    file_idx: usize,
    func: GenericFunctionDef,
    tree: Rc<similarity_core::tree::TreeNode>,
}

/// Check for duplicates across files, in parallel.
/// Pre-parses all function bodies into AST trees (O(N) parsers),
/// then compares pairs using only calculate_tsed (no parser creation per pair).
pub fn check_cross_file_duplicates_parallel(
    file_data: &[GenericFileData],
    threshold: f64,
    options: &TSEDOptions,
    language: &str,
    config: Option<&GenericParserConfig>,
) -> Vec<(String, SimilarityResult<GenericFunctionDef>, String)> {
    // Pre-split lines once per file
    let file_lines: Vec<Vec<&str>> = file_data.iter().map(|d| d.content.lines().collect()).collect();

    // Pre-parse all function bodies into trees (one parser per function, not per pair)
    // Note: Rc<TreeNode> is not Send, so we collect sequentially per file, then merge.
    // We use a flat Vec grouped by file for the parallel pair comparison.
    let parsed_funcs: Vec<ParsedFunc> = file_data
        .iter()
        .enumerate()
        .flat_map(|(file_idx, data)| {
            let file_str = data.path.to_string_lossy();
            let lines = &file_lines[file_idx];
            let mut parser = match make_parser(language, config) {
                Ok(p) => p,
                Err(_) => return Vec::new(),
            };

            data.functions
                .iter()
                .filter(|func| func.end_line - func.start_line + 1 >= options.min_lines)
                .filter_map(|func| {
                    let body = extract_function_body(lines, func);
                    let tree =
                        parser.parse(&body, &format!("{}:{}", file_str, func.name)).ok()?;
                    Some(ParsedFunc { file_idx, func: func.clone(), tree })
                })
                .collect::<Vec<_>>()
        })
        .collect();

    // Generate cross-file pairs
    let mut pairs_to_check = Vec::new();
    for i in 0..parsed_funcs.len() {
        for j in (i + 1)..parsed_funcs.len() {
            if parsed_funcs[i].file_idx != parsed_funcs[j].file_idx {
                pairs_to_check.push((i, j));
            }
        }
    }

    // Compare pairs - calculate_tsed is pure computation, no parser needed.
    // Since Rc<TreeNode> is not Send, we process sequentially.
    pairs_to_check
        .iter()
        .filter_map(|&(i, j)| {
            let pf1 = &parsed_funcs[i];
            let pf2 = &parsed_funcs[j];

            let similarity = calculate_tsed(&pf1.tree, &pf2.tree, options);

            if similarity >= threshold {
                let file1 = file_data[pf1.file_idx].path.to_string_lossy().to_string();
                let file2 = file_data[pf2.file_idx].path.to_string_lossy().to_string();
                Some((
                    file1,
                    SimilarityResult::new(pf1.func.clone(), pf2.func.clone(), similarity),
                    file2,
                ))
            } else {
                None
            }
        })
        .collect()
}

fn extract_function_body(lines: &[&str], func: &GenericFunctionDef) -> String {
    let start_idx = (func.body_start_line.saturating_sub(1)) as usize;
    let end_idx = std::cmp::min(func.body_end_line as usize, lines.len());

    if start_idx >= lines.len() {
        return String::new();
    }

    lines[start_idx..end_idx].join("\n")
}
