use anyhow::Result;
use clap::Parser;
use similarity_core::generic_parser_config::GenericParserConfig;
use std::path::PathBuf;

mod check;
mod language_util;
mod parallel;

#[derive(Parser)]
#[command(name = "similarity-generic")]
#[command(about = "Generic code similarity analyzer using tree-sitter")]
#[command(version)]
struct Cli {
    /// Paths to analyze (files or directories)
    #[arg(default_value = ".", required_unless_present_any = ["supported", "show_config"])]
    paths: Vec<String>,

    /// Language configuration file (JSON)
    #[arg(short, long, conflicts_with_all = ["language", "supported", "show_config"])]
    config: Option<PathBuf>,

    /// Language name (if using built-in config)
    #[arg(short, long, conflicts_with_all = ["config", "supported", "show_config"])]
    language: Option<String>,

    /// Similarity threshold (0.0-1.0)
    #[arg(short, long, default_value = "0.85")]
    threshold: f64,

    /// Print code in output
    #[arg(short, long)]
    print: bool,

    /// File extensions to check
    #[arg(short, long, value_delimiter = ',')]
    extensions: Option<Vec<String>>,

    /// Minimum lines for functions to be considered
    #[arg(short, long, default_value = "3")]
    min_lines: Option<u32>,

    /// Minimum tokens for functions to be considered
    #[arg(long)]
    min_tokens: Option<u32>,

    /// Rename cost for APTED algorithm
    #[arg(short, long, default_value = "0.3")]
    rename_cost: f64,

    /// Disable size penalty for very different sized functions
    #[arg(long)]
    no_size_penalty: bool,

    /// Filter functions by name (substring match)
    #[arg(long)]
    filter_function: Option<String>,

    /// Filter functions by body content (substring match)
    #[arg(long)]
    filter_function_body: Option<String>,

    /// Exclude paths matching glob patterns
    #[arg(long)]
    exclude: Vec<String>,

    /// Show extracted functions
    #[arg(long)]
    show_functions: bool,

    /// Show supported languages
    #[arg(long, conflicts_with_all = ["config", "language", "show_functions", "show_config"])]
    supported: bool,

    /// Show example configuration for a language
    #[arg(long, value_name = "LANGUAGE", conflicts_with_all = ["config", "language", "show_functions", "supported"])]
    show_config: Option<String>,

    /// Enable experimental overlap detection mode
    #[arg(long = "experimental-overlap")]
    overlap: bool,

    /// Minimum window size for overlap detection (number of nodes)
    #[arg(long, default_value = "8")]
    overlap_min_window: u32,

    /// Maximum window size for overlap detection (number of nodes)
    #[arg(long, default_value = "25")]
    overlap_max_window: u32,

    /// Size tolerance for overlap detection (0.0-1.0)
    #[arg(long, default_value = "0.25")]
    overlap_size_tolerance: f64,

    /// Exit with code 1 if duplicates are found
    #[arg(long)]
    fail_on_duplicates: bool,

    /// Skip test functions
    #[arg(long)]
    skip_tests: bool,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    // Handle --supported
    if cli.supported {
        println!("Supported languages for generic tree-sitter parser:");
        for (name, desc) in language_util::supported_languages() {
            println!("  {name:10} - {desc}");
        }
        println!();
        println!("Note: For Python, TypeScript, and Rust, use the dedicated implementations:");
        println!("  similarity-py  - Optimized Python analyzer");
        println!("  similarity-ts  - Optimized TypeScript/JavaScript analyzer");
        return Ok(());
    }

    // Handle --show-config
    if let Some(lang) = &cli.show_config {
        let config = language_util::config_for_language(lang).ok_or_else(|| {
            anyhow::anyhow!(
                "Unknown language: {}. Use --supported to see available languages.",
                lang
            )
        })?;
        println!("{}", serde_json::to_string_pretty(&config)?);
        return Ok(());
    }

    // Resolve language config
    let (language_name, custom_config) = resolve_language(&cli)?;

    // Handle --show-functions
    if cli.show_functions {
        return show_functions(&cli.paths, &language_name, custom_config.as_ref(), &cli);
    }

    let mut total_duplicates = 0;

    // Normal similarity detection
    if !cli.overlap {
        let duplicate_count = check::check_paths(
            cli.paths.clone(),
            cli.threshold,
            cli.rename_cost,
            cli.extensions.as_ref(),
            cli.min_lines.unwrap_or(3),
            cli.min_tokens,
            cli.no_size_penalty,
            cli.print,
            cli.filter_function.as_ref(),
            cli.filter_function_body.as_ref(),
            &cli.exclude,
            &language_name,
            custom_config.as_ref(),
            cli.skip_tests,
        )?;
        total_duplicates += duplicate_count;
    }

    // Overlap detection
    if cli.overlap {
        let overlap_count = check_overlaps(
            &cli.paths,
            &language_name,
            custom_config.as_ref(),
            &cli,
        )?;
        total_duplicates += overlap_count;
    }

    if cli.fail_on_duplicates && total_duplicates > 0 {
        std::process::exit(1);
    }

    Ok(())
}

/// Resolve language from --config or --language
fn resolve_language(cli: &Cli) -> Result<(String, Option<GenericParserConfig>)> {
    if let Some(config_path) = &cli.config {
        let config = GenericParserConfig::from_file(config_path)
            .map_err(|e| anyhow::anyhow!("Failed to load config: {}", e))?;
        let lang = config.language.clone();
        Ok((lang, Some(config)))
    } else if let Some(lang) = &cli.language {
        if !language_util::is_supported(lang) {
            eprintln!("Error: Language '{lang}' is not supported by similarity-generic.");
            eprintln!("Use --supported to see available languages.");
            if matches!(
                lang.as_str(),
                "python" | "py" | "rust" | "rs" | "javascript" | "js" | "typescript" | "ts"
            ) {
                eprintln!();
                eprintln!("Note: For {lang}, use the dedicated implementation:");
                match lang.as_str() {
                    "python" | "py" => eprintln!("  similarity-py"),
                    "rust" | "rs" => eprintln!("  similarity-rs (planned)"),
                    "javascript" | "js" | "typescript" | "ts" => eprintln!("  similarity-ts"),
                    _ => {}
                }
            }
            return Err(anyhow::anyhow!("Unsupported language"));
        }
        Ok((language_util::normalize_language(lang).to_string(), None))
    } else {
        Err(anyhow::anyhow!("Either --config or --language must be provided"))
    }
}

/// Show extracted functions for all files
fn show_functions(
    paths: &[String],
    language: &str,
    config: Option<&GenericParserConfig>,
    cli: &Cli,
) -> Result<()> {
    use similarity_core::{cli_file_utils::collect_files, language_parser::LanguageParser};

    let lang_extensions = language_util::extensions_for_language(language);
    let exts: Vec<&str> = cli
        .extensions
        .as_ref()
        .map_or(lang_extensions, |v| v.iter().map(String::as_str).collect());

    let files = collect_files(paths, &exts)?;
    if files.is_empty() {
        println!("No files found in the specified paths.");
        return Ok(());
    }

    for file in &files {
        let content = std::fs::read_to_string(file)?;
        let filename = file.to_string_lossy();

        let mut parser = language_util::make_parser(language, config)?;

        match parser.extract_functions(&content, &filename) {
            Ok(functions) => {
                println!("{} ({} functions):", filename, functions.len());
                for func in &functions {
                    println!(
                        "  {}:{}-{} {}",
                        filename, func.start_line, func.end_line, func.name
                    );
                }
            }
            Err(e) => {
                eprintln!("Error parsing {}: {}", filename, e);
            }
        }
    }

    Ok(())
}

/// Directory-aware overlap detection
fn check_overlaps(
    paths: &[String],
    language: &str,
    config: Option<&GenericParserConfig>,
    cli: &Cli,
) -> Result<usize> {
    use similarity_core::{OverlapOptions, cli_file_utils::collect_files, find_overlaps_across_files_generic};
    use std::collections::HashMap;

    let lang_extensions = language_util::extensions_for_language(language);
    let exts: Vec<&str> = cli
        .extensions
        .as_ref()
        .map_or(lang_extensions, |v| v.iter().map(String::as_str).collect());

    let files = collect_files(paths, &exts)?;
    if files.is_empty() {
        println!("No files found in the specified paths.");
        return Ok(0);
    }

    println!("Checking {} files for overlapping code...\n", files.len());

    let mut file_contents = HashMap::new();
    for file in &files {
        match std::fs::read_to_string(file) {
            Ok(content) => {
                file_contents.insert(file.to_string_lossy().to_string(), content);
            }
            Err(e) => {
                eprintln!("Error reading {}: {}", file.display(), e);
            }
        }
    }

    let options = OverlapOptions {
        min_window_size: cli.overlap_min_window,
        max_window_size: cli.overlap_max_window,
        threshold: cli.threshold,
        size_tolerance: cli.overlap_size_tolerance,
    };

    let mut parser = language_util::make_parser(language, config)?;

    let overlaps = find_overlaps_across_files_generic(&mut parser, &file_contents, &options)
        .map_err(|e| anyhow::anyhow!("Failed to find overlaps: {}", e))?;

    if overlaps.is_empty() {
        println!("\nNo code overlaps found!");
    } else {
        println!("\nCode overlaps found:");
        println!("{}", "-".repeat(60));

        for overlap_with_files in &overlaps {
            let overlap = &overlap_with_files.overlap;
            let source_path = get_relative_path(&overlap_with_files.source_file);
            let target_path = get_relative_path(&overlap_with_files.target_file);

            println!(
                "\nSimilarity: {:.2}% | {} nodes | {}",
                overlap.similarity * 100.0,
                overlap.node_count,
                overlap.node_type
            );
            println!(
                "  {}:{} | L{}-{} in function: {}",
                source_path,
                overlap.source_lines.0,
                overlap.source_lines.0,
                overlap.source_lines.1,
                overlap.source_function
            );
            println!(
                "  {}:{} | L{}-{} in function: {}",
                target_path,
                overlap.target_lines.0,
                overlap.target_lines.0,
                overlap.target_lines.1,
                overlap.target_function
            );

            if cli.print
                && let Some(source_content) = file_contents.get(&overlap_with_files.source_file)
                && let Some(target_content) = file_contents.get(&overlap_with_files.target_file)
            {
                println!("\n\x1b[36m--- Source Code ---\x1b[0m");
                if let Ok(source_segment) = extract_code_lines(
                    source_content,
                    overlap.source_lines.0,
                    overlap.source_lines.1,
                ) {
                    println!("{source_segment}");
                }

                println!("\n\x1b[36m--- Target Code ---\x1b[0m");
                if let Ok(target_segment) = extract_code_lines(
                    target_content,
                    overlap.target_lines.0,
                    overlap.target_lines.1,
                ) {
                    println!("{target_segment}");
                }
            }
        }

        println!("\nTotal overlaps found: {}", overlaps.len());
    }

    Ok(overlaps.len())
}

fn get_relative_path(file_path: &str) -> String {
    if let Ok(current_dir) = std::env::current_dir() {
        std::path::Path::new(file_path)
            .strip_prefix(&current_dir)
            .unwrap_or(std::path::Path::new(file_path))
            .to_string_lossy()
            .to_string()
    } else {
        file_path.to_string()
    }
}

fn extract_code_lines(code: &str, start_line: u32, end_line: u32) -> Result<String, String> {
    let lines: Vec<_> = code.lines().collect();

    if start_line as usize > lines.len() || end_line as usize > lines.len() {
        return Err("Line numbers out of bounds".to_string());
    }

    let start = (start_line as usize).saturating_sub(1);
    let end = (end_line as usize).min(lines.len());

    Ok(lines[start..end].join("\n"))
}
