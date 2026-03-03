pub mod css_comparator;
pub mod css_parser;
pub mod css_rule_converter;
pub mod duplicate_analyzer;
pub mod parser;
pub mod scss_flattener;
pub mod scss_simple_flattener;
pub mod shorthand_expander;
pub mod specificity;

pub use css_comparator::{
    CssRule, CssSimilarityResult, SerializableCssRule, calculate_rule_similarity, compare_css_rules,
};
pub use css_rule_converter::{convert_to_css_rule, parse_css_to_rules};
pub use duplicate_analyzer::{
    DuplicateAnalysisResult, DuplicateAnalyzer, DuplicateRule, DuplicateType,
    SerializableDuplicateRule,
};
pub use parser::CssParser;
pub use scss_flattener::{FlatRule, flatten_scss_rules};
pub use shorthand_expander::expand_shorthand_properties;
pub use specificity::{SelectorAnalysis, Specificity, calculate_specificity};
