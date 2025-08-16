//! JavaScript lexer implementation

use super::token::{Token, TokenKind, Keyword};
use crate::error::{Error, Result, Span};

/// JavaScript lexer
pub struct Lexer<'a> {
    source: &'a str,
    current: usize,
    line: u32,
    column: u32,
    start_line: u32,
    start_column: u32,
    token_start: usize,
}

impl<'a> Lexer<'a> {
    /// Create a new lexer for the given source
    pub fn new(source: &'a str) -> Self {
        Self {
            source,
            current: 0,
            line: 1,
            column: 1,
            start_line: 1,
            start_column: 1,
            token_start: 0,
        }
    }
    
    /// Tokenize the entire source and return a vector of tokens
    pub fn tokenize(&mut self) -> Result<Vec<Token>> {
        let mut tokens = Vec::new();
        
        while !self.is_at_end() {
            // Skip whitespace but track position
            if self.current_char().is_whitespace() {
                if self.current_char() == '\n' {
                    self.line += 1;
                    self.column = 1;
                } else {
                    self.column += 1;
                }
                self.advance();
                continue;
            }
            
            // Skip comments
            if self.current_char() == '/' && self.peek() == Some('/') {
                self.skip_line_comment();
                continue;
            }
            
            if self.current_char() == '/' && self.peek() == Some('*') {
                self.skip_block_comment()?;
                continue;
            }
            
            // Start of a new token
            self.start_line = self.line;
            self.start_column = self.column;
            self.token_start = self.current;
            
            let token = self.scan_token()?;
            tokens.push(token);
        }
        
        // Add EOF token
        let eof_span = Span::new(self.current, self.current, self.line, self.column);
        tokens.push(Token::new(TokenKind::Eof, eof_span, String::new()));
        
        Ok(tokens)
    }
    
    /// Scan a single token
    fn scan_token(&mut self) -> Result<Token> {
        let start_pos = self.current;
        let c = self.advance();
        
        let kind = match c {
            // Numbers
            '0'..='9' => self.scan_number()?,
            
            // Strings
            '"' | '\'' => self.scan_string(c)?,
            
            // Identifiers and keywords
            'a'..='z' | 'A'..='Z' | '_' | '$' => self.scan_identifier(),
            
            // Single character tokens that we'll handle in the next task
            '(' => TokenKind::LeftParen,
            ')' => TokenKind::RightParen,
            '{' => TokenKind::LeftBrace,
            '}' => TokenKind::RightBrace,
            '[' => TokenKind::LeftBracket,
            ']' => TokenKind::RightBracket,
            ',' => TokenKind::Comma,
            ';' => TokenKind::Semicolon,
            '.' => TokenKind::Dot,
            ':' => TokenKind::Colon,
            '?' => TokenKind::Question,
            '~' => TokenKind::Tilde,
            
            // Complex operators
            '+' => self.scan_plus_operator()?,
            '-' => self.scan_minus_operator()?,
            '*' => self.scan_star_operator()?,
            '/' => TokenKind::Slash, // Already handled division
            '%' => self.scan_percent_operator()?,
            '=' => self.scan_equal_operator()?,
            '!' => self.scan_bang_operator()?,
            '<' => self.scan_less_operator()?,
            '>' => self.scan_greater_operator()?,
            '&' => self.scan_amp_operator()?,
            '|' => self.scan_pipe_operator()?,
            '^' => TokenKind::Caret,
            
            _ => {
                return Err(Error::lexer(
                    format!("Unexpected character: '{}'", c),
                    self.make_span(self.token_start, self.current),
                ));
            }
        };
        
        let end = self.current;
        let span = self.make_span(start_pos, end);
        let text = self.source[start_pos..end].to_string();
        
        Ok(Token::new(kind, span, text))
    }
    
    /// Scan a number literal
    fn scan_number(&mut self) -> Result<TokenKind> {
        // Scan integer part
        while self.current_char().is_ascii_digit() {
            self.advance();
        }
        
        // Check for decimal point
        if self.current_char() == '.' && self.peek().map(|c| c.is_ascii_digit()).unwrap_or(false) {
            self.advance(); // consume '.'
            
            // Scan fractional part
            while self.current_char().is_ascii_digit() {
                self.advance();
            }
        }
        
        // Check for exponent
        if matches!(self.current_char(), 'e' | 'E') {
            self.advance(); // consume 'e' or 'E'
            
            // Optional sign
            if matches!(self.current_char(), '+' | '-') {
                self.advance();
            }
            
            // Exponent digits
            if !self.current_char().is_ascii_digit() {
                return Err(Error::lexer(
                    "Invalid number: expected digits after exponent".to_string(),
                    self.make_span(self.current, self.current + 1),
                ));
            }
            
            while self.current_char().is_ascii_digit() {
                self.advance();
            }
        }
        
        // Parse the number
        let number_text = &self.source[self.token_start..self.current];
        
        match number_text.parse::<f64>() {
            Ok(value) => Ok(TokenKind::Number(value)),
            Err(_) => Err(Error::lexer(
                format!("Invalid number: {}", number_text),
                self.make_span(self.token_start, self.current),
            )),
        }
    }
    
    /// Scan a string literal
    fn scan_string(&mut self, quote: char) -> Result<TokenKind> {
        let mut value = String::new();
        
        while !self.is_at_end() && self.current_char() != quote {
            if self.current_char() == '\\' {
                // Handle escape sequences
                self.advance(); // consume '\'
                
                if self.is_at_end() {
                    return Err(Error::lexer(
                        "Unterminated string: unexpected end of input after escape".to_string(),
                        self.make_span(self.current - 1, self.current),
                    ));
                }
                
                let escaped = match self.advance() {
                    'n' => '\n',
                    't' => '\t',
                    'r' => '\r',
                    '\\' => '\\',
                    '\'' => '\'',
                    '"' => '"',
                    '0' => '\0',
                    'x' => {
                        // Hexadecimal escape sequence \xHH
                        let hex = self.scan_hex_escape(2)?;
                        char::from(hex as u8)
                    },
                    'u' => {
                        // Unicode escape sequence \uHHHH
                        if self.current_char() == '{' {
                            // \u{HHHHHH}
                            self.advance(); // consume '{'
                            let mut hex_digits = String::new();
                            while self.current_char() != '}' && !self.is_at_end() {
                                if self.current_char().is_ascii_hexdigit() {
                                    hex_digits.push(self.advance());
                                } else {
                                    return Err(Error::lexer(
                                        "Invalid unicode escape sequence".to_string(),
                                        self.make_span(self.current, self.current + 1),
                                    ));
                                }
                            }
                            if self.current_char() == '}' {
                                self.advance(); // consume '}'
                            }
                            
                            let code_point = u32::from_str_radix(&hex_digits, 16)
                                .map_err(|_| Error::lexer(
                                    "Invalid unicode escape sequence".to_string(),
                                    self.make_span(self.current - hex_digits.len(), self.current),
                                ))?;
                                
                            char::from_u32(code_point).ok_or_else(|| Error::lexer(
                                "Invalid unicode code point".to_string(),
                                self.make_span(self.current - hex_digits.len(), self.current),
                            ))?
                        } else {
                            // \uHHHH
                            let hex = self.scan_hex_escape(4)?;
                            char::from_u32(hex).ok_or_else(|| Error::lexer(
                                "Invalid unicode escape sequence".to_string(),
                                self.make_span(self.current - 4, self.current),
                            ))?
                        }
                    },
                    c => c, // For any other character, include it literally
                };
                
                value.push(escaped);
            } else {
                if self.current_char() == '\n' {
                    self.line += 1;
                    self.column = 1;
                } else {
                    self.column += 1;
                }
                value.push(self.advance());
            }
        }
        
        if self.is_at_end() {
            return Err(Error::lexer(
                "Unterminated string literal".to_string(),
                self.make_span(self.token_start, self.current),
            ));
        }
        
        // Consume closing quote
        self.advance();
        
        Ok(TokenKind::String(value))
    }
    
    /// Scan an identifier or keyword
    fn scan_identifier(&mut self) -> TokenKind {
        while self.is_identifier_continue(self.current_char()) {
            self.advance();
        }
        
        let text = &self.source[self.token_start..self.current];
        
        // Check if it's a keyword
        if let Some(keyword) = Keyword::from_str(text) {
            match keyword {
                Keyword::True => TokenKind::Boolean(true),
                Keyword::False => TokenKind::Boolean(false),
                Keyword::Null => TokenKind::Null,
                Keyword::Undefined => TokenKind::Undefined,
                _ => TokenKind::Keyword(keyword),
            }
        } else {
            TokenKind::Identifier(text.to_string())
        }
    }
    
    /// Check if a character can start an identifier
    fn is_identifier_start(&self, c: char) -> bool {
        c.is_alphabetic() || c == '_' || c == '$' || unicode_xid::UnicodeXID::is_xid_start(c)
    }
    
    /// Check if a character can continue an identifier
    fn is_identifier_continue(&self, c: char) -> bool {
        c.is_alphanumeric() || c == '_' || c == '$' || unicode_xid::UnicodeXID::is_xid_continue(c)
    }
    
    /// Scan hexadecimal escape sequence
    fn scan_hex_escape(&mut self, digits: usize) -> Result<u32> {
        let mut hex_string = String::new();
        
        for _ in 0..digits {
            if self.is_at_end() || !self.current_char().is_ascii_hexdigit() {
                return Err(Error::lexer(
                    format!("Invalid hex escape sequence: expected {} hex digits", digits),
                    self.make_span(self.current, self.current + 1),
                ));
            }
            hex_string.push(self.advance());
        }
        
        u32::from_str_radix(&hex_string, 16).map_err(|_| Error::lexer(
            "Invalid hex escape sequence".to_string(),
            self.make_span(self.current - digits, self.current),
        ))
    }
    
    /// Skip line comment (// ...)
    fn skip_line_comment(&mut self) {
        while !self.is_at_end() && self.current_char() != '\n' {
            self.advance();
        }
    }
    
    /// Skip block comment (/* ... */)
    fn skip_block_comment(&mut self) -> Result<()> {
        self.advance(); // consume '/'
        self.advance(); // consume '*'
        
        while !self.is_at_end() {
            if self.current_char() == '*' && self.peek() == Some('/') {
                self.advance(); // consume '*'
                self.advance(); // consume '/'
                return Ok(());
            }
            
            if self.current_char() == '\n' {
                self.line += 1;
                self.column = 1;
            } else {
                self.column += 1;
            }
            self.advance();
        }
        
        Err(Error::lexer(
            "Unterminated block comment".to_string(),
            self.make_span(self.current, self.current),
        ))
    }
    
    /// Get the current character
    fn current_char(&self) -> char {
        if self.is_at_end() {
            '\0'
        } else {
            self.source.chars().nth(self.current).unwrap_or('\0')
        }
    }
    
    /// Peek at the next character
    fn peek(&self) -> Option<char> {
        if self.current + 1 >= self.source.len() {
            None
        } else {
            self.source.chars().nth(self.current + 1)
        }
    }
    
    /// Advance to the next character
    fn advance(&mut self) -> char {
        let c = self.current_char();
        self.current += c.len_utf8();
        self.column += 1;
        c
    }
    
    /// Check if we're at the end of the source
    fn is_at_end(&self) -> bool {
        self.current >= self.source.len()
    }
    
    /// Create a span from start to end positions
    fn make_span(&self, start: usize, end: usize) -> Span {
        Span::new(start, end, self.start_line, self.start_column)
    }
    
    /// Get the start position of the current token
    fn start_position(&self) -> usize {
        // This is a simplification - in a real implementation we'd track
        // the byte position more carefully
        let mut pos = 0;
        let mut line = 1;
        let mut col = 1;
        
        for c in self.source.chars() {
            if line == self.start_line && col == self.start_column {
                return pos;
            }
            
            if c == '\n' {
                line += 1;
                col = 1;
            } else {
                col += 1;
            }
            pos += c.len_utf8();
        }
        
        pos
    }
    
    /// Scan + or ++ or +=
    fn scan_plus_operator(&mut self) -> Result<TokenKind> {
        if self.current_char() == '+' {
            self.advance();
            Ok(TokenKind::PlusPlus)
        } else if self.current_char() == '=' {
            self.advance();
            Ok(TokenKind::PlusEqual)
        } else {
            Ok(TokenKind::Plus)
        }
    }
    
    /// Scan - or -- or -=
    fn scan_minus_operator(&mut self) -> Result<TokenKind> {
        if self.current_char() == '-' {
            self.advance();
            Ok(TokenKind::MinusMinus)
        } else if self.current_char() == '=' {
            self.advance();
            Ok(TokenKind::MinusEqual)
        } else {
            Ok(TokenKind::Minus)
        }
    }
    
    /// Scan * or ** or *=
    fn scan_star_operator(&mut self) -> Result<TokenKind> {
        if self.current_char() == '*' {
            self.advance();
            Ok(TokenKind::StarStar)
        } else if self.current_char() == '=' {
            self.advance();
            Ok(TokenKind::StarEqual)
        } else {
            Ok(TokenKind::Star)
        }
    }
    
    /// Scan % or %=
    fn scan_percent_operator(&mut self) -> Result<TokenKind> {
        if self.current_char() == '=' {
            self.advance();
            Ok(TokenKind::PercentEqual)
        } else {
            Ok(TokenKind::Percent)
        }
    }
    
    /// Scan = or == or === or =>
    fn scan_equal_operator(&mut self) -> Result<TokenKind> {
        if self.current_char() == '=' {
            self.advance();
            if self.current_char() == '=' {
                self.advance();
                Ok(TokenKind::EqualEqualEqual)
            } else {
                Ok(TokenKind::EqualEqual)
            }
        } else if self.current_char() == '>' {
            self.advance();
            Ok(TokenKind::Arrow)
        } else {
            Ok(TokenKind::Equal)
        }
    }
    
    /// Scan ! or != or !==
    fn scan_bang_operator(&mut self) -> Result<TokenKind> {
        if self.current_char() == '=' {
            self.advance();
            if self.current_char() == '=' {
                self.advance();
                Ok(TokenKind::BangEqualEqual)
            } else {
                Ok(TokenKind::BangEqual)
            }
        } else {
            Ok(TokenKind::Bang)
        }
    }
    
    /// Scan < or <= or <<
    fn scan_less_operator(&mut self) -> Result<TokenKind> {
        if self.current_char() == '=' {
            self.advance();
            Ok(TokenKind::LessEqual)
        } else if self.current_char() == '<' {
            self.advance();
            Ok(TokenKind::LessLess)
        } else {
            Ok(TokenKind::Less)
        }
    }
    
    /// Scan > or >= or >> or >>>
    fn scan_greater_operator(&mut self) -> Result<TokenKind> {
        if self.current_char() == '=' {
            self.advance();
            Ok(TokenKind::GreaterEqual)
        } else if self.current_char() == '>' {
            self.advance();
            if self.current_char() == '>' {
                self.advance();
                Ok(TokenKind::GreaterGreaterGreater)
            } else {
                Ok(TokenKind::GreaterGreater)
            }
        } else {
            Ok(TokenKind::Greater)
        }
    }
    
    /// Scan & or &&
    fn scan_amp_operator(&mut self) -> Result<TokenKind> {
        if self.current_char() == '&' {
            self.advance();
            Ok(TokenKind::AmpAmp)
        } else {
            Ok(TokenKind::Amp)
        }
    }
    
    /// Scan | or ||
    fn scan_pipe_operator(&mut self) -> Result<TokenKind> {
        if self.current_char() == '|' {
            self.advance();
            Ok(TokenKind::PipePipe)
        } else {
            Ok(TokenKind::Pipe)
        }
    }
}