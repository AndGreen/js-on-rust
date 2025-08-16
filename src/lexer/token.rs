//! Token definitions for JavaScript lexer

use crate::error::Span;
use std::fmt;

/// A JavaScript token with position information
#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
    pub text: String,
}

impl Token {
    pub fn new(kind: TokenKind, span: Span, text: String) -> Self {
        Self { kind, span, text }
    }
}

/// JavaScript token types
#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    // Literals
    Number(f64),
    String(String),
    Boolean(bool),
    Null,
    Undefined,
    
    // Identifiers and keywords
    Identifier(String),
    Keyword(Keyword),
    
    // Arithmetic operators
    Plus,           // +
    Minus,          // -
    Star,           // *
    Slash,          // /
    Percent,        // %
    StarStar,       // **
    
    // Assignment operators  
    Equal,          // =
    PlusEqual,      // +=
    MinusEqual,     // -=
    StarEqual,      // *=
    SlashEqual,     // /=
    PercentEqual,   // %=
    
    // Comparison operators
    EqualEqual,     // ==
    EqualEqualEqual, // ===
    BangEqual,      // !=
    BangEqualEqual, // !==
    Less,           // <
    Greater,        // >
    LessEqual,      // <=
    GreaterEqual,   // >=
    
    // Logical operators
    AmpAmp,         // &&
    PipePipe,       // ||
    Bang,           // !
    
    // Bitwise operators
    Amp,            // &
    Pipe,           // |
    Caret,          // ^
    Tilde,          // ~
    LessLess,       // <<
    GreaterGreater, // >>
    GreaterGreaterGreater, // >>>
    
    // Increment/decrement
    PlusPlus,       // ++
    MinusMinus,     // --
    
    // Punctuation
    LeftParen,      // (
    RightParen,     // )
    LeftBrace,      // {
    RightBrace,     // }
    LeftBracket,    // [
    RightBracket,   // ]
    Semicolon,      // ;
    Comma,          // ,
    Dot,            // .
    Colon,          // :
    Question,       // ?
    Arrow,          // =>
    
    // Special tokens
    Eof,
    Newline,
    
    // Error token
    Error(String),
}

/// JavaScript keywords
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Keyword {
    // Declarations
    Function,
    Var,
    Let,
    Const,
    Class,
    
    // Control flow
    If,
    Else,
    While,
    For,
    Do,
    Switch,
    Case,
    Default,
    Break,
    Continue,
    Return,
    
    // Exception handling
    Try,
    Catch,
    Finally,
    Throw,
    
    // Other keywords
    New,
    This,
    Super,
    Extends,
    Static,
    
    // Literals as keywords
    True,
    False,
    Null,
    Undefined,
    
    // Module system
    Import,
    Export,
    From,
    As,
    DefaultKeyword,
    
    // Async/await
    Async,
    Await,
    
    // Other
    Typeof,
    Instanceof,
    In,
    Of,
    With,
    Delete,
    Void,
    Yield,
}

impl Keyword {
    /// Parse a keyword from a string
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "function" => Some(Keyword::Function),
            "var" => Some(Keyword::Var),
            "let" => Some(Keyword::Let),
            "const" => Some(Keyword::Const),
            "class" => Some(Keyword::Class),
            "if" => Some(Keyword::If),
            "else" => Some(Keyword::Else),
            "while" => Some(Keyword::While),
            "for" => Some(Keyword::For),
            "do" => Some(Keyword::Do),
            "switch" => Some(Keyword::Switch),
            "case" => Some(Keyword::Case),
            "default" => Some(Keyword::Default),
            "break" => Some(Keyword::Break),
            "continue" => Some(Keyword::Continue),
            "return" => Some(Keyword::Return),
            "try" => Some(Keyword::Try),
            "catch" => Some(Keyword::Catch),
            "finally" => Some(Keyword::Finally),
            "throw" => Some(Keyword::Throw),
            "new" => Some(Keyword::New),
            "this" => Some(Keyword::This),
            "super" => Some(Keyword::Super),
            "extends" => Some(Keyword::Extends),
            "static" => Some(Keyword::Static),
            "true" => Some(Keyword::True),
            "false" => Some(Keyword::False),
            "null" => Some(Keyword::Null),
            "undefined" => Some(Keyword::Undefined),
            "import" => Some(Keyword::Import),
            "export" => Some(Keyword::Export),
            "from" => Some(Keyword::From),
            "as" => Some(Keyword::As),
            "async" => Some(Keyword::Async),
            "await" => Some(Keyword::Await),
            "typeof" => Some(Keyword::Typeof),
            "instanceof" => Some(Keyword::Instanceof),
            "in" => Some(Keyword::In),
            "of" => Some(Keyword::Of),
            "with" => Some(Keyword::With),
            "delete" => Some(Keyword::Delete),
            "void" => Some(Keyword::Void),
            "yield" => Some(Keyword::Yield),
            _ => None,
        }
    }
    
    /// Get the string representation of a keyword
    pub fn as_str(self) -> &'static str {
        match self {
            Keyword::Function => "function",
            Keyword::Var => "var",
            Keyword::Let => "let",
            Keyword::Const => "const",
            Keyword::Class => "class",
            Keyword::If => "if",
            Keyword::Else => "else",
            Keyword::While => "while",
            Keyword::For => "for",
            Keyword::Do => "do",
            Keyword::Switch => "switch",
            Keyword::Case => "case",
            Keyword::Default => "default",
            Keyword::Break => "break",
            Keyword::Continue => "continue",
            Keyword::Return => "return",
            Keyword::Try => "try",
            Keyword::Catch => "catch",
            Keyword::Finally => "finally",
            Keyword::Throw => "throw",
            Keyword::New => "new",
            Keyword::This => "this",
            Keyword::Super => "super",
            Keyword::Extends => "extends",
            Keyword::Static => "static",
            Keyword::True => "true",
            Keyword::False => "false",
            Keyword::Null => "null",
            Keyword::Undefined => "undefined",
            Keyword::Import => "import",
            Keyword::Export => "export",
            Keyword::From => "from",
            Keyword::As => "as",
            Keyword::DefaultKeyword => "default",
            Keyword::Async => "async",
            Keyword::Await => "await",
            Keyword::Typeof => "typeof",
            Keyword::Instanceof => "instanceof",
            Keyword::In => "in",
            Keyword::Of => "of",
            Keyword::With => "with",
            Keyword::Delete => "delete",
            Keyword::Void => "void",
            Keyword::Yield => "yield",
        }
    }
}

impl fmt::Display for TokenKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TokenKind::Number(n) => write!(f, "{}", n),
            TokenKind::String(s) => write!(f, "\"{}\"", s),
            TokenKind::Boolean(b) => write!(f, "{}", b),
            TokenKind::Null => write!(f, "null"),
            TokenKind::Undefined => write!(f, "undefined"),
            TokenKind::Identifier(name) => write!(f, "{}", name),
            TokenKind::Keyword(kw) => write!(f, "{}", kw.as_str()),
            TokenKind::Plus => write!(f, "+"),
            TokenKind::Minus => write!(f, "-"),
            TokenKind::Star => write!(f, "*"),
            TokenKind::Slash => write!(f, "/"),
            TokenKind::Percent => write!(f, "%"),
            TokenKind::StarStar => write!(f, "**"),
            TokenKind::Equal => write!(f, "="),
            TokenKind::PlusEqual => write!(f, "+="),
            TokenKind::MinusEqual => write!(f, "-="),
            TokenKind::StarEqual => write!(f, "*="),
            TokenKind::SlashEqual => write!(f, "/="),
            TokenKind::PercentEqual => write!(f, "%="),
            TokenKind::EqualEqual => write!(f, "=="),
            TokenKind::EqualEqualEqual => write!(f, "==="),
            TokenKind::BangEqual => write!(f, "!="),
            TokenKind::BangEqualEqual => write!(f, "!=="),
            TokenKind::Less => write!(f, "<"),
            TokenKind::Greater => write!(f, ">"),
            TokenKind::LessEqual => write!(f, "<="),
            TokenKind::GreaterEqual => write!(f, ">="),
            TokenKind::AmpAmp => write!(f, "&&"),
            TokenKind::PipePipe => write!(f, "||"),
            TokenKind::Bang => write!(f, "!"),
            TokenKind::Amp => write!(f, "&"),
            TokenKind::Pipe => write!(f, "|"),
            TokenKind::Caret => write!(f, "^"),
            TokenKind::Tilde => write!(f, "~"),
            TokenKind::LessLess => write!(f, "<<"),
            TokenKind::GreaterGreater => write!(f, ">>"),
            TokenKind::GreaterGreaterGreater => write!(f, ">>>"),
            TokenKind::PlusPlus => write!(f, "++"),
            TokenKind::MinusMinus => write!(f, "--"),
            TokenKind::LeftParen => write!(f, "("),
            TokenKind::RightParen => write!(f, ")"),
            TokenKind::LeftBrace => write!(f, "{{"),
            TokenKind::RightBrace => write!(f, "}}"),
            TokenKind::LeftBracket => write!(f, "["),
            TokenKind::RightBracket => write!(f, "]"),
            TokenKind::Semicolon => write!(f, ";"),
            TokenKind::Comma => write!(f, ","),
            TokenKind::Dot => write!(f, "."),
            TokenKind::Colon => write!(f, ":"),
            TokenKind::Question => write!(f, "?"),
            TokenKind::Arrow => write!(f, "=>"),
            TokenKind::Eof => write!(f, "EOF"),
            TokenKind::Newline => write!(f, "\\n"),
            TokenKind::Error(msg) => write!(f, "ERROR({})", msg),
        }
    }
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.kind)
    }
}