//! Lexer context and configuration management
//!
//! Provides configuration options and shared state for the lexer
//! and its component scanners.

use std::collections::HashSet;

/// Configuration flags for lexer behavior
#[derive(Debug, Clone)]
pub struct LexerConfig {
    /// Allow Unicode identifiers beyond ASCII
    pub unicode_identifiers: bool,
    /// Strict mode parsing (affects keyword recognition)
    pub strict_mode: bool,
    /// Target ECMAScript version
    pub ecma_version: EcmaVersion,
    /// Allow JSX syntax
    pub jsx: bool,
    /// Allow experimental features
    pub experimental: bool,
    /// Case sensitivity for identifiers
    pub case_sensitive: bool,
}

impl Default for LexerConfig {
    fn default() -> Self {
        Self {
            unicode_identifiers: true,
            strict_mode: false,
            ecma_version: EcmaVersion::ES2020,
            jsx: false,
            experimental: false,
            case_sensitive: true,
        }
    }
}

/// Supported ECMAScript versions
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum EcmaVersion {
    ES5,
    ES2015,
    ES2016,
    ES2017,
    ES2018,
    ES2019,
    ES2020,
    ES2021,
    ES2022,
    ES2023,
    ESNext,
}

impl EcmaVersion {
    /// Check if version supports specific features
    pub fn supports_async_await(self) -> bool {
        self >= EcmaVersion::ES2017
    }
    
    pub fn supports_bigint(self) -> bool {
        self >= EcmaVersion::ES2020
    }
    
    pub fn supports_optional_chaining(self) -> bool {
        self >= EcmaVersion::ES2020
    }
    
    pub fn supports_nullish_coalescing(self) -> bool {
        self >= EcmaVersion::ES2020
    }
    
    pub fn supports_private_fields(self) -> bool {
        self >= EcmaVersion::ES2022
    }
}

/// Lexer context holding configuration and state
#[derive(Debug, Clone)]
pub struct LexerContext {
    /// Configuration options
    pub config: LexerConfig,
    /// Reserved words based on context
    reserved_words: HashSet<String>,
    /// Contextual keywords (may be identifiers in some contexts)
    contextual_keywords: HashSet<String>,
    /// Current nesting level for tracking context
    nesting_level: usize,
    /// Whether we're inside a template literal
    in_template: bool,
    /// Whether we're inside a regular expression
    in_regex: bool,
}

impl LexerContext {
    /// Create new lexer context with default configuration
    pub fn new() -> Self {
        Self::with_config(LexerConfig::default())
    }
    
    /// Create lexer context with specific configuration
    pub fn with_config(config: LexerConfig) -> Self {
        let mut context = Self {
            config,
            reserved_words: HashSet::new(),
            contextual_keywords: HashSet::new(),
            nesting_level: 0,
            in_template: false,
            in_regex: false,
        };
        
        context.initialize_reserved_words();
        context
    }
    
    /// Initialize reserved words based on configuration
    fn initialize_reserved_words(&mut self) {
        // ECMAScript 5 reserved words
        let es5_reserved = [
            "break", "case", "catch", "continue", "debugger", "default", "delete",
            "do", "else", "finally", "for", "function", "if", "in", "instanceof",
            "new", "return", "switch", "this", "throw", "try", "typeof", "var",
            "void", "while", "with",
        ];
        
        for word in &es5_reserved {
            self.reserved_words.insert(word.to_string());
        }
        
        // Additional ES6+ reserved words
        if self.config.ecma_version >= EcmaVersion::ES2015 {
            let es6_reserved = [
                "class", "const", "enum", "export", "extends", "import", "super",
                "implements", "interface", "let", "package", "private", "protected",
                "public", "static", "yield",
            ];
            
            for word in &es6_reserved {
                self.reserved_words.insert(word.to_string());
            }
        }
        
        // Strict mode additional reserved words
        if self.config.strict_mode {
            let strict_reserved = [
                "implements", "interface", "let", "package", "private", "protected",
                "public", "static", "yield",
            ];
            
            for word in &strict_reserved {
                self.reserved_words.insert(word.to_string());
            }
        }
        
        // Contextual keywords (may be identifiers in some contexts)
        if self.config.ecma_version >= EcmaVersion::ES2017 {
            self.contextual_keywords.insert("async".to_string());
            self.contextual_keywords.insert("await".to_string());
        }
        
        if self.config.ecma_version >= EcmaVersion::ES2015 {
            self.contextual_keywords.insert("from".to_string());
            self.contextual_keywords.insert("of".to_string());
            self.contextual_keywords.insert("as".to_string());
        }
    }
    
    /// Check if identifier is a reserved word
    pub fn is_reserved_word(&self, identifier: &str) -> bool {
        self.reserved_words.contains(identifier)
    }
    
    /// Check if identifier is a contextual keyword
    pub fn is_contextual_keyword(&self, identifier: &str) -> bool {
        self.contextual_keywords.contains(identifier)
    }
    
    /// Enter a new nesting level (e.g., entering braces)
    pub fn enter_nesting(&mut self) {
        self.nesting_level += 1;
    }
    
    /// Exit a nesting level
    pub fn exit_nesting(&mut self) {
        if self.nesting_level > 0 {
            self.nesting_level -= 1;
        }
    }
    
    /// Get current nesting level
    pub fn nesting_level(&self) -> usize {
        self.nesting_level
    }
    
    /// Enter template literal context
    pub fn enter_template(&mut self) {
        self.in_template = true;
    }
    
    /// Exit template literal context
    pub fn exit_template(&mut self) {
        self.in_template = false;
    }
    
    /// Check if inside template literal
    pub fn in_template(&self) -> bool {
        self.in_template
    }
    
    /// Enter regular expression context
    pub fn enter_regex(&mut self) {
        self.in_regex = true;
    }
    
    /// Exit regular expression context
    pub fn exit_regex(&mut self) {
        self.in_regex = false;
    }
    
    /// Check if inside regular expression
    pub fn in_regex(&self) -> bool {
        self.in_regex
    }
    
    /// Reset context state (keep configuration)
    pub fn reset(&mut self) {
        self.nesting_level = 0;
        self.in_template = false;
        self.in_regex = false;
    }
}

impl Default for LexerContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Builder for lexer configuration
#[derive(Debug)]
pub struct LexerConfigBuilder {
    config: LexerConfig,
}

impl LexerConfigBuilder {
    /// Create new configuration builder
    pub fn new() -> Self {
        Self {
            config: LexerConfig::default(),
        }
    }
    
    /// Enable or disable Unicode identifiers
    pub fn unicode_identifiers(mut self, enabled: bool) -> Self {
        self.config.unicode_identifiers = enabled;
        self
    }
    
    /// Enable or disable strict mode
    pub fn strict_mode(mut self, enabled: bool) -> Self {
        self.config.strict_mode = enabled;
        self
    }
    
    /// Set ECMAScript version
    pub fn ecma_version(mut self, version: EcmaVersion) -> Self {
        self.config.ecma_version = version;
        self
    }
    
    /// Enable or disable JSX support
    pub fn jsx(mut self, enabled: bool) -> Self {
        self.config.jsx = enabled;
        self
    }
    
    /// Enable or disable experimental features
    pub fn experimental(mut self, enabled: bool) -> Self {
        self.config.experimental = enabled;
        self
    }
    
    /// Set case sensitivity
    pub fn case_sensitive(mut self, enabled: bool) -> Self {
        self.config.case_sensitive = enabled;
        self
    }
    
    /// Build the final configuration
    pub fn build(self) -> LexerConfig {
        self.config
    }
}

impl Default for LexerConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_default_config() {
        let config = LexerConfig::default();
        assert!(config.unicode_identifiers);
        assert!(!config.strict_mode);
        assert_eq!(config.ecma_version, EcmaVersion::ES2020);
        assert!(!config.jsx);
    }
    
    #[test]
    fn test_config_builder() {
        let config = LexerConfigBuilder::new()
            .strict_mode(true)
            .ecma_version(EcmaVersion::ES2022)
            .jsx(true)
            .build();
        
        assert!(config.strict_mode);
        assert_eq!(config.ecma_version, EcmaVersion::ES2022);
        assert!(config.jsx);
    }
    
    #[test]
    fn test_reserved_words() {
        let context = LexerContext::new();
        
        // ES5 reserved words
        assert!(context.is_reserved_word("function"));
        assert!(context.is_reserved_word("var"));
        assert!(context.is_reserved_word("if"));
        
        // Non-reserved
        assert!(!context.is_reserved_word("hello"));
        assert!(!context.is_reserved_word("world"));
    }
    
    #[test]
    fn test_contextual_keywords() {
        let config = LexerConfigBuilder::new()
            .ecma_version(EcmaVersion::ES2017)
            .build();
        let context = LexerContext::with_config(config);
        
        assert!(context.is_contextual_keyword("async"));
        assert!(context.is_contextual_keyword("await"));
        assert!(!context.is_contextual_keyword("function"));
    }
    
    #[test]
    fn test_nesting_tracking() {
        let mut context = LexerContext::new();
        
        assert_eq!(context.nesting_level(), 0);
        
        context.enter_nesting();
        assert_eq!(context.nesting_level(), 1);
        
        context.enter_nesting();
        assert_eq!(context.nesting_level(), 2);
        
        context.exit_nesting();
        assert_eq!(context.nesting_level(), 1);
        
        context.exit_nesting();
        assert_eq!(context.nesting_level(), 0);
        
        // Should not go below 0
        context.exit_nesting();
        assert_eq!(context.nesting_level(), 0);
    }
    
    #[test]
    fn test_context_flags() {
        let mut context = LexerContext::new();
        
        assert!(!context.in_template());
        assert!(!context.in_regex());
        
        context.enter_template();
        assert!(context.in_template());
        
        context.enter_regex();
        assert!(context.in_regex());
        
        context.exit_template();
        assert!(!context.in_template());
        assert!(context.in_regex());
        
        context.exit_regex();
        assert!(!context.in_regex());
    }
    
    #[test]
    fn test_ecma_version_features() {
        assert!(!EcmaVersion::ES5.supports_async_await());
        assert!(EcmaVersion::ES2017.supports_async_await());
        
        assert!(!EcmaVersion::ES2018.supports_bigint());
        assert!(EcmaVersion::ES2020.supports_bigint());
        
        assert!(EcmaVersion::ES2022.supports_private_fields());
        assert!(!EcmaVersion::ES2020.supports_private_fields());
    }
}