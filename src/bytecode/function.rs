//! Bytecode function representation
//!
//! This module defines the structure for compiled JavaScript functions,
//! including bytecode instructions, constant pool, and debug information.

use std::fmt;
use crate::error::Span;
use super::instruction::{Bytecode, LocalIndex};
use super::constant_pool::ConstantPool;

/// Debug information for mapping bytecode back to source code
#[derive(Debug, Clone, PartialEq)]
pub struct DebugInfo {
    /// Maps bytecode instruction index to source code span
    pub source_map: Vec<Option<Span>>,
    /// Original source code (for error reporting)
    pub source_code: Option<String>,
    /// Line number information for each instruction
    pub line_numbers: Vec<Option<u32>>,
}

impl DebugInfo {
    /// Create new debug info with the given capacity
    pub fn new(capacity: usize) -> Self {
        Self {
            source_map: Vec::with_capacity(capacity),
            source_code: None,
            line_numbers: Vec::with_capacity(capacity),
        }
    }
    
    /// Create empty debug info (for synthetic functions)
    pub fn empty() -> Self {
        Self {
            source_map: Vec::new(),
            source_code: None,
            line_numbers: Vec::new(),
        }
    }
    
    /// Add debug information for an instruction
    pub fn add_instruction(&mut self, span: Option<Span>) {
        let line_number = span.as_ref().map(|s| s.line);
        self.source_map.push(span);
        self.line_numbers.push(line_number);
    }
    
    /// Set the original source code
    pub fn set_source_code(&mut self, source: String) {
        self.source_code = Some(source);
    }
    
    /// Get span information for a bytecode instruction
    pub fn get_span(&self, instruction_index: usize) -> Option<&Span> {
        self.source_map.get(instruction_index).and_then(|s| s.as_ref())
    }
    
    /// Get line number for a bytecode instruction
    pub fn get_line_number(&self, instruction_index: usize) -> Option<u32> {
        self.line_numbers.get(instruction_index).copied().flatten()
    }
}

/// Represents a compiled JavaScript function with bytecode
#[derive(Debug, Clone, PartialEq)]
pub struct BytecodeFunction {
    /// Function name (for debugging and stack traces)
    pub name: String,
    
    /// Number of parameters this function expects
    pub arity: u8,
    
    /// Total number of local variable slots needed
    /// This includes parameters, local variables, and temporary slots
    pub locals_count: LocalIndex,
    
    /// Maximum stack depth required during execution
    pub max_stack_size: usize,
    
    /// The bytecode instructions that make up this function
    pub bytecode: Vec<Bytecode>,
    
    /// Pool of constants referenced by the bytecode
    pub constants: ConstantPool,
    
    /// Debug information for mapping back to source
    pub debug_info: DebugInfo,
    
    /// Whether this function is a generator
    pub is_generator: bool,
    
    /// Whether this function is async
    pub is_async: bool,
    
    /// Whether this is an arrow function (affects 'this' binding)
    pub is_arrow: bool,
}

impl BytecodeFunction {
    /// Create a new bytecode function
    pub fn new(
        name: String,
        arity: u8,
        locals_count: LocalIndex,
        max_stack_size: usize,
    ) -> Self {
        Self {
            name,
            arity,
            locals_count,
            max_stack_size,
            bytecode: Vec::new(),
            constants: ConstantPool::new(),
            debug_info: DebugInfo::empty(),
            is_generator: false,
            is_async: false,
            is_arrow: false,
        }
    }
    
    /// Create a new main function (for top-level code)
    pub fn new_main() -> Self {
        Self::new(
            "<main>".to_string(),
            0,  // no parameters
            0,  // will be set during compilation
            0,  // will be calculated during compilation
        )
    }
    
    /// Add a bytecode instruction
    pub fn add_instruction(&mut self, instruction: Bytecode) {
        self.bytecode.push(instruction);
        // Add empty debug info for now (will be filled by compiler)
        self.debug_info.add_instruction(None);
    }
    
    /// Add a bytecode instruction with span information
    pub fn add_instruction_with_span(&mut self, instruction: Bytecode, span: Span) {
        self.bytecode.push(instruction);
        self.debug_info.add_instruction(Some(span));
    }
    
    /// Get the current bytecode length (useful for jump targets)
    pub fn current_offset(&self) -> usize {
        self.bytecode.len()
    }
    
    /// Replace an instruction at the given offset (for backpatching jumps)
    pub fn patch_instruction(&mut self, offset: usize, instruction: Bytecode) {
        if let Some(existing) = self.bytecode.get_mut(offset) {
            *existing = instruction;
        }
    }
    
    /// Get instruction at offset (for analysis and optimization)
    pub fn get_instruction(&self, offset: usize) -> Option<&Bytecode> {
        self.bytecode.get(offset)
    }
    
    /// Calculate and update the maximum stack size
    /// This is important for VM stack allocation
    pub fn calculate_stack_size(&mut self) {
        let mut current_stack = 0i32;
        let mut max_stack = 0i32;
        
        for instruction in &self.bytecode {
            // Update current stack based on pops and pushes
            current_stack -= instruction.stack_pop_count() as i32;
            current_stack += instruction.stack_push_count() as i32;
            
            // Track maximum stack usage
            max_stack = max_stack.max(current_stack);
        }
        
        self.max_stack_size = max_stack.max(0) as usize;
    }
    
    /// Set function flags for special function types
    pub fn set_flags(&mut self, is_generator: bool, is_async: bool, is_arrow: bool) {
        self.is_generator = is_generator;
        self.is_async = is_async;
        self.is_arrow = is_arrow;
    }
    
    /// Get a human-readable signature of this function
    pub fn signature(&self) -> String {
        let mut sig = String::new();
        
        if self.is_async {
            sig.push_str("async ");
        }
        
        sig.push_str("function");
        
        if self.is_generator {
            sig.push('*');
        }
        
        if !self.name.is_empty() && self.name != "<anonymous>" {
            sig.push(' ');
            sig.push_str(&self.name);
        }
        
        sig.push('(');
        if self.arity > 0 {
            for i in 0..self.arity {
                if i > 0 {
                    sig.push_str(", ");
                }
                sig.push_str(&format!("arg{}", i));
            }
        }
        sig.push(')');
        
        sig
    }
    
    /// Get statistics about this function for profiling
    pub fn stats(&self) -> FunctionStats {
        let total_instructions = self.bytecode.len();
        let control_flow_instructions = self.bytecode.iter()
            .filter(|instr| instr.is_control_flow())
            .count();
        
        FunctionStats {
            name: self.name.clone(),
            instruction_count: total_instructions,
            control_flow_count: control_flow_instructions,
            constants_count: self.constants.len(),
            locals_count: self.locals_count as usize,
            max_stack_size: self.max_stack_size,
            arity: self.arity,
        }
    }
}

/// Statistics about a compiled function
#[derive(Debug, Clone)]
pub struct FunctionStats {
    pub name: String,
    pub instruction_count: usize,
    pub control_flow_count: usize,
    pub constants_count: usize,
    pub locals_count: usize,
    pub max_stack_size: usize,
    pub arity: u8,
}

impl fmt::Display for FunctionStats {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Function: {}", self.name)?;
        writeln!(f, "  Instructions: {}", self.instruction_count)?;
        writeln!(f, "  Control flow: {}", self.control_flow_count)?;
        writeln!(f, "  Constants: {}", self.constants_count)?;
        writeln!(f, "  Locals: {}", self.locals_count)?;
        writeln!(f, "  Max stack: {}", self.max_stack_size)?;
        write!(f, "  Arity: {}", self.arity)
    }
}

impl fmt::Display for BytecodeFunction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{} {{", self.signature())?;
        writeln!(f, "  locals: {}, stack: {}", self.locals_count, self.max_stack_size)?;
        
        for (i, instruction) in self.bytecode.iter().enumerate() {
            let line_info = self.debug_info.get_line_number(i)
                .map(|line| format!(" ; line {}", line))
                .unwrap_or_default();
            
            writeln!(f, "  {:4}: {}{}", i, instruction, line_info)?;
        }
        
        write!(f, "}}")
    }
}