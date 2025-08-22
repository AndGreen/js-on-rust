//! Position tracking and span management for lexer
//!
//! Provides UTF-8 safe position tracking with line/column information
//! and span creation for error reporting and debugging.

use crate::error::Span;

/// Tracks position in source code with UTF-8 safety
#[derive(Debug, Clone)]
pub struct Position {
    /// Byte offset in source
    pub byte_offset: usize,
    /// Line number (1-based)
    pub line: u32,
    /// Column number (1-based, measured in Unicode scalar values)
    pub column: u32,
}

impl Position {
    /// Create a new position at the start of source
    pub fn new() -> Self {
        Self {
            byte_offset: 0,
            line: 1,
            column: 1,
        }
    }
    
    /// Create position with specific values
    pub fn with_values(byte_offset: usize, line: u32, column: u32) -> Self {
        Self {
            byte_offset,
            line,
            column,
        }
    }
    
    /// Advance position by one character
    pub fn advance(&mut self, ch: char) {
        self.byte_offset += ch.len_utf8();
        if ch == '\n' {
            self.line += 1;
            self.column = 1;
        } else {
            self.column += 1;
        }
    }
    
    /// Advance position by byte count (for non-character advances)
    pub fn advance_bytes(&mut self, count: usize) {
        self.byte_offset += count;
        self.column += count as u32; // Approximation for non-newline content
    }
    
    /// Create a span from this position to another
    pub fn span_to(&self, end: &Position) -> Span {
        Span::new(
            self.byte_offset,
            end.byte_offset,
            self.line,
            self.column,
        )
    }
    
    /// Create a span for a single character at this position
    pub fn span_for_char(&self, ch: char) -> Span {
        let mut end = self.clone();
        end.advance(ch);
        self.span_to(&end)
    }
    
    /// Create a span covering a range of bytes from this position
    pub fn span_with_length(&self, byte_length: usize) -> Span {
        let end_pos = Position::with_values(
            self.byte_offset + byte_length,
            self.line,
            self.column + byte_length as u32, // Approximation
        );
        self.span_to(&end_pos)
    }
}

impl Default for Position {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper for tracking token start positions
#[derive(Debug, Clone)]
pub struct TokenPosition {
    /// Position where current token started
    pub start: Position,
    /// Current position in source
    pub current: Position,
}

impl TokenPosition {
    /// Create new token position tracker
    pub fn new() -> Self {
        Self {
            start: Position::new(),
            current: Position::new(),
        }
    }
    
    /// Mark the start of a new token
    pub fn mark_token_start(&mut self) {
        self.start = self.current.clone();
    }
    
    /// Advance current position by one character
    pub fn advance(&mut self, ch: char) {
        self.current.advance(ch);
    }
    
    /// Advance current position by byte count
    pub fn advance_bytes(&mut self, count: usize) {
        self.current.advance_bytes(count);
    }
    
    /// Get span for current token
    pub fn current_token_span(&self) -> Span {
        self.start.span_to(&self.current)
    }
    
    /// Check if at specific byte offset
    pub fn at_byte_offset(&self, offset: usize) -> bool {
        self.current.byte_offset == offset
    }
    
    /// Get current line number
    pub fn line(&self) -> u32 {
        self.current.line
    }
    
    /// Get current column number
    pub fn column(&self) -> u32 {
        self.current.column
    }
    
    /// Get current byte offset
    pub fn byte_offset(&self) -> usize {
        self.current.byte_offset
    }
}

impl Default for TokenPosition {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_position_advance() {
        let mut pos = Position::new();
        
        // Test normal character
        pos.advance('a');
        assert_eq!(pos.byte_offset, 1);
        assert_eq!(pos.line, 1);
        assert_eq!(pos.column, 2);
        
        // Test newline
        pos.advance('\n');
        assert_eq!(pos.byte_offset, 2);
        assert_eq!(pos.line, 2);
        assert_eq!(pos.column, 1);
        
        // Test multi-byte UTF-8 character
        pos.advance('🚀');
        assert_eq!(pos.byte_offset, 6); // '\n' (1) + '🚀' (4)
        assert_eq!(pos.line, 2);
        assert_eq!(pos.column, 2);
    }
    
    #[test]
    fn test_span_creation() {
        let start = Position::with_values(0, 1, 1);
        let end = Position::with_values(5, 1, 6);
        
        let span = start.span_to(&end);
        assert_eq!(span.start, 0);
        assert_eq!(span.end, 5);
        assert_eq!(span.line, 1);
        assert_eq!(span.column, 1);
    }
    
    #[test]
    fn test_token_position() {
        let mut token_pos = TokenPosition::new();
        
        // Mark start of token
        token_pos.mark_token_start();
        
        // Advance through "hello"
        for ch in "hello".chars() {
            token_pos.advance(ch);
        }
        
        let span = token_pos.current_token_span();
        assert_eq!(span.start, 0);
        assert_eq!(span.end, 5);
        assert_eq!(span.line, 1);
        assert_eq!(span.column, 1);
    }
    
    #[test]
    fn test_unicode_positioning() {
        let mut pos = Position::new();
        
        // Test with mixed ASCII and Unicode
        let text = "café🚀";
        for ch in text.chars() {
            pos.advance(ch);
        }
        
        // café (4) + 🚀 (4) = 8 bytes, but 5 columns
        assert_eq!(pos.byte_offset, 8);
        assert_eq!(pos.column, 6); // 1 + 4 + 1
    }
}