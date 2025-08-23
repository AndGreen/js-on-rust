//! Bytecode disassembler
//!
//! This module provides utilities for converting bytecode back to human-readable
//! assembly-like format. This is essential for debugging, optimization analysis,
//! and developer tooling.

use std::fmt::Write;
use super::function::BytecodeFunction;
use super::instruction::Bytecode;

/// Options for controlling disassembly output
#[derive(Debug, Clone)]
pub struct DisassemblyOptions {
    /// Show line numbers from source code
    pub show_line_numbers: bool,
    /// Show constant values inline instead of just indices
    pub show_constant_values: bool,
    /// Show detailed instruction analysis (stack effects, etc.)
    pub show_analysis: bool,
    /// Add extra spacing for readability
    pub pretty_format: bool,
    /// Show jump targets with labels
    pub show_jump_labels: bool,
}

impl Default for DisassemblyOptions {
    fn default() -> Self {
        Self {
            show_line_numbers: true,
            show_constant_values: true,
            show_analysis: false,
            pretty_format: true,
            show_jump_labels: true,
        }
    }
}

/// Bytecode disassembler for converting bytecode to human-readable format
pub struct Disassembler {
    options: DisassemblyOptions,
}

impl Disassembler {
    /// Create a new disassembler with default options
    pub fn new() -> Self {
        Self {
            options: DisassemblyOptions::default(),
        }
    }
    
    /// Create a new disassembler with custom options
    pub fn with_options(options: DisassemblyOptions) -> Self {
        Self { options }
    }
    
    /// Disassemble a complete function to a formatted string
    pub fn disassemble_function(&self, function: &BytecodeFunction) -> String {
        let mut output = String::new();
        
        // Function header
        if self.options.pretty_format {
            writeln!(output, "=== {} ===", function.signature()).unwrap();
            writeln!(output, "Locals: {}, Max stack: {}", 
                function.locals_count, function.max_stack_size).unwrap();
            writeln!(output).unwrap();
        }
        
        // Constant pool (if requested and not empty)
        if self.options.show_constant_values && !function.constants.is_empty() {
            writeln!(output, "Constants:").unwrap();
            for (index, value) in function.constants.iter() {
                writeln!(output, "  #{}: {}", index, value).unwrap();
            }
            writeln!(output).unwrap();
        }
        
        // Find jump targets for labeling
        let jump_targets = if self.options.show_jump_labels {
            self.find_jump_targets(function)
        } else {
            Vec::new()
        };
        
        // Disassemble instructions
        writeln!(output, "Bytecode:").unwrap();
        for (offset, instruction) in function.bytecode.iter().enumerate() {
            // Show jump labels
            if self.options.show_jump_labels && jump_targets.contains(&offset) {
                writeln!(output, "L{}:", offset).unwrap();
            }
            
            let line = self.disassemble_instruction(
                instruction, 
                offset, 
                function
            );
            writeln!(output, "{}", line).unwrap();
        }
        
        output
    }
    
    /// Disassemble a single instruction with context
    pub fn disassemble_instruction(
        &self,
        instruction: &Bytecode,
        offset: usize,
        function: &BytecodeFunction,
    ) -> String {
        let mut line = String::new();
        
        // Offset
        write!(line, "  {:4}:", offset).unwrap();
        
        // Instruction mnemonic and operands  
        let instruction_str = self.format_instruction_with_offset(instruction, offset, function);
        write!(line, " {:<20}", instruction_str).unwrap();
        
        // Comments and analysis
        let mut comments = Vec::new();
        
        // Line number comment
        if self.options.show_line_numbers {
            if let Some(line_num) = function.debug_info.get_line_number(offset) {
                comments.push(format!("line {}", line_num));
            }
        }
        
        // Instruction analysis
        if self.options.show_analysis {
            let stack_effect = instruction.stack_pop_count() as i32 - instruction.stack_push_count() as i32;
            if stack_effect != 0 {
                comments.push(format!("stack: {}", stack_effect));
            }
            
            if instruction.is_control_flow() {
                comments.push("control flow".to_string());
            }
        }
        
        // Add comments
        if !comments.is_empty() {
            write!(line, " ; {}", comments.join(", ")).unwrap();
        }
        
        line
    }
    
    /// Format a single instruction with its operands and offset context
    fn format_instruction_with_offset(&self, instruction: &Bytecode, offset: usize, function: &BytecodeFunction) -> String {
        match instruction {
            // Jump instructions with labels
            Bytecode::Jump(jump_offset) => {
                if self.options.show_jump_labels {
                    let target = (offset as i32 + 1 + *jump_offset as i32) as usize;
                    format!("Jump L{} ({})", target, jump_offset)
                } else {
                    format!("Jump {}", jump_offset)
                }
            }
            
            Bytecode::JumpIfFalse(jump_offset) => {
                if self.options.show_jump_labels {
                    let target = (offset as i32 + 1 + *jump_offset as i32) as usize;
                    format!("JumpIfFalse L{} ({})", target, jump_offset)
                } else {
                    format!("JumpIfFalse {}", jump_offset)
                }
            }
            
            Bytecode::JumpIfTrue(jump_offset) => {
                if self.options.show_jump_labels {
                    let target = (offset as i32 + 1 + *jump_offset as i32) as usize;
                    format!("JumpIfTrue L{} ({})", target, jump_offset)
                } else {
                    format!("JumpIfTrue {}", jump_offset)
                }
            }
            
            Bytecode::JumpIfNullish(jump_offset) => {
                if self.options.show_jump_labels {
                    let target = (offset as i32 + 1 + *jump_offset as i32) as usize;
                    format!("JumpIfNullish L{} ({})", target, jump_offset)
                } else {
                    format!("JumpIfNullish {}", jump_offset)
                }
            }
            
            // For all other instructions, use the original method
            _ => self.format_instruction(instruction, function),
        }
    }
    
    /// Format a single instruction with its operands
    fn format_instruction(&self, instruction: &Bytecode, function: &BytecodeFunction) -> String {
        match instruction {
            // Instructions with constant references
            Bytecode::LdaConst(idx) => {
                if self.options.show_constant_values {
                    if let Some(value) = function.constants.get(*idx) {
                        format!("LdaConst #{} ({})", idx, value)
                    } else {
                        format!("LdaConst #{} (invalid)", idx)
                    }
                } else {
                    format!("LdaConst #{}", idx)
                }
            }
            
            Bytecode::LdaGlobal(idx) => {
                if self.options.show_constant_values {
                    if let Some(value) = function.constants.get(*idx) {
                        format!("LdaGlobal #{} ({})", idx, value)
                    } else {
                        format!("LdaGlobal #{} (invalid)", idx)
                    }
                } else {
                    format!("LdaGlobal #{}", idx)
                }
            }
            
            Bytecode::StaGlobal(idx) => {
                if self.options.show_constant_values {
                    if let Some(value) = function.constants.get(*idx) {
                        format!("StaGlobal #{} ({})", idx, value)
                    } else {
                        format!("StaGlobal #{} (invalid)", idx)
                    }
                } else {
                    format!("StaGlobal #{}", idx)
                }
            }
            
            Bytecode::LdaNamed(idx) => {
                if self.options.show_constant_values {
                    if let Some(value) = function.constants.get(*idx) {
                        format!("LdaNamed #{} ({})", idx, value)
                    } else {
                        format!("LdaNamed #{} (invalid)", idx)
                    }
                } else {
                    format!("LdaNamed #{}", idx)
                }
            }
            
            Bytecode::StaNamed(idx) => {
                if self.options.show_constant_values {
                    if let Some(value) = function.constants.get(*idx) {
                        format!("StaNamed #{} ({})", idx, value)
                    } else {
                        format!("StaNamed #{} (invalid)", idx)
                    }
                } else {
                    format!("StaNamed #{}", idx)
                }
            }
            
            Bytecode::CreateArray(idx) => {
                if self.options.show_constant_values {
                    if let Some(value) = function.constants.get(*idx) {
                        format!("CreateArray #{} ({})", idx, value)
                    } else {
                        format!("CreateArray #{} (invalid)", idx)
                    }
                } else {
                    format!("CreateArray #{}", idx)
                }
            }
            
            Bytecode::CreateClosure(idx) => {
                if self.options.show_constant_values {
                    if let Some(value) = function.constants.get(*idx) {
                        format!("CreateClosure #{} ({})", idx, value)
                    } else {
                        format!("CreateClosure #{} (invalid)", idx)
                    }
                } else {
                    format!("CreateClosure #{}", idx)
                }
            }
            
            // Default formatting for other instructions  
            _ => instruction.to_string(),
        }
    }
    
    /// Find all jump targets in the function for labeling
    fn find_jump_targets(&self, function: &BytecodeFunction) -> Vec<usize> {
        let mut targets = Vec::new();
        
        for (offset, instruction) in function.bytecode.iter().enumerate() {
            match instruction {
                Bytecode::Jump(jump_offset) |
                Bytecode::JumpIfFalse(jump_offset) |
                Bytecode::JumpIfTrue(jump_offset) |
                Bytecode::JumpIfNullish(jump_offset) => {
                    // Jump target is relative to the next instruction
                    let target = (offset as i32 + 1 + *jump_offset as i32) as usize;
                    if target <= function.bytecode.len() {
                        targets.push(target);
                    }
                }
                _ => {}
            }
        }
        
        targets.sort();
        targets.dedup();
        targets
    }
    
    /// Set disassembly options
    pub fn set_options(&mut self, options: DisassemblyOptions) {
        self.options = options;
    }
}

impl Default for Disassembler {
    fn default() -> Self {
        Self::new()
    }
}

// Convenience functions for quick disassembly
impl Disassembler {
    /// Quick disassembly with default options
    pub fn quick_disassemble(function: &BytecodeFunction) -> String {
        let disassembler = Disassembler::new();
        disassembler.disassemble_function(function)
    }
    
    /// Minimal disassembly (no constants, no line numbers)
    pub fn minimal_disassemble(function: &BytecodeFunction) -> String {
        let options = DisassemblyOptions {
            show_line_numbers: false,
            show_constant_values: false,
            show_analysis: false,
            pretty_format: false,
            show_jump_labels: false,
        };
        let disassembler = Disassembler::with_options(options);
        disassembler.disassemble_function(function)
    }
    
    /// Detailed disassembly with all analysis information
    pub fn detailed_disassemble(function: &BytecodeFunction) -> String {
        let options = DisassemblyOptions {
            show_line_numbers: true,
            show_constant_values: true,
            show_analysis: true,
            pretty_format: true,
            show_jump_labels: true,
        };
        let disassembler = Disassembler::with_options(options);
        disassembler.disassemble_function(function)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bytecode::{BytecodeFunction, Bytecode};
    
    #[test]
    fn test_simple_disassembly() {
        let mut function = BytecodeFunction::new_main();
        
        // Add some constants
        let const_42 = function.constants.add_number(42.0);
        let const_hello = function.constants.add_string("hello".to_string());
        
        // Add some instructions
        function.add_instruction(Bytecode::LdaConst(const_42));
        function.add_instruction(Bytecode::LdaConst(const_hello));
        function.add_instruction(Bytecode::Add);
        function.add_instruction(Bytecode::Return);
        
        let disassembly = Disassembler::quick_disassemble(&function);
        
        // Basic checks
        assert!(disassembly.contains("LdaConst"));
        assert!(disassembly.contains("Add"));
        assert!(disassembly.contains("Return"));
        assert!(disassembly.contains("42"));
        assert!(disassembly.contains("hello"));
    }
    
    #[test]
    fn test_jump_disassembly() {
        let mut function = BytecodeFunction::new_main();
        
        function.add_instruction(Bytecode::LdaConst(0));
        function.add_instruction(Bytecode::JumpIfFalse(1)); // Jump forward 1 instruction to instruction 3
        function.add_instruction(Bytecode::Return);
        function.add_instruction(Bytecode::ReturnUndefined); // Jump target should be here (index 3)
        
        let disassembly = Disassembler::quick_disassemble(&function);
        
        // Should contain jump with label
        assert!(disassembly.contains("JumpIfFalse"));
        assert!(disassembly.contains("L3")); // Jump target label (offset=1, jump=1 -> target=1+1+1=3)
    }
}