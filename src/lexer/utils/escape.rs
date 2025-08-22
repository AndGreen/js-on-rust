//! Escape sequence utilities for JavaScript lexer
//!
//! Handles parsing and validation of escape sequences in string literals,
//! template literals, and regular expressions according to the ECMAScript
//! specification.

use crate::error::{Error, Result, Span};
use crate::lexer::core::Input;

/// Escape sequence parser and validator
pub struct EscapeSequenceParser;

/// Represents a parsed escape sequence
#[derive(Debug, Clone, PartialEq)]
pub enum EscapeSequence {
    /// Simple character escape (\n, \t, etc.)
    Character(char),
    /// Null character (\0)
    Null,
    /// Hexadecimal escape (\xHH)
    Hexadecimal(u8),
    /// Unicode escape (\uHHHH)
    Unicode(char),
    /// Unicode code point escape (\u{HHHHHH})
    UnicodeCodePoint(char),
    /// Octal escape (\377) - legacy, not in strict mode
    Octal(u8),
    /// Line continuation (backslash followed by line terminator)
    LineContinuation,
    /// Invalid escape sequence
    Invalid(String),
}

impl EscapeSequenceParser {
    /// Parse an escape sequence starting after the backslash
    pub fn parse_escape_sequence(input: &mut Input) -> Result<EscapeSequence> {
        if input.is_at_end() {
            return Err(Error::lexer(
                "Unexpected end of input after escape character".to_string(),
                input.current_token_span(),
            ));
        }
        
        let escape_char = input.advance();
        
        match escape_char {
            // Simple character escapes
            'n' => Ok(EscapeSequence::Character('\n')),
            't' => Ok(EscapeSequence::Character('\t')),
            'r' => Ok(EscapeSequence::Character('\r')),
            'b' => Ok(EscapeSequence::Character('\u{0008}')), // backspace
            'f' => Ok(EscapeSequence::Character('\u{000C}')), // form feed
            'v' => Ok(EscapeSequence::Character('\u{000B}')), // vertical tab
            '\\' => Ok(EscapeSequence::Character('\\')),
            '\'' => Ok(EscapeSequence::Character('\'')),
            '"' => Ok(EscapeSequence::Character('"')),
            '/' => Ok(EscapeSequence::Character('/')), // JSON compatibility
            
            // Null character
            '0' => {
                // Check if followed by a digit (would make it octal)
                if input.current_char().is_ascii_digit() {
                    Self::parse_octal_escape(input, escape_char)
                } else {
                    Ok(EscapeSequence::Null)
                }
            }
            
            // Hexadecimal escape
            'x' => Self::parse_hex_escape(input),
            
            // Unicode escapes
            'u' => Self::parse_unicode_escape(input),
            
            // Line continuation
            '\n' => Ok(EscapeSequence::LineContinuation),
            '\r' => {
                // Handle CRLF
                if input.current_char() == '\n' {
                    input.advance();
                }
                Ok(EscapeSequence::LineContinuation)
            }
            '\u{2028}' | '\u{2029}' => Ok(EscapeSequence::LineContinuation),
            
            // Octal escapes (legacy)
            '1'..='7' => Self::parse_octal_escape(input, escape_char),
            
            // Any other character - include literally (non-standard but common)
            ch => Ok(EscapeSequence::Character(ch)),
        }
    }
    
    /// Parse hexadecimal escape sequence (\xHH)
    fn parse_hex_escape(input: &mut Input) -> Result<EscapeSequence> {
        let mut hex_digits = String::new();
        
        for _ in 0..2 {
            if input.is_at_end() || !input.current_char().is_ascii_hexdigit() {
                return Ok(EscapeSequence::Invalid(format!("\\x{}", hex_digits)));
            }
            hex_digits.push(input.advance());
        }
        
        match u8::from_str_radix(&hex_digits, 16) {
            Ok(value) => Ok(EscapeSequence::Hexadecimal(value)),
            Err(_) => Ok(EscapeSequence::Invalid(format!("\\x{}", hex_digits))),
        }
    }
    
    /// Parse Unicode escape sequence (\uHHHH or \u{HHHHHH})
    fn parse_unicode_escape(input: &mut Input) -> Result<EscapeSequence> {
        if input.current_char() == '{' {
            // Extended Unicode escape \u{HHHHHH}
            input.advance(); // consume '{'
            
            let mut hex_digits = String::new();
            
            while !input.is_at_end() && input.current_char() != '}' {
                let ch = input.current_char();
                if ch.is_ascii_hexdigit() {
                    hex_digits.push(input.advance());
                } else {
                    return Ok(EscapeSequence::Invalid(format!("\\u{{{}", hex_digits)));
                }
                
                // Limit to reasonable length
                if hex_digits.len() > 6 {
                    return Ok(EscapeSequence::Invalid(format!("\\u{{{}", hex_digits)));
                }
            }
            
            if input.current_char() != '}' {
                return Ok(EscapeSequence::Invalid(format!("\\u{{{}", hex_digits)));
            }
            
            input.advance(); // consume '}'
            
            if hex_digits.is_empty() {
                return Ok(EscapeSequence::Invalid("\\u{}".to_string()));
            }
            
            match u32::from_str_radix(&hex_digits, 16) {
                Ok(code_point) => {
                    match char::from_u32(code_point) {
                        Some(ch) => Ok(EscapeSequence::UnicodeCodePoint(ch)),
                        None => Ok(EscapeSequence::Invalid(format!("\\u{{{}}}", hex_digits))),
                    }
                }
                Err(_) => Ok(EscapeSequence::Invalid(format!("\\u{{{}}}", hex_digits))),
            }
        } else {
            // Standard Unicode escape \uHHHH
            let mut hex_digits = String::new();
            
            for _ in 0..4 {
                if input.is_at_end() || !input.current_char().is_ascii_hexdigit() {
                    return Ok(EscapeSequence::Invalid(format!("\\u{}", hex_digits)));
                }
                hex_digits.push(input.advance());
            }
            
            match u32::from_str_radix(&hex_digits, 16) {
                Ok(code_point) => {
                    match char::from_u32(code_point) {
                        Some(ch) => Ok(EscapeSequence::Unicode(ch)),
                        None => Ok(EscapeSequence::Invalid(format!("\\u{}", hex_digits))),
                    }
                }
                Err(_) => Ok(EscapeSequence::Invalid(format!("\\u{}", hex_digits))),
            }
        }
    }
    
    /// Parse octal escape sequence (\377)
    fn parse_octal_escape(input: &mut Input, first_digit: char) -> Result<EscapeSequence> {
        let mut octal_digits = String::new();
        octal_digits.push(first_digit);
        
        // Parse up to 2 more octal digits
        for _ in 0..2 {
            if input.is_at_end() || !input.current_char().is_ascii_digit() {
                break;
            }
            
            let digit = input.current_char();
            if digit > '7' {
                break;
            }
            
            octal_digits.push(input.advance());
        }
        
        match u8::from_str_radix(&octal_digits, 8) {
            Ok(value) => Ok(EscapeSequence::Octal(value)),
            Err(_) => Ok(EscapeSequence::Invalid(format!("\\{}", octal_digits))),
        }
    }
    
    /// Convert escape sequence to character
    pub fn to_character(escape: &EscapeSequence) -> Option<char> {
        match escape {
            EscapeSequence::Character(ch) => Some(*ch),
            EscapeSequence::Null => Some('\0'),
            EscapeSequence::Hexadecimal(byte) => Some(*byte as char),
            EscapeSequence::Unicode(ch) => Some(*ch),
            EscapeSequence::UnicodeCodePoint(ch) => Some(*ch),
            EscapeSequence::Octal(byte) => Some(*byte as char),
            EscapeSequence::LineContinuation => None, // Line continuation doesn't produce a character
            EscapeSequence::Invalid(_) => None,
        }
    }
    
    /// Check if escape sequence is valid in strict mode
    pub fn is_valid_in_strict_mode(escape: &EscapeSequence) -> bool {
        // Octal escapes are not allowed in strict mode
        !matches!(escape, EscapeSequence::Octal(_))
    }
    
    /// Get the string representation of an escape sequence
    pub fn to_string(escape: &EscapeSequence) -> String {
        match escape {
            EscapeSequence::Character(ch) => match ch {
                '\n' => "\\n".to_string(),
                '\t' => "\\t".to_string(),
                '\r' => "\\r".to_string(),
                '\u{0008}' => "\\b".to_string(),
                '\u{000C}' => "\\f".to_string(),
                '\u{000B}' => "\\v".to_string(),
                '\\' => "\\\\".to_string(),
                '\'' => "\\'".to_string(),
                '"' => "\\\"".to_string(),
                '/' => "\\/".to_string(),
                _ => ch.to_string(),
            },
            EscapeSequence::Null => "\\0".to_string(),
            EscapeSequence::Hexadecimal(byte) => format!("\\x{:02X}", byte),
            EscapeSequence::Unicode(ch) => format!("\\u{:04X}", *ch as u32),
            EscapeSequence::UnicodeCodePoint(ch) => format!("\\u{{{:X}}}", *ch as u32),
            EscapeSequence::Octal(byte) => format!("\\{:o}", byte),
            EscapeSequence::LineContinuation => "".to_string(),
            EscapeSequence::Invalid(s) => s.clone(),
        }
    }
    
    /// Validate escape sequence in template literal context
    pub fn validate_in_template(escape: &EscapeSequence) -> bool {
        // Template literals allow invalid escape sequences in tagged templates
        match escape {
            EscapeSequence::Invalid(_) => false, // But not in regular templates
            _ => true,
        }
    }
    
    /// Parse template literal escape sequences (more permissive)
    pub fn parse_template_escape(input: &mut Input) -> Result<EscapeSequence> {
        // Template literals are more permissive with escape sequences
        // Invalid sequences are preserved as-is in tagged templates
        match Self::parse_escape_sequence(input) {
            Ok(seq) => Ok(seq),
            Err(_) => {
                // In template literals, invalid escapes might be preserved
                // For now, treat as invalid
                Ok(EscapeSequence::Invalid("\\".to_string()))
            }
        }
    }
}

/// Escape sequence validation utilities
pub struct EscapeValidator;

impl EscapeValidator {
    /// Validate all escape sequences in a string
    pub fn validate_string_escapes(content: &str, strict_mode: bool) -> Result<Vec<EscapeSequence>> {
        let mut escapes = Vec::new();
        let mut input = Input::new(content);
        
        while !input.is_at_end() {
            if input.current_char() == '\\' {
                input.advance(); // consume backslash
                let escape = EscapeSequenceParser::parse_escape_sequence(&mut input)?;
                
                if strict_mode && !EscapeSequenceParser::is_valid_in_strict_mode(&escape) {
                    return Err(Error::lexer(
                        "Octal escape sequences are not allowed in strict mode".to_string(),
                        input.current_token_span(),
                    ));
                }
                
                escapes.push(escape);
            } else {
                input.advance();
            }
        }
        
        Ok(escapes)
    }
    
    /// Check if a string contains any escape sequences
    pub fn has_escape_sequences(content: &str) -> bool {
        content.contains('\\')
    }
    
    /// Count escape sequences in a string
    pub fn count_escape_sequences(content: &str) -> usize {
        content.matches('\\').count()
    }
    
    /// Unescape a string with escape sequences
    pub fn unescape_string(content: &str) -> Result<String> {
        let mut result = String::new();
        let mut input = Input::new(content);
        
        while !input.is_at_end() {
            if input.current_char() == '\\' {
                input.advance(); // consume backslash
                let escape = EscapeSequenceParser::parse_escape_sequence(&mut input)?;
                
                if let Some(ch) = EscapeSequenceParser::to_character(&escape) {
                    result.push(ch);
                }
                // Line continuations don't add characters
            } else {
                result.push(input.advance());
            }
        }
        
        Ok(result)
    }
    
    /// Escape a string for JavaScript string literal
    pub fn escape_string(content: &str) -> String {
        let mut result = String::new();
        
        for ch in content.chars() {
            match ch {
                '\n' => result.push_str("\\n"),
                '\t' => result.push_str("\\t"),
                '\r' => result.push_str("\\r"),
                '\u{0008}' => result.push_str("\\b"),
                '\u{000C}' => result.push_str("\\f"),
                '\u{000B}' => result.push_str("\\v"),
                '\\' => result.push_str("\\\\"),
                '"' => result.push_str("\\\""),
                '\'' => result.push_str("\\'"),
                _ if ch.is_control() => {
                    if (ch as u32) <= 0xFF {
                        result.push_str(&format!("\\x{:02X}", ch as u32));
                    } else {
                        result.push_str(&format!("\\u{:04X}", ch as u32));
                    }
                }
                _ => result.push(ch),
            }
        }
        
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    fn parse_escape(input_str: &str) -> Result<EscapeSequence> {
        let mut input = Input::new(input_str);
        EscapeSequenceParser::parse_escape_sequence(&mut input)
    }
    
    #[test]
    fn test_simple_escapes() {
        assert_eq!(parse_escape("n").unwrap(), EscapeSequence::Character('\n'));
        assert_eq!(parse_escape("t").unwrap(), EscapeSequence::Character('\t'));
        assert_eq!(parse_escape("r").unwrap(), EscapeSequence::Character('\r'));
        assert_eq!(parse_escape("\\").unwrap(), EscapeSequence::Character('\\'));
        assert_eq!(parse_escape("'").unwrap(), EscapeSequence::Character('\''));
        assert_eq!(parse_escape("\"").unwrap(), EscapeSequence::Character('"'));
    }
    
    #[test]
    fn test_null_escape() {
        assert_eq!(parse_escape("0").unwrap(), EscapeSequence::Null);
    }
    
    #[test]
    fn test_hex_escapes() {
        assert_eq!(parse_escape("x41").unwrap(), EscapeSequence::Hexadecimal(0x41));
        assert_eq!(parse_escape("x20").unwrap(), EscapeSequence::Hexadecimal(0x20));
        assert_eq!(parse_escape("xFF").unwrap(), EscapeSequence::Hexadecimal(0xFF));
        
        // Invalid hex escapes
        assert!(matches!(parse_escape("x").unwrap(), EscapeSequence::Invalid(_)));
        assert!(matches!(parse_escape("xGG").unwrap(), EscapeSequence::Invalid(_)));
    }
    
    #[test]
    fn test_unicode_escapes() {
        assert_eq!(parse_escape("u0041").unwrap(), EscapeSequence::Unicode('A'));
        assert_eq!(parse_escape("u0020").unwrap(), EscapeSequence::Unicode(' '));
        
        // Invalid unicode escapes
        assert!(matches!(parse_escape("u").unwrap(), EscapeSequence::Invalid(_)));
        assert!(matches!(parse_escape("uGGGG").unwrap(), EscapeSequence::Invalid(_)));
    }
    
    #[test]
    fn test_unicode_code_point_escapes() {
        assert_eq!(parse_escape("u{41}").unwrap(), EscapeSequence::UnicodeCodePoint('A'));
        assert_eq!(parse_escape("u{1F680}").unwrap(), EscapeSequence::UnicodeCodePoint('🚀'));
        
        // Invalid code point escapes
        assert!(matches!(parse_escape("u{").unwrap(), EscapeSequence::Invalid(_)));
        assert!(matches!(parse_escape("u{}").unwrap(), EscapeSequence::Invalid(_)));
        assert!(matches!(parse_escape("u{GGGG}").unwrap(), EscapeSequence::Invalid(_)));
    }
    
    #[test]
    fn test_octal_escapes() {
        assert_eq!(parse_escape("377").unwrap(), EscapeSequence::Octal(255));
        assert_eq!(parse_escape("123").unwrap(), EscapeSequence::Octal(83));
    }
    
    #[test]
    fn test_line_continuation() {
        assert_eq!(parse_escape("\n").unwrap(), EscapeSequence::LineContinuation);
        assert_eq!(parse_escape("\r").unwrap(), EscapeSequence::LineContinuation);
    }
    
    #[test]
    fn test_to_character() {
        assert_eq!(EscapeSequenceParser::to_character(&EscapeSequence::Character('A')), Some('A'));
        assert_eq!(EscapeSequenceParser::to_character(&EscapeSequence::Null), Some('\0'));
        assert_eq!(EscapeSequenceParser::to_character(&EscapeSequence::Hexadecimal(65)), Some('A'));
        assert_eq!(EscapeSequenceParser::to_character(&EscapeSequence::Unicode('A')), Some('A'));
        assert_eq!(EscapeSequenceParser::to_character(&EscapeSequence::LineContinuation), None);
    }
    
    #[test]
    fn test_strict_mode_validation() {
        assert!(EscapeSequenceParser::is_valid_in_strict_mode(&EscapeSequence::Character('A')));
        assert!(EscapeSequenceParser::is_valid_in_strict_mode(&EscapeSequence::Unicode('A')));
        assert!(!EscapeSequenceParser::is_valid_in_strict_mode(&EscapeSequence::Octal(123)));
    }
    
    #[test]
    fn test_to_string() {
        assert_eq!(EscapeSequenceParser::to_string(&EscapeSequence::Character('\n')), "\\n");
        assert_eq!(EscapeSequenceParser::to_string(&EscapeSequence::Null), "\\0");
        assert_eq!(EscapeSequenceParser::to_string(&EscapeSequence::Hexadecimal(65)), "\\x41");
        assert_eq!(EscapeSequenceParser::to_string(&EscapeSequence::Unicode('A')), "\\u0041");
    }
    
    #[test]
    fn test_string_validation() {
        let escapes = EscapeValidator::validate_string_escapes("hello\\nworld\\t", false).unwrap();
        assert_eq!(escapes.len(), 2);
        assert_eq!(escapes[0], EscapeSequence::Character('\n'));
        assert_eq!(escapes[1], EscapeSequence::Character('\t'));
    }
    
    #[test]
    fn test_unescape_string() {
        let result = EscapeValidator::unescape_string("hello\\nworld\\t").unwrap();
        assert_eq!(result, "hello\nworld\t");
        
        let result = EscapeValidator::unescape_string("\\u0041\\x42\\103").unwrap();
        assert_eq!(result, "ABC");
    }
    
    #[test]
    fn test_escape_string() {
        let result = EscapeValidator::escape_string("hello\nworld\t");
        assert_eq!(result, "hello\\nworld\\t");
        
        let result = EscapeValidator::escape_string("quote\"inside");
        assert_eq!(result, "quote\\\"inside");
    }
    
    #[test]
    fn test_has_escape_sequences() {
        assert!(EscapeValidator::has_escape_sequences("hello\\nworld"));
        assert!(!EscapeValidator::has_escape_sequences("hello world"));
    }
    
    #[test]
    fn test_count_escape_sequences() {
        assert_eq!(EscapeValidator::count_escape_sequences("hello\\nworld\\t"), 2);
        assert_eq!(EscapeValidator::count_escape_sequences("hello world"), 0);
    }
}