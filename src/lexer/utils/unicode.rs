//! Unicode utilities for JavaScript lexer
//!
//! Provides Unicode identifier validation, normalization, and category
//! classification according to the ECMAScript specification.

use unicode_xid::UnicodeXID;

/// Unicode identifier validation and normalization utilities
pub struct UnicodeHelper;

impl UnicodeHelper {
    /// Check if a character can start an identifier according to ECMAScript spec
    /// 
    /// From ECMAScript 2022 spec:
    /// IdentifierStart :: Letter | $ | _ | \ UnicodeEscapeSequence
    pub fn is_identifier_start(ch: char) -> bool {
        match ch {
            // ASCII fast path
            'a'..='z' | 'A'..='Z' | '_' | '$' => true,
            // Unicode ID_Start
            _ => UnicodeXID::is_xid_start(ch),
        }
    }
    
    /// Check if a character can continue an identifier according to ECMAScript spec
    /// 
    /// From ECMAScript 2022 spec:
    /// IdentifierPart :: IdentifierStart | Digit | ConnectingPunctuation | CombiningMark | ZeroWidthNonJoiner | ZeroWidthJoiner
    pub fn is_identifier_continue(ch: char) -> bool {
        match ch {
            // ASCII fast path
            'a'..='z' | 'A'..='Z' | '0'..='9' | '_' | '$' => true,
            // Zero-width non-joiner and joiner
            '\u{200C}' | '\u{200D}' => true,
            // Unicode ID_Continue
            _ => UnicodeXID::is_xid_continue(ch),
        }
    }
    
    /// Check if a character is a Unicode line terminator
    /// 
    /// LineTerminator ::
    ///   <LF> | <CR> | <LS> | <PS>
    pub fn is_line_terminator(ch: char) -> bool {
        matches!(ch, '\u{000A}' | '\u{000D}' | '\u{2028}' | '\u{2029}')
    }
    
    /// Check if a character is ECMAScript whitespace
    /// 
    /// WhiteSpace ::
    ///   <TAB> | <VT> | <FF> | <SP> | <NBSP> | <ZWNBSP> | <USP>
    pub fn is_whitespace(ch: char) -> bool {
        match ch {
            // ASCII whitespace
            '\u{0009}' |  // TAB
            '\u{000B}' |  // VT (Vertical Tab)
            '\u{000C}' |  // FF (Form Feed) 
            '\u{0020}' |  // SP (Space)
            '\u{00A0}' |  // NBSP (Non-breaking space)
            '\u{FEFF}'    // ZWNBSP (Zero Width No-Break Space, BOM)
                => true,
            // Unicode category Zs (Space Separator)
            _ => ch.is_whitespace() && !Self::is_line_terminator(ch),
        }
    }
    
    /// Normalize Unicode identifier to canonical form
    /// 
    /// This implements Unicode normalization form KC (NFKC)
    /// as recommended by the ECMAScript specification.
    pub fn normalize_identifier(identifier: &str) -> String {
        // For now, we'll use a simple approach
        // In a full implementation, we'd use the unicode-normalization crate
        identifier.to_string()
    }
    
    /// Check if an identifier is normalized
    pub fn is_normalized(identifier: &str) -> bool {
        // Simplified check - in practice would compare with normalized form
        identifier.chars().all(|ch| !ch.is_control() || matches!(ch, '\u{200C}' | '\u{200D}'))
    }
    
    /// Get Unicode category information for a character
    pub fn get_unicode_category(ch: char) -> UnicodeCategory {
        match ch {
            // ASCII fast paths
            'a'..='z' => UnicodeCategory::LowercaseLetter,
            'A'..='Z' => UnicodeCategory::UppercaseLetter,
            '0'..='9' => UnicodeCategory::DecimalNumber,
            ' ' => UnicodeCategory::SpaceSeparator,
            '_' => UnicodeCategory::ConnectorPunctuation,
            '$' => UnicodeCategory::CurrencySymbol,
            
            // Unicode categories
            _ => {
                // This is a simplified mapping - a full implementation would
                // use the unicode-categories crate or similar
                if ch.is_alphabetic() {
                    if ch.is_lowercase() {
                        UnicodeCategory::LowercaseLetter
                    } else if ch.is_uppercase() {
                        UnicodeCategory::UppercaseLetter
                    } else {
                        UnicodeCategory::OtherLetter
                    }
                } else if ch.is_numeric() {
                    UnicodeCategory::DecimalNumber
                } else if ch.is_whitespace() {
                    UnicodeCategory::SpaceSeparator
                } else if ch.is_control() {
                    UnicodeCategory::Control
                } else {
                    UnicodeCategory::Other
                }
            }
        }
    }
    
    /// Validate that a string contains only valid identifier characters
    pub fn validate_identifier(identifier: &str) -> Result<(), UnicodeError> {
        if identifier.is_empty() {
            return Err(UnicodeError::EmptyIdentifier);
        }
        
        let mut chars = identifier.chars();
        let first_char = chars.next().unwrap();
        
        if !Self::is_identifier_start(first_char) {
            return Err(UnicodeError::InvalidIdentifierStart(first_char));
        }
        
        for (pos, ch) in chars.enumerate() {
            if !Self::is_identifier_continue(ch) {
                return Err(UnicodeError::InvalidIdentifierCharacter {
                    character: ch,
                    position: pos + 1,
                });
            }
        }
        
        Ok(())
    }
    
    /// Check if a character needs to be escaped in string literals
    pub fn needs_escaping(ch: char) -> bool {
        match ch {
            // Control characters
            '\0'..='\u{001F}' => true,
            // DEL
            '\u{007F}' => true,
            // Line terminators
            '\u{000A}' | '\u{000D}' | '\u{2028}' | '\u{2029}' => true,
            // String delimiters
            '"' | '\'' | '`' => true,
            // Escape character
            '\\' => true,
            _ => false,
        }
    }
    
    /// Get the escaped representation of a character
    pub fn escape_character(ch: char) -> String {
        match ch {
            '\0' => "\\0".to_string(),
            '\t' => "\\t".to_string(),
            '\n' => "\\n".to_string(),
            '\r' => "\\r".to_string(),
            '\\' => "\\\\".to_string(),
            '"' => "\\\"".to_string(),
            '\'' => "\\'".to_string(),
            '\u{000B}' => "\\v".to_string(),
            '\u{000C}' => "\\f".to_string(),
            '\u{0008}' => "\\b".to_string(),
            _ if ch.is_control() || ch as u32 > 0x10FFFF => {
                if (ch as u32) <= 0xFF {
                    format!("\\x{:02X}", ch as u32)
                } else if (ch as u32) <= 0xFFFF {
                    format!("\\u{:04X}", ch as u32)
                } else {
                    format!("\\u{{{:X}}}", ch as u32)
                }
            }
            _ => ch.to_string(),
        }
    }
    
    /// Check if a character is a valid JSON string character
    pub fn is_valid_json_char(ch: char) -> bool {
        match ch {
            // Unescaped characters in JSON strings
            '\u{0020}'..='\u{0021}' |  // Space to !
            '\u{0023}'..='\u{005B}' |  // # to [
            '\u{005D}'..='\u{10FFFF}' => true, // ] to end of Unicode
            _ => false,
        }
    }
    
    /// Convert a UTF-8 byte sequence to a character safely
    pub fn utf8_to_char(bytes: &[u8]) -> Result<char, UnicodeError> {
        let s = std::str::from_utf8(bytes)
            .map_err(|_| UnicodeError::InvalidUtf8)?;
        
        let mut chars = s.chars();
        let ch = chars.next().ok_or(UnicodeError::EmptySequence)?;
        
        if chars.next().is_some() {
            return Err(UnicodeError::MultipleCharacters);
        }
        
        Ok(ch)
    }
}

/// Unicode character categories (simplified)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnicodeCategory {
    /// Uppercase letter (Lu)
    UppercaseLetter,
    /// Lowercase letter (Ll)
    LowercaseLetter,
    /// Titlecase letter (Lt)
    TitlecaseLetter,
    /// Modifier letter (Lm)
    ModifierLetter,
    /// Other letter (Lo)
    OtherLetter,
    /// Non-spacing mark (Mn)
    NonSpacingMark,
    /// Spacing combining mark (Mc)
    SpacingMark,
    /// Enclosing mark (Me)
    EnclosingMark,
    /// Decimal number (Nd)
    DecimalNumber,
    /// Letter number (Nl)
    LetterNumber,
    /// Other number (No)
    OtherNumber,
    /// Connector punctuation (Pc)
    ConnectorPunctuation,
    /// Dash punctuation (Pd)
    DashPunctuation,
    /// Open punctuation (Ps)
    OpenPunctuation,
    /// Close punctuation (Pe)
    ClosePunctuation,
    /// Initial punctuation (Pi)
    InitialPunctuation,
    /// Final punctuation (Pf)
    FinalPunctuation,
    /// Other punctuation (Po)
    OtherPunctuation,
    /// Math symbol (Sm)
    MathSymbol,
    /// Currency symbol (Sc)
    CurrencySymbol,
    /// Modifier symbol (Sk)
    ModifierSymbol,
    /// Other symbol (So)
    OtherSymbol,
    /// Space separator (Zs)
    SpaceSeparator,
    /// Line separator (Zl)
    LineSeparator,
    /// Paragraph separator (Zp)
    ParagraphSeparator,
    /// Control (Cc)
    Control,
    /// Format (Cf)
    Format,
    /// Surrogate (Cs)
    Surrogate,
    /// Private use (Co)
    PrivateUse,
    /// Unassigned (Cn)
    Unassigned,
    /// Other/Unknown
    Other,
}

/// Unicode processing errors
#[derive(Debug, Clone, PartialEq)]
pub enum UnicodeError {
    /// Empty identifier
    EmptyIdentifier,
    /// Invalid character at start of identifier
    InvalidIdentifierStart(char),
    /// Invalid character in identifier
    InvalidIdentifierCharacter { character: char, position: usize },
    /// Invalid UTF-8 sequence
    InvalidUtf8,
    /// Empty byte sequence
    EmptySequence,
    /// Multiple characters in sequence
    MultipleCharacters,
    /// Invalid Unicode code point
    InvalidCodePoint(u32),
    /// Normalization error
    NormalizationError(String),
}

impl std::fmt::Display for UnicodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UnicodeError::EmptyIdentifier => write!(f, "Empty identifier"),
            UnicodeError::InvalidIdentifierStart(ch) => {
                write!(f, "Invalid identifier start character: '{}'", ch)
            }
            UnicodeError::InvalidIdentifierCharacter { character, position } => {
                write!(f, "Invalid identifier character '{}' at position {}", character, position)
            }
            UnicodeError::InvalidUtf8 => write!(f, "Invalid UTF-8 sequence"),
            UnicodeError::EmptySequence => write!(f, "Empty byte sequence"),
            UnicodeError::MultipleCharacters => write!(f, "Multiple characters in sequence"),
            UnicodeError::InvalidCodePoint(cp) => write!(f, "Invalid Unicode code point: U+{:X}", cp),
            UnicodeError::NormalizationError(msg) => write!(f, "Normalization error: {}", msg),
        }
    }
}

impl std::error::Error for UnicodeError {}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_identifier_start() {
        // ASCII letters
        assert!(UnicodeHelper::is_identifier_start('a'));
        assert!(UnicodeHelper::is_identifier_start('Z'));
        assert!(UnicodeHelper::is_identifier_start('_'));
        assert!(UnicodeHelper::is_identifier_start('$'));
        
        // Not identifier start
        assert!(!UnicodeHelper::is_identifier_start('0'));
        assert!(!UnicodeHelper::is_identifier_start(' '));
        assert!(!UnicodeHelper::is_identifier_start('!'));
        
        // Unicode letters
        assert!(UnicodeHelper::is_identifier_start('α')); // Greek alpha
        assert!(UnicodeHelper::is_identifier_start("café".chars().nth(1).unwrap())); // é
    }
    
    #[test]
    fn test_identifier_continue() {
        // ASCII letters and digits
        assert!(UnicodeHelper::is_identifier_continue('a'));
        assert!(UnicodeHelper::is_identifier_continue('Z'));
        assert!(UnicodeHelper::is_identifier_continue('0'));
        assert!(UnicodeHelper::is_identifier_continue('9'));
        assert!(UnicodeHelper::is_identifier_continue('_'));
        assert!(UnicodeHelper::is_identifier_continue('$'));
        
        // Zero-width characters
        assert!(UnicodeHelper::is_identifier_continue('\u{200C}')); // ZWNJ
        assert!(UnicodeHelper::is_identifier_continue('\u{200D}')); // ZWJ
        
        // Not identifier continue
        assert!(!UnicodeHelper::is_identifier_continue(' '));
        assert!(!UnicodeHelper::is_identifier_continue('!'));
        assert!(!UnicodeHelper::is_identifier_continue('@'));
    }
    
    #[test]
    fn test_line_terminators() {
        assert!(UnicodeHelper::is_line_terminator('\n'));    // LF
        assert!(UnicodeHelper::is_line_terminator('\r'));    // CR
        assert!(UnicodeHelper::is_line_terminator('\u{2028}')); // Line Separator
        assert!(UnicodeHelper::is_line_terminator('\u{2029}')); // Paragraph Separator
        
        assert!(!UnicodeHelper::is_line_terminator(' '));
        assert!(!UnicodeHelper::is_line_terminator('\t'));
        assert!(!UnicodeHelper::is_line_terminator('a'));
    }
    
    #[test]
    fn test_whitespace() {
        assert!(UnicodeHelper::is_whitespace(' '));      // Space
        assert!(UnicodeHelper::is_whitespace('\t'));     // Tab
        assert!(UnicodeHelper::is_whitespace('\u{000B}')); // VT
        assert!(UnicodeHelper::is_whitespace('\u{000C}')); // FF
        assert!(UnicodeHelper::is_whitespace('\u{00A0}')); // NBSP
        assert!(UnicodeHelper::is_whitespace('\u{FEFF}')); // BOM
        
        // Line terminators are not whitespace in our classification
        assert!(!UnicodeHelper::is_whitespace('\n'));
        assert!(!UnicodeHelper::is_whitespace('\r'));
        
        assert!(!UnicodeHelper::is_whitespace('a'));
        assert!(!UnicodeHelper::is_whitespace('0'));
    }
    
    #[test]
    fn test_identifier_validation() {
        // Valid identifiers
        assert!(UnicodeHelper::validate_identifier("hello").is_ok());
        assert!(UnicodeHelper::validate_identifier("_private").is_ok());
        assert!(UnicodeHelper::validate_identifier("$jquery").is_ok());
        assert!(UnicodeHelper::validate_identifier("test123").is_ok());
        assert!(UnicodeHelper::validate_identifier("café").is_ok());
        
        // Invalid identifiers
        assert!(UnicodeHelper::validate_identifier("").is_err());
        assert!(UnicodeHelper::validate_identifier("123abc").is_err());
        assert!(UnicodeHelper::validate_identifier("hello world").is_err());
        assert!(UnicodeHelper::validate_identifier("hello!").is_err());
    }
    
    #[test]
    fn test_character_escaping() {
        assert!(UnicodeHelper::needs_escaping('\0'));
        assert!(UnicodeHelper::needs_escaping('\n'));
        assert!(UnicodeHelper::needs_escaping('\t'));
        assert!(UnicodeHelper::needs_escaping('"'));
        assert!(UnicodeHelper::needs_escaping('\''));
        assert!(UnicodeHelper::needs_escaping('\\'));
        
        assert!(!UnicodeHelper::needs_escaping('a'));
        assert!(!UnicodeHelper::needs_escaping(' '));
        assert!(!UnicodeHelper::needs_escaping('0'));
    }
    
    #[test]
    fn test_escape_character() {
        assert_eq!(UnicodeHelper::escape_character('\0'), "\\0");
        assert_eq!(UnicodeHelper::escape_character('\n'), "\\n");
        assert_eq!(UnicodeHelper::escape_character('\t'), "\\t");
        assert_eq!(UnicodeHelper::escape_character('\r'), "\\r");
        assert_eq!(UnicodeHelper::escape_character('\\'), "\\\\");
        assert_eq!(UnicodeHelper::escape_character('"'), "\\\"");
        assert_eq!(UnicodeHelper::escape_character('\''), "\\'");
        
        assert_eq!(UnicodeHelper::escape_character('a'), "a");
        assert_eq!(UnicodeHelper::escape_character(' '), " ");
    }
    
    #[test]
    fn test_unicode_categories() {
        assert_eq!(UnicodeHelper::get_unicode_category('a'), UnicodeCategory::LowercaseLetter);
        assert_eq!(UnicodeHelper::get_unicode_category('A'), UnicodeCategory::UppercaseLetter);
        assert_eq!(UnicodeHelper::get_unicode_category('0'), UnicodeCategory::DecimalNumber);
        assert_eq!(UnicodeHelper::get_unicode_category(' '), UnicodeCategory::SpaceSeparator);
        assert_eq!(UnicodeHelper::get_unicode_category('_'), UnicodeCategory::ConnectorPunctuation);
        assert_eq!(UnicodeHelper::get_unicode_category('$'), UnicodeCategory::CurrencySymbol);
    }
    
    #[test]
    fn test_utf8_conversion() {
        let bytes = "a".as_bytes();
        assert_eq!(UnicodeHelper::utf8_to_char(bytes).unwrap(), 'a');
        
        let bytes = "🚀".as_bytes();
        assert_eq!(UnicodeHelper::utf8_to_char(bytes).unwrap(), '🚀');
        
        // Invalid UTF-8
        let invalid_bytes = &[0xFF, 0xFE];
        assert!(UnicodeHelper::utf8_to_char(invalid_bytes).is_err());
        
        // Empty sequence
        assert!(UnicodeHelper::utf8_to_char(&[]).is_err());
    }
    
    #[test]
    fn test_json_characters() {
        assert!(UnicodeHelper::is_valid_json_char('a'));
        assert!(UnicodeHelper::is_valid_json_char(' '));
        assert!(UnicodeHelper::is_valid_json_char('!'));
        assert!(UnicodeHelper::is_valid_json_char('#'));
        
        assert!(!UnicodeHelper::is_valid_json_char('"'));
        assert!(!UnicodeHelper::is_valid_json_char('\\'));
        assert!(!UnicodeHelper::is_valid_json_char('\n'));
        assert!(!UnicodeHelper::is_valid_json_char('\0'));
    }
}