//! Modular JavaScript lexer
//!
//! The main orchestrating lexer that coordinates all specialized scanners
//! to tokenize JavaScript source code efficiently and accurately.

use crate::error::{Error, Result};
use crate::lexer::core::{Input, LexerContext};
use crate::lexer::scanners::{
    Scanner, NumberScanner, StringScanner, IdentifierScanner, 
    OperatorScanner, CommentScanner,
};
use crate::lexer::tokens::{Token, TokenKind};
use crate::lexer::{NumberLiteral, StringLiteral, IdentifierToken, OperatorToken, CommentToken};
use crate::lexer::utils::{LexerValidator, UnicodeHelper};

/// Main lexer that orchestrates specialized scanners
pub struct ModularLexer<'a> {
    /// Input handler
    input: Input<'a>,
    /// Lexer context and configuration
    context: LexerContext,
    /// Specialized scanners
    scanners: ScannerSet,
    /// Validation engine
    validator: Option<LexerValidator>,
    /// Configuration flags
    config: LexerFlags,
}

/// Set of specialized scanners
struct ScannerSet {
    number: NumberScanner,
    string: StringScanner,
    identifier: IdentifierScanner,
    operator: OperatorScanner,
    comment: CommentScanner,
}

/// Lexer configuration flags
#[derive(Debug, Clone)]
pub struct LexerFlags {
    /// Skip trivia tokens (whitespace, comments)
    pub skip_trivia: bool,
    /// Preserve comment content
    pub preserve_comments: bool,
    /// Enable validation
    pub validate_tokens: bool,
    /// Track trivia with tokens
    pub track_trivia: bool,
    /// Enable error recovery
    pub error_recovery: bool,
}

impl Default for LexerFlags {
    fn default() -> Self {
        Self {
            skip_trivia: true,
            preserve_comments: false,
            validate_tokens: false,
            track_trivia: false,
            error_recovery: true,
        }
    }
}

impl<'a> ModularLexer<'a> {
    /// Create a new modular lexer
    pub fn new(source: &'a str, context: LexerContext) -> Self {
        let validator = if context.config.experimental {
            Some(LexerValidator::new(&context))
        } else {
            None
        };
        
        Self {
            input: Input::new(source),
            context,
            scanners: ScannerSet {
                number: NumberScanner::new(),
                string: StringScanner::new(),
                identifier: IdentifierScanner::new(),
                operator: OperatorScanner::new(),
                comment: CommentScanner::new(),
            },
            validator,
            config: LexerFlags::default(),
        }
    }
    
    /// Create lexer with custom configuration
    pub fn with_config(source: &'a str, context: LexerContext, config: LexerFlags) -> Self {
        let mut lexer = Self::new(source, context);
        lexer.config = config;
        lexer
    }
    
    /// Create lexer with validation enabled
    pub fn with_validation(source: &'a str, context: LexerContext) -> Self {
        let mut config = LexerFlags::default();
        config.validate_tokens = true;
        
        let mut lexer = Self::with_config(source, context, config);
        lexer.validator = Some(LexerValidator::new(&lexer.context));
        lexer
    }
    
    /// Tokenize the entire source
    pub fn tokenize(&mut self) -> Result<Vec<Token>> {
        let mut tokens = Vec::new();
        
        while !self.input.is_at_end() {
            // Skip whitespace and optionally collect as trivia
            if self.skip_whitespace_and_trivia(&mut tokens)? {
                continue;
            }
            
            // Try to scan a token
            match self.scan_next_token()? {
                Some(token) => {
                    // Validate token if enabled
                    if let Some(validator) = &mut self.validator {
                        if self.config.validate_tokens {
                            validator.validate_token(&token)?;
                        }
                    }
                    
                    tokens.push(token);
                }
                None => {
                    // No scanner could handle the current character
                    return self.handle_unexpected_character();
                }
            }
        }
        
        // Add EOF token
        let eof_span = self.input.current_token_span();
        tokens.push(Token::new(TokenKind::Eof, eof_span, String::new()));
        
        // Final validation if enabled
        if let Some(validator) = &mut self.validator {
            if self.config.validate_tokens {
                validator.validate_tokens(&tokens)?;
            }
        }
        
        Ok(tokens)
    }
    
    /// Scan the next token using appropriate scanner
    fn scan_next_token(&mut self) -> Result<Option<Token>> {
        self.input.mark_token_start();
        
        // Try each scanner in order of precedence
        if let Some(result) = self.scanners.comment.try_scan(&mut self.input, &self.context) {
            return self.handle_comment_token(result?);
        }
        
        if let Some(result) = self.scanners.number.try_scan(&mut self.input, &self.context) {
            return self.handle_number_token(result?);
        }
        
        if let Some(result) = self.scanners.string.try_scan(&mut self.input, &self.context) {
            return self.handle_string_token(result?);
        }
        
        if let Some(result) = self.scanners.identifier.try_scan(&mut self.input, &self.context) {
            return self.handle_identifier_token(result?);
        }
        
        if let Some(result) = self.scanners.operator.try_scan(&mut self.input, &self.context) {
            return self.handle_operator_token(result?);
        }
        
        Ok(None)
    }
    
    /// Handle number token from scanner
    fn handle_number_token(&mut self, number_literal: NumberLiteral) -> Result<Option<Token>> {
        let span = self.input.current_token_span();
        let text = self.input.token_text().to_string();
        let token_kind = TokenKind::Number(number_literal);
        
        let token = Token::new(token_kind, span, text);
        Ok(Some(token))
    }
    
    /// Handle string token from scanner
    fn handle_string_token(&mut self, string_literal: StringLiteral) -> Result<Option<Token>> {
        let span = self.input.current_token_span();
        let text = self.input.token_text().to_string();
        let token_kind = TokenKind::String(string_literal);
        
        let token = Token::new(token_kind, span, text);
        Ok(Some(token))
    }
    
    /// Handle identifier token from scanner
    fn handle_identifier_token(&mut self, identifier_token: IdentifierToken) -> Result<Option<Token>> {
        let span = self.input.current_token_span();
        let text = self.input.token_text().to_string();
        let token_kind = TokenKind::from(identifier_token);
        
        let token = Token::new(token_kind, span, text);
        Ok(Some(token))
    }
    
    /// Handle operator token from scanner
    fn handle_operator_token(&mut self, operator_token: OperatorToken) -> Result<Option<Token>> {
        let span = self.input.current_token_span();
        let text = self.input.token_text().to_string();
        let token_kind = TokenKind::Operator(operator_token);
        
        let token = Token::new(token_kind, span, text);
        Ok(Some(token))
    }
    
    /// Handle comment token from scanner
    fn handle_comment_token(&mut self, comment_token: CommentToken) -> Result<Option<Token>> {
        if self.config.skip_trivia && !self.config.preserve_comments {
            return Ok(None); // Skip comments entirely
        }
        
        let span = self.input.current_token_span();
        let text = self.input.token_text().to_string();
        let token_kind = TokenKind::Comment(comment_token);
        
        let token = Token::new(token_kind, span, text);
        Ok(Some(token))
    }
    
    /// Skip whitespace and optionally collect as trivia
    fn skip_whitespace_and_trivia(&mut self, tokens: &mut Vec<Token>) -> Result<bool> {
        if !UnicodeHelper::is_whitespace(self.input.current_char()) && 
           !UnicodeHelper::is_line_terminator(self.input.current_char()) {
            return Ok(false);
        }
        
        self.input.mark_token_start();
        let mut whitespace_content = String::new();
        let mut is_line_terminator = false;
        
        // Collect consecutive whitespace
        while !self.input.is_at_end() {
            let ch = self.input.current_char();
            
            if UnicodeHelper::is_line_terminator(ch) {
                whitespace_content.push(self.input.advance());
                is_line_terminator = true;
                break; // Line terminators are significant
            } else if UnicodeHelper::is_whitespace(ch) {
                whitespace_content.push(self.input.advance());
            } else {
                break;
            }
        }
        
        if whitespace_content.is_empty() {
            return Ok(false);
        }
        
        // Create trivia token if tracking is enabled
        if self.config.track_trivia || !self.config.skip_trivia {
            let span = self.input.current_token_span();
            let token_kind = if is_line_terminator {
                TokenKind::LineTerminator(whitespace_content.clone())
            } else {
                TokenKind::Whitespace(whitespace_content.clone())
            };
            
            let token = Token::new(token_kind, span, whitespace_content);
            tokens.push(token);
        }
        
        Ok(true)
    }
    
    /// Handle unexpected character
    fn handle_unexpected_character(&mut self) -> Result<Vec<Token>> {
        let ch = self.input.current_char();
        let span = self.input.current_token_span();
        
        if self.config.error_recovery {
            // Try to recover by skipping the character
            self.input.advance();
            
            // Create an error token
            let error_token = Token::error_recovery(
                format!("Unexpected character: '{}'", ch),
                span,
            );
            
            // Continue tokenizing
            let mut tokens = vec![error_token];
            tokens.extend(self.tokenize()?);
            Ok(tokens)
        } else {
            Err(Error::lexer(
                format!("Unexpected character: '{}'", ch),
                span,
            ))
        }
    }
    
    /// Get the next token (iterator-style interface)
    pub fn next_token(&mut self) -> Result<Option<Token>> {
        if self.input.is_at_end() {
            return Ok(None);
        }
        
        // Skip whitespace if configured
        if self.config.skip_trivia {
            self.skip_whitespace_only()?;
        }
        
        if self.input.is_at_end() {
            return Ok(None);
        }
        
        self.scan_next_token()
    }
    
    /// Skip whitespace without creating trivia tokens
    fn skip_whitespace_only(&mut self) -> Result<()> {
        while !self.input.is_at_end() {
            let ch = self.input.current_char();
            if UnicodeHelper::is_whitespace(ch) || UnicodeHelper::is_line_terminator(ch) {
                self.input.advance();
            } else {
                break;
            }
        }
        Ok(())
    }
    
    /// Get current position in input
    pub fn position(&self) -> usize {
        self.input.byte_offset()
    }
    
    /// Get current line number
    pub fn line(&self) -> u32 {
        self.input.line()
    }
    
    /// Get current column number
    pub fn column(&self) -> u32 {
        self.input.column()
    }
    
    /// Check if at end of input
    pub fn is_at_end(&self) -> bool {
        self.input.is_at_end()
    }
    
    /// Get validation results if validation is enabled
    pub fn validation_results(&self) -> Option<(&[crate::lexer::utils::ValidationWarning], &[crate::lexer::utils::ValidationError])> {
        self.validator.as_ref().map(|v| (v.warnings(), v.errors()))
    }
    
    /// Get lexer configuration
    pub fn config(&self) -> &LexerFlags {
        &self.config
    }
    
    /// Get lexer context
    pub fn context(&self) -> &LexerContext {
        &self.context
    }
}

/// Builder for creating lexers with custom configuration
pub struct LexerBuilder<'a> {
    source: &'a str,
    context: Option<LexerContext>,
    config: LexerFlags,
}

impl<'a> LexerBuilder<'a> {
    /// Create a new lexer builder
    pub fn new(source: &'a str) -> Self {
        Self {
            source,
            context: None,
            config: LexerFlags::default(),
        }
    }
    
    /// Set lexer context
    pub fn context(mut self, context: LexerContext) -> Self {
        self.context = Some(context);
        self
    }
    
    /// Enable trivia tracking
    pub fn track_trivia(mut self, enable: bool) -> Self {
        self.config.track_trivia = enable;
        self
    }
    
    /// Enable comment preservation
    pub fn preserve_comments(mut self, enable: bool) -> Self {
        self.config.preserve_comments = enable;
        self
    }
    
    /// Enable validation
    pub fn validate(mut self, enable: bool) -> Self {
        self.config.validate_tokens = enable;
        self
    }
    
    /// Enable error recovery
    pub fn error_recovery(mut self, enable: bool) -> Self {
        self.config.error_recovery = enable;
        self
    }
    
    /// Skip trivia tokens
    pub fn skip_trivia(mut self, skip: bool) -> Self {
        self.config.skip_trivia = skip;
        self
    }
    
    /// Build the lexer
    pub fn build(self) -> ModularLexer<'a> {
        let context = self.context.unwrap_or_default();
        ModularLexer::with_config(self.source, context, self.config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::core::{LexerConfigBuilder, EcmaVersion};
    
    #[test]
    fn test_basic_tokenization() {
        let source = "let x = 42;";
        let context = LexerContext::new();
        let mut lexer = ModularLexer::new(source, context);
        
        let tokens = lexer.tokenize().unwrap();
        
        // Should have: let, x, =, 42, ;, EOF
        assert_eq!(tokens.len(), 6);
        assert!(matches!(tokens[0].kind, TokenKind::Keyword(_)));
        assert!(matches!(tokens[1].kind, TokenKind::Identifier(_)));
        assert!(matches!(tokens[2].kind, TokenKind::Operator(_)));
        assert!(matches!(tokens[3].kind, TokenKind::Number(_)));
        assert!(matches!(tokens[4].kind, TokenKind::Operator(_)));
        assert!(matches!(tokens[5].kind, TokenKind::Eof));
    }
    
    #[test]
    fn test_string_tokenization() {
        let source = r#""hello world""#;
        let context = LexerContext::new();
        let mut lexer = ModularLexer::new(source, context);
        
        let tokens = lexer.tokenize().unwrap();
        
        assert_eq!(tokens.len(), 2); // string, EOF
        assert!(matches!(tokens[0].kind, TokenKind::String(_)));
        if let TokenKind::String(StringLiteral::String(content)) = &tokens[0].kind {
            assert_eq!(content, "hello world");
        }
    }
    
    #[test]
    fn test_comment_handling() {
        let source = "// comment\nlet x = 1;";
        let context = LexerContext::new();
        
        // With comments skipped (default)
        let mut lexer = ModularLexer::new(source, context.clone());
        let tokens = lexer.tokenize().unwrap();
        
        // Should not include comment token
        assert!(tokens.iter().all(|t| !matches!(t.kind, TokenKind::Comment(_))));
        
        // With comments preserved
        let config = LexerFlags {
            skip_trivia: false,
            preserve_comments: true,
            ..Default::default()
        };
        let mut lexer = ModularLexer::with_config(source, context, config);
        let tokens = lexer.tokenize().unwrap();
        
        // Should include comment token
        assert!(tokens.iter().any(|t| matches!(t.kind, TokenKind::Comment(_))));
    }
    
    #[test]
    fn test_error_recovery() {
        let source = "let x = @; // invalid character";
        let context = LexerContext::new();
        let config = LexerFlags {
            error_recovery: true,
            ..Default::default()
        };
        let mut lexer = ModularLexer::with_config(source, context, config);
        
        let tokens = lexer.tokenize().unwrap();
        
        // Should recover and continue tokenizing
        assert!(tokens.iter().any(|t| matches!(t.kind, TokenKind::Invalid(_))));
        assert!(tokens.iter().any(|t| matches!(t.kind, TokenKind::Keyword(_))));
    }
    
    #[test]
    fn test_unicode_handling() {
        let source = "let café = '🚀';";
        let context = LexerContext::new();
        let mut lexer = ModularLexer::new(source, context);
        
        let tokens = lexer.tokenize().unwrap();
        
        // Should handle Unicode identifiers and strings
        let has_unicode_identifier = tokens.iter().any(|t| {
            matches!(&t.kind, TokenKind::Identifier(name) if name == "café")
        });
        assert!(has_unicode_identifier);
        
        let has_unicode_string = tokens.iter().any(|t| {
            matches!(&t.kind, TokenKind::String(StringLiteral::String(content)) if content == "🚀")
        });
        assert!(has_unicode_string);
    }
    
    #[test]
    fn test_lexer_builder() {
        let source = "function test() {}";
        let context = LexerConfigBuilder::new()
            .ecma_version(EcmaVersion::ES2015)
            .build();
        
        let mut lexer = LexerBuilder::new(source)
            .context(LexerContext::with_config(context))
            .preserve_comments(true)
            .validate(true)
            .build();
        
        let tokens = lexer.tokenize().unwrap();
        assert!(!tokens.is_empty());
    }
    
    #[test]
    fn test_iterator_interface() {
        let source = "a b c";
        let context = LexerContext::new();
        let mut lexer = ModularLexer::new(source, context);
        
        let mut tokens = Vec::new();
        while let Some(token) = lexer.next_token().unwrap() {
            tokens.push(token);
        }
        
        assert_eq!(tokens.len(), 3); // a, b, c
        assert!(tokens.iter().all(|t| matches!(t.kind, TokenKind::Identifier(_))));
    }
}