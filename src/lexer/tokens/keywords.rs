//! JavaScript keyword system
//!
//! Provides fast keyword lookup, contextual classification, and version-specific
//! keyword handling for different ECMAScript versions and strict mode.

use crate::lexer::core::{LexerContext, EcmaVersion};
use std::collections::{HashMap, HashSet};
use std::sync::LazyLock;

/// JavaScript keyword categories
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum KeywordCategory {
    /// Core reserved words (function, var, if, etc.)
    Reserved,
    /// Strict mode reserved words
    StrictReserved,
    /// Future reserved words
    FutureReserved,
    /// Contextual keywords (async, await, from, of)
    Contextual,
    /// Literal keywords (true, false, null, undefined)
    Literal,
}

/// Information about a keyword
#[derive(Debug, Clone)]
pub struct KeywordInfo {
    /// The keyword string
    pub keyword: &'static str,
    /// Category of the keyword
    pub category: KeywordCategory,
    /// Minimum ECMAScript version where this keyword is available
    pub min_version: EcmaVersion,
    /// Whether this keyword is only reserved in strict mode
    pub strict_only: bool,
    /// Whether this keyword can be used as an identifier in some contexts
    pub contextual: bool,
}

impl KeywordInfo {
    /// Create new keyword info
    pub const fn new(
        keyword: &'static str,
        category: KeywordCategory,
        min_version: EcmaVersion,
        strict_only: bool,
        contextual: bool,
    ) -> Self {
        Self {
            keyword,
            category,
            min_version,
            strict_only,
            contextual,
        }
    }
    
    /// Check if this keyword is available in the given context
    pub fn is_available(&self, context: &LexerContext) -> bool {
        // Check version requirement
        if context.config.ecma_version < self.min_version {
            return false;
        }
        
        // Check strict mode requirement
        if self.strict_only && !context.config.strict_mode {
            return false;
        }
        
        true
    }
    
    /// Check if this keyword is reserved in the given context
    pub fn is_reserved(&self, context: &LexerContext) -> bool {
        if !self.is_available(context) {
            return false;
        }
        
        match self.category {
            KeywordCategory::Reserved => true,
            KeywordCategory::StrictReserved => context.config.strict_mode,
            KeywordCategory::FutureReserved => true,
            KeywordCategory::Contextual => false, // Contextual keywords are not always reserved
            KeywordCategory::Literal => false, // Literals are not reserved words
        }
    }
}

/// Static keyword database
static KEYWORD_DATABASE: LazyLock<HashMap<&'static str, KeywordInfo>> = LazyLock::new(|| {
    use KeywordCategory::*;
    use EcmaVersion::*;
    
    let keywords = [
        // ES5 reserved words
        KeywordInfo::new("break", Reserved, ES5, false, false),
        KeywordInfo::new("case", Reserved, ES5, false, false),
        KeywordInfo::new("catch", Reserved, ES5, false, false),
        KeywordInfo::new("continue", Reserved, ES5, false, false),
        KeywordInfo::new("debugger", Reserved, ES5, false, false),
        KeywordInfo::new("default", Reserved, ES5, false, false),
        KeywordInfo::new("delete", Reserved, ES5, false, false),
        KeywordInfo::new("do", Reserved, ES5, false, false),
        KeywordInfo::new("else", Reserved, ES5, false, false),
        KeywordInfo::new("finally", Reserved, ES5, false, false),
        KeywordInfo::new("for", Reserved, ES5, false, false),
        KeywordInfo::new("function", Reserved, ES5, false, false),
        KeywordInfo::new("if", Reserved, ES5, false, false),
        KeywordInfo::new("in", Reserved, ES5, false, false),
        KeywordInfo::new("instanceof", Reserved, ES5, false, false),
        KeywordInfo::new("new", Reserved, ES5, false, false),
        KeywordInfo::new("return", Reserved, ES5, false, false),
        KeywordInfo::new("switch", Reserved, ES5, false, false),
        KeywordInfo::new("this", Reserved, ES5, false, false),
        KeywordInfo::new("throw", Reserved, ES5, false, false),
        KeywordInfo::new("try", Reserved, ES5, false, false),
        KeywordInfo::new("typeof", Reserved, ES5, false, false),
        KeywordInfo::new("var", Reserved, ES5, false, false),
        KeywordInfo::new("void", Reserved, ES5, false, false),
        KeywordInfo::new("while", Reserved, ES5, false, false),
        KeywordInfo::new("with", Reserved, ES5, false, false),
        
        // ES5 strict mode reserved words
        KeywordInfo::new("implements", StrictReserved, ES5, true, false),
        KeywordInfo::new("interface", StrictReserved, ES5, true, false),
        KeywordInfo::new("let", StrictReserved, ES5, true, false),
        KeywordInfo::new("package", StrictReserved, ES5, true, false),
        KeywordInfo::new("private", StrictReserved, ES5, true, false),
        KeywordInfo::new("protected", StrictReserved, ES5, true, false),
        KeywordInfo::new("public", StrictReserved, ES5, true, false),
        KeywordInfo::new("static", StrictReserved, ES5, true, false),
        KeywordInfo::new("yield", StrictReserved, ES5, true, false),
        
        // ES5 future reserved words
        KeywordInfo::new("enum", FutureReserved, ES5, false, false),
        
        // ES2015 keywords
        KeywordInfo::new("class", Reserved, ES2015, false, false),
        KeywordInfo::new("const", Reserved, ES2015, false, false),
        KeywordInfo::new("export", Reserved, ES2015, false, false),
        KeywordInfo::new("extends", Reserved, ES2015, false, false),
        KeywordInfo::new("import", Reserved, ES2015, false, false),
        KeywordInfo::new("super", Reserved, ES2015, false, false),
        
        // ES2015 contextual keywords
        KeywordInfo::new("from", Contextual, ES2015, false, true),
        KeywordInfo::new("of", Contextual, ES2015, false, true),
        KeywordInfo::new("as", Contextual, ES2015, false, true),
        KeywordInfo::new("target", Contextual, ES2015, false, true),
        
        // ES2017 contextual keywords
        KeywordInfo::new("async", Contextual, ES2017, false, true),
        KeywordInfo::new("await", Contextual, ES2017, false, true),
        
        // ES2020 contextual keywords
        KeywordInfo::new("meta", Contextual, ES2020, false, true),
        
        // Literals (not keywords but handled similarly)
        KeywordInfo::new("true", Literal, ES5, false, false),
        KeywordInfo::new("false", Literal, ES5, false, false),
        KeywordInfo::new("null", Literal, ES5, false, false),
        KeywordInfo::new("undefined", Literal, ES5, false, false),
    ];
    
    keywords.into_iter().map(|info| (info.keyword, info)).collect()
});

/// Fast keyword lookup and classification system
#[derive(Debug)]
pub struct KeywordSystem {
    /// Cached keyword sets for different contexts
    cached_reserved: HashMap<(EcmaVersion, bool), HashSet<&'static str>>,
    /// Cached contextual keyword sets
    cached_contextual: HashMap<EcmaVersion, HashSet<&'static str>>,
}

impl Default for KeywordSystem {
    fn default() -> Self {
        Self::new()
    }
}

impl KeywordSystem {
    /// Create a new keyword system
    pub fn new() -> Self {
        Self {
            cached_reserved: HashMap::new(),
            cached_contextual: HashMap::new(),
        }
    }
    
    /// Look up keyword information
    pub fn lookup(&self, word: &str) -> Option<&KeywordInfo> {
        KEYWORD_DATABASE.get(word)
    }
    
    /// Check if a word is a keyword in the given context
    pub fn is_keyword(&self, word: &str, context: &LexerContext) -> bool {
        if let Some(info) = self.lookup(word) {
            info.is_reserved(context)
        } else {
            false
        }
    }
    
    /// Check if a word is a contextual keyword
    pub fn is_contextual_keyword(&self, word: &str, context: &LexerContext) -> bool {
        if let Some(info) = self.lookup(word) {
            info.is_available(context) && info.contextual
        } else {
            false
        }
    }
    
    /// Check if a word is a literal
    pub fn is_literal(&self, word: &str) -> bool {
        if let Some(info) = self.lookup(word) {
            matches!(info.category, KeywordCategory::Literal)
        } else {
            false
        }
    }
    
    /// Get all reserved words for a given context
    pub fn get_reserved_words(&mut self, context: &LexerContext) -> &HashSet<&'static str> {
        let key = (context.config.ecma_version, context.config.strict_mode);
        
        self.cached_reserved.entry(key).or_insert_with(|| {
            KEYWORD_DATABASE
                .values()
                .filter(|info| info.is_reserved(context))
                .map(|info| info.keyword)
                .collect()
        })
    }
    
    /// Get all contextual keywords for a given ECMAScript version
    pub fn get_contextual_keywords(&mut self, version: EcmaVersion) -> &HashSet<&'static str> {
        self.cached_contextual.entry(version).or_insert_with(|| {
            KEYWORD_DATABASE
                .values()
                .filter(|info| info.min_version <= version && info.contextual)
                .map(|info| info.keyword)
                .collect()
        })
    }
    
    /// Get keyword category
    pub fn get_category(&self, word: &str) -> Option<KeywordCategory> {
        self.lookup(word).map(|info| info.category)
    }
    
    /// Check if a word is available in the given ECMAScript version
    pub fn is_available_in_version(&self, word: &str, version: EcmaVersion) -> bool {
        if let Some(info) = self.lookup(word) {
            info.min_version <= version
        } else {
            false
        }
    }
    
    /// Get all keywords by category
    pub fn get_keywords_by_category(&self, category: KeywordCategory) -> Vec<&'static str> {
        KEYWORD_DATABASE
            .values()
            .filter(|info| info.category == category)
            .map(|info| info.keyword)
            .collect()
    }
    
    /// Get keywords introduced in a specific version
    pub fn get_keywords_by_version(&self, version: EcmaVersion) -> Vec<&'static str> {
        KEYWORD_DATABASE
            .values()
            .filter(|info| info.min_version == version)
            .map(|info| info.keyword)
            .collect()
    }
    
    /// Check if identifier conflicts with any keyword in any supported version
    pub fn conflicts_with_future_keywords(&self, word: &str) -> bool {
        KEYWORD_DATABASE.contains_key(word)
    }
    
    /// Get suggestions for misspelled keywords
    pub fn suggest_corrections(&self, word: &str) -> Vec<&'static str> {
        let word_lower = word.to_lowercase();
        let mut suggestions = Vec::new();
        
        for &keyword in KEYWORD_DATABASE.keys() {
            // Simple edit distance heuristic
            if self.edit_distance(&word_lower, keyword) <= 2 {
                suggestions.push(keyword);
            }
        }
        
        suggestions.sort_by_key(|&s| self.edit_distance(&word_lower, s));
        suggestions.truncate(3); // Limit to top 3 suggestions
        suggestions
    }
    
    /// Simple edit distance calculation
    fn edit_distance(&self, a: &str, b: &str) -> usize {
        let a_chars: Vec<char> = a.chars().collect();
        let b_chars: Vec<char> = b.chars().collect();
        let a_len = a_chars.len();
        let b_len = b_chars.len();
        
        let mut dp = vec![vec![0; b_len + 1]; a_len + 1];
        
        for i in 0..=a_len {
            dp[i][0] = i;
        }
        for j in 0..=b_len {
            dp[0][j] = j;
        }
        
        for i in 1..=a_len {
            for j in 1..=b_len {
                let cost = if a_chars[i - 1] == b_chars[j - 1] { 0 } else { 1 };
                dp[i][j] = std::cmp::min(
                    std::cmp::min(dp[i - 1][j] + 1, dp[i][j - 1] + 1),
                    dp[i - 1][j - 1] + cost,
                );
            }
        }
        
        dp[a_len][b_len]
    }
}

/// Global keyword system instance
static KEYWORD_SYSTEM: LazyLock<std::sync::Mutex<KeywordSystem>> = 
    LazyLock::new(|| std::sync::Mutex::new(KeywordSystem::new()));

/// Convenience functions for quick keyword checking
pub fn is_keyword(word: &str, context: &LexerContext) -> bool {
    KEYWORD_SYSTEM.lock().unwrap().is_keyword(word, context)
}

pub fn is_contextual_keyword(word: &str, context: &LexerContext) -> bool {
    KEYWORD_SYSTEM.lock().unwrap().is_contextual_keyword(word, context)
}

pub fn is_literal(word: &str) -> bool {
    KEYWORD_SYSTEM.lock().unwrap().is_literal(word)
}

pub fn lookup_keyword(word: &str) -> Option<KeywordInfo> {
    KEYWORD_DATABASE.get(word).cloned()
}

pub fn suggest_keyword_corrections(word: &str) -> Vec<&'static str> {
    KEYWORD_SYSTEM.lock().unwrap().suggest_corrections(word)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::core::{LexerContext, LexerConfigBuilder};
    
    #[test]
    fn test_keyword_lookup() {
        let system = KeywordSystem::new();
        
        let info = system.lookup("function").unwrap();
        assert_eq!(info.keyword, "function");
        assert_eq!(info.category, KeywordCategory::Reserved);
        assert_eq!(info.min_version, EcmaVersion::ES5);
        assert!(!info.strict_only);
        
        assert!(system.lookup("nonexistent").is_none());
    }
    
    #[test]
    fn test_es5_keywords() {
        let context = LexerContext::with_config(
            LexerConfigBuilder::new()
                .ecma_version(EcmaVersion::ES5)
                .build()
        );
        let system = KeywordSystem::new();
        
        assert!(system.is_keyword("function", &context));
        assert!(system.is_keyword("var", &context));
        assert!(system.is_keyword("if", &context));
        assert!(!system.is_keyword("class", &context)); // ES2015
        assert!(!system.is_keyword("hello", &context));
    }
    
    #[test]
    fn test_es2015_keywords() {
        let context = LexerContext::with_config(
            LexerConfigBuilder::new()
                .ecma_version(EcmaVersion::ES2015)
                .build()
        );
        let system = KeywordSystem::new();
        
        assert!(system.is_keyword("function", &context));
        assert!(system.is_keyword("class", &context));
        assert!(system.is_keyword("const", &context));
        assert!(system.is_keyword("let", &context));
        assert!(!system.is_keyword("async", &context)); // Contextual, not reserved
    }
    
    #[test]
    fn test_strict_mode_keywords() {
        let strict_context = LexerContext::with_config(
            LexerConfigBuilder::new()
                .ecma_version(EcmaVersion::ES5)
                .strict_mode(true)
                .build()
        );
        let normal_context = LexerContext::with_config(
            LexerConfigBuilder::new()
                .ecma_version(EcmaVersion::ES5)
                .strict_mode(false)
                .build()
        );
        let system = KeywordSystem::new();
        
        assert!(system.is_keyword("implements", &strict_context));
        assert!(!system.is_keyword("implements", &normal_context));
        
        assert!(system.is_keyword("private", &strict_context));
        assert!(!system.is_keyword("private", &normal_context));
    }
    
    #[test]
    fn test_contextual_keywords() {
        let context = LexerContext::with_config(
            LexerConfigBuilder::new()
                .ecma_version(EcmaVersion::ES2017)
                .build()
        );
        let system = KeywordSystem::new();
        
        assert!(system.is_contextual_keyword("async", &context));
        assert!(system.is_contextual_keyword("await", &context));
        assert!(system.is_contextual_keyword("from", &context));
        assert!(system.is_contextual_keyword("of", &context));
        
        assert!(!system.is_contextual_keyword("function", &context)); // Regular keyword
        assert!(!system.is_contextual_keyword("hello", &context)); // Not a keyword
    }
    
    #[test]
    fn test_literals() {
        let system = KeywordSystem::new();
        
        assert!(system.is_literal("true"));
        assert!(system.is_literal("false"));
        assert!(system.is_literal("null"));
        assert!(system.is_literal("undefined"));
        assert!(!system.is_literal("function"));
    }
    
    #[test]
    fn test_keyword_categories() {
        let system = KeywordSystem::new();
        
        assert_eq!(system.get_category("function"), Some(KeywordCategory::Reserved));
        assert_eq!(system.get_category("async"), Some(KeywordCategory::Contextual));
        assert_eq!(system.get_category("true"), Some(KeywordCategory::Literal));
        assert_eq!(system.get_category("implements"), Some(KeywordCategory::StrictReserved));
        assert_eq!(system.get_category("enum"), Some(KeywordCategory::FutureReserved));
        assert_eq!(system.get_category("nonexistent"), None);
    }
    
    #[test]
    fn test_version_availability() {
        let system = KeywordSystem::new();
        
        assert!(system.is_available_in_version("function", EcmaVersion::ES5));
        assert!(system.is_available_in_version("class", EcmaVersion::ES2015));
        assert!(!system.is_available_in_version("class", EcmaVersion::ES5));
        assert!(system.is_available_in_version("async", EcmaVersion::ES2017));
        assert!(!system.is_available_in_version("async", EcmaVersion::ES2015));
    }
    
    #[test]
    fn test_keywords_by_category() {
        let system = KeywordSystem::new();
        
        let reserved = system.get_keywords_by_category(KeywordCategory::Reserved);
        assert!(reserved.contains(&"function"));
        assert!(reserved.contains(&"var"));
        assert!(reserved.contains(&"class"));
        
        let contextual = system.get_keywords_by_category(KeywordCategory::Contextual);
        assert!(contextual.contains(&"async"));
        assert!(contextual.contains(&"await"));
        assert!(contextual.contains(&"from"));
        
        let literals = system.get_keywords_by_category(KeywordCategory::Literal);
        assert!(literals.contains(&"true"));
        assert!(literals.contains(&"false"));
        assert!(literals.contains(&"null"));
    }
    
    #[test]
    fn test_keywords_by_version() {
        let system = KeywordSystem::new();
        
        let es2015_keywords = system.get_keywords_by_version(EcmaVersion::ES2015);
        assert!(es2015_keywords.contains(&"class"));
        assert!(es2015_keywords.contains(&"const"));
        assert!(!es2015_keywords.contains(&"function")); // ES5
        assert!(!es2015_keywords.contains(&"async")); // ES2017
    }
    
    #[test]
    fn test_keyword_suggestions() {
        let system = KeywordSystem::new();
        
        let suggestions = system.suggest_corrections("functon");
        assert!(suggestions.contains(&"function"));
        
        let suggestions = system.suggest_corrections("clas");
        assert!(suggestions.contains(&"class"));
        
        let suggestions = system.suggest_corrections("varr");
        assert!(suggestions.contains(&"var"));
    }
    
    #[test]
    fn test_future_keyword_conflicts() {
        let system = KeywordSystem::new();
        
        assert!(system.conflicts_with_future_keywords("async"));
        assert!(system.conflicts_with_future_keywords("class"));
        assert!(system.conflicts_with_future_keywords("enum"));
        assert!(!system.conflicts_with_future_keywords("hello"));
        assert!(!system.conflicts_with_future_keywords("myFunction"));
    }
    
    #[test]
    fn test_convenience_functions() {
        let context = LexerContext::with_config(
            LexerConfigBuilder::new()
                .ecma_version(EcmaVersion::ES2017)
                .build()
        );
        
        assert!(is_keyword("function", &context));
        assert!(is_contextual_keyword("async", &context));
        assert!(is_literal("true"));
        
        let info = lookup_keyword("function").unwrap();
        assert_eq!(info.keyword, "function");
        
        let suggestions = suggest_keyword_corrections("functon");
        assert!(!suggestions.is_empty());
    }
}