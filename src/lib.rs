//! V8-like JavaScript Engine
//! 
//! A JavaScript engine implementation in Rust featuring:
//! - Tiered compilation (Interpreter → Baseline JIT → Optimizing JIT)
//! - Hidden classes for fast property access
//! - Inline caches for adaptive optimization
//! - Generational garbage collection

pub mod error;
pub mod lexer;
pub mod parser;

// Re-exports for convenience
pub use error::{Error, Result};
pub use lexer::{Lexer, Token, TokenKind};
pub use parser::{Parser, ast};

/// Engine version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Engine entry point
pub struct Engine;

impl Engine {
    /// Create a new engine instance
    pub fn new() -> Self {
        Self
    }
    
    /// Execute JavaScript source code
    pub fn execute(&mut self, source: &str) -> Result<()> {
        // TODO: Implement execution pipeline:
        // 1. Parse source to AST
        // 2. Compile AST to bytecode  
        // 3. Execute bytecode in interpreter
        // 4. Profile and JIT compile hot functions
        
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize()?;
        
        let mut parser = Parser::new(tokens);
        let _ast = parser.parse()?;
        
        // TODO: Continue with bytecode generation and execution
        println!("Successfully parsed source code");
        Ok(())
    }
}

impl Default for Engine {
    fn default() -> Self {
        Self::new()
    }
}