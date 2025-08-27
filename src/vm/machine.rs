//! Virtual Machine Implementation
//!
//! This module implements the stack-based virtual machine that executes
//! JavaScript bytecode with an accumulator register.

use std::collections::HashMap;
use std::rc::Rc;

use crate::bytecode::{BytecodeFunction, Bytecode, ConstantValue};
use crate::error::{Error, Result};
use super::value::Value;
use super::frame::{CallFrame, CallStack};
use super::builtins::Builtins;

/// Stack-based virtual machine with accumulator
pub struct VM {
    /// The accumulator register
    accumulator: Value,
    
    /// Operand stack for temporary values
    stack: Vec<Value>,
    
    /// Call stack for function invocations
    call_stack: CallStack,
    
    /// Global variables
    globals: HashMap<String, Value>,
    
    /// Built-in functions registry
    builtins: Builtins,
    
    /// Bytecode functions (for function calls)
    functions: Vec<Rc<BytecodeFunction>>,
    
    /// Debug mode flag
    debug: bool,
    
    /// Maximum stack size to prevent overflow
    max_stack_size: usize,
}

impl VM {
    /// Create a new VM instance
    pub fn new() -> Self {
        Self {
            accumulator: Value::Undefined,
            stack: Vec::with_capacity(256),
            call_stack: CallStack::new(1000),
            globals: HashMap::new(),
            builtins: Builtins::new(),
            functions: Vec::new(),
            debug: false,
            max_stack_size: 10000,
        }
    }
    
    /// Create a new VM with debug mode enabled
    pub fn new_with_debug() -> Self {
        let mut vm = Self::new();
        vm.debug = true;
        vm
    }
    
    /// Execute a bytecode function
    pub fn execute(&mut self, function: BytecodeFunction) -> Result<Value> {
        // Store the main function
        let main_func = Rc::new(function);
        self.functions.push(main_func.clone());
        
        // Create and push the main frame
        let main_frame = CallFrame::new_main(main_func);
        self.call_stack.push(main_frame)
            .map_err(|e| Error::Runtime { message: e, span: None })?;
        
        // Run the interpreter loop
        let result = self.run();
        
        // Return the final value (from accumulator or undefined)
        match result {
            Ok(_) => Ok(self.accumulator.clone()),
            Err(e) => Err(e),
        }
    }
    
    /// Main interpreter loop
    fn run(&mut self) -> Result<()> {
        loop {
            // Check if we have an active frame
            if self.call_stack.is_empty() {
                break; // No more frames to execute
            }
            
            // Get frame info we need
            let (instruction, ip, at_end) = {
                let frame = self.call_stack.current_frame_mut()
                    .ok_or_else(|| Error::Runtime { message: "No active frame".to_string(), span: None })?;
                
                // Check if we've reached the end of the function
                if frame.is_at_end() {
                    (None, 0, true)
                } else {
                    let instruction = frame.function.bytecode[frame.ip()].clone();
                    let ip = frame.ip();
                    frame.advance_ip(1);
                    (Some(instruction), ip, false)
                }
            };
            
            if at_end {
                // Implicit return undefined
                self.accumulator = Value::Undefined;
                if !self.handle_return()? {
                    break; // Main function returned
                }
                continue;
            }
            
            let instruction = instruction.unwrap();
            
            if self.debug {
                self.debug_instruction(&instruction, ip);
            }
            
            // Execute the instruction
            self.execute_instruction(instruction)?;
        }
        
        Ok(())
    }
    
    /// Execute a single bytecode instruction
    fn execute_instruction(&mut self, instruction: Bytecode) -> Result<()> {
        match instruction {
            // === Load/Store Operations ===
            Bytecode::LdaConst(idx) => {
                let constant = self.get_constant(idx)?;
                self.accumulator = self.constant_to_value(constant)?;
            }
            
            Bytecode::LdaLocal(idx) => {
                let frame = self.call_stack.current_frame()
                    .ok_or_else(|| Error::Runtime { message: "No active frame".to_string(), span: None })?;
                self.accumulator = frame.get_local(idx as usize)
                    .map_err(|e| Error::Runtime { message: e, span: None })?
                    .clone();
            }
            
            Bytecode::StaLocal(idx) => {
                let frame = self.call_stack.current_frame_mut()
                    .ok_or_else(|| Error::Runtime { message: "No active frame".to_string(), span: None })?;
                frame.set_local(idx as usize, self.accumulator.clone())
                    .map_err(|e| Error::Runtime { message: e, span: None })?;
            }
            
            Bytecode::LdaGlobal(idx) => {
                let name = self.get_constant_string(idx)?;
                self.accumulator = self.globals.get(&name)
                    .cloned()
                    .unwrap_or(Value::Undefined);
            }
            
            Bytecode::StaGlobal(idx) => {
                let name = self.get_constant_string(idx)?;
                self.globals.insert(name, self.accumulator.clone());
            }
            
            // === Stack Operations ===
            Bytecode::Push => {
                if self.stack.len() >= self.max_stack_size {
                    return Err(Error::Runtime { message: "Stack overflow".to_string(), span: None });
                }
                self.stack.push(self.accumulator.clone());
            }
            
            Bytecode::Pop => {
                self.accumulator = self.stack.pop()
                    .unwrap_or(Value::Undefined);
            }
            
            // === Arithmetic Operations ===
            Bytecode::Add => {
                let left = self.stack.pop()
                    .ok_or_else(|| Error::Runtime { message: "Stack underflow in Add".to_string(), span: None })?;
                let right = self.accumulator.clone();
                
                // JavaScript addition: string concatenation or numeric addition
                self.accumulator = match (&left, &right) {
                    (Value::String(s1), Value::String(s2)) => {
                        Value::string(format!("{}{}", s1, s2))
                    }
                    (Value::String(s), _) => {
                        Value::string(format!("{}{}", s, right.to_string()))
                    }
                    (_, Value::String(s)) => {
                        Value::string(format!("{}{}", left.to_string(), s))
                    }
                    _ => {
                        Value::Number(left.to_number() + right.to_number())
                    }
                };
            }
            
            Bytecode::Sub => {
                let left = self.stack.pop()
                    .ok_or_else(|| Error::Runtime { message: "Stack underflow in Sub".to_string(), span: None })?;
                let right = self.accumulator.clone();
                self.accumulator = Value::Number(
                    left.to_number() - right.to_number()
                );
            }
            
            Bytecode::Mul => {
                let left = self.stack.pop()
                    .ok_or_else(|| Error::Runtime { message: "Stack underflow in Mul".to_string(), span: None })?;
                let right = self.accumulator.clone();
                self.accumulator = Value::Number(
                    left.to_number() * right.to_number()
                );
            }
            
            Bytecode::Div => {
                let left = self.stack.pop()
                    .ok_or_else(|| Error::Runtime { message: "Stack underflow in Div".to_string(), span: None })?;
                let right = self.accumulator.clone();
                self.accumulator = Value::Number(
                    left.to_number() / right.to_number()
                );
            }
            
            Bytecode::Mod => {
                let left = self.stack.pop()
                    .ok_or_else(|| Error::Runtime { message: "Stack underflow in Mod".to_string(), span: None })?;
                let right = self.accumulator.clone();
                self.accumulator = Value::Number(
                    left.to_number() % right.to_number()
                );
            }
            
            Bytecode::Pow => {
                let left = self.stack.pop()
                    .ok_or_else(|| Error::Runtime { message: "Stack underflow in Pow".to_string(), span: None })?;
                let right = self.accumulator.clone();
                self.accumulator = Value::Number(
                    left.to_number().powf(right.to_number())
                );
            }
            
            // === Comparison Operations ===
            Bytecode::Eq => {
                let left = self.stack.pop()
                    .ok_or_else(|| Error::Runtime { message: "Stack underflow in Eq".to_string(), span: None })?;
                let right = self.accumulator.clone();
                self.accumulator = Value::Boolean(left.loose_eq(&right));
            }
            
            Bytecode::Ne => {
                let left = self.stack.pop()
                    .ok_or_else(|| Error::Runtime { message: "Stack underflow in Ne".to_string(), span: None })?;
                let right = self.accumulator.clone();
                self.accumulator = Value::Boolean(!left.loose_eq(&right));
            }
            
            Bytecode::StrictEq => {
                let left = self.stack.pop()
                    .ok_or_else(|| Error::Runtime { message: "Stack underflow in StrictEq".to_string(), span: None })?;
                let right = self.accumulator.clone();
                self.accumulator = Value::Boolean(left.strict_eq(&right));
            }
            
            Bytecode::StrictNe => {
                let left = self.stack.pop()
                    .ok_or_else(|| Error::Runtime { message: "Stack underflow in StrictNe".to_string(), span: None })?;
                let right = self.accumulator.clone();
                self.accumulator = Value::Boolean(!left.strict_eq(&right));
            }
            
            Bytecode::Lt => {
                let left = self.stack.pop()
                    .ok_or_else(|| Error::Runtime { message: "Stack underflow in Lt".to_string(), span: None })?;
                let right = self.accumulator.clone();
                self.accumulator = Value::Boolean(
                    left.to_number() < right.to_number()
                );
            }
            
            Bytecode::Gt => {
                let left = self.stack.pop()
                    .ok_or_else(|| Error::Runtime { message: "Stack underflow in Gt".to_string(), span: None })?;
                let right = self.accumulator.clone();
                self.accumulator = Value::Boolean(
                    left.to_number() > right.to_number()
                );
            }
            
            Bytecode::Le => {
                let left = self.stack.pop()
                    .ok_or_else(|| Error::Runtime { message: "Stack underflow in Le".to_string(), span: None })?;
                let right = self.accumulator.clone();
                self.accumulator = Value::Boolean(
                    left.to_number() <= right.to_number()
                );
            }
            
            Bytecode::Ge => {
                let left = self.stack.pop()
                    .ok_or_else(|| Error::Runtime { message: "Stack underflow in Ge".to_string(), span: None })?;
                let right = self.accumulator.clone();
                self.accumulator = Value::Boolean(
                    left.to_number() >= right.to_number()
                );
            }
            
            // === Logical Operations ===
            Bytecode::LogicalAnd => {
                let right = self.stack.pop()
                    .ok_or_else(|| Error::Runtime { message: "Stack underflow in LogicalAnd".to_string(), span: None })?;
                // JavaScript short-circuit: return first falsy or last value
                if !self.accumulator.is_truthy() {
                    // accumulator is already the result (first falsy value)
                } else {
                    self.accumulator = right;
                }
            }
            
            Bytecode::LogicalOr => {
                let right = self.stack.pop()
                    .ok_or_else(|| Error::Runtime { message: "Stack underflow in LogicalOr".to_string(), span: None })?;
                // JavaScript short-circuit: return first truthy or last value
                if self.accumulator.is_truthy() {
                    // accumulator is already the result (first truthy value)
                } else {
                    self.accumulator = right;
                }
            }
            
            Bytecode::LogicalNot => {
                self.accumulator = Value::Boolean(!self.accumulator.is_truthy());
            }
            
            // === Bitwise Operations ===
            Bytecode::BitwiseAnd => {
                let right = self.stack.pop()
                    .ok_or_else(|| Error::Runtime { message: "Stack underflow in BitwiseAnd".to_string(), span: None })?;
                let left = self.accumulator.to_number() as i32;
                let right = right.to_number() as i32;
                self.accumulator = Value::Number((left & right) as f64);
            }
            
            Bytecode::BitwiseOr => {
                let right = self.stack.pop()
                    .ok_or_else(|| Error::Runtime { message: "Stack underflow in BitwiseOr".to_string(), span: None })?;
                let left = self.accumulator.to_number() as i32;
                let right = right.to_number() as i32;
                self.accumulator = Value::Number((left | right) as f64);
            }
            
            Bytecode::BitwiseXor => {
                let right = self.stack.pop()
                    .ok_or_else(|| Error::Runtime { message: "Stack underflow in BitwiseXor".to_string(), span: None })?;
                let left = self.accumulator.to_number() as i32;
                let right = right.to_number() as i32;
                self.accumulator = Value::Number((left ^ right) as f64);
            }
            
            Bytecode::BitwiseNot => {
                let value = self.accumulator.to_number() as i32;
                self.accumulator = Value::Number((!value) as f64);
            }
            
            Bytecode::LeftShift => {
                let right = self.stack.pop()
                    .ok_or_else(|| Error::Runtime { message: "Stack underflow in LeftShift".to_string(), span: None })?;
                let left = self.accumulator.to_number() as i32;
                let right = right.to_number() as u32;
                self.accumulator = Value::Number((left << (right & 31)) as f64);
            }
            
            Bytecode::RightShift => {
                let right = self.stack.pop()
                    .ok_or_else(|| Error::Runtime { message: "Stack underflow in RightShift".to_string(), span: None })?;
                let left = self.accumulator.to_number() as i32;
                let right = right.to_number() as u32;
                self.accumulator = Value::Number((left >> (right & 31)) as f64);
            }
            
            Bytecode::UnsignedRightShift => {
                let right = self.stack.pop()
                    .ok_or_else(|| Error::Runtime { message: "Stack underflow in UnsignedRightShift".to_string(), span: None })?;
                let left = self.accumulator.to_number() as u32;
                let right = right.to_number() as u32;
                self.accumulator = Value::Number((left >> (right & 31)) as f64);
            }
            
            // === Unary Operations ===
            Bytecode::UnaryPlus => {
                self.accumulator = Value::Number(self.accumulator.to_number());
            }
            
            Bytecode::UnaryMinus => {
                self.accumulator = Value::Number(-self.accumulator.to_number());
            }
            
            Bytecode::TypeOf => {
                self.accumulator = Value::string(self.accumulator.type_of());
            }
            
            // === Control Flow ===
            Bytecode::Jump(offset) => {
                let frame = self.call_stack.current_frame_mut()
                    .ok_or_else(|| Error::Runtime { message: "No active frame".to_string(), span: None })?;
                frame.jump_relative(offset);
            }
            
            Bytecode::JumpIfFalse(offset) => {
                if !self.accumulator.is_truthy() {
                    let frame = self.call_stack.current_frame_mut()
                        .ok_or_else(|| Error::Runtime { message: "No active frame".to_string(), span: None })?;
                    frame.jump_relative(offset);
                }
            }
            
            Bytecode::JumpIfTrue(offset) => {
                if self.accumulator.is_truthy() {
                    let frame = self.call_stack.current_frame_mut()
                        .ok_or_else(|| Error::Runtime { message: "No active frame".to_string(), span: None })?;
                    frame.jump_relative(offset);
                }
            }
            
            Bytecode::JumpIfNullish(offset) => {
                if matches!(self.accumulator, Value::Null | Value::Undefined) {
                    let frame = self.call_stack.current_frame_mut()
                        .ok_or_else(|| Error::Runtime { message: "No active frame".to_string(), span: None })?;
                    frame.jump_relative(offset);
                }
            }
            
            // === Function Operations ===
            Bytecode::Return => {
                // Value to return is in accumulator
                self.handle_return()?;
            }
            
            Bytecode::ReturnUndefined => {
                self.accumulator = Value::Undefined;
                self.handle_return()?;
            }
            
            Bytecode::Call(arg_count) => {
                // For now, we only support built-in function calls
                // The function name should be a string in the accumulator
                if let Value::String(name) = &self.accumulator {
                    if let Some(builtin_fn) = self.builtins.get(&**name) {
                        // Pop arguments from stack
                        let mut args = Vec::with_capacity(arg_count as usize);
                        for _ in 0..arg_count {
                            args.push(self.stack.pop()
                                .unwrap_or(Value::Undefined));
                        }
                        args.reverse(); // Arguments were pushed in order
                        
                        // Call the built-in function
                        self.accumulator = builtin_fn(&args);
                    } else {
                        return Err(Error::Runtime { message: format!("Unknown function: {}", name), span: None });
                    }
                } else {
                    return Err(Error::Runtime { message: "Can only call functions".to_string(), span: None });
                }
            }
            
            // === Object Operations (stubs for now) ===
            Bytecode::CreateObject => {
                // Create an empty object
                use std::rc::Rc;
                use super::value::ObjectData;
                self.accumulator = Value::Object(Rc::new(ObjectData {
                    properties: HashMap::new(),
                }));
            }
            
            Bytecode::CreateArray(_idx) => {
                // Create an empty array (represented as object for now)
                use std::rc::Rc;
                use super::value::ObjectData;
                self.accumulator = Value::Object(Rc::new(ObjectData {
                    properties: HashMap::new(),
                }));
            }
            
            Bytecode::CreateClosure(_idx) => {
                // Create a closure (not fully implemented yet)
                self.accumulator = Value::Undefined;
            }
            
            Bytecode::LdaNamed(_idx) => {
                // Load named property (not fully implemented)
                self.accumulator = Value::Undefined;
            }
            
            Bytecode::StaNamed(_idx) => {
                // Store named property (not fully implemented)
                // For now, just consume the value
            }
            
            Bytecode::LdaKeyed => {
                // Load keyed property (not fully implemented)
                self.accumulator = Value::Undefined;
            }
            
            Bytecode::StaKeyed => {
                // Store keyed property (not fully implemented)
                // For now, just consume the value
            }
            
            // === Debugging Operations ===
            Bytecode::Nop => {
                // No operation - do nothing
            }
            
            Bytecode::Debugger => {
                // Debugger breakpoint - for now just continue
                if self.debug {
                    if let Some(frame) = self.call_stack.current_frame() {
                        println!("[DEBUGGER] Breakpoint hit at ip: {}", frame.ip());
                    }
                }
            }
        }
        
        Ok(())
    }
    
    /// Handle function return
    fn handle_return(&mut self) -> Result<bool> {
        // Pop current frame
        let current_frame = self.call_stack.pop()
            .ok_or_else(|| Error::Runtime { message: "No frame to return from".to_string(), span: None })?;
        
        // Check if we're returning from main
        if self.call_stack.is_empty() {
            return Ok(false); // Signal to stop execution
        }
        
        // Restore caller's context (if returning from a function call)
        if let Some(return_address) = current_frame.return_address {
            let frame = self.call_stack.current_frame_mut()
                .ok_or_else(|| Error::Runtime { message: "No caller frame".to_string(), span: None })?;
            frame.ip = return_address;
        }
        
        Ok(true) // Continue execution
    }
    
    /// Get a constant value from the current function's constant pool
    fn get_constant(&self, idx: u16) -> Result<&ConstantValue> {
        let frame = self.call_stack.current_frame()
            .ok_or_else(|| Error::Runtime { message: "No active frame".to_string(), span: None })?;
        frame.function.constants.get(idx)
            .ok_or_else(|| Error::Runtime { message: format!("Invalid constant index: {}", idx), span: None })
    }
    
    /// Get a string constant from the pool
    fn get_constant_string(&self, idx: u16) -> Result<String> {
        match self.get_constant(idx)? {
            ConstantValue::String(s) => Ok(s.clone()),
            _ => Err(Error::Runtime { message: "Expected string constant".to_string(), span: None }),
        }
    }
    
    /// Convert a constant pool value to a VM value
    fn constant_to_value(&self, constant: &ConstantValue) -> Result<Value> {
        Ok(match constant {
            ConstantValue::Number(n) => Value::Number(n.0),
            ConstantValue::String(s) => Value::string(s.clone()),
            ConstantValue::Boolean(b) => Value::Boolean(*b),
            ConstantValue::Null => Value::Null,
            ConstantValue::Undefined => Value::Undefined,
            _ => Value::Undefined, // Other constant types not yet supported
        })
    }
    
    /// Debug helper to print current instruction
    fn debug_instruction(&self, instruction: &Bytecode, ip: usize) {
        println!("[{:04}] {:?} | acc: {:?} | stack: {:?}", 
                 ip, instruction, self.accumulator, self.stack);
    }
}

impl Default for VM {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bytecode::{Compiler};
    use crate::parser::{Parser};
    use crate::lexer::{Lexer};
    
    fn compile_and_run(source: &str) -> Result<Value> {
        // Tokenize
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize()?;
        
        // Parse
        let mut parser = Parser::new(tokens);
        let ast = parser.parse()?;
        
        // Compile
        let compiler = Compiler::new_main(source);
        let bytecode = compiler.compile(&ast)?;
        
        // Execute
        let mut vm = VM::new();
        vm.execute(bytecode)
    }
    
    #[test]
    fn test_arithmetic() {
        assert_eq!(compile_and_run("2 + 3").unwrap(), Value::Number(5.0));
        assert_eq!(compile_and_run("10 - 4").unwrap(), Value::Number(6.0));
        assert_eq!(compile_and_run("3 * 7").unwrap(), Value::Number(21.0));
        assert_eq!(compile_and_run("15 / 3").unwrap(), Value::Number(5.0));
        assert_eq!(compile_and_run("10 % 3").unwrap(), Value::Number(1.0));
    }
    
    #[test]
    fn test_variables() {
        assert_eq!(compile_and_run("let x = 42; x").unwrap(), Value::Number(42.0));
        assert_eq!(compile_and_run("let a = 10; let b = 20; a + b").unwrap(), Value::Number(30.0));
    }
    
    #[test]
    fn test_comparisons() {
        assert_eq!(compile_and_run("5 > 3").unwrap(), Value::Boolean(true));
        assert_eq!(compile_and_run("2 < 1").unwrap(), Value::Boolean(false));
        assert_eq!(compile_and_run("10 == 10").unwrap(), Value::Boolean(true));
        assert_eq!(compile_and_run("5 != 5").unwrap(), Value::Boolean(false));
    }
    
    #[test]
    fn test_logical() {
        assert_eq!(compile_and_run("!true").unwrap(), Value::Boolean(false));
        assert_eq!(compile_and_run("!false").unwrap(), Value::Boolean(true));
        assert_eq!(compile_and_run("true && false").unwrap(), Value::Boolean(false));
        assert_eq!(compile_and_run("true || false").unwrap(), Value::Boolean(true));
    }
}