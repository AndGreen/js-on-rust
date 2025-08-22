//! Comment scanner
//!
//! Handles scanning of JavaScript comments including:
//! - Line comments (// comment text)
//! - Block comments (/* comment text */)
//! - Nested block comments (not standard JS, but useful for debugging)
//! - HTML-style comments (<!-- and -->, legacy support)

use super::{Scanner, LookaheadScanner, CharClassifier, EcmaCharClassifier, lexer_error};
use crate::error::Result;
use crate::lexer::core::{Input, LexerContext};

/// Comment token types
#[derive(Debug, Clone, PartialEq)]
pub enum CommentToken {
    /// Line comment (// text)
    LineComment(String),
    /// Block comment (/* text */)
    BlockComment(String),
    /// HTML opening comment (<!-- text, legacy support)
    HtmlComment(String),
}

/// Comment scanner for JavaScript comments
#[derive(Debug, Default)]
pub struct CommentScanner {
    /// Whether to preserve comment content (useful for documentation tools)
    preserve_content: bool,
    /// Whether to allow HTML-style comments (legacy support)
    allow_html_comments: bool,
}

impl CommentScanner {
    /// Create a new comment scanner
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Create comment scanner with content preservation
    pub fn with_content_preservation(preserve_content: bool) -> Self {
        Self {
            preserve_content,
            allow_html_comments: false,
        }
    }
    
    /// Create comment scanner with HTML comment support
    pub fn with_html_support(allow_html_comments: bool) -> Self {
        Self {
            preserve_content: true, // HTML comments need content
            allow_html_comments,
        }
    }
    
    /// Scan a comment
    fn scan_comment(&self, input: &mut Input, _context: &LexerContext) -> Result<CommentToken> {
        input.mark_token_start();
        
        let first_char = input.current_char();
        
        match first_char {
            '/' => {
                if input.peek_char() == Some('/') {
                    self.scan_line_comment(input)
                } else if input.peek_char() == Some('*') {
                    self.scan_block_comment(input)
                } else {
                    Err(lexer_error(
                        "Internal error: comment scanner called on non-comment",
                        input.current_token_span(),
                    ))
                }
            }
            '<' if self.allow_html_comments => {
                self.scan_html_comment(input)
            }
            _ => Err(lexer_error(
                "Internal error: comment scanner called on non-comment character",
                input.current_token_span(),
            )),
        }
    }
    
    /// Scan line comment (// ...)
    fn scan_line_comment(&self, input: &mut Input) -> Result<CommentToken> {
        // Consume '//'
        input.advance();
        input.advance();
        
        let mut content = String::new();
        
        // Read until end of line or end of input
        while !input.is_at_end() && !EcmaCharClassifier::is_line_terminator(input.current_char()) {
            if self.preserve_content {
                content.push(input.current_char());
            }
            input.advance();
        }
        
        // Don't consume the line terminator - leave it for the main lexer
        
        Ok(CommentToken::LineComment(content))
    }
    
    /// Scan block comment (/* ... */)
    fn scan_block_comment(&self, input: &mut Input) -> Result<CommentToken> {
        // Consume '/*'
        input.advance();
        input.advance();
        
        let mut content = String::new();
        let mut nesting_level = 1; // For potential nested comment support
        
        while !input.is_at_end() && nesting_level > 0 {
            let ch = input.current_char();
            
            if ch == '*' && input.peek_char() == Some('/') {
                // End of block comment
                input.advance(); // consume '*'
                input.advance(); // consume '/'
                nesting_level -= 1;
                
                if nesting_level == 0 {
                    break;
                }
                
                if self.preserve_content {
                    content.push_str("*/");
                }
            } else if ch == '/' && input.peek_char() == Some('*') {
                // Potential nested comment (not standard JS, but handle gracefully)
                input.advance(); // consume '/'
                input.advance(); // consume '*'
                
                if self.preserve_content {
                    content.push_str("/*");
                }
                // Note: We don't increment nesting_level for standard JS compliance
            } else {
                if self.preserve_content {
                    content.push(ch);
                }
                input.advance();
            }
        }
        
        if nesting_level > 0 {
            return Err(lexer_error(
                "Unterminated block comment",
                input.current_token_span(),
            ));
        }
        
        Ok(CommentToken::BlockComment(content))
    }
    
    /// Scan HTML-style comment (<!-- ...)
    fn scan_html_comment(&self, input: &mut Input) -> Result<CommentToken> {
        // Check for '<!--'
        if !self.matches_html_comment_start(input) {
            return Err(lexer_error(
                "Internal error: HTML comment scanner called incorrectly",
                input.current_token_span(),
            ));
        }
        
        // Consume '<!--'
        input.advance(); // <
        input.advance(); // !
        input.advance(); // -
        input.advance(); // -
        
        let mut content = String::new();
        
        // Read until '-->' or end of input
        while !input.is_at_end() {
            if input.current_char() == '-' && 
               input.peek_char() == Some('-') && 
               input.peek_nth(2) == Some('>') {
                // End of HTML comment
                input.advance(); // -
                input.advance(); // -
                input.advance(); // >
                break;
            }
            
            if self.preserve_content {
                content.push(input.current_char());
            }
            input.advance();
        }
        
        Ok(CommentToken::HtmlComment(content))
    }
    
    /// Check if current position matches HTML comment start
    fn matches_html_comment_start(&self, input: &Input) -> bool {
        input.current_char() == '<' &&
        input.peek_char() == Some('!') &&
        input.peek_nth(2) == Some('-') &&
        input.peek_nth(3) == Some('-')
    }
    
    /// Check if current position could start a comment
    fn can_start_comment(&self, input: &Input) -> bool {
        match input.current_char() {
            '/' => {
                matches!(input.peek_char(), Some('/') | Some('*'))
            }
            '<' if self.allow_html_comments => {
                self.matches_html_comment_start(input)
            }
            _ => false,
        }
    }
}

impl Scanner for CommentScanner {
    type Token = CommentToken;
    
    fn try_scan(&mut self, input: &mut Input, context: &LexerContext) -> Option<Result<Self::Token>> {
        if self.can_scan(input, context) {
            Some(self.scan_comment(input, context))
        } else {
            None
        }
    }
    
    fn can_scan(&self, input: &Input, _context: &LexerContext) -> bool {
        self.can_start_comment(input)
    }
    
    fn name(&self) -> &'static str {
        "CommentScanner"
    }
}

impl LookaheadScanner for CommentScanner {
    fn would_match(&self, input: &mut Input, context: &LexerContext) -> bool {
        self.can_scan(input, context)
    }
    
    fn expected_length(&self, input: &mut Input, _context: &LexerContext) -> Option<usize> {
        if !self.can_start_comment(input) {
            return None;
        }
        
        let start = input.byte_offset();
        let mut temp_input = input.clone();
        
        match temp_input.current_char() {
            '/' if temp_input.peek_char() == Some('/') => {
                // Line comment - scan to end of line
                temp_input.advance(); // /
                temp_input.advance(); // /
                
                while !temp_input.is_at_end() && 
                      !EcmaCharClassifier::is_line_terminator(temp_input.current_char()) {
                    temp_input.advance();
                }
            }
            '/' if temp_input.peek_char() == Some('*') => {
                // Block comment - scan to */
                temp_input.advance(); // /
                temp_input.advance(); // *
                
                while !temp_input.is_at_end() {
                    if temp_input.current_char() == '*' && temp_input.peek_char() == Some('/') {
                        temp_input.advance(); // *
                        temp_input.advance(); // /
                        break;
                    }
                    temp_input.advance();
                }
            }
            '<' if self.allow_html_comments && self.matches_html_comment_start(&temp_input) => {
                // HTML comment - scan to -->
                temp_input.advance(); // <
                temp_input.advance(); // !
                temp_input.advance(); // -
                temp_input.advance(); // -
                
                while !temp_input.is_at_end() {
                    if temp_input.current_char() == '-' && 
                       temp_input.peek_char() == Some('-') && 
                       temp_input.peek_nth(2) == Some('>') {
                        temp_input.advance(); // -
                        temp_input.advance(); // -
                        temp_input.advance(); // >
                        break;
                    }
                    temp_input.advance();
                }
            }
            _ => return None,
        }
        
        Some(temp_input.byte_offset() - start)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::core::LexerContext;
    
    fn scan_comment(source: &str) -> Result<CommentToken> {
        let mut input = Input::new(source);
        let mut scanner = CommentScanner::with_content_preservation(true);
        let context = LexerContext::new();
        
        scanner.try_scan(&mut input, &context).unwrap()
    }
    
    fn scan_comment_no_content(source: &str) -> Result<CommentToken> {
        let mut input = Input::new(source);
        let mut scanner = CommentScanner::new();
        let context = LexerContext::new();
        
        scanner.try_scan(&mut input, &context).unwrap()
    }
    
    #[test]
    fn test_line_comments() {
        match scan_comment("// hello world").unwrap() {
            CommentToken::LineComment(content) => assert_eq!(content, " hello world"),
            _ => panic!("Expected line comment"),
        }
        
        match scan_comment("//").unwrap() {
            CommentToken::LineComment(content) => assert_eq!(content, ""),
            _ => panic!("Expected empty line comment"),
        }
        
        match scan_comment("// comment\ncode").unwrap() {
            CommentToken::LineComment(content) => assert_eq!(content, " comment"),
            _ => panic!("Expected line comment"),
        }
    }
    
    #[test]
    fn test_block_comments() {
        match scan_comment("/* hello world */").unwrap() {
            CommentToken::BlockComment(content) => assert_eq!(content, " hello world "),
            _ => panic!("Expected block comment"),
        }
        
        match scan_comment("/**/").unwrap() {
            CommentToken::BlockComment(content) => assert_eq!(content, ""),
            _ => panic!("Expected empty block comment"),
        }
        
        match scan_comment("/* multi\nline\ncomment */").unwrap() {
            CommentToken::BlockComment(content) => assert_eq!(content, " multi\nline\ncomment "),
            _ => panic!("Expected multiline block comment"),
        }
    }
    
    #[test]
    fn test_html_comments() {
        let mut scanner = CommentScanner::with_html_support(true);
        let mut input = Input::new("<!-- HTML comment -->");
        let context = LexerContext::new();
        
        match scanner.try_scan(&mut input, &context).unwrap().unwrap() {
            CommentToken::HtmlComment(content) => assert_eq!(content, " HTML comment "),
            _ => panic!("Expected HTML comment"),
        }
    }
    
    #[test]
    fn test_content_preservation() {
        // With content preservation
        match scan_comment("// hello").unwrap() {
            CommentToken::LineComment(content) => assert_eq!(content, " hello"),
            _ => panic!("Expected line comment with content"),
        }
        
        // Without content preservation
        match scan_comment_no_content("// hello").unwrap() {
            CommentToken::LineComment(content) => assert_eq!(content, ""),
            _ => panic!("Expected line comment without content"),
        }
    }
    
    #[test]
    fn test_unterminated_block_comment() {
        let mut input = Input::new("/* unterminated");
        let mut scanner = CommentScanner::new();
        let context = LexerContext::new();
        
        assert!(scanner.try_scan(&mut input, &context).unwrap().is_err());
    }
    
    #[test]
    fn test_nested_comment_markers() {
        // Block comment containing // should work
        match scan_comment("/* line // comment inside */").unwrap() {
            CommentToken::BlockComment(content) => assert_eq!(content, " line // comment inside "),
            _ => panic!("Expected block comment"),
        }
        
        // Block comment containing /* should work (no nesting in standard JS)
        match scan_comment("/* block /* inside */ outside").unwrap() {
            CommentToken::BlockComment(content) => assert_eq!(content, " block /* inside "),
            _ => panic!("Expected block comment"),
        }
    }
    
    #[test]
    fn test_can_scan() {
        let scanner = CommentScanner::new();
        let context = LexerContext::new();
        
        assert!(scanner.can_scan(&Input::new("// comment"), &context));
        assert!(scanner.can_scan(&Input::new("/* comment */"), &context));
        assert!(!scanner.can_scan(&Input::new("hello"), &context));
        assert!(!scanner.can_scan(&Input::new("123"), &context));
        
        // HTML comments require special configuration
        assert!(!scanner.can_scan(&Input::new("<!-- comment -->"), &context));
        
        let html_scanner = CommentScanner::with_html_support(true);
        assert!(html_scanner.can_scan(&Input::new("<!-- comment -->"), &context));
    }
    
    #[test]
    fn test_expected_length() {
        let mut scanner = CommentScanner::new();
        let context = LexerContext::new();
        
        let mut input = Input::new("// hello");
        assert_eq!(scanner.expected_length(&mut input, &context), Some(8));
        
        let mut input2 = Input::new("/* hello */");
        assert_eq!(scanner.expected_length(&mut input2, &context), Some(11));
        
        let mut input3 = Input::new("// comment\ncode");
        assert_eq!(scanner.expected_length(&mut input3, &context), Some(10));
    }
    
    #[test]
    fn test_unicode_in_comments() {
        match scan_comment("// Привет мир! 🚀").unwrap() {
            CommentToken::LineComment(content) => assert_eq!(content, " Привет мир! 🚀"),
            _ => panic!("Expected line comment with Unicode"),
        }
        
        match scan_comment("/* café naïve 🎉 */").unwrap() {
            CommentToken::BlockComment(content) => assert_eq!(content, " café naïve 🎉 "),
            _ => panic!("Expected block comment with Unicode"),
        }
    }
    
    #[test]
    fn test_edge_cases() {
        // Single slash should not match
        let scanner = CommentScanner::new();
        let context = LexerContext::new();
        assert!(!scanner.can_scan(&Input::new("/"), &context));
        
        // Incomplete block comment start
        assert!(!scanner.can_scan(&Input::new("/*"), &context));
        
        // HTML comment without proper configuration
        assert!(!scanner.can_scan(&Input::new("<!--"), &context));
    }
}