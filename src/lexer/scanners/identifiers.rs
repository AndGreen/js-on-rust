//! Identifier and keyword scanner
//!
//! Handles scanning of JavaScript identifiers and keywords including:
//! - ASCII identifiers (hello, _private, $jquery)
//! - Unicode identifiers (café, naïve)
//! - Keywords (function, var, let, const, etc.)
//! - Contextual keywords (async, await, from, of)
//! - Reserved words based on context

use super::{Scanner, LookaheadScanner, CharClassifier, EcmaCharClassifier, lexer_error};
use crate::error::{Result, Span};
use crate::lexer::core::{Input, LexerContext};

/// Identifier or keyword token
#[derive(Debug, Clone, PartialEq)]
pub enum IdentifierToken {
    /// Regular identifier
    Identifier(String),
    /// Reserved keyword
    Keyword(String),
    /// Contextual keyword (may be identifier in some contexts)
    ContextualKeyword(String),
    /// Boolean literal
    Boolean(bool),
    /// Null literal
    Null,
    /// Undefined literal
    Undefined,
}

/// Identifier scanner for JavaScript identifiers and keywords
#[derive(Debug, Default)]
pub struct IdentifierScanner;

impl IdentifierScanner {
    /// Create a new identifier scanner
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Scan an identifier or keyword
    fn scan_identifier(&self, input: &mut Input, context: &LexerContext) -> Result<IdentifierToken> {
        input.mark_token_start();
        
        // First character must be identifier start
        if !self.is_identifier_start(input.current_char(), context) {
            return Err(lexer_error(
                "Internal error: identifier scanner called on non-identifier character",
                input.current_token_span(),
            ));
        }
        
        // Consume first character
        input.advance();
        
        // Consume continuation characters
        while !input.is_at_end() && self.is_identifier_continue(input.current_char(), context) {
            input.advance();
        }
        
        let text = input.token_text().to_string();
        
        // Classify the identifier
        self.classify_identifier(text, context)
    }
    
    /// Check if character can start an identifier
    fn is_identifier_start(&self, ch: char, context: &LexerContext) -> bool {
        if ch.is_ascii_alphabetic() || ch == '_' || ch == '$' {
            true
        } else if context.config.unicode_identifiers {
            unicode_xid::UnicodeXID::is_xid_start(ch)
        } else {
            false
        }
    }
    
    /// Check if character can continue an identifier
    fn is_identifier_continue(&self, ch: char, context: &LexerContext) -> bool {
        if ch.is_ascii_alphanumeric() || ch == '_' || ch == '$' {
            true
        } else if context.config.unicode_identifiers {
            unicode_xid::UnicodeXID::is_xid_continue(ch)
        } else {
            false
        }
    }
    
    /// Classify an identifier as keyword, contextual keyword, or regular identifier
    fn classify_identifier(&self, text: String, context: &LexerContext) -> Result<IdentifierToken> {
        // Check for literal values first
        match text.as_str() {
            "true" => return Ok(IdentifierToken::Boolean(true)),
            "false" => return Ok(IdentifierToken::Boolean(false)),
            "null" => return Ok(IdentifierToken::Null),
            "undefined" => return Ok(IdentifierToken::Undefined),
            _ => {}
        }
        
        // Check if it's a reserved word
        if context.is_reserved_word(&text) {
            return Ok(IdentifierToken::Keyword(text));
        }
        
        // Check if it's a contextual keyword
        if context.is_contextual_keyword(&text) {
            return Ok(IdentifierToken::ContextualKeyword(text));
        }
        
        // Check for version-specific keywords
        if self.is_version_specific_keyword(&text, context) {
            return Ok(IdentifierToken::Keyword(text));
        }
        
        // Regular identifier
        Ok(IdentifierToken::Identifier(text))
    }
    
    /// Check if identifier is a keyword specific to certain ECMAScript versions
    fn is_version_specific_keyword(&self, text: &str, context: &LexerContext) -> bool {
        use crate::lexer::core::EcmaVersion;
        
        match text {
            // ES2015+ keywords
            "class" | "const" | "let" | "super" | "extends" | "static" |
            "import" | "export" | "default" => {
                context.config.ecma_version >= EcmaVersion::ES2015
            }
            
            // ES2017+ keywords
            "async" | "await" => {
                context.config.ecma_version >= EcmaVersion::ES2017
            }
            
            // Strict mode keywords
            "implements" | "interface" | "package" | "private" | 
            "protected" | "public" | "yield" => {
                context.config.strict_mode
            }
            
            _ => false,
        }
    }
    
    /// Validate identifier according to JavaScript rules
    fn validate_identifier(&self, text: &str, context: &LexerContext) -> Result<()> {
        if text.is_empty() {
            return Err(lexer_error(
                "Empty identifier",
                Span::new(0, 0, 1, 1),
            ));
        }
        
        let mut chars = text.chars();
        let first_char = chars.next().unwrap();
        
        if !self.is_identifier_start(first_char, context) {
            return Err(lexer_error(
                format!("Invalid identifier start character: '{}'", first_char),
                Span::new(0, first_char.len_utf8(), 1, 1),
            ));
        }
        
        for (i, ch) in chars.enumerate() {
            if !self.is_identifier_continue(ch, context) {
                return Err(lexer_error(
                    format!("Invalid identifier character: '{}'", ch),
                    Span::new(i + first_char.len_utf8(), i + first_char.len_utf8() + ch.len_utf8(), 1, 1),
                ));
            }
        }
        
        Ok(())
    }
}

impl Scanner for IdentifierScanner {
    type Token = IdentifierToken;
    
    fn try_scan(&mut self, input: &mut Input, context: &LexerContext) -> Option<Result<Self::Token>> {
        if self.can_scan(input, context) {
            Some(self.scan_identifier(input, context))
        } else {
            None
        }
    }
    
    fn can_scan(&self, input: &Input, context: &LexerContext) -> bool {
        self.is_identifier_start(input.current_char(), context)
    }
    
    fn name(&self) -> &'static str {
        "IdentifierScanner"
    }
}

impl LookaheadScanner for IdentifierScanner {
    fn would_match(&self, input: &mut Input, context: &LexerContext) -> bool {
        self.can_scan(input, context)
    }
    
    fn expected_length(&self, input: &mut Input, context: &LexerContext) -> Option<usize> {
        if !self.can_scan(input, context) {
            return None;
        }
        
        let start = input.byte_offset();
        let mut temp_input = input.clone();
        
        // Scan identifier characters
        if self.is_identifier_start(temp_input.current_char(), context) {
            temp_input.advance();
            
            while !temp_input.is_at_end() && self.is_identifier_continue(temp_input.current_char(), context) {
                temp_input.advance();
            }
        }
        
        Some(temp_input.byte_offset() - start)
    }
}

/// Helper functions for keyword checking
impl IdentifierScanner {
    /// Check if a string is a JavaScript keyword
    pub fn is_keyword(text: &str, context: &LexerContext) -> bool {
        matches!(
            text,
            "break" | "case" | "catch" | "continue" | "debugger" | "default" | "delete" |
            "do" | "else" | "finally" | "for" | "function" | "if" | "in" | "instanceof" |
            "new" | "return" | "switch" | "this" | "throw" | "try" | "typeof" | "var" |
            "void" | "while" | "with"
        ) || (context.config.ecma_version >= crate::lexer::core::EcmaVersion::ES2015 && matches!(
            text,
            "class" | "const" | "enum" | "export" | "extends" | "import" | "super" |
            "let" | "static" | "yield"
        )) || (context.config.strict_mode && matches!(
            text,
            "implements" | "interface" | "package" | "private" | "protected" | "public"
        ))
    }
    
    /// Check if a string is a future reserved word
    pub fn is_future_reserved_word(text: &str, context: &LexerContext) -> bool {
        matches!(text, "enum") ||
        (context.config.strict_mode && matches!(
            text,
            "implements" | "interface" | "package" | "private" | "protected" | "public"
        ))
    }
    
    /// Get all keywords for the current context
    pub fn get_keywords(context: &LexerContext) -> Vec<&'static str> {
        let mut keywords = vec![
            "break", "case", "catch", "continue", "debugger", "default", "delete",
            "do", "else", "finally", "for", "function", "if", "in", "instanceof",
            "new", "return", "switch", "this", "throw", "try", "typeof", "var",
            "void", "while", "with",
        ];
        
        if context.config.ecma_version >= crate::lexer::core::EcmaVersion::ES2015 {
            keywords.extend([
                "class", "const", "enum", "export", "extends", "import", "super",
                "let", "static", "yield",
            ]);
        }
        
        if context.config.strict_mode {
            keywords.extend([
                "implements", "interface", "package", "private", "protected", "public",
            ]);
        }
        
        keywords
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::core::{LexerContext, LexerConfigBuilder, EcmaVersion};
    
    fn scan_identifier(source: &str) -> Result<IdentifierToken> {
        let mut input = Input::new(source);
        let mut scanner = IdentifierScanner::new();
        let context = LexerContext::new();
        
        scanner.try_scan(&mut input, &context).unwrap()
    }
    
    fn scan_identifier_with_context(source: &str, context: &LexerContext) -> Result<IdentifierToken> {
        let mut input = Input::new(source);
        let mut scanner = IdentifierScanner::new();
        
        scanner.try_scan(&mut input, context).unwrap()
    }
    
    #[test]
    fn test_simple_identifiers() {
        match scan_identifier("hello").unwrap() {
            IdentifierToken::Identifier(s) => assert_eq!(s, "hello"),
            _ => panic!("Expected identifier"),
        }
        
        match scan_identifier("_private").unwrap() {
            IdentifierToken::Identifier(s) => assert_eq!(s, "_private"),
            _ => panic!("Expected identifier"),
        }
        
        match scan_identifier("$jquery").unwrap() {
            IdentifierToken::Identifier(s) => assert_eq!(s, "$jquery"),
            _ => panic!("Expected identifier"),
        }
    }
    
    #[test]
    fn test_keywords() {
        match scan_identifier("function").unwrap() {
            IdentifierToken::Keyword(s) => assert_eq!(s, "function"),
            _ => panic!("Expected keyword"),
        }
        
        match scan_identifier("var").unwrap() {
            IdentifierToken::Keyword(s) => assert_eq!(s, "var"),
            _ => panic!("Expected keyword"),
        }
        
        match scan_identifier("if").unwrap() {
            IdentifierToken::Keyword(s) => assert_eq!(s, "if"),
            _ => panic!("Expected keyword"),
        }
    }
    
    #[test]
    fn test_boolean_literals() {
        match scan_identifier("true").unwrap() {
            IdentifierToken::Boolean(true) => {},
            _ => panic!("Expected true boolean"),
        }
        
        match scan_identifier("false").unwrap() {
            IdentifierToken::Boolean(false) => {},
            _ => panic!("Expected false boolean"),
        }
    }
    
    #[test]
    fn test_null_undefined() {
        match scan_identifier("null").unwrap() {
            IdentifierToken::Null => {},
            _ => panic!("Expected null"),
        }
        
        match scan_identifier("undefined").unwrap() {
            IdentifierToken::Undefined => {},
            _ => panic!("Expected undefined"),
        }
    }
    
    #[test]
    fn test_es6_keywords() {
        let config = LexerConfigBuilder::new()
            .ecma_version(EcmaVersion::ES2015)
            .build();
        let context = LexerContext::with_config(config);
        
        match scan_identifier_with_context("class", &context).unwrap() {
            IdentifierToken::Keyword(s) => assert_eq!(s, "class"),
            _ => panic!("Expected keyword"),
        }
        
        match scan_identifier_with_context("const", &context).unwrap() {
            IdentifierToken::Keyword(s) => assert_eq!(s, "const"),
            _ => panic!("Expected keyword"),
        }
        
        match scan_identifier_with_context("let", &context).unwrap() {
            IdentifierToken::Keyword(s) => assert_eq!(s, "let"),
            _ => panic!("Expected keyword"),
        }
    }
    
    #[test]
    fn test_contextual_keywords() {
        let config = LexerConfigBuilder::new()
            .ecma_version(EcmaVersion::ES2017)
            .build();
        let context = LexerContext::with_config(config);
        
        match scan_identifier_with_context("async", &context).unwrap() {
            IdentifierToken::ContextualKeyword(s) => assert_eq!(s, "async"),
            _ => panic!("Expected contextual keyword"),
        }
        
        match scan_identifier_with_context("await", &context).unwrap() {
            IdentifierToken::ContextualKeyword(s) => assert_eq!(s, "await"),
            _ => panic!("Expected contextual keyword"),
        }
    }
    
    #[test]
    fn test_unicode_identifiers() {
        match scan_identifier("café").unwrap() {
            IdentifierToken::Identifier(s) => assert_eq!(s, "café"),
            _ => panic!("Expected identifier"),
        }
        
        match scan_identifier("naïve").unwrap() {
            IdentifierToken::Identifier(s) => assert_eq!(s, "naïve"),
            _ => panic!("Expected identifier"),
        }
    }
    
    #[test]
    fn test_unicode_disabled() {
        let config = LexerConfigBuilder::new()
            .unicode_identifiers(false)
            .build();
        let context = LexerContext::with_config(config);
        
        let scanner = IdentifierScanner::new();
        
        // Should not be able to scan Unicode identifiers
        assert!(!scanner.can_scan(&Input::new("café"), &context));
    }
    
    #[test]
    fn test_can_scan() {
        let scanner = IdentifierScanner::new();
        let context = LexerContext::new();
        
        assert!(scanner.can_scan(&Input::new("hello"), &context));
        assert!(scanner.can_scan(&Input::new("_private"), &context));
        assert!(scanner.can_scan(&Input::new("$jquery"), &context));
        assert!(scanner.can_scan(&Input::new("function"), &context));
        assert!(!scanner.can_scan(&Input::new("123"), &context));
        assert!(!scanner.can_scan(&Input::new("\"string\""), &context));
    }
    
    #[test]
    fn test_expected_length() {
        let mut scanner = IdentifierScanner::new();
        let context = LexerContext::new();
        
        let mut input = Input::new("hello");
        assert_eq!(scanner.expected_length(&mut input, &context), Some(5));
        
        let mut input2 = Input::new("_private123");
        assert_eq!(scanner.expected_length(&mut input2, &context), Some(11));
        
        let mut input3 = Input::new("$");
        assert_eq!(scanner.expected_length(&mut input3, &context), Some(1));
    }
    
    #[test]
    fn test_keyword_helpers() {
        let context = LexerContext::new();
        
        assert!(IdentifierScanner::is_keyword("function", &context));
        assert!(IdentifierScanner::is_keyword("var", &context));
        assert!(!IdentifierScanner::is_keyword("hello", &context));
        
        let keywords = IdentifierScanner::get_keywords(&context);
        assert!(keywords.contains(&"function"));
        assert!(keywords.contains(&"var"));
        assert!(keywords.contains(&"if"));
    }
}