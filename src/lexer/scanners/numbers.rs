//! Number literal scanner
//!
//! Handles scanning of JavaScript number literals including:
//! - Decimal numbers (42, 3.14, 1e10)
//! - Hexadecimal numbers (0xFF)
//! - Octal numbers (0o777, legacy 0777)
//! - Binary numbers (0b1010)
//! - BigInt literals (123n)

use super::{Scanner, LookaheadScanner, CharClassifier, EcmaCharClassifier, lexer_error};
use crate::error::{Result, Span};
use crate::lexer::core::{Input, LexerContext};

/// Types of number literals
#[derive(Debug, Clone, PartialEq)]
pub enum NumberLiteral {
    /// Regular JavaScript number (f64)
    Number(f64),
    /// BigInt literal
    BigInt(String),
}

/// Number base for parsing
#[derive(Debug, Clone, Copy, PartialEq)]
enum NumberBase {
    Binary = 2,
    Octal = 8,
    Decimal = 10,
    Hexadecimal = 16,
}

/// Number scanner for JavaScript number literals
#[derive(Debug, Default)]
pub struct NumberScanner;

impl NumberScanner {
    /// Create a new number scanner
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Scan a complete number literal
    fn scan_number(&self, input: &mut Input, context: &LexerContext) -> Result<NumberLiteral> {
        input.mark_token_start();
        
        // Check for special base prefixes
        if input.current_char() == '0' && !input.is_at_end() {
            match input.peek_char() {
                Some('x') | Some('X') => return self.scan_hex_number(input),
                Some('o') | Some('O') => return self.scan_octal_number(input),
                Some('b') | Some('B') => return self.scan_binary_number(input),
                Some('0'..='9') => {
                    // Could be legacy octal or decimal
                    return self.scan_decimal_or_legacy_octal(input, context);
                }
                _ => {
                    // Just a zero, fall through to decimal parsing
                }
            }
        }
        
        // Scan as decimal number
        self.scan_decimal_number(input)
    }
    
    /// Scan hexadecimal number (0x...)
    fn scan_hex_number(&self, input: &mut Input) -> Result<NumberLiteral> {
        input.advance(); // consume '0'
        input.advance(); // consume 'x' or 'X'
        
        let mut has_digits = false;
        
        // Scan hex digits
        while EcmaCharClassifier::is_hex_digit(input.current_char()) {
            input.advance();
            has_digits = true;
        }
        
        if !has_digits {
            return Err(lexer_error(
                "Invalid hexadecimal number: no digits after 0x",
                input.current_token_span(),
            ));
        }
        
        // Check for BigInt suffix
        if input.current_char() == 'n' {
            input.advance();
            return Ok(NumberLiteral::BigInt(input.token_text().to_string()));
        }
        
        // Parse as regular number
        let text = input.token_text();
        let without_prefix = &text[2..]; // Remove "0x"
        
        match u64::from_str_radix(without_prefix, 16) {
            Ok(value) => Ok(NumberLiteral::Number(value as f64)),
            Err(_) => Err(lexer_error(
                format!("Invalid hexadecimal number: {}", text),
                input.current_token_span(),
            )),
        }
    }
    
    /// Scan octal number (0o...)
    fn scan_octal_number(&self, input: &mut Input) -> Result<NumberLiteral> {
        input.advance(); // consume '0'
        input.advance(); // consume 'o' or 'O'
        
        let mut has_digits = false;
        
        // Scan octal digits
        while EcmaCharClassifier::is_octal_digit(input.current_char()) {
            input.advance();
            has_digits = true;
        }
        
        if !has_digits {
            return Err(lexer_error(
                "Invalid octal number: no digits after 0o",
                input.current_token_span(),
            ));
        }
        
        // Check for BigInt suffix
        if input.current_char() == 'n' {
            input.advance();
            return Ok(NumberLiteral::BigInt(input.token_text().to_string()));
        }
        
        // Parse as regular number
        let text = input.token_text();
        let without_prefix = &text[2..]; // Remove "0o"
        
        match u64::from_str_radix(without_prefix, 8) {
            Ok(value) => Ok(NumberLiteral::Number(value as f64)),
            Err(_) => Err(lexer_error(
                format!("Invalid octal number: {}", text),
                input.current_token_span(),
            )),
        }
    }
    
    /// Scan binary number (0b...)
    fn scan_binary_number(&self, input: &mut Input) -> Result<NumberLiteral> {
        input.advance(); // consume '0'
        input.advance(); // consume 'b' or 'B'
        
        let mut has_digits = false;
        
        // Scan binary digits
        while EcmaCharClassifier::is_binary_digit(input.current_char()) {
            input.advance();
            has_digits = true;
        }
        
        if !has_digits {
            return Err(lexer_error(
                "Invalid binary number: no digits after 0b",
                input.current_token_span(),
            ));
        }
        
        // Check for BigInt suffix
        if input.current_char() == 'n' {
            input.advance();
            return Ok(NumberLiteral::BigInt(input.token_text().to_string()));
        }
        
        // Parse as regular number
        let text = input.token_text();
        let without_prefix = &text[2..]; // Remove "0b"
        
        match u64::from_str_radix(without_prefix, 2) {
            Ok(value) => Ok(NumberLiteral::Number(value as f64)),
            Err(_) => Err(lexer_error(
                format!("Invalid binary number: {}", text),
                input.current_token_span(),
            )),
        }
    }
    
    /// Scan decimal number or legacy octal
    fn scan_decimal_or_legacy_octal(&self, input: &mut Input, context: &LexerContext) -> Result<NumberLiteral> {
        let start_pos = input.byte_offset();
        
        // If strict mode, don't allow legacy octal
        if context.config.strict_mode {
            return self.scan_decimal_number(input);
        }
        
        // Check if it's a valid legacy octal (all digits 0-7)
        let mut temp_input = input.clone();
        temp_input.advance(); // skip leading '0'
        
        let mut is_octal = true;
        let mut has_8_or_9 = false;
        
        while EcmaCharClassifier::is_decimal_digit(temp_input.current_char()) {
            let ch = temp_input.current_char();
            if ch == '8' || ch == '9' {
                has_8_or_9 = true;
                is_octal = false;
            }
            temp_input.advance();
        }
        
        // If we have a decimal point or exponent, it's decimal
        if temp_input.current_char() == '.' || 
           temp_input.current_char() == 'e' || temp_input.current_char() == 'E' {
            return self.scan_decimal_number(input);
        }
        
        if is_octal && !has_8_or_9 {
            // Parse as legacy octal
            input.advance(); // consume '0'
            
            while EcmaCharClassifier::is_octal_digit(input.current_char()) {
                input.advance();
            }
            
            let text = input.token_text();
            match u64::from_str_radix(&text[1..], 8) {
                Ok(value) => Ok(NumberLiteral::Number(value as f64)),
                Err(_) => self.scan_decimal_number_from_start(input, start_pos),
            }
        } else {
            // Parse as decimal
            self.scan_decimal_number(input)
        }
    }
    
    /// Scan decimal number
    fn scan_decimal_number(&self, input: &mut Input) -> Result<NumberLiteral> {
        // Scan integer part
        while EcmaCharClassifier::is_decimal_digit(input.current_char()) {
            input.advance();
        }
        
        // Check for decimal point
        if input.current_char() == '.' {
            input.advance();
            
            // Scan fractional part
            while EcmaCharClassifier::is_decimal_digit(input.current_char()) {
                input.advance();
            }
        }
        
        // Check for exponent
        if input.current_char() == 'e' || input.current_char() == 'E' {
            input.advance();
            
            // Optional sign
            if input.current_char() == '+' || input.current_char() == '-' {
                input.advance();
            }
            
            // Exponent digits
            if !EcmaCharClassifier::is_decimal_digit(input.current_char()) {
                return Err(lexer_error(
                    "Invalid number: expected digits after exponent",
                    input.current_token_span(),
                ));
            }
            
            while EcmaCharClassifier::is_decimal_digit(input.current_char()) {
                input.advance();
            }
        }
        
        // Check for BigInt suffix
        if input.current_char() == 'n' {
            input.advance();
            return Ok(NumberLiteral::BigInt(input.token_text().to_string()));
        }
        
        // Parse the number
        let text = input.token_text();
        match text.parse::<f64>() {
            Ok(value) => Ok(NumberLiteral::Number(value)),
            Err(_) => Err(lexer_error(
                format!("Invalid number: {}", text),
                input.current_token_span(),
            )),
        }
    }
    
    /// Scan decimal number from a specific start position
    fn scan_decimal_number_from_start(&self, input: &mut Input, _start_pos: usize) -> Result<NumberLiteral> {
        // Reset to start position and scan as decimal
        // This is a fallback for complex octal/decimal disambiguation
        self.scan_decimal_number(input)
    }
}

impl Scanner for NumberScanner {
    type Token = NumberLiteral;
    
    fn try_scan(&mut self, input: &mut Input, context: &LexerContext) -> Option<Result<Self::Token>> {
        if self.can_scan(input, context) {
            Some(self.scan_number(input, context))
        } else {
            None
        }
    }
    
    fn can_scan(&self, input: &Input, _context: &LexerContext) -> bool {
        EcmaCharClassifier::is_decimal_digit(input.current_char())
    }
    
    fn name(&self) -> &'static str {
        "NumberScanner"
    }
}

impl LookaheadScanner for NumberScanner {
    fn would_match(&self, input: &mut Input, context: &LexerContext) -> bool {
        self.can_scan(input, context)
    }
    
    fn expected_length(&self, input: &mut Input, _context: &LexerContext) -> Option<usize> {
        if !self.can_scan(input, _context) {
            return None;
        }
        
        let start = input.byte_offset();
        let mut temp_input = input.clone();
        
        // Quick estimate: scan until we hit a non-numeric character
        while EcmaCharClassifier::is_decimal_digit(temp_input.current_char()) ||
              temp_input.current_char() == '.' ||
              temp_input.current_char() == 'e' ||
              temp_input.current_char() == 'E' ||
              temp_input.current_char() == '+' ||
              temp_input.current_char() == '-' ||
              temp_input.current_char() == 'x' ||
              temp_input.current_char() == 'X' ||
              temp_input.current_char() == 'o' ||
              temp_input.current_char() == 'O' ||
              temp_input.current_char() == 'b' ||
              temp_input.current_char() == 'B' ||
              EcmaCharClassifier::is_hex_digit(temp_input.current_char()) {
            temp_input.advance();
        }
        
        // Check for BigInt suffix
        if temp_input.current_char() == 'n' {
            temp_input.advance();
        }
        
        Some(temp_input.byte_offset() - start)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::core::LexerContext;
    
    fn scan_number(source: &str) -> Result<NumberLiteral> {
        let mut input = Input::new(source);
        let mut scanner = NumberScanner::new();
        let context = LexerContext::new();
        
        scanner.try_scan(&mut input, &context).unwrap()
    }
    
    #[test]
    fn test_decimal_integers() {
        assert_eq!(scan_number("42").unwrap(), NumberLiteral::Number(42.0));
        assert_eq!(scan_number("0").unwrap(), NumberLiteral::Number(0.0));
        assert_eq!(scan_number("123").unwrap(), NumberLiteral::Number(123.0));
    }
    
    #[test]
    fn test_decimal_floats() {
        assert_eq!(scan_number("3.14").unwrap(), NumberLiteral::Number(3.14));
        assert_eq!(scan_number("0.5").unwrap(), NumberLiteral::Number(0.5));
        assert_eq!(scan_number("123.456").unwrap(), NumberLiteral::Number(123.456));
    }
    
    #[test]
    fn test_scientific_notation() {
        assert_eq!(scan_number("1e10").unwrap(), NumberLiteral::Number(1e10));
        assert_eq!(scan_number("2.5e-3").unwrap(), NumberLiteral::Number(2.5e-3));
        assert_eq!(scan_number("1E+5").unwrap(), NumberLiteral::Number(1e5));
    }
    
    #[test]
    fn test_hexadecimal() {
        assert_eq!(scan_number("0xFF").unwrap(), NumberLiteral::Number(255.0));
        assert_eq!(scan_number("0x10").unwrap(), NumberLiteral::Number(16.0));
        assert_eq!(scan_number("0xDEADBEEF").unwrap(), NumberLiteral::Number(0xDEADBEEFu32 as f64));
    }
    
    #[test]
    fn test_octal() {
        assert_eq!(scan_number("0o777").unwrap(), NumberLiteral::Number(511.0));
        assert_eq!(scan_number("0o10").unwrap(), NumberLiteral::Number(8.0));
    }
    
    #[test]
    fn test_binary() {
        assert_eq!(scan_number("0b1010").unwrap(), NumberLiteral::Number(10.0));
        assert_eq!(scan_number("0b11111111").unwrap(), NumberLiteral::Number(255.0));
    }
    
    #[test]
    fn test_bigint() {
        match scan_number("123n").unwrap() {
            NumberLiteral::BigInt(s) => assert_eq!(s, "123n"),
            _ => panic!("Expected BigInt"),
        }
        
        match scan_number("0xFFn").unwrap() {
            NumberLiteral::BigInt(s) => assert_eq!(s, "0xFFn"),
            _ => panic!("Expected BigInt"),
        }
    }
    
    #[test]
    fn test_invalid_numbers() {
        assert!(scan_number("0x").is_err());
        assert!(scan_number("0o").is_err());
        assert!(scan_number("0b").is_err());
        assert!(scan_number("1e").is_err());
    }
    
    #[test]
    fn test_can_scan() {
        let scanner = NumberScanner::new();
        let context = LexerContext::new();
        
        assert!(scanner.can_scan(&Input::new("123"), &context));
        assert!(scanner.can_scan(&Input::new("0xFF"), &context));
        assert!(!scanner.can_scan(&Input::new("abc"), &context));
        assert!(!scanner.can_scan(&Input::new(""), &context));
    }
    
    #[test]
    fn test_expected_length() {
        let mut scanner = NumberScanner::new();
        let context = LexerContext::new();
        
        let mut input = Input::new("123.45");
        assert_eq!(scanner.expected_length(&mut input, &context), Some(6));
        
        let mut input2 = Input::new("0xFF");
        assert_eq!(scanner.expected_length(&mut input2, &context), Some(4));
        
        let mut input3 = Input::new("1e10");
        assert_eq!(scanner.expected_length(&mut input3, &context), Some(4));
    }
}