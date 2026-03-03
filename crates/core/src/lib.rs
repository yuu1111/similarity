#![allow(clippy::uninlined_format_args)]

pub mod apted;
pub mod ast_exchange;
pub mod ast_fingerprint;
pub mod class_comparator;
pub mod class_extractor;
pub mod css_structure_adapter;
pub mod enhanced_similarity;
pub mod fast_similarity;
pub mod function_extractor;
pub mod generic_overlap_detector;
pub mod generic_parser_config;
pub mod generic_tree_sitter_parser;
pub mod language_parser;
pub mod overlap_detector;
pub mod parser;
pub mod rust_structure_adapter;
pub mod structure_comparator;
pub mod subtree_fingerprint;
pub mod tree;
pub mod tsed;
pub mod type_comparator;
pub mod type_extractor;
pub mod type_fingerprint;
pub mod type_normalizer;
pub mod typescript_structure_adapter;
pub mod unified_type_comparator;

// CLI utilities
pub mod cli_file_utils;
pub mod cli_output;
pub mod cli_parallel;

pub use apted::{APTEDOptions, compute_edit_distance};
pub use enhanced_similarity::{
    EnhancedSimilarityOptions, calculate_enhanced_similarity, calculate_semantic_similarity,
};
pub use function_extractor::{
    FunctionDefinition, FunctionType, SimilarityResult, compare_functions, extract_functions,
    find_similar_functions_across_files, find_similar_functions_in_file,
};
pub use parser::{ast_to_tree_node, parse_and_convert_to_tree};
pub use tree::TreeNode;
pub use tsed::{TSEDOptions, calculate_tsed, calculate_tsed_from_code};

// Type-related exports
pub use type_comparator::{
    MatchedProperty, SimilarTypePair, TypeComparisonOptions, TypeComparisonResult, TypeDifferences,
    TypeLiteralComparisonPair, TypeMismatch, compare_type_literal_with_type, compare_types,
    find_duplicate_types, find_similar_type_literals, find_similar_type_literals_pairs,
    find_similar_types, group_similar_types,
};
pub use type_extractor::{
    PropertyDefinition, TypeDefinition, TypeKind, TypeLiteralContext, TypeLiteralDefinition,
    extract_type_literals_from_code, extract_type_literals_from_files, extract_types_from_code,
    extract_types_from_files,
};
pub use type_normalizer::{
    NormalizationOptions, NormalizedType, PropertyMatch, calculate_property_similarity,
    calculate_type_similarity, find_property_matches, normalize_type,
};
pub use unified_type_comparator::{
    UnifiedType, UnifiedTypeComparisonPair, find_similar_unified_types,
    find_similar_unified_types_structured,
};

// Structure comparator exports
pub use css_structure_adapter::{CssBatchComparator, CssStructDef, CssStructureComparator};
pub use rust_structure_adapter::{
    RustEnumDef, RustFieldDef, RustStructDef, RustStructureComparator, RustVariantDef,
    RustVariantType,
};
pub use structure_comparator::{
    ComparisonOptions, MemberComparisonStrategy, MemberMatch, SourceLocation, Structure,
    StructureComparator, StructureComparisonResult, StructureDifferences, StructureIdentifier,
    StructureKind, StructureMember, StructureMetadata, compute_structure_fingerprint,
    should_compare_fingerprints,
};
pub use typescript_structure_adapter::{BatchComparator, TypeScriptStructureComparator};

// Fast similarity exports
pub use ast_fingerprint::AstFingerprint;
pub use fast_similarity::{
    FastSimilarityOptions, find_similar_functions_across_files_fast, find_similar_functions_fast,
};

// Subtree fingerprint exports
pub use subtree_fingerprint::{
    IndexedFunction, OverlapOptions, PartialOverlap, SubtreeFingerprint, create_sliding_windows,
    detect_partial_overlaps, generate_subtree_fingerprints,
};

// Overlap detector exports
pub use overlap_detector::{
    DetailedOverlap, PartialOverlapWithFiles, find_function_overlaps, find_overlaps_across_files,
    find_overlaps_with_similarity,
};

// Generic overlap detector exports
pub use generic_overlap_detector::{
    DetailedOverlap as GenericDetailedOverlap,
    PartialOverlapWithFiles as GenericPartialOverlapWithFiles, find_function_overlaps_generic,
    find_overlaps_across_files_generic, find_overlaps_with_similarity_generic,
};

// Class-related exports
pub use class_comparator::{
    ClassComparisonResult, ClassDifferences, MethodMismatch, NormalizedClass, PropertyMismatch,
    SimilarClassPair, compare_classes, find_similar_classes, find_similar_classes_across_files,
    normalize_class,
};
pub use class_extractor::{
    ClassDefinition, ClassMethod, ClassProperty, MethodKind, extract_classes_from_code,
    extract_classes_from_files,
};

#[cfg(test)]
mod structure_comparator_tests;
