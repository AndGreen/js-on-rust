//! Operator and punctuation scanner
//!
//! Handles scanning of JavaScript operators and punctuation including:
//! - Arithmetic operators (+, -, *, /, %, **)
//! - Assignment operators (=, +=, -=, *=, /=, %=)
//! - Comparison operators (==, ===, !=, !==, <, >, <=, >=)
//! - Logical operators (&&, ||, !)
//! - Bitwise operators (&, |, ^, ~, <<, >>, >>>)
//! - Increment/decrement (++, --)
//! - Punctuation ({}, [], (), ;, ,, ., :, ?, =>)

use super::{Scanner, LookaheadScanner, lexer_error};
use crate::error::Result;
use crate::lexer::core::{Input, LexerContext};
use std::collections::HashMap;

/// Operator or punctuation token
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum OperatorToken {
    // Arithmetic operators
    Plus,               // +
    Minus,              // -
    Star,               // *
    Slash,              // /
    Percent,            // %
    StarStar,           // **
    
    // Assignment operators
    Equal,              // =
    PlusEqual,          // +=
    MinusEqual,         // -=
    StarEqual,          // *=
    SlashEqual,         // /=
    PercentEqual,       // %=
    StarStarEqual,      // **=
    
    // Comparison operators
    EqualEqual,         // ==
    EqualEqualEqual,    // ===
    BangEqual,          // !=
    BangEqualEqual,     // !==
    Less,               // <
    Greater,            // >
    LessEqual,          // <=
    GreaterEqual,       // >=
    
    // Logical operators
    AmpAmp,             // &&
    PipePipe,           // ||
    Bang,               // !
    
    // Bitwise operators
    Amp,                // &
    Pipe,               // |
    Caret,              // ^
    Tilde,              // ~
    LessLess,           // <<
    GreaterGreater,     // >>
    GreaterGreaterGreater, // >>>
    
    // Bitwise assignment operators
    AmpEqual,           // &=
    PipeEqual,          // |=
    CaretEqual,         // ^=
    LessLessEqual,      // <<=
    GreaterGreaterEqual, // >>=
    GreaterGreaterGreaterEqual, // >>>=
    
    // Increment/decrement
    PlusPlus,           // ++
    MinusMinus,         // --
    
    // Punctuation
    LeftParen,          // (
    RightParen,         // )
    LeftBrace,          // {
    RightBrace,         // }
    LeftBracket,        // [
    RightBracket,       // ]
    Semicolon,          // ;
    Comma,              // ,
    Dot,                // .
    Colon,              // :
    Question,           // ?
    Arrow,              // =>
    
    // ES2020+ operators
    QuestionQuestion,   // ??
    QuestionQuestionEqual, // ??=
    QuestionDot,        // ?.
    
    // ES2021+ operators
    AmpAmpEqual,        // &&=
    PipePipeEqual,      // ||=
}

/// Operator precedence information
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Precedence {
    Lowest = 0,
    Assignment = 1,
    Conditional = 2,
    LogicalOr = 3,
    LogicalAnd = 4,
    BitwiseOr = 5,
    BitwiseXor = 6,
    BitwiseAnd = 7,
    Equality = 8,
    Relational = 9,
    Shift = 10,
    Additive = 11,
    Multiplicative = 12,
    Exponentiation = 13,
    Unary = 14,
    Postfix = 15,
    Primary = 16,
}

/// Operator information for parsing
#[derive(Debug, Clone)]
pub struct OperatorInfo {
    pub token: OperatorToken,
    pub precedence: Precedence,
    pub right_associative: bool,
    pub unary: bool,
    pub binary: bool,
}

/// Operator scanner using table-driven approach
#[derive(Debug)]
pub struct OperatorScanner {
    /// Mapping from operator strings to tokens (sorted by length, longest first)
    operator_table: Vec<(String, OperatorToken)>,
    /// Operator information lookup
    operator_info: HashMap<OperatorToken, OperatorInfo>,
}

impl Default for OperatorScanner {
    fn default() -> Self {
        Self::new()
    }
}

impl OperatorScanner {
    /// Create a new operator scanner
    pub fn new() -> Self {
        let mut scanner = Self {
            operator_table: Vec::new(),
            operator_info: HashMap::new(),
        };
        
        scanner.initialize_operators();
        scanner
    }
    
    /// Initialize the operator table and precedence information
    fn initialize_operators(&mut self) {
        use OperatorToken::*;
        use Precedence::*;
        
        // Define operators with their string representations
        let operators = [
            // 3-character operators (must come first for proper matching)
            ("===", EqualEqualEqual),
            ("!==", BangEqualEqual),
            (">>>", GreaterGreaterGreater),
            ("**=", StarStarEqual),
            ("&&&", AmpAmpEqual),
            ("||=", PipePipeEqual),
            ("??=", QuestionQuestionEqual),
            ("<<=", LessLessEqual),
            (">>=", GreaterGreaterEqual),
            (">>>=", GreaterGreaterGreaterEqual), // 4 chars, but handle it
            
            // 2-character operators
            ("==", EqualEqual),
            ("!=", BangEqual),
            ("<=", LessEqual),
            (">=", GreaterEqual),
            ("&&", AmpAmp),
            ("||", PipePipe),
            ("<<", LessLess),
            (">>", GreaterGreater),
            ("++", PlusPlus),
            ("--", MinusMinus),
            ("+=", PlusEqual),
            ("-=", MinusEqual),
            ("*=", StarEqual),
            ("/=", SlashEqual),
            ("%=", PercentEqual),
            ("&=", AmpEqual),
            ("|=", PipeEqual),
            ("^=", CaretEqual),
            ("**", StarStar),
            ("=>", Arrow),
            ("??", QuestionQuestion),
            ("?.", QuestionDot),
            
            // 1-character operators
            ("+", Plus),
            ("-", Minus),
            ("*", Star),
            ("/", Slash),
            ("%", Percent),
            ("=", Equal),
            ("<", Less),
            (">", Greater),
            ("!", Bang),
            ("&", Amp),
            ("|", Pipe),
            ("^", Caret),
            ("~", Tilde),
            ("(", LeftParen),
            (")", RightParen),
            ("{", LeftBrace),
            ("}", RightBrace),
            ("[", LeftBracket),
            ("]", RightBracket),
            (";", Semicolon),
            (",", Comma),
            (".", Dot),
            (":", Colon),
            ("?", Question),
        ];
        
        // Sort by length (longest first) for proper matching
        self.operator_table = operators
            .iter()
            .map(|(s, t)| (s.to_string(), t.clone()))
            .collect();
        self.operator_table.sort_by(|a, b| b.0.len().cmp(&a.0.len()));
        
        // Initialize operator information
        self.initialize_operator_info();
    }
    
    /// Initialize operator precedence and associativity information
    fn initialize_operator_info(&mut self) {
        use OperatorToken::*;
        use Precedence::*;
        
        let operators_info = [
            // Assignment operators (right associative)
            (Equal, OperatorInfo { token: Equal, precedence: Assignment, right_associative: true, unary: false, binary: true }),
            (PlusEqual, OperatorInfo { token: PlusEqual, precedence: Assignment, right_associative: true, unary: false, binary: true }),
            (MinusEqual, OperatorInfo { token: MinusEqual, precedence: Assignment, right_associative: true, unary: false, binary: true }),
            (StarEqual, OperatorInfo { token: StarEqual, precedence: Assignment, right_associative: true, unary: false, binary: true }),
            (SlashEqual, OperatorInfo { token: SlashEqual, precedence: Assignment, right_associative: true, unary: false, binary: true }),
            (PercentEqual, OperatorInfo { token: PercentEqual, precedence: Assignment, right_associative: true, unary: false, binary: true }),
            (StarStarEqual, OperatorInfo { token: StarStarEqual, precedence: Assignment, right_associative: true, unary: false, binary: true }),
            (AmpEqual, OperatorInfo { token: AmpEqual, precedence: Assignment, right_associative: true, unary: false, binary: true }),
            (PipeEqual, OperatorInfo { token: PipeEqual, precedence: Assignment, right_associative: true, unary: false, binary: true }),
            (CaretEqual, OperatorInfo { token: CaretEqual, precedence: Assignment, right_associative: true, unary: false, binary: true }),
            (LessLessEqual, OperatorInfo { token: LessLessEqual, precedence: Assignment, right_associative: true, unary: false, binary: true }),
            (GreaterGreaterEqual, OperatorInfo { token: GreaterGreaterEqual, precedence: Assignment, right_associative: true, unary: false, binary: true }),
            (GreaterGreaterGreaterEqual, OperatorInfo { token: GreaterGreaterGreaterEqual, precedence: Assignment, right_associative: true, unary: false, binary: true }),
            (QuestionQuestionEqual, OperatorInfo { token: QuestionQuestionEqual, precedence: Assignment, right_associative: true, unary: false, binary: true }),
            (AmpAmpEqual, OperatorInfo { token: AmpAmpEqual, precedence: Assignment, right_associative: true, unary: false, binary: true }),
            (PipePipeEqual, OperatorInfo { token: PipePipeEqual, precedence: Assignment, right_associative: true, unary: false, binary: true }),
            
            // Conditional operator
            (Question, OperatorInfo { token: Question, precedence: Conditional, right_associative: true, unary: false, binary: false }),
            
            // Logical operators
            (PipePipe, OperatorInfo { token: PipePipe, precedence: LogicalOr, right_associative: false, unary: false, binary: true }),
            (AmpAmp, OperatorInfo { token: AmpAmp, precedence: LogicalAnd, right_associative: false, unary: false, binary: true }),
            (QuestionQuestion, OperatorInfo { token: QuestionQuestion, precedence: LogicalOr, right_associative: false, unary: false, binary: true }),
            
            // Bitwise operators
            (Pipe, OperatorInfo { token: Pipe, precedence: BitwiseOr, right_associative: false, unary: false, binary: true }),
            (Caret, OperatorInfo { token: Caret, precedence: BitwiseXor, right_associative: false, unary: false, binary: true }),
            (Amp, OperatorInfo { token: Amp, precedence: BitwiseAnd, right_associative: false, unary: false, binary: true }),
            
            // Equality operators
            (EqualEqual, OperatorInfo { token: EqualEqual, precedence: Equality, right_associative: false, unary: false, binary: true }),
            (EqualEqualEqual, OperatorInfo { token: EqualEqualEqual, precedence: Equality, right_associative: false, unary: false, binary: true }),
            (BangEqual, OperatorInfo { token: BangEqual, precedence: Equality, right_associative: false, unary: false, binary: true }),
            (BangEqualEqual, OperatorInfo { token: BangEqualEqual, precedence: Equality, right_associative: false, unary: false, binary: true }),
            
            // Relational operators
            (Less, OperatorInfo { token: Less, precedence: Relational, right_associative: false, unary: false, binary: true }),
            (Greater, OperatorInfo { token: Greater, precedence: Relational, right_associative: false, unary: false, binary: true }),
            (LessEqual, OperatorInfo { token: LessEqual, precedence: Relational, right_associative: false, unary: false, binary: true }),
            (GreaterEqual, OperatorInfo { token: GreaterEqual, precedence: Relational, right_associative: false, unary: false, binary: true }),
            
            // Shift operators
            (LessLess, OperatorInfo { token: LessLess, precedence: Shift, right_associative: false, unary: false, binary: true }),
            (GreaterGreater, OperatorInfo { token: GreaterGreater, precedence: Shift, right_associative: false, unary: false, binary: true }),
            (GreaterGreaterGreater, OperatorInfo { token: GreaterGreaterGreater, precedence: Shift, right_associative: false, unary: false, binary: true }),
            
            // Additive operators
            (Plus, OperatorInfo { token: Plus, precedence: Additive, right_associative: false, unary: true, binary: true }),
            (Minus, OperatorInfo { token: Minus, precedence: Additive, right_associative: false, unary: true, binary: true }),
            
            // Multiplicative operators
            (Star, OperatorInfo { token: Star, precedence: Multiplicative, right_associative: false, unary: false, binary: true }),
            (Slash, OperatorInfo { token: Slash, precedence: Multiplicative, right_associative: false, unary: false, binary: true }),
            (Percent, OperatorInfo { token: Percent, precedence: Multiplicative, right_associative: false, unary: false, binary: true }),
            
            // Exponentiation (right associative)
            (StarStar, OperatorInfo { token: StarStar, precedence: Exponentiation, right_associative: true, unary: false, binary: true }),
            
            // Unary operators
            (Bang, OperatorInfo { token: Bang, precedence: Unary, right_associative: true, unary: true, binary: false }),
            (Tilde, OperatorInfo { token: Tilde, precedence: Unary, right_associative: true, unary: true, binary: false }),
            
            // Postfix operators
            (PlusPlus, OperatorInfo { token: PlusPlus, precedence: Postfix, right_associative: false, unary: true, binary: false }),
            (MinusMinus, OperatorInfo { token: MinusMinus, precedence: Postfix, right_associative: false, unary: true, binary: false }),
            
            // Punctuation (no precedence)
            (LeftParen, OperatorInfo { token: LeftParen, precedence: Primary, right_associative: false, unary: false, binary: false }),
            (RightParen, OperatorInfo { token: RightParen, precedence: Primary, right_associative: false, unary: false, binary: false }),
            (LeftBrace, OperatorInfo { token: LeftBrace, precedence: Primary, right_associative: false, unary: false, binary: false }),
            (RightBrace, OperatorInfo { token: RightBrace, precedence: Primary, right_associative: false, unary: false, binary: false }),
            (LeftBracket, OperatorInfo { token: LeftBracket, precedence: Primary, right_associative: false, unary: false, binary: false }),
            (RightBracket, OperatorInfo { token: RightBracket, precedence: Primary, right_associative: false, unary: false, binary: false }),
            (Semicolon, OperatorInfo { token: Semicolon, precedence: Lowest, right_associative: false, unary: false, binary: false }),
            (Comma, OperatorInfo { token: Comma, precedence: Lowest, right_associative: false, unary: false, binary: false }),
            (Dot, OperatorInfo { token: Dot, precedence: Primary, right_associative: false, unary: false, binary: false }),
            (Colon, OperatorInfo { token: Colon, precedence: Lowest, right_associative: false, unary: false, binary: false }),
            (Arrow, OperatorInfo { token: Arrow, precedence: Lowest, right_associative: false, unary: false, binary: false }),
            (QuestionDot, OperatorInfo { token: QuestionDot, precedence: Primary, right_associative: false, unary: false, binary: false }),
        ];
        
        for (token, info) in operators_info {
            self.operator_info.insert(token, info);
        }
    }
    
    /// Scan an operator or punctuation token
    fn scan_operator(&self, input: &mut Input, context: &LexerContext) -> Result<OperatorToken> {
        input.mark_token_start();
        
        // Try to match operators from longest to shortest
        for (op_str, token) in &self.operator_table {
            if self.matches_operator_at_position(input, op_str) {
                // Check if this operator is available in the current context
                if self.is_operator_available(token, context) {
                    // Advance past the operator
                    for _ in 0..op_str.len() {
                        input.advance();
                    }
                    return Ok(token.clone());
                }
            }
        }
        
        Err(lexer_error(
            format!("Unexpected character: '{}'", input.current_char()),
            input.current_token_span(),
        ))
    }
    
    /// Check if an operator string matches at the current position
    fn matches_operator_at_position(&self, input: &Input, op_str: &str) -> bool {
        let remaining = input.remaining();
        remaining.starts_with(op_str)
    }
    
    /// Check if an operator is available in the current context
    fn is_operator_available(&self, token: &OperatorToken, context: &LexerContext) -> bool {
        use crate::lexer::core::EcmaVersion;
        use OperatorToken::*;
        
        match token {
            // ES2020+ operators
            QuestionQuestion | QuestionQuestionEqual | QuestionDot => {
                context.config.ecma_version >= EcmaVersion::ES2020
            }
            
            // ES2021+ operators
            AmpAmpEqual | PipePipeEqual => {
                context.config.ecma_version >= EcmaVersion::ES2021
            }
            
            // ES2016+ operators
            StarStar | StarStarEqual => {
                context.config.ecma_version >= EcmaVersion::ES2016
            }
            
            // All other operators are available in all versions
            _ => true,
        }
    }
    
    /// Get operator information
    pub fn get_operator_info(&self, token: &OperatorToken) -> Option<&OperatorInfo> {
        self.operator_info.get(token)
    }
    
    /// Check if token is a binary operator
    pub fn is_binary_operator(&self, token: &OperatorToken) -> bool {
        self.operator_info.get(token)
            .map(|info| info.binary)
            .unwrap_or(false)
    }
    
    /// Check if token is a unary operator
    pub fn is_unary_operator(&self, token: &OperatorToken) -> bool {
        self.operator_info.get(token)
            .map(|info| info.unary)
            .unwrap_or(false)
    }
    
    /// Check if token is right associative
    pub fn is_right_associative(&self, token: &OperatorToken) -> bool {
        self.operator_info.get(token)
            .map(|info| info.right_associative)
            .unwrap_or(false)
    }
}

impl Scanner for OperatorScanner {
    type Token = OperatorToken;
    
    fn try_scan(&mut self, input: &mut Input, context: &LexerContext) -> Option<Result<Self::Token>> {
        if self.can_scan(input, context) {
            Some(self.scan_operator(input, context))
        } else {
            None
        }
    }
    
    fn can_scan(&self, input: &Input, context: &LexerContext) -> bool {
        // Check if any operator matches at current position
        for (op_str, token) in &self.operator_table {
            if self.matches_operator_at_position(input, op_str) && 
               self.is_operator_available(token, context) {
                return true;
            }
        }
        false
    }
    
    fn name(&self) -> &'static str {
        "OperatorScanner"
    }
}

impl LookaheadScanner for OperatorScanner {
    fn would_match(&self, input: &mut Input, context: &LexerContext) -> bool {
        self.can_scan(input, context)
    }
    
    fn expected_length(&self, input: &mut Input, context: &LexerContext) -> Option<usize> {
        // Find the longest matching operator
        for (op_str, token) in &self.operator_table {
            if self.matches_operator_at_position(input, op_str) && 
               self.is_operator_available(token, context) {
                return Some(op_str.len());
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::core::{LexerContext, LexerConfigBuilder, EcmaVersion};
    
    fn scan_operator(source: &str) -> Result<OperatorToken> {
        let mut input = Input::new(source);
        let mut scanner = OperatorScanner::new();
        let context = LexerContext::new();
        
        scanner.try_scan(&mut input, &context).unwrap()
    }
    
    fn scan_operator_with_context(source: &str, context: &LexerContext) -> Result<OperatorToken> {
        let mut input = Input::new(source);
        let mut scanner = OperatorScanner::new();
        
        scanner.try_scan(&mut input, context).unwrap()
    }
    
    #[test]
    fn test_arithmetic_operators() {
        assert_eq!(scan_operator("+").unwrap(), OperatorToken::Plus);
        assert_eq!(scan_operator("-").unwrap(), OperatorToken::Minus);
        assert_eq!(scan_operator("*").unwrap(), OperatorToken::Star);
        assert_eq!(scan_operator("/").unwrap(), OperatorToken::Slash);
        assert_eq!(scan_operator("%").unwrap(), OperatorToken::Percent);
    }
    
    #[test]
    fn test_assignment_operators() {
        assert_eq!(scan_operator("=").unwrap(), OperatorToken::Equal);
        assert_eq!(scan_operator("+=").unwrap(), OperatorToken::PlusEqual);
        assert_eq!(scan_operator("-=").unwrap(), OperatorToken::MinusEqual);
        assert_eq!(scan_operator("*=").unwrap(), OperatorToken::StarEqual);
        assert_eq!(scan_operator("/=").unwrap(), OperatorToken::SlashEqual);
        assert_eq!(scan_operator("%=").unwrap(), OperatorToken::PercentEqual);
    }
    
    #[test]
    fn test_comparison_operators() {
        assert_eq!(scan_operator("==").unwrap(), OperatorToken::EqualEqual);
        assert_eq!(scan_operator("===").unwrap(), OperatorToken::EqualEqualEqual);
        assert_eq!(scan_operator("!=").unwrap(), OperatorToken::BangEqual);
        assert_eq!(scan_operator("!==").unwrap(), OperatorToken::BangEqualEqual);
        assert_eq!(scan_operator("<").unwrap(), OperatorToken::Less);
        assert_eq!(scan_operator(">").unwrap(), OperatorToken::Greater);
        assert_eq!(scan_operator("<=").unwrap(), OperatorToken::LessEqual);
        assert_eq!(scan_operator(">=").unwrap(), OperatorToken::GreaterEqual);
    }
    
    #[test]
    fn test_logical_operators() {
        assert_eq!(scan_operator("&&").unwrap(), OperatorToken::AmpAmp);
        assert_eq!(scan_operator("||").unwrap(), OperatorToken::PipePipe);
        assert_eq!(scan_operator("!").unwrap(), OperatorToken::Bang);
    }
    
    #[test]
    fn test_bitwise_operators() {
        assert_eq!(scan_operator("&").unwrap(), OperatorToken::Amp);
        assert_eq!(scan_operator("|").unwrap(), OperatorToken::Pipe);
        assert_eq!(scan_operator("^").unwrap(), OperatorToken::Caret);
        assert_eq!(scan_operator("~").unwrap(), OperatorToken::Tilde);
        assert_eq!(scan_operator("<<").unwrap(), OperatorToken::LessLess);
        assert_eq!(scan_operator(">>").unwrap(), OperatorToken::GreaterGreater);
        assert_eq!(scan_operator(">>>").unwrap(), OperatorToken::GreaterGreaterGreater);
    }
    
    #[test]
    fn test_increment_decrement() {
        assert_eq!(scan_operator("++").unwrap(), OperatorToken::PlusPlus);
        assert_eq!(scan_operator("--").unwrap(), OperatorToken::MinusMinus);
    }
    
    #[test]
    fn test_punctuation() {
        assert_eq!(scan_operator("(").unwrap(), OperatorToken::LeftParen);
        assert_eq!(scan_operator(")").unwrap(), OperatorToken::RightParen);
        assert_eq!(scan_operator("{").unwrap(), OperatorToken::LeftBrace);
        assert_eq!(scan_operator("}").unwrap(), OperatorToken::RightBrace);
        assert_eq!(scan_operator("[").unwrap(), OperatorToken::LeftBracket);
        assert_eq!(scan_operator("]").unwrap(), OperatorToken::RightBracket);
        assert_eq!(scan_operator(";").unwrap(), OperatorToken::Semicolon);
        assert_eq!(scan_operator(",").unwrap(), OperatorToken::Comma);
        assert_eq!(scan_operator(".").unwrap(), OperatorToken::Dot);
        assert_eq!(scan_operator(":").unwrap(), OperatorToken::Colon);
        assert_eq!(scan_operator("?").unwrap(), OperatorToken::Question);
        assert_eq!(scan_operator("=>").unwrap(), OperatorToken::Arrow);
    }
    
    #[test]
    fn test_es2016_operators() {
        let config = LexerConfigBuilder::new()
            .ecma_version(EcmaVersion::ES2016)
            .build();
        let context = LexerContext::with_config(config);
        
        assert_eq!(scan_operator_with_context("**", &context).unwrap(), OperatorToken::StarStar);
        assert_eq!(scan_operator_with_context("**=", &context).unwrap(), OperatorToken::StarStarEqual);
    }
    
    #[test]
    fn test_es2020_operators() {
        let config = LexerConfigBuilder::new()
            .ecma_version(EcmaVersion::ES2020)
            .build();
        let context = LexerContext::with_config(config);
        
        assert_eq!(scan_operator_with_context("??", &context).unwrap(), OperatorToken::QuestionQuestion);
        assert_eq!(scan_operator_with_context("??=", &context).unwrap(), OperatorToken::QuestionQuestionEqual);
        assert_eq!(scan_operator_with_context("?.", &context).unwrap(), OperatorToken::QuestionDot);
    }
    
    #[test]
    fn test_operator_precedence() {
        let scanner = OperatorScanner::new();
        
        let info = scanner.get_operator_info(&OperatorToken::Plus).unwrap();
        assert_eq!(info.precedence, Precedence::Additive);
        assert!(!info.right_associative);
        
        let info = scanner.get_operator_info(&OperatorToken::Equal).unwrap();
        assert_eq!(info.precedence, Precedence::Assignment);
        assert!(info.right_associative);
        
        let info = scanner.get_operator_info(&OperatorToken::StarStar).unwrap();
        assert_eq!(info.precedence, Precedence::Exponentiation);
        assert!(info.right_associative);
    }
    
    #[test]
    fn test_longest_match() {
        // Should match "===" not "==" + "="
        assert_eq!(scan_operator("===").unwrap(), OperatorToken::EqualEqualEqual);
        
        // Should match ">>>" not ">>" + ">"
        assert_eq!(scan_operator(">>>").unwrap(), OperatorToken::GreaterGreaterGreater);
        
        // Should match "++" not "+" + "+"
        assert_eq!(scan_operator("++").unwrap(), OperatorToken::PlusPlus);
    }
    
    #[test]
    fn test_can_scan() {
        let scanner = OperatorScanner::new();
        let context = LexerContext::new();
        
        assert!(scanner.can_scan(&Input::new("+"), &context));
        assert!(scanner.can_scan(&Input::new("==="), &context));
        assert!(scanner.can_scan(&Input::new("("), &context));
        assert!(!scanner.can_scan(&Input::new("hello"), &context));
        assert!(!scanner.can_scan(&Input::new("123"), &context));
    }
    
    #[test]
    fn test_expected_length() {
        let mut scanner = OperatorScanner::new();
        let context = LexerContext::new();
        
        let mut input = Input::new("===");
        assert_eq!(scanner.expected_length(&mut input, &context), Some(3));
        
        let mut input2 = Input::new("++");
        assert_eq!(scanner.expected_length(&mut input2, &context), Some(2));
        
        let mut input3 = Input::new("+");
        assert_eq!(scanner.expected_length(&mut input3, &context), Some(1));
    }
}