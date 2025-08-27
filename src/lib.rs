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
pub mod bytecode;
pub mod vm;

// Re-exports for convenience
pub use error::{Error, Result};
pub use lexer::{Lexer, Token, TokenKind};
pub use parser::{Parser, ast};
pub use bytecode::{BytecodeFunction, Disassembler, ConstantPool, Bytecode, Compiler};
pub use vm::{VM, Value};
use ast::PrettyPrint;

/// Engine version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Engine entry point
pub struct Engine {
    /// Enable detailed AST debugging output
    pub ast_debug_mode: bool,
    /// Enable bytecode debugging output
    pub bytecode_debug_mode: bool,
}

impl Engine {
    /// Create a new engine instance
    pub fn new() -> Self {
        Self {
            ast_debug_mode: false,
            bytecode_debug_mode: false,
        }
    }
    
    /// Create a new engine instance with AST debug mode enabled
    pub fn new_with_ast_debug() -> Self {
        Self {
            ast_debug_mode: true,
            bytecode_debug_mode: false,
        }
    }
    
    /// Create a new engine instance with bytecode debug mode enabled
    pub fn new_with_bytecode_debug() -> Self {
        Self {
            ast_debug_mode: false,
            bytecode_debug_mode: true,
        }
    }
    
    /// Create a new engine instance with all debug modes enabled
    pub fn new_with_all_debug() -> Self {
        Self {
            ast_debug_mode: true,
            bytecode_debug_mode: true,
        }
    }
    
    /// Execute JavaScript source code
    pub fn execute(&mut self, source: &str) -> Result<Value> {
        // Execution pipeline:
        // 1. Parse source to AST ✓
        // 2. Compile AST to bytecode ✓ 
        // 3. Execute bytecode in interpreter ✓
        // 4. Profile and JIT compile hot functions (Phase 5 - TODO)
        
        // Step 1: Tokenize the source code
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize()?;
        
        // Step 2: Parse tokens into AST
        let mut parser = Parser::new(tokens);
        let ast = parser.parse()?;
        
        // Display the parsed AST if requested
        if self.ast_debug_mode {
            println!("AST (detailed tree):");
            println!("{}", ast.pretty_print(0));
            println!();
        }
        
        // Step 3: Compile AST to bytecode
        let bytecode_function = self.compile_to_bytecode(&ast, source)?;
        
        // Display the bytecode if requested
        if self.bytecode_debug_mode {
            println!("Bytecode:");
            println!("{}", Disassembler::quick_disassemble(&bytecode_function));
            println!();
        }
        
        // Step 4: Execute bytecode in VM
        let mut vm = if self.bytecode_debug_mode {
            VM::new_with_debug()
        } else {
            VM::new()
        };
        
        let result = vm.execute(bytecode_function)?;
        
        // Print the result if it's not undefined (for REPL)
        if !matches!(result, Value::Undefined) {
            println!("{}", result);
        }
        
        Ok(result)
    }
    
    /// Compile AST to bytecode using the real compiler
    fn compile_to_bytecode(&self, ast: &ast::Program, source: &str) -> Result<BytecodeFunction> {
        // Create a compiler for the main program
        let compiler = Compiler::new_main(source);
        
        // Compile the AST to bytecode
        compiler.compile(ast)
    }
}

impl Default for Engine {
    fn default() -> Self {
        Self::new()
    }
}