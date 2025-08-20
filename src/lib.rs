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
use ast::PrettyPrint;

/// Engine version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Engine entry point
pub struct Engine {
    /// Enable detailed AST debugging output
    pub ast_debug_mode: bool,
}

impl Engine {
    /// Create a new engine instance
    pub fn new() -> Self {
        Self {
            ast_debug_mode: false,
        }
    }
    
    /// Create a new engine instance with AST debug mode enabled
    pub fn new_with_ast_debug() -> Self {
        Self {
            ast_debug_mode: true,
        }
    }
    
    /// Execute JavaScript source code
    pub fn execute(&mut self, source: &str) -> Result<()> {
        // TODO: Implement execution pipeline:
        // 1. Parse source to AST ✓
        // 2. Compile AST to bytecode  
        // 3. Execute bytecode in interpreter
        // 4. Profile and JIT compile hot functions
        
        // Step 1: Tokenize the source code
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize()?;
        
        // Step 2: Parse tokens into AST
        let mut parser = Parser::new(tokens);
        let ast = parser.parse()?;
        
        // Display the parsed AST for demonstration
        if self.ast_debug_mode {
            println!("Successfully parsed JavaScript (detailed AST tree):");
            println!("{}", ast.pretty_print(0));
        } else {
            println!("Successfully parsed JavaScript:");
            println!("{}", ast);
        }
        
        // TODO: Continue with bytecode generation and execution
        Ok(())
    }
}

impl Default for Engine {
    fn default() -> Self {
        Self::new()
    }
}