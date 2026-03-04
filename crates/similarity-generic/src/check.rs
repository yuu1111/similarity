use crate::language_util::extensions_for_language;
use crate::parallel::{
    check_cross_file_duplicates_parallel, check_within_file_duplicates_parallel,
    load_files_parallel,
};
use ignore::WalkBuilder;
use similarity_core::{
    TSEDOptions,
    cli_file_utils::collect_files,
    cli_output::{format_function_output, show_function_code},
    cli_parallel::SimilarityResult,
    generic_parser_config::GenericParserConfig,
    language_parser::GenericFunctionDef,
};
use std::collections::HashSet;
use std::path::{Path, PathBuf};

fn create_exclude_matcher(exclude_patterns: &[String]) -> Option<globset::GlobSet> {
    if exclude_patterns.is_empty() {
        return None;
    }

    let mut builder = globset::GlobSetBuilder::new();
    for pattern in exclude_patterns {
        if let Ok(glob) = globset::Glob::new(pattern) {
            builder.add(glob);
        }

        if !pattern.starts_with("**") {
            let prefixed = format!("**/{}", pattern);
            if let Ok(glob) = globset::Glob::new(&prefixed) {
                builder.add(glob);
            }

            let suffixed = format!("{}/**", pattern.trim_end_matches('/'));
            if let Ok(glob) = globset::Glob::new(&suffixed) {
                builder.add(glob);
            }

            let both = format!("**/{}", suffixed);
            if let Ok(glob) = globset::Glob::new(&both) {
                builder.add(glob);
            }
        }
    }

    builder.build().ok()
}

struct DuplicateResult {
    file1: PathBuf,
    file2: PathBuf,
    result: SimilarityResult<GenericFunctionDef>,
}

impl DuplicateResult {
    fn priority(&self) -> f64 {
        let avg_lines = ((self.result.func1.end_line - self.result.func1.start_line + 1)
            + (self.result.func2.end_line - self.result.func2.start_line + 1))
            as f64
            / 2.0;
        self.result.similarity * avg_lines
    }
}

#[allow(clippy::too_many_arguments)]
pub fn check_paths(
    paths: Vec<String>,
    threshold: f64,
    rename_cost: f64,
    extensions: Option<&Vec<String>>,
    min_lines: u32,
    min_tokens: Option<u32>,
    no_size_penalty: bool,
    print: bool,
    filter_function: Option<&String>,
    filter_function_body: Option<&String>,
    exclude_patterns: &[String],
    language: &str,
    config: Option<&GenericParserConfig>,
    skip_tests: bool,
) -> anyhow::Result<usize> {
    let lang_extensions = extensions_for_language(language);
    let exts: Vec<&str> =
        extensions.map_or(lang_extensions, |v| v.iter().map(String::as_str).collect());

    let files = if exclude_patterns.is_empty() {
        collect_files(&paths, &exts)?
    } else {
        collect_files_with_exclude(&paths, &exts, exclude_patterns)?
    };

    if files.is_empty() {
        println!("No files found in the specified paths.");
        return Ok(0);
    }

    println!("Checking {} files for duplicates...", files.len());

    let mut options = TSEDOptions::default();
    options.apted_options.rename_cost = rename_cost;
    options.min_lines = min_lines;
    options.min_tokens = min_tokens;
    options.size_penalty = !no_size_penalty;
    options.skip_test = skip_tests;

    let mut all_results = Vec::new();

    // Load all files once (read + parse)
    let file_data = load_files_parallel(&files, language, config);

    // Within-file duplicates (uses already-loaded data, no re-read)
    let within_file_results =
        check_within_file_duplicates_parallel(&file_data, threshold, &options, language, config);

    for (file, similar_pairs) in within_file_results {
        for result in similar_pairs {
            all_results.push(DuplicateResult { file1: file.clone(), file2: file.clone(), result });
        }
    }

    // Cross-file duplicates (pre-parses trees, then compares without creating parsers per pair)
    let cross_file_results =
        check_cross_file_duplicates_parallel(&file_data, threshold, &options, language, config);

    for (file1, result, file2) in cross_file_results {
        all_results.push(DuplicateResult {
            file1: PathBuf::from(file1),
            file2: PathBuf::from(file2),
            result,
        });
    }

    let duplicate_count =
        display_all_results(all_results, print, filter_function, filter_function_body);

    Ok(duplicate_count)
}

fn collect_files_with_exclude(
    paths: &[String],
    extensions: &[&str],
    exclude_patterns: &[String],
) -> anyhow::Result<Vec<PathBuf>> {
    let exclude_matcher = create_exclude_matcher(exclude_patterns);
    let mut files = Vec::new();
    let mut visited = HashSet::new();

    for path_str in paths {
        let path = Path::new(path_str);

        if path.is_file() {
            if let Some(ext) = path.extension()
                && let Some(ext_str) = ext.to_str()
                && extensions.contains(&ext_str)
                && let Ok(canonical) = path.canonicalize()
                && visited.insert(canonical.clone())
            {
                files.push(path.to_path_buf());
            }
        } else if path.is_dir() {
            let walker = WalkBuilder::new(path)
                .follow_links(false)
                .git_ignore(true)
                .git_global(true)
                .git_exclude(true)
                .build();

            for entry in walker {
                let entry = entry?;
                let entry_path = entry.path();

                if !entry_path.is_file() {
                    continue;
                }

                if let Some(ref matcher) = exclude_matcher {
                    if matcher.is_match(entry_path) {
                        continue;
                    }
                    if let Ok(current_dir) = std::env::current_dir()
                        && let Ok(relative) = entry_path.strip_prefix(&current_dir)
                        && matcher.is_match(relative)
                    {
                        continue;
                    }
                }

                if let Some(ext) = entry_path.extension()
                    && let Some(ext_str) = ext.to_str()
                    && extensions.contains(&ext_str)
                    && let Ok(canonical) = entry_path.canonicalize()
                    && visited.insert(canonical.clone())
                {
                    files.push(entry_path.to_path_buf());
                }
            }
        } else {
            eprintln!("Path does not exist or is not accessible: {}", path_str);
        }
    }

    files.sort();
    Ok(files)
}

fn display_all_results(
    mut all_results: Vec<DuplicateResult>,
    print: bool,
    filter_function: Option<&String>,
    filter_function_body: Option<&String>,
) -> usize {
    if all_results.is_empty() {
        println!("\nNo duplicate functions found!");
        return 0;
    }

    // Apply filters
    if filter_function.is_some() || filter_function_body.is_some() {
        all_results.retain(|dup| {
            if let Some(filter) = filter_function
                && !dup.result.func1.name.contains(filter)
                && !dup.result.func2.name.contains(filter)
            {
                return false;
            }

            if let Some(filter) = filter_function_body {
                let mut match_found = false;

                if let Ok(content) = std::fs::read_to_string(&dup.file1) {
                    let body = similarity_core::cli_output::extract_lines_from_content(
                        &content,
                        dup.result.func1.start_line,
                        dup.result.func1.end_line,
                    );
                    if body.contains(filter) {
                        match_found = true;
                    }
                }

                if !match_found
                    && let Ok(content) = std::fs::read_to_string(&dup.file2)
                {
                    let body = similarity_core::cli_output::extract_lines_from_content(
                        &content,
                        dup.result.func2.start_line,
                        dup.result.func2.end_line,
                    );
                    if body.contains(filter) {
                        match_found = true;
                    }
                }

                if !match_found {
                    return false;
                }
            }

            true
        });
    }

    if all_results.is_empty() {
        println!("\nNo duplicate functions found matching the filters!");
        return 0;
    }

    // Sort by priority
    all_results.sort_by(|a, b| {
        b.priority().partial_cmp(&a.priority()).unwrap_or(std::cmp::Ordering::Equal)
    });

    println!("\nFound {} duplicate pairs:", all_results.len());
    println!("{}", "-".repeat(60));

    for dup in &all_results {
        let (relative_path1, relative_path2) = if let Ok(current_dir) = std::env::current_dir() {
            (
                dup.file1
                    .strip_prefix(&current_dir)
                    .unwrap_or(&dup.file1)
                    .to_string_lossy()
                    .to_string(),
                dup.file2
                    .strip_prefix(&current_dir)
                    .unwrap_or(&dup.file2)
                    .to_string_lossy()
                    .to_string(),
            )
        } else {
            (dup.file1.to_string_lossy().to_string(), dup.file2.to_string_lossy().to_string())
        };

        let line_count1 = dup.result.func1.end_line - dup.result.func1.start_line + 1;
        let line_count2 = dup.result.func2.end_line - dup.result.func2.start_line + 1;
        let min_lines = line_count1.min(line_count2);
        let max_lines = line_count1.max(line_count2);
        let avg_lines = (line_count1 + line_count2) as f64 / 2.0;
        let score = dup.result.similarity * avg_lines;

        println!(
            "\nSimilarity: {:.2}%, Score: {:.1} points (lines {}~{}, avg: {:.1})",
            dup.result.similarity * 100.0,
            score,
            min_lines,
            max_lines,
            avg_lines
        );
        println!(
            "  {}",
            format_function_output(
                &relative_path1,
                &format!(
                    "{} {}",
                    if dup.result.func1.is_method { "method" } else { "function" },
                    &dup.result.func1.name
                ),
                dup.result.func1.start_line,
                dup.result.func1.end_line,
            )
        );
        println!(
            "  {}",
            format_function_output(
                &relative_path2,
                &format!(
                    "{} {}",
                    if dup.result.func2.is_method { "method" } else { "function" },
                    &dup.result.func2.name
                ),
                dup.result.func2.start_line,
                dup.result.func2.end_line,
            )
        );

        if let (Some(class1), Some(class2)) =
            (&dup.result.func1.class_name, &dup.result.func2.class_name)
        {
            println!("  Classes: {} <-> {}", class1, class2);
        }

        if print {
            show_function_code(
                &relative_path1,
                &dup.result.func1.name,
                dup.result.func1.start_line,
                dup.result.func1.end_line,
            );
            show_function_code(
                &relative_path2,
                &dup.result.func2.name,
                dup.result.func2.start_line,
                dup.result.func2.end_line,
            );
        }
    }

    all_results.len()
}
