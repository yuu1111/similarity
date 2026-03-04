use anyhow::Result;
use similarity_core::generic_parser_config::GenericParserConfig;
use similarity_core::generic_tree_sitter_parser::GenericTreeSitterParser;

// Include auto-generated language configs
include!(concat!(env!("OUT_DIR"), "/language_configs.rs"));

/// Normalize language name aliases to canonical form
pub fn normalize_language(lang: &str) -> &str {
    match lang {
        "c++" => "cpp",
        "cs" => "csharp",
        "rb" => "ruby",
        _ => lang,
    }
}

/// Return file extensions for a given language
pub fn extensions_for_language(lang: &str) -> Vec<&'static str> {
    match normalize_language(lang) {
        "go" => vec!["go"],
        "java" => vec!["java"],
        "c" => vec!["c", "h"],
        "cpp" => vec!["cpp", "cc", "cxx", "hpp", "hxx", "h"],
        "csharp" => vec!["cs"],
        "ruby" => vec!["rb"],
        _ => vec![],
    }
}

/// Get the tree-sitter Language object for a given language name
pub fn tree_sitter_language(lang: &str) -> Option<tree_sitter::Language> {
    match normalize_language(lang) {
        "go" => Some(tree_sitter_go::LANGUAGE.into()),
        "java" => Some(tree_sitter_java::LANGUAGE.into()),
        "c" => Some(tree_sitter_c::LANGUAGE.into()),
        "cpp" => Some(tree_sitter_cpp::LANGUAGE.into()),
        "csharp" => Some(tree_sitter_c_sharp::LANGUAGE.into()),
        "ruby" => Some(tree_sitter_ruby::LANGUAGE.into()),
        _ => None,
    }
}

/// Load a GenericParserConfig for a language, trying embedded JSON first, then hardcoded fallback
pub fn config_for_language(lang: &str) -> Option<GenericParserConfig> {
    let normalized = normalize_language(lang);

    // Try embedded JSON configs first
    if let Some(config_json) = LANGUAGE_CONFIGS.get(normalized)
        && let Ok(config) = serde_json::from_str(config_json)
    {
        return Some(config);
    }

    // Hardcoded fallback
    match normalized {
        "go" => Some(GenericParserConfig::go()),
        "java" => Some(GenericParserConfig::java()),
        "c" => Some(GenericParserConfig::c()),
        "cpp" => Some(GenericParserConfig::cpp()),
        "csharp" => Some(GenericParserConfig::csharp()),
        "ruby" => Some(GenericParserConfig::ruby()),
        _ => None,
    }
}

/// Create a new GenericTreeSitterParser for a language.
/// This creates a fresh parser instance (suitable for use in rayon closures,
/// since tree_sitter::Parser is not Send/Sync).
pub fn create_parser(lang: &str) -> Result<GenericTreeSitterParser> {
    let config = config_for_language(lang)
        .ok_or_else(|| anyhow::anyhow!("Unsupported language: {}", lang))?;
    let ts_lang = tree_sitter_language(lang)
        .ok_or_else(|| anyhow::anyhow!("No tree-sitter grammar for: {}", lang))?;
    GenericTreeSitterParser::new(ts_lang, config)
        .map_err(|e| anyhow::anyhow!("Failed to create parser: {}", e))
}

/// Create a parser from a pre-loaded config (for --config mode)
pub fn create_parser_from_config(config: &GenericParserConfig) -> Result<GenericTreeSitterParser> {
    let ts_lang = tree_sitter_language(&config.language)
        .ok_or_else(|| anyhow::anyhow!("No tree-sitter grammar for: {}", config.language))?;
    GenericTreeSitterParser::new(ts_lang, config.clone())
        .map_err(|e| anyhow::anyhow!("Failed to create parser: {}", e))
}

/// Create a parser from either a custom config or a language name.
/// Consolidates the two-branch pattern used across parallel and main modules.
pub fn make_parser(
    language: &str,
    config: Option<&GenericParserConfig>,
) -> Result<GenericTreeSitterParser> {
    match config {
        Some(cfg) => create_parser_from_config(cfg),
        None => create_parser(language),
    }
}

/// Check if a language is supported
pub fn is_supported(lang: &str) -> bool {
    tree_sitter_language(lang).is_some()
}

/// List of all supported languages
pub fn supported_languages() -> &'static [(&'static str, &'static str)] {
    &[
        ("go", "Go language"),
        ("java", "Java language"),
        ("c", "C language"),
        ("cpp", "C++ language"),
        ("csharp", "C# language"),
        ("ruby", "Ruby language"),
    ]
}
