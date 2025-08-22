//! Validation utilities for JavaScript lexer
//!
//! Provides comprehensive validation for tokens, identifiers, literals,
//! and other lexical constructs with detailed error reporting and
//! suggestions for common mistakes.

use crate::error::{Result, Span};
use crate::lexer::OperatorToken;
use crate::lexer::core::{LexerContext, EcmaVersion};
use crate::lexer::tokens::{Token, TokenKind};
use crate::lexer::utils::unicode::UnicodeHelper;
use std::collections::HashSet;

/// Lexer validation engine
pub struct LexerValidator {
    /// Strict mode validation rules
    strict_mode: bool,
    /// Target ECMAScript version
    ecma_version: EcmaVersion,
    /// Collected validation warnings
    warnings: Vec<ValidationWarning>,
    /// Collected validation errors
    errors: Vec<ValidationError>,
}

/// Validation warning (non-fatal issues)
#[derive(Debug, Clone, PartialEq)]
pub struct ValidationWarning {
    /// Warning message
    pub message: String,
    /// Location of the warning
    pub span: Span,
    /// Warning category
    pub category: WarningCategory,
    /// Suggested fix
    pub suggestion: Option<String>,
}

/// Validation error (fatal issues)
#[derive(Debug, Clone, PartialEq)]
pub struct ValidationError {
    /// Error message
    pub message: String,
    /// Location of the error
    pub span: Span,
    /// Error category
    pub category: ErrorCategory,
    /// Suggested fix
    pub suggestion: Option<String>,
}

/// Warning categories
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WarningCategory {
    /// Deprecated feature usage
    Deprecated,
    /// Potentially confusing identifier
    ConfusingIdentifier,
    /// Unnecessary escape sequence
    UnnecessaryEscape,
    /// Potential Unicode issue
    Unicode,
    /// Style/convention issue
    Style,
    /// Performance consideration
    Performance,
    /// Version compatibility issue
    Version,
}

/// Error categories
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorCategory {
    /// Syntax error
    Syntax,
    /// Invalid character or sequence
    InvalidCharacter,
    /// Unicode-related error
    Unicode,
    /// Version compatibility error
    Version,
    /// Strict mode violation
    StrictMode,
    /// Invalid literal
    InvalidLiteral,
}

impl LexerValidator {
    /// Create a new validator
    pub fn new(context: &LexerContext) -> Self {
        Self {
            strict_mode: context.config.strict_mode,
            ecma_version: context.config.ecma_version,
            warnings: Vec::new(),
            errors: Vec::new(),
        }
    }
    
    /// Validate a single token
    pub fn validate_token(&mut self, token: &Token) -> Result<()> {
        match &token.kind {
            TokenKind::Identifier(name) => self.validate_identifier(name, &token.span),
            TokenKind::Number(literal) => self.validate_number_literal(literal, &token.span),
            TokenKind::String(literal) => self.validate_string_literal(literal, &token.span),
            TokenKind::Keyword(keyword) => self.validate_keyword_usage(keyword, &token.span),
            TokenKind::ContextualKeyword(keyword) => self.validate_contextual_keyword(keyword, &token.span),
            _ => Ok(()),
        }
    }
    
    /// Validate a sequence of tokens
    pub fn validate_tokens(&mut self, tokens: &[Token]) -> Result<()> {
        for token in tokens {
            self.validate_token(token)?;
        }
        
        // Additional sequence validations
        self.validate_token_sequence(tokens)?;
        
        Ok(())
    }
    
    /// Validate an identifier
    fn validate_identifier(&mut self, name: &str, span: &Span) -> Result<()> {
        // Check basic Unicode validity
        match UnicodeHelper::validate_identifier(name) {
            Ok(()) => {},
            Err(unicode_error) => {
                self.add_error(
                    ErrorCategory::Unicode,
                    format!("Invalid identifier: {}", unicode_error),
                    span.clone(),
                    None,
                );
                return Ok(()); // Continue validation
            }
        }
        
        // Check for confusing identifiers
        if self.is_confusing_identifier(name) {
            self.add_warning(
                WarningCategory::ConfusingIdentifier,
                format!("Potentially confusing identifier: '{}'", name),
                span.clone(),
                Some("Consider using a clearer name".to_string()),
            );
        }
        
        // Check for deprecated patterns
        if self.is_deprecated_identifier_pattern(name) {
            self.add_warning(
                WarningCategory::Deprecated,
                format!("Identifier '{}' follows a deprecated pattern", name),
                span.clone(),
                None,
            );
        }
        
        // Check for version-specific issues
        if self.has_version_issues(name) {
            self.add_warning(
                WarningCategory::Version,
                format!("Identifier '{}' may conflict with future keywords", name),
                span.clone(),
                Some("Consider renaming to avoid future conflicts".to_string()),
            );
        }
        
        Ok(())
    }
    
    /// Validate a number literal
    fn validate_number_literal(&mut self, literal: &crate::lexer::scanners::numbers::NumberLiteral, span: &Span) -> Result<()> {
        use crate::lexer::scanners::numbers::NumberLiteral;
        
        match literal {
            NumberLiteral::Number(value) => {
                // Check for precision issues
                if value.is_infinite() {
                    self.add_warning(
                        WarningCategory::Performance,
                        "Number literal results in infinity".to_string(),
                        span.clone(),
                        Some("Consider using a smaller value".to_string()),
                    );
                }
                
                if value.is_nan() {
                    self.add_error(
                        ErrorCategory::InvalidLiteral,
                        "Number literal results in NaN".to_string(),
                        span.clone(),
                        None,
                    );
                }
                
                // Check for precision loss
                if value.fract() == 0.0 && value.abs() > 2_f64.powi(53) {
                    self.add_warning(
                        WarningCategory::Performance,
                        "Large integer may lose precision".to_string(),
                        span.clone(),
                        Some("Consider using BigInt for large integers".to_string()),
                    );
                }
            }
            NumberLiteral::BigInt(_) => {
                // BigInt availability check
                if self.ecma_version < EcmaVersion::ES2020 {
                    self.add_error(
                        ErrorCategory::Version,
                        "BigInt literals require ES2020 or later".to_string(),
                        span.clone(),
                        Some("Use regular numbers or update target version".to_string()),
                    );
                }
            }
        }
        
        Ok(())
    }
    
    /// Validate a string literal
    fn validate_string_literal(&mut self, literal: &crate::lexer::scanners::strings::StringLiteral, span: &Span) -> Result<()> {
        use crate::lexer::scanners::strings::StringLiteral;
        
        match literal {
            StringLiteral::String(content) => {
                // Check for unnecessary escapes
                self.check_unnecessary_escapes(content, span);
                
                // Check for Unicode issues
                self.check_unicode_issues(content, span);
                
                // Check for very long strings
                if content.len() > 10000 {
                    self.add_warning(
                        WarningCategory::Performance,
                        "Very long string literal".to_string(),
                        span.clone(),
                        Some("Consider breaking into smaller strings or loading from external source".to_string()),
                    );
                }
            }
            StringLiteral::Template(_) => {
                // Template literal validation would go here
                // For now, just check version compatibility
                if self.ecma_version < EcmaVersion::ES2015 {
                    self.add_error(
                        ErrorCategory::Version,
                        "Template literals require ES2015 or later".to_string(),
                        span.clone(),
                        Some("Use string concatenation or update target version".to_string()),
                    );
                }
            }
        }
        
        Ok(())
    }
    
    /// Validate keyword usage
    fn validate_keyword_usage(&mut self, keyword: &str, span: &Span) -> Result<()> {
        // Check if keyword is available in target version
        match keyword {
            "class" | "const" | "let" | "super" | "extends" | "static" => {
                if self.ecma_version < EcmaVersion::ES2015 {
                    self.add_error(
                        ErrorCategory::Version,
                        format!("Keyword '{}' requires ES2015 or later", keyword),
                        span.clone(),
                        Some("Update target ECMAScript version".to_string()),
                    );
                }
            }
            "async" | "await" => {
                if self.ecma_version < EcmaVersion::ES2017 {
                    self.add_error(
                        ErrorCategory::Version,
                        format!("Keyword '{}' requires ES2017 or later", keyword),
                        span.clone(),
                        Some("Update target ECMAScript version".to_string()),
                    );
                }
            }
            _ => {}
        }
        
        Ok(())
    }
    
    /// Validate contextual keyword usage
    fn validate_contextual_keyword(&mut self, keyword: &str, span: &Span) -> Result<()> {
        match keyword {
            "async" | "await" => {
                if self.ecma_version < EcmaVersion::ES2017 {
                    self.add_warning(
                        WarningCategory::Version,
                        format!("Contextual keyword '{}' requires ES2017 or later", keyword),
                        span.clone(),
                        Some("Update target ECMAScript version or use regular identifier".to_string()),
                    );
                }
            }
            _ => {}
        }
        
        Ok(())
    }
    
    /// Validate token sequence for patterns and common issues
    fn validate_token_sequence(&mut self, tokens: &[Token]) -> Result<()> {
        // Check for common mistake patterns
        for window in tokens.windows(2) {
            if let [first, second] = window {
                self.check_adjacent_tokens(first, second);
            }
        }
        
        // Check for automatic semicolon insertion opportunities
        self.check_asi_opportunities(tokens);
        
        Ok(())
    }
    
    /// Check adjacent tokens for common issues
    fn check_adjacent_tokens(&mut self, first: &Token, second: &Token) {
        // Check for potential ASI issues
        if first.kind.is_statement_terminator() && second.can_start_expression() {
            if first.line() != second.line() {
                self.add_warning(
                    WarningCategory::Style,
                    "Potential automatic semicolon insertion".to_string(),
                    second.span.clone(),
                    Some("Add explicit semicolon".to_string()),
                );
            }
        }
        
        // Check for confusing operator sequences
        if let (TokenKind::Operator(op1), TokenKind::Operator(op2)) = (&first.kind, &second.kind) {
            if self.is_confusing_operator_sequence(op1, op2) {
                self.add_warning(
                    WarningCategory::Style,
                    "Potentially confusing operator sequence".to_string(),
                    Span::new(first.start(), second.end(), first.line(), first.column()),
                    Some("Add parentheses for clarity".to_string()),
                );
            }
        }
    }
    
    /// Check for automatic semicolon insertion opportunities
    fn check_asi_opportunities(&mut self, tokens: &[Token]) {
        // Implementation would analyze token patterns for ASI
        // This is a simplified check
        for token in tokens {
            if token.metadata.is_new_line && token.can_start_expression() {
                // Previous statement might be affected by ASI
                self.add_warning(
                    WarningCategory::Style,
                    "Statement may be affected by automatic semicolon insertion".to_string(),
                    token.span.clone(),
                    Some("Add explicit semicolon to previous statement".to_string()),
                );
            }
        }
    }
    
    /// Check if identifier is potentially confusing
    fn is_confusing_identifier(&self, name: &str) -> bool {
        // Check for lookalike characters
        let confusing_chars = ['℮', 'ℯ', '𝕖', '𝖊', '𝖾', '𝗲', '𝘦', '𝙚', '𝚎']; // Various 'e' lookalikes
        name.chars().any(|c| confusing_chars.contains(&c)) ||
        // Check for mixing scripts
        self.mixes_scripts(name) ||
        // Check for very similar to keywords
        self.similar_to_keyword(name)
    }
    
    /// Check if identifier mixes scripts
    fn mixes_scripts(&self, name: &str) -> bool {
        let mut has_latin = false;
        let mut has_other = false;
        
        for ch in name.chars() {
            if ch.is_ascii_alphabetic() {
                has_latin = true;
            } else if ch.is_alphabetic() && !ch.is_ascii() {
                has_other = true;
            }
        }
        
        has_latin && has_other
    }
    
    /// Check if identifier is similar to a keyword
    fn similar_to_keyword(&self, name: &str) -> bool {
        let keywords = [
            "function", "var", "let", "const", "if", "else", "for", "while",
            "return", "class", "async", "await", "import", "export",
        ];
        
        keywords.iter().any(|&keyword| {
            self.edit_distance(name, keyword) == 1 && name.len() > 2
        })
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
    
    /// Check for deprecated identifier patterns
    fn is_deprecated_identifier_pattern(&self, name: &str) -> bool {
        // Hungarian notation
        name.starts_with("str") || name.starts_with("int") || name.starts_with("obj") ||
        // Old-style private markers
        name.starts_with("__") ||
        // Common typos that are valid but likely mistakes
        name.ends_with("_")
    }
    
    /// Check for version-specific identifier issues
    fn has_version_issues(&self, name: &str) -> bool {
        // Future reserved words that might conflict
        let future_keywords = ["enum", "implements", "interface", "package"];
        future_keywords.contains(&name)
    }
    
    /// Check for unnecessary escape sequences in strings
    fn check_unnecessary_escapes(&mut self, content: &str, span: &Span) {
        if content.contains("\\'") && !content.contains('\'') {
            self.add_warning(
                WarningCategory::UnnecessaryEscape,
                "Unnecessary escape sequence in string".to_string(),
                span.clone(),
                Some("Remove unnecessary backslash".to_string()),
            );
        }
    }
    
    /// Check for Unicode issues in strings
    fn check_unicode_issues(&mut self, content: &str, span: &Span) {
        // Check for potentially problematic Unicode characters
        for ch in content.chars() {
            if ch.is_control() && !matches!(ch, '\t' | '\n' | '\r') {
                self.add_warning(
                    WarningCategory::Unicode,
                    format!("String contains control character: U+{:04X}", ch as u32),
                    span.clone(),
                    Some("Consider using escape sequence".to_string()),
                );
            }
        }
    }
    
    /// Check for confusing operator sequences
    fn is_confusing_operator_sequence(&self, _op1: &OperatorToken, _op2: &OperatorToken) -> bool {
        // Implementation would check for patterns like == vs ===, etc.
        false
    }
    
    /// Add a validation warning
    fn add_warning(&mut self, category: WarningCategory, message: String, span: Span, suggestion: Option<String>) {
        self.warnings.push(ValidationWarning {
            message,
            span,
            category,
            suggestion,
        });
    }
    
    /// Add a validation error
    fn add_error(&mut self, category: ErrorCategory, message: String, span: Span, suggestion: Option<String>) {
        self.errors.push(ValidationError {
            message,
            span,
            category,
            suggestion,
        });
    }
    
    /// Get all warnings
    pub fn warnings(&self) -> &[ValidationWarning] {
        &self.warnings
    }
    
    /// Get all errors
    pub fn errors(&self) -> &[ValidationError] {
        &self.errors
    }
    
    /// Check if validation passed (no errors)
    pub fn is_valid(&self) -> bool {
        self.errors.is_empty()
    }
    
    /// Clear all warnings and errors
    pub fn clear(&mut self) {
        self.warnings.clear();
        self.errors.clear();
    }
    
    /// Get validation summary
    pub fn summary(&self) -> ValidationSummary {
        ValidationSummary {
            error_count: self.errors.len(),
            warning_count: self.warnings.len(),
            has_unicode_issues: self.warnings.iter().any(|w| w.category == WarningCategory::Unicode),
            has_version_issues: self.errors.iter().any(|e| e.category == ErrorCategory::Version),
            has_strict_mode_issues: self.errors.iter().any(|e| e.category == ErrorCategory::StrictMode),
        }
    }
}

/// Validation summary
#[derive(Debug, Clone, PartialEq)]
pub struct ValidationSummary {
    pub error_count: usize,
    pub warning_count: usize,
    pub has_unicode_issues: bool,
    pub has_version_issues: bool,
    pub has_strict_mode_issues: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::core::{LexerContext, LexerConfigBuilder};
    use crate::lexer::tokens::TokenBuilder;
    use crate::lexer::{NumberLiteral, StringLiteral};
    
    #[test]
    fn test_identifier_validation() {
        let context = LexerContext::new();
        let mut validator = LexerValidator::new(&context);
        
        let span = Span::new(0, 5, 1, 1);
        
        // Valid identifier
        validator.validate_identifier("hello", &span).unwrap();
        assert!(validator.is_valid());
        
        // Confusing identifier (mixing scripts)
        validator.validate_identifier("hеllo", &span).unwrap(); // Cyrillic 'е'
        assert!(!validator.warnings().is_empty());
    }
    
    #[test]
    fn test_number_validation() {
        let context = LexerContext::new();
        let mut validator = LexerValidator::new(&context);
        
        let span = Span::new(0, 5, 1, 1);
        
        // Large number
        let large_number = NumberLiteral::Number(2_f64.powi(54));
        validator.validate_number_literal(&large_number, &span).unwrap();
        assert!(!validator.warnings().is_empty());
        
        // Infinity
        let infinity = NumberLiteral::Number(f64::INFINITY);
        validator.validate_number_literal(&infinity, &span).unwrap();
        assert!(!validator.warnings().is_empty());
    }
    
    #[test]
    fn test_bigint_version_validation() {
        let context = LexerContext::with_config(
            LexerConfigBuilder::new()
                .ecma_version(EcmaVersion::ES5)
                .build()
        );
        let mut validator = LexerValidator::new(&context);
        
        let span = Span::new(0, 5, 1, 1);
        let bigint = NumberLiteral::BigInt("123n".to_string());
        
        validator.validate_number_literal(&bigint, &span).unwrap();
        assert!(!validator.errors().is_empty());
        assert_eq!(validator.errors()[0].category, ErrorCategory::Version);
    }
    
    #[test]
    fn test_string_validation() {
        let context = LexerContext::new();
        let mut validator = LexerValidator::new(&context);
        
        let span = Span::new(0, 5, 1, 1);
        
        // String with unnecessary escape
        let string_with_escape = StringLiteral::String("hello\\'world".to_string());
        validator.validate_string_literal(&string_with_escape, &span).unwrap();
        assert!(!validator.warnings().is_empty());
    }
    
    #[test]
    fn test_keyword_version_validation() {
        let context = LexerContext::with_config(
            LexerConfigBuilder::new()
                .ecma_version(EcmaVersion::ES5)
                .build()
        );
        let mut validator = LexerValidator::new(&context);
        
        let span = Span::new(0, 5, 1, 1);
        
        validator.validate_keyword_usage("class", &span).unwrap();
        assert!(!validator.errors().is_empty());
        assert_eq!(validator.errors()[0].category, ErrorCategory::Version);
    }
    
    #[test]
    fn test_validation_summary() {
        let context = LexerContext::new();
        let mut validator = LexerValidator::new(&context);
        
        let span = Span::new(0, 5, 1, 1);
        
        // Add some warnings and errors
        validator.add_warning(
            WarningCategory::Unicode,
            "Unicode issue".to_string(),
            span.clone(),
            None,
        );
        validator.add_error(
            ErrorCategory::Version,
            "Version issue".to_string(),
            span,
            None,
        );
        
        let summary = validator.summary();
        assert_eq!(summary.error_count, 1);
        assert_eq!(summary.warning_count, 1);
        assert!(summary.has_unicode_issues);
        assert!(summary.has_version_issues);
        assert!(!summary.has_strict_mode_issues);
    }
    
    #[test]
    fn test_confusing_identifier_detection() {
        let context = LexerContext::new();
        let validator = LexerValidator::new(&context);
        
        // Similar to keyword
        assert!(validator.similar_to_keyword("functon")); // typo in 'function'
        assert!(!validator.similar_to_keyword("myFunction"));
        
        // Mixed scripts
        assert!(validator.mixes_scripts("hеllo")); // Cyrillic 'е'
        assert!(!validator.mixes_scripts("hello"));
        assert!(!validator.mixes_scripts("привет")); // All Cyrillic
    }
    
    #[test]
    fn test_deprecated_patterns() {
        let context = LexerContext::new();
        let validator = LexerValidator::new(&context);
        
        assert!(validator.is_deprecated_identifier_pattern("strName"));
        assert!(validator.is_deprecated_identifier_pattern("__private"));
        assert!(validator.is_deprecated_identifier_pattern("value_"));
        assert!(!validator.is_deprecated_identifier_pattern("myVariable"));
    }
}