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

// Re-exports for convenience
pub use error::{Error, Result};
pub use lexer::{Lexer, Token, TokenKind};
pub use parser::{Parser, ast};
pub use bytecode::{BytecodeFunction, Disassembler, ConstantPool, Bytecode};
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
    pub fn execute(&mut self, source: &str) -> Result<()> {
        // TODO: Implement execution pipeline:
        // 1. Parse source to AST ✓
        // 2. Compile AST to bytecode ✓ (stub)
        // 3. Execute bytecode in interpreter (Phase 2.3)
        // 4. Profile and JIT compile hot functions (Phase 5)
        
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
        
        // Step 3: Compile AST to bytecode (stub implementation)
        let bytecode_function = self.compile_to_bytecode(&ast, source)?;
        
        // Display the bytecode if requested
        if self.bytecode_debug_mode {
            println!("Bytecode:");
            println!("{}", Disassembler::quick_disassemble(&bytecode_function));
            println!();
        }
        
        // Step 4: Execute bytecode (placeholder)
        println!("Successfully compiled to bytecode:");
        println!("  Function: {}", bytecode_function.signature());
        println!("  Instructions: {}", bytecode_function.bytecode.len());
        println!("  Constants: {}", bytecode_function.constants.len());
        println!("  Locals: {}", bytecode_function.locals_count);
        
        // TODO: Execute in virtual machine (Phase 2.3)
        Ok(())
    }
    
    /// Compile AST to bytecode (stub implementation for Phase 2.1)
    /// This will be fully implemented in Phase 2.2
    fn compile_to_bytecode(&self, _ast: &ast::Program, source: &str) -> Result<BytecodeFunction> {
        // Create a simple stub function that demonstrates the bytecode system
        let mut function = BytecodeFunction::new_main();
        function.debug_info.set_source_code(source.to_string());
        
        // Add some example constants
        let const_42 = function.constants.add_number(42.0);
        let const_hello = function.constants.add_string("Hello, Bytecode!".to_string());
        
        // Add some example bytecode instructions (stub)
        function.add_instruction(Bytecode::LdaConst(const_42));
        function.add_instruction(Bytecode::StaLocal(0));
        function.add_instruction(Bytecode::LdaConst(const_hello));
        function.add_instruction(Bytecode::StaLocal(1));
        function.add_instruction(Bytecode::LdaLocal(0));
        function.add_instruction(Bytecode::LdaLocal(1));
        function.add_instruction(Bytecode::Add); // This will concatenate in JS
        function.add_instruction(Bytecode::Return);
        
        // Set basic function properties
        function.locals_count = 2; // Two local variables
        function.calculate_stack_size(); // Calculate maximum stack usage
        
        Ok(function)
    }
}

impl Default for Engine {
    fn default() -> Self {
        Self::new()
    }
}