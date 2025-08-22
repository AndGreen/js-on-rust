//! String literal scanner
//!
//! Handles scanning of JavaScript string literals including:
//! - Single and double quoted strings
//! - Escape sequences (\n, \t, \', \", \\, etc.)
//! - Unicode escape sequences (\uXXXX, \u{XXXXXX})
//! - Hexadecimal escape sequences (\xXX)
//! - Template literals (future extension point)

use super::{Scanner, LookaheadScanner, CharClassifier, EcmaCharClassifier, lexer_error};
use crate::error::{Result, Span};
use crate::lexer::core::{Input, LexerContext};

/// String literal types
#[derive(Debug, Clone, PartialEq)]
pub enum StringLiteral {
    /// Regular string literal ("hello" or 'hello')
    String(String),
    /// Template literal (`hello ${world}`) - for future implementation
    Template(Vec<TemplatePart>),
}

/// Parts of a template literal
#[derive(Debug, Clone, PartialEq)]
pub enum TemplatePart {
    /// String content
    Text(String),
    /// Expression placeholder
    Expression,
}

/// String scanner for JavaScript string literals
#[derive(Debug, Default)]
pub struct StringScanner;

impl StringScanner {
    /// Create a new string scanner
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Scan a string literal
    fn scan_string(&self, input: &mut Input, _context: &LexerContext) -> Result<StringLiteral> {
        input.mark_token_start();
        
        let quote_char = input.current_char();
        if quote_char != '"' && quote_char != '\'' {
            return Err(lexer_error(
                "Internal error: string scanner called on non-quote character",
                input.current_token_span(),
            ));
        }
        
        input.advance(); // consume opening quote
        
        let mut content = String::new();
        
        while !input.is_at_end() && input.current_char() != quote_char {
            let ch = input.current_char();
            
            if ch == '\\' {
                // Handle escape sequence
                input.advance(); // consume backslash
                
                if input.is_at_end() {
                    return Err(lexer_error(
                        "Unterminated string: unexpected end after escape character",
                        input.current_token_span(),
                    ));
                }
                
                let escaped_char = self.parse_escape_sequence(input)?;
                content.push(escaped_char);
            } else if EcmaCharClassifier::is_line_terminator(ch) {
                return Err(lexer_error(
                    "Unterminated string: unexpected line terminator",
                    input.current_token_span(),
                ));
            } else {
                content.push(ch);
                input.advance();
            }
        }
        
        if input.is_at_end() {
            return Err(lexer_error(
                "Unterminated string literal",
                input.current_token_span(),
            ));
        }
        
        // Consume closing quote
        input.advance();
        
        Ok(StringLiteral::String(content))
    }
    
    /// Parse an escape sequence starting after the backslash
    fn parse_escape_sequence(&self, input: &mut Input) -> Result<char> {
        let escape_char = input.advance();
        
        match escape_char {
            // Simple escape sequences
            'n' => Ok('\n'),
            't' => Ok('\t'),
            'r' => Ok('\r'),
            'b' => Ok('\u{0008}'), // backspace
            'f' => Ok('\u{000C}'), // form feed
            'v' => Ok('\u{000B}'), // vertical tab
            '0' => Ok('\0'),
            '\\' => Ok('\\'),
            '\'' => Ok('\''),
            '"' => Ok('"'),
            
            // Line continuation (backslash followed by line terminator)
            '\n' => Ok('\n'), // Actually removes the newline in JS
            '\r' => {
                // Handle CRLF
                if input.current_char() == '\n' {
                    input.advance();
                }
                Ok('\n')
            }
            '\u{2028}' | '\u{2029}' => Ok(escape_char), // Unicode line separators
            
            // Hexadecimal escape sequence \xXX
            'x' => self.parse_hex_escape_sequence(input, 2),
            
            // Unicode escape sequences
            'u' => self.parse_unicode_escape_sequence(input),
            
            // Octal escape sequences (legacy, not in strict mode)
            '0'..='7' => {
                // For now, treat as literal character
                // Full octal parsing would require context checking
                Ok(escape_char)
            }
            
            // Any other character - include literally
            ch => Ok(ch),
        }
    }
    
    /// Parse hexadecimal escape sequence (\xXX)
    fn parse_hex_escape_sequence(&self, input: &mut Input, digit_count: usize) -> Result<char> {
        let mut hex_digits = String::new();
        
        for _ in 0..digit_count {
            if input.is_at_end() || !EcmaCharClassifier::is_hex_digit(input.current_char()) {
                return Err(lexer_error(
                    format!("Invalid hex escape sequence: expected {} hex digits", digit_count),
                    input.current_token_span(),
                ));
            }
            hex_digits.push(input.advance());
        }
        
        match u32::from_str_radix(&hex_digits, 16) {
            Ok(code_point) if code_point <= 0xFF => {
                Ok(code_point as u8 as char)
            }
            Ok(_) => Err(lexer_error(
                "Invalid hex escape sequence: value too large",
                input.current_token_span(),
            )),
            Err(_) => Err(lexer_error(
                "Invalid hex escape sequence",
                input.current_token_span(),
            )),
        }
    }
    
    /// Parse Unicode escape sequence (\uXXXX or \u{XXXXXX})
    fn parse_unicode_escape_sequence(&self, input: &mut Input) -> Result<char> {
        if input.current_char() == '{' {
            // Extended Unicode escape \u{XXXXXX}
            input.advance(); // consume '{'
            
            let mut hex_digits = String::new();
            
            while !input.is_at_end() && input.current_char() != '}' {
                let ch = input.current_char();
                if EcmaCharClassifier::is_hex_digit(ch) {
                    hex_digits.push(input.advance());
                } else {
                    return Err(lexer_error(
                        "Invalid Unicode escape sequence: non-hex digit",
                        input.current_token_span(),
                    ));
                }
                
                // Limit to reasonable length
                if hex_digits.len() > 6 {
                    return Err(lexer_error(
                        "Invalid Unicode escape sequence: too many digits",
                        input.current_token_span(),
                    ));
                }
            }
            
            if input.current_char() != '}' {
                return Err(lexer_error(
                    "Invalid Unicode escape sequence: missing closing brace",
                    input.current_token_span(),
                ));
            }
            
            input.advance(); // consume '}'
            
            if hex_digits.is_empty() {
                return Err(lexer_error(
                    "Invalid Unicode escape sequence: no hex digits",
                    input.current_token_span(),
                ));
            }
            
            match u32::from_str_radix(&hex_digits, 16) {
                Ok(code_point) => {
                    char::from_u32(code_point).ok_or_else(|| lexer_error(
                        "Invalid Unicode escape sequence: invalid code point",
                        input.current_token_span(),
                    ))
                }
                Err(_) => Err(lexer_error(
                    "Invalid Unicode escape sequence",
                    input.current_token_span(),
                )),
            }
        } else {
            // Standard Unicode escape \uXXXX
            let mut hex_digits = String::new();
            
            for _ in 0..4 {
                if input.is_at_end() || !EcmaCharClassifier::is_hex_digit(input.current_char()) {
                    return Err(lexer_error(
                        "Invalid Unicode escape sequence: expected 4 hex digits",
                        input.current_token_span(),
                    ));
                }
                hex_digits.push(input.advance());
            }
            
            match u32::from_str_radix(&hex_digits, 16) {
                Ok(code_point) => {
                    char::from_u32(code_point).ok_or_else(|| lexer_error(
                        "Invalid Unicode escape sequence: invalid code point",
                        input.current_token_span(),
                    ))
                }
                Err(_) => Err(lexer_error(
                    "Invalid Unicode escape sequence",
                    input.current_token_span(),
                )),
            }
        }
    }
}

impl Scanner for StringScanner {
    type Token = StringLiteral;
    
    fn try_scan(&mut self, input: &mut Input, context: &LexerContext) -> Option<Result<Self::Token>> {
        if self.can_scan(input, context) {
            Some(self.scan_string(input, context))
        } else {
            None
        }
    }
    
    fn can_scan(&self, input: &Input, _context: &LexerContext) -> bool {
        matches!(input.current_char(), '"' | '\'')
    }
    
    fn name(&self) -> &'static str {
        "StringScanner"
    }
}

impl LookaheadScanner for StringScanner {
    fn would_match(&self, input: &mut Input, context: &LexerContext) -> bool {
        self.can_scan(input, context)
    }
    
    fn expected_length(&self, input: &mut Input, _context: &LexerContext) -> Option<usize> {
        if !self.can_scan(input, _context) {
            return None;
        }
        
        let start = input.byte_offset();
        let quote_char = input.current_char();
        let mut temp_input = input.clone();
        
        temp_input.advance(); // skip opening quote
        
        // Scan until closing quote or end
        while !temp_input.is_at_end() && temp_input.current_char() != quote_char {
            let ch = temp_input.current_char();
            
            if ch == '\\' {
                temp_input.advance(); // skip backslash
                if !temp_input.is_at_end() {
                    temp_input.advance(); // skip escaped character
                }
            } else if EcmaCharClassifier::is_line_terminator(ch) {
                // Unterminated string
                break;
            } else {
                temp_input.advance();
            }
        }
        
        if !temp_input.is_at_end() && temp_input.current_char() == quote_char {
            temp_input.advance(); // closing quote
        }
        
        Some(temp_input.byte_offset() - start)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::core::LexerContext;
    
    fn scan_string(source: &str) -> Result<StringLiteral> {
        let mut input = Input::new(source);
        let mut scanner = StringScanner::new();
        let context = LexerContext::new();
        
        scanner.try_scan(&mut input, &context).unwrap()
    }
    
    #[test]
    fn test_simple_strings() {
        match scan_string("\"hello\"").unwrap() {
            StringLiteral::String(s) => assert_eq!(s, "hello"),
            _ => panic!("Expected string literal"),
        }
        
        match scan_string("'world'").unwrap() {
            StringLiteral::String(s) => assert_eq!(s, "world"),
            _ => panic!("Expected string literal"),
        }
    }
    
    #[test]
    fn test_empty_strings() {
        match scan_string("\"\"").unwrap() {
            StringLiteral::String(s) => assert_eq!(s, ""),
            _ => panic!("Expected empty string literal"),
        }
        
        match scan_string("''").unwrap() {
            StringLiteral::String(s) => assert_eq!(s, ""),
            _ => panic!("Expected empty string literal"),
        }
    }
    
    #[test]
    fn test_escape_sequences() {
        match scan_string("\"hello\\nworld\"").unwrap() {
            StringLiteral::String(s) => assert_eq!(s, "hello\nworld"),
            _ => panic!("Expected string literal"),
        }
        
        match scan_string("\"tab\\there\"").unwrap() {
            StringLiteral::String(s) => assert_eq!(s, "tab\there"),
            _ => panic!("Expected string literal"),
        }
        
        match scan_string("\"quote\\\"inside\"").unwrap() {
            StringLiteral::String(s) => assert_eq!(s, "quote\"inside"),
            _ => panic!("Expected string literal"),
        }
    }
    
    #[test]
    fn test_hex_escape_sequences() {
        match scan_string("\"\\x41\"").unwrap() {
            StringLiteral::String(s) => assert_eq!(s, "A"),
            _ => panic!("Expected string literal"),
        }
        
        match scan_string("\"\\x20\"").unwrap() {
            StringLiteral::String(s) => assert_eq!(s, " "),
            _ => panic!("Expected string literal"),
        }
    }
    
    #[test]
    fn test_unicode_escape_sequences() {
        // Standard Unicode escape
        match scan_string("\"\\u0041\"").unwrap() {
            StringLiteral::String(s) => assert_eq!(s, "A"),
            _ => panic!("Expected string literal"),
        }
        
        // Extended Unicode escape
        match scan_string("\"\\u{41}\"").unwrap() {
            StringLiteral::String(s) => assert_eq!(s, "A"),
            _ => panic!("Expected string literal"),
        }
        
        // Extended Unicode with more digits
        match scan_string("\"\\u{1F680}\"").unwrap() {
            StringLiteral::String(s) => assert_eq!(s, "🚀"),
            _ => panic!("Expected string literal"),
        }
    }
    
    #[test]
    fn test_unicode_content() {
        match scan_string("\"café\"").unwrap() {
            StringLiteral::String(s) => assert_eq!(s, "café"),
            _ => panic!("Expected string literal"),
        }
        
        match scan_string("\"🚀🎉\"").unwrap() {
            StringLiteral::String(s) => assert_eq!(s, "🚀🎉"),
            _ => panic!("Expected string literal"),
        }
    }
    
    #[test]
    fn test_unterminated_strings() {
        assert!(scan_string("\"hello").is_err());
        assert!(scan_string("'world").is_err());
        assert!(scan_string("\"hello\n\"").is_err()); // Newline in string
    }
    
    #[test]
    fn test_invalid_escape_sequences() {
        assert!(scan_string("\"\\x\"").is_err()); // Incomplete hex escape
        assert!(scan_string("\"\\u\"").is_err()); // Incomplete unicode escape
        assert!(scan_string("\"\\u{\"").is_err()); // Incomplete extended unicode
        assert!(scan_string("\"\\u{GGGG}\"").is_err()); // Invalid hex digits
    }
    
    #[test]
    fn test_can_scan() {
        let scanner = StringScanner::new();
        let context = LexerContext::new();
        
        assert!(scanner.can_scan(&Input::new("\"hello\""), &context));
        assert!(scanner.can_scan(&Input::new("'world'"), &context));
        assert!(!scanner.can_scan(&Input::new("hello"), &context));
        assert!(!scanner.can_scan(&Input::new("123"), &context));
    }
    
    #[test]
    fn test_expected_length() {
        let mut scanner = StringScanner::new();
        let context = LexerContext::new();
        
        let mut input = Input::new("\"hello\"");
        assert_eq!(scanner.expected_length(&mut input, &context), Some(7));
        
        let mut input2 = Input::new("'world'");
        assert_eq!(scanner.expected_length(&mut input2, &context), Some(7));
        
        let mut input3 = Input::new("\"hello\\nworld\"");
        assert_eq!(scanner.expected_length(&mut input3, &context), Some(14));
    }
}