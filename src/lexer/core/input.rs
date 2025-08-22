//! Input handling and character access for lexer
//!
//! Provides UTF-8 safe character iteration, lookahead, and substring extraction
//! while maintaining position tracking information.

use super::position::TokenPosition;
use crate::error::{Error, Result};

/// UTF-8 safe input handler for lexer
#[derive(Debug, Clone)]
pub struct Input<'a> {
    /// Source code being lexed
    source: &'a str,
    /// Current position tracker
    position: TokenPosition,
    /// Current character and its byte position
    current: Option<(usize, char)>,
}

impl<'a> Input<'a> {
    /// Create new input handler for source
    pub fn new(source: &'a str) -> Self {
        let current = source.char_indices().next();
        
        Self {
            source,
            position: TokenPosition::new(),
            current,
        }
    }
    
    /// Get current character
    pub fn current_char(&self) -> char {
        self.current.map(|(_, ch)| ch).unwrap_or('\0')
    }
    
    /// Check if at end of input
    pub fn is_at_end(&self) -> bool {
        self.current.is_none()
    }
    
    /// Peek at next character without advancing
    pub fn peek_char(&self) -> Option<char> {
        if let Some((current_pos, _)) = self.current {
            let next_pos = current_pos + self.current_char().len_utf8();
            self.source[next_pos..].chars().next()
        } else {
            None
        }
    }
    
    /// Peek ahead n characters
    pub fn peek_nth(&self, n: usize) -> Option<char> {
        if n == 0 {
            return Some(self.current_char());
        }
        
        if let Some((current_pos, _)) = self.current {
            let remaining = &self.source[current_pos..];
            remaining.chars().nth(n)
        } else {
            None
        }
    }
    
    /// Advance to next character
    pub fn advance(&mut self) -> char {
        let ch = self.current_char();
        
        if let Some((current_pos, _)) = self.current {
            // Update position tracker
            self.position.advance(ch);
            
            // Move to next character
            let next_pos = current_pos + ch.len_utf8();
            if next_pos < self.source.len() {
                let mut char_indices = self.source[next_pos..].char_indices();
                self.current = char_indices.next().map(|(offset, ch)| (next_pos + offset, ch));
            } else {
                self.current = None;
            }
        }
        
        ch
    }
    
    /// Mark start of new token
    pub fn mark_token_start(&mut self) {
        self.position.mark_token_start();
    }
    
    /// Get span for current token
    pub fn current_token_span(&self) -> crate::error::Span {
        self.position.current_token_span()
    }
    
    /// Get current byte offset
    pub fn byte_offset(&self) -> usize {
        self.position.byte_offset()
    }
    
    /// Get current line number
    pub fn line(&self) -> u32 {
        self.position.line()
    }
    
    /// Get current column number
    pub fn column(&self) -> u32 {
        self.position.column()
    }
    
    /// Extract substring from token start to current position
    pub fn token_text(&self) -> &'a str {
        let start = self.position.start.byte_offset;
        let end = self.position.current.byte_offset;
        self.safe_slice(start, end)
    }
    
    /// Extract substring from specific byte range
    pub fn slice(&self, start: usize, end: usize) -> &'a str {
        self.safe_slice(start, end)
    }
    
    /// Safely slice source respecting UTF-8 boundaries
    fn safe_slice(&self, start: usize, end: usize) -> &'a str {
        let start = start.min(self.source.len());
        let end = end.min(self.source.len());
        
        // Ensure we're at character boundaries
        let start_boundary = self.find_char_boundary(start, false);
        let end_boundary = self.find_char_boundary(end, true);
        
        &self.source[start_boundary..end_boundary]
    }
    
    /// Find the nearest character boundary
    fn find_char_boundary(&self, mut byte_pos: usize, round_up: bool) -> usize {
        if byte_pos >= self.source.len() {
            return self.source.len();
        }
        
        // If we're already at a character boundary, return as-is
        if self.source.is_char_boundary(byte_pos) {
            return byte_pos;
        }
        
        // Find the nearest boundary
        if round_up {
            while byte_pos < self.source.len() && !self.source.is_char_boundary(byte_pos) {
                byte_pos += 1;
            }
        } else {
            while byte_pos > 0 && !self.source.is_char_boundary(byte_pos) {
                byte_pos -= 1;
            }
        }
        
        byte_pos
    }
    
    /// Check if current character matches predicate
    pub fn matches<F>(&self, predicate: F) -> bool
    where
        F: FnOnce(char) -> bool,
    {
        if self.is_at_end() {
            false
        } else {
            predicate(self.current_char())
        }
    }
    
    /// Check if next character matches predicate
    pub fn peek_matches<F>(&mut self, predicate: F) -> bool
    where
        F: FnOnce(char) -> bool,
    {
        if let Some(ch) = self.peek_char() {
            predicate(ch)
        } else {
            false
        }
    }
    
    /// Advance while characters match predicate
    pub fn advance_while<F>(&mut self, mut predicate: F)
    where
        F: FnMut(char) -> bool,
    {
        while !self.is_at_end() && predicate(self.current_char()) {
            self.advance();
        }
    }
    
    /// Skip whitespace characters
    pub fn skip_whitespace(&mut self) {
        self.advance_while(|ch| ch.is_whitespace());
    }
    
    /// Get remaining source from current position
    pub fn remaining(&self) -> &'a str {
        let start = self.position.current.byte_offset;
        &self.source[start.min(self.source.len())..]
    }
    
    /// Get source slice starting from current position with specific length
    pub fn slice_from_current(&self, length: usize) -> &'a str {
        let start = self.position.current.byte_offset;
        let end = start + length;
        self.safe_slice(start, end)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_basic_iteration() {
        let source = "hello";
        let mut input = Input::new(source);
        
        assert_eq!(input.current_char(), 'h');
        assert_eq!(input.advance(), 'h');
        assert_eq!(input.current_char(), 'e');
        assert_eq!(input.advance(), 'e');
        assert_eq!(input.current_char(), 'l');
    }
    
    #[test]
    fn test_unicode_handling() {
        let source = "café🚀";
        let mut input = Input::new(source);
        
        input.mark_token_start();
        
        // Advance through each character
        assert_eq!(input.advance(), 'c');
        assert_eq!(input.advance(), 'a');
        assert_eq!(input.advance(), 'f');
        assert_eq!(input.advance(), 'é');
        assert_eq!(input.advance(), '🚀');
        
        assert!(input.is_at_end());
        
        // Check token text extraction
        let text = input.token_text();
        assert_eq!(text, "café🚀");
    }
    
    #[test]
    fn test_peek_functionality() {
        let source = "abc";
        let mut input = Input::new(source);
        
        assert_eq!(input.current_char(), 'a');
        assert_eq!(input.peek_char(), Some('b'));
        assert_eq!(input.peek_nth(2), Some('c'));
        assert_eq!(input.peek_nth(3), None);
        
        // Peek shouldn't advance position
        assert_eq!(input.current_char(), 'a');
    }
    
    #[test]
    fn test_safe_slicing() {
        let source = "Hello, 世界!";
        let input = Input::new(source);
        
        // Test normal ASCII slice
        let slice1 = input.slice(0, 5);
        assert_eq!(slice1, "Hello");
        
        // Test Unicode slice (世 is 3 bytes, 界 is 3 bytes)
        let slice2 = input.slice(7, 13);
        assert_eq!(slice2, "世界");
        
        // Test boundary safety - should not panic on mid-character index
        let slice3 = input.slice(7, 8); // Mid-character boundary
        assert_eq!(slice3, ""); // Should round to nearest boundary
    }
    
    #[test]
    fn test_position_tracking() {
        let source = "line1\nline2\n";
        let mut input = Input::new(source);
        
        // Start of line 1
        assert_eq!(input.line(), 1);
        assert_eq!(input.column(), 1);
        
        // Advance to newline
        while input.current_char() != '\n' {
            input.advance();
        }
        input.advance(); // Skip newline
        
        // Start of line 2
        assert_eq!(input.line(), 2);
        assert_eq!(input.column(), 1);
    }
    
    #[test]
    fn test_advance_while() {
        let source = "123abc";
        let mut input = Input::new(source);
        
        input.mark_token_start();
        input.advance_while(|ch| ch.is_ascii_digit());
        
        assert_eq!(input.current_char(), 'a');
        assert_eq!(input.token_text(), "123");
    }
    
    #[test]
    fn test_matches_predicates() {
        let source = "a1";
        let mut input = Input::new(source);
        
        assert!(input.matches(|ch| ch.is_alphabetic()));
        assert!(input.peek_matches(|ch| ch.is_ascii_digit()));
        
        input.advance();
        assert!(input.matches(|ch| ch.is_ascii_digit()));
    }
}