//! AST to Bytecode Compiler
//!
//! This module implements the compiler that translates parsed JavaScript AST
//! into bytecode instructions for the stack-based virtual machine.
//!
//! # Architecture
//!
//! - **Compiler**: Main compilation context with scope management
//! - **Scope**: Tracks local variables and their indices  
//! - **JumpPatch**: Forward jump resolution for control flow
//! - **LoopContext**: Break/continue handling in loops

use std::collections::HashMap;
use crate::error::{Error, Result, Span};
use crate::parser::ast::{Program, Stmt, Expr, BinaryOp, UnaryOp, Literal};
use super::{BytecodeFunction, Bytecode, ConstIndex, LocalIndex};

/// Local variable slot assignment
#[derive(Debug, Clone)]
pub struct LocalSlot {
    pub name: String,
    pub index: LocalIndex,
    pub is_parameter: bool,
    pub span: Span,
}

/// Scope for variable resolution and local slot management
#[derive(Debug, Clone)]
pub struct Scope {
    /// Variables defined in this scope
    pub locals: HashMap<String, LocalSlot>,
    /// Parent scope index (for nested scopes)
    pub parent: Option<usize>,
    /// Scope type (function, block, etc.)
    pub scope_type: ScopeType,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ScopeType {
    Function,
    Block,
    Loop,
}

/// Jump patch site for forward jumps
#[derive(Debug, Clone)]
pub struct JumpPatch {
    /// Instruction offset to patch
    pub instruction_offset: usize,
    /// Target label identifier
    pub target_label: String,
}

/// Loop context for break/continue handling
#[derive(Debug, Clone)]
pub struct LoopContext {
    /// Label for continue statements (jump to loop condition)
    pub continue_label: String,
    /// Patches for break statements (jump to loop end)
    pub break_patches: Vec<usize>,
    /// Patches for continue statements
    pub continue_patches: Vec<usize>,
}

/// Main AST to Bytecode compiler
pub struct Compiler {
    /// The function being compiled
    function: BytecodeFunction,
    /// Stack of scopes (for nested scopes)
    scopes: Vec<Scope>,
    /// Counter for next local variable index
    next_local_index: LocalIndex,
}

impl Compiler {
    /// Create a new compiler for a main program
    pub fn new_main(source: &str) -> Self {
        let mut function = BytecodeFunction::new_main();
        function.debug_info.set_source_code(source.to_string());
        
        Self {
            function,
            scopes: vec![Scope {
                locals: HashMap::new(),
                parent: None,
                scope_type: ScopeType::Function,
            }],
            next_local_index: 0,
        }
    }
    
    /// Create a new compiler for a function
    pub fn new_function(name: String, params: &[String], source: &str) -> Self {
        let arity = params.len() as u8;
        let mut function = BytecodeFunction::new(name, arity, 0, 0);
        function.debug_info.set_source_code(source.to_string());
        
        // Create function scope with parameters
        let mut scope = Scope {
            locals: HashMap::new(),
            parent: None,
            scope_type: ScopeType::Function,
        };
        
        let mut next_local = 0;
        // Parameters are local variables 0..arity
        for (i, param) in params.iter().enumerate() {
            scope.locals.insert(param.clone(), LocalSlot {
                name: param.clone(),
                index: i as LocalIndex,
                is_parameter: true,
                span: Span::new(0, 0, 1, 1), // TODO: Get real span from AST
            });
            next_local = i as LocalIndex + 1;
        }
        
        Self {
            function,
            scopes: vec![scope],
            next_local_index: next_local,
        }
    }
    
    
    /// Enter a new scope
    fn enter_scope(&mut self, scope_type: ScopeType) {
        let parent_index = self.scopes.len() - 1;
        self.scopes.push(Scope {
            locals: HashMap::new(),
            parent: Some(parent_index),
            scope_type,
        });
    }
    
    /// Exit the current scope
    fn exit_scope(&mut self) {
        if self.scopes.len() > 1 {
            self.scopes.pop();
        }
    }
    
    /// Resolve a variable name to a local slot
    fn resolve_variable(&self, name: &str) -> Option<&LocalSlot> {
        // Search from current scope up the scope chain
        for scope in self.scopes.iter().rev() {
            if let Some(slot) = scope.locals.get(name) {
                return Some(slot);
            }
        }
        None
    }
    
    /// Declare a new local variable in the current scope
    fn declare_local(&mut self, name: String, span: Span) -> Result<LocalIndex> {
        let current_scope = self.scopes.last_mut()
            .ok_or_else(|| Error::runtime("No current scope for variable declaration", Some(span)))?;
        
        // Check if variable already exists in current scope
        if current_scope.locals.contains_key(&name) {
            return Err(Error::runtime(
                format!("Variable '{}' already declared in this scope", name),
                Some(span)
            ));
        }
        
        let index = self.next_local_index;
        current_scope.locals.insert(name.clone(), LocalSlot {
            name: name.clone(),
            index,
            is_parameter: false,
            span,
        });
        
        self.next_local_index += 1;
        self.function.locals_count = self.next_local_index;
        
        Ok(index)
    }
    
    /// Add an instruction to the function
    fn emit(&mut self, instruction: Bytecode) {
        self.function.add_instruction(instruction);
    }
    
    /// Add an instruction with span information
    fn emit_with_span(&mut self, instruction: Bytecode, span: Span) {
        self.function.add_instruction_with_span(instruction, span);
    }
    
    
    /// Add a constant to the function's constant pool
    fn add_constant_number(&mut self, value: f64) -> ConstIndex {
        self.function.constants.add_number(value)
    }
    
    /// Add a string constant to the function's constant pool
    fn add_constant_string(&mut self, value: String) -> ConstIndex {
        self.function.constants.add_string(value)
    }
    
    /// Add a property name constant to the function's constant pool
    fn add_constant_property_name(&mut self, name: String) -> ConstIndex {
        self.function.constants.add_property_name(name)
    }
    
    
    /// Compile a program to bytecode
    pub fn compile(mut self, program: &Program) -> Result<BytecodeFunction> {
        // Compile all statements in the program
        let num_statements = program.statements.len();
        for (i, stmt) in program.statements.iter().enumerate() {
            let is_last = i == num_statements - 1;
            
            // For the last statement, if it's an expression, keep its value as return value
            if is_last && matches!(stmt, Stmt::Expression(_)) {
                // Compile expression without popping the result
                if let Stmt::Expression(expr) = stmt {
                    self.compile_expression(expr)?;
                    // Don't pop - leave value in accumulator for implicit return
                }
            } else {
                self.compile_statement(stmt)?;
            }
        }
        
        // Ensure program ends with a return
        if let Some(last_instruction) = self.function.bytecode.last() {
            if !matches!(last_instruction, Bytecode::Return | Bytecode::ReturnUndefined) {
                // If the last statement was an expression, return its value
                // Otherwise return undefined
                self.emit(Bytecode::Return);
            }
        } else {
            self.emit(Bytecode::ReturnUndefined);
        }
        
        // Calculate final stack size
        self.function.calculate_stack_size();
        
        Ok(self.function)
    }
    
    /// Compile a statement to bytecode
    fn compile_statement(&mut self, stmt: &Stmt) -> Result<()> {
        match stmt {
            Stmt::Expression(expr) => {
                self.compile_expression(expr)?;
                // Pop the result since it's not used
                self.emit(Bytecode::Pop);
                Ok(())
            }
            
            Stmt::VarDecl { name, init, span } => {
                // Declare the variable in the current scope
                let local_index = self.declare_local(name.clone(), *span)?;
                
                // If there's an initializer, compile it and store the result
                if let Some(init_expr) = init {
                    self.compile_expression(init_expr)?;
                } else {
                    // Default initialize with undefined
                    let undefined_const = self.function.constants.add_undefined();
                    self.emit(Bytecode::LdaConst(undefined_const));
                }
                
                // Store in the local variable
                self.emit_with_span(Bytecode::StaLocal(local_index), *span);
                Ok(())
            }
            
            Stmt::Return { value, span } => {
                if let Some(expr) = value {
                    self.compile_expression(expr)?;
                    self.emit_with_span(Bytecode::Return, *span);
                } else {
                    self.emit_with_span(Bytecode::ReturnUndefined, *span);
                }
                Ok(())
            }
            
            Stmt::Block { statements, .. } => {
                self.enter_scope(ScopeType::Block);
                for stmt in statements {
                    self.compile_statement(stmt)?;
                }
                self.exit_scope();
                Ok(())
            }
            
            // TODO: Implement other statements in next tasks
            _ => {
                Ok(()) // Placeholder for now
            }
        }
    }
    
    /// Compile an expression to bytecode (result left in accumulator)
    fn compile_expression(&mut self, expr: &Expr) -> Result<()> {
        match expr {
            Expr::Literal(literal) => {
                self.compile_literal(literal)
            }
            
            Expr::Identifier { name, span } => {
                // Try to resolve as local variable first
                if let Some(local) = self.resolve_variable(name) {
                    self.emit_with_span(Bytecode::LdaLocal(local.index), *span);
                } else {
                    // Global variable access
                    let name_const = self.add_constant_property_name(name.clone());
                    self.emit_with_span(Bytecode::LdaGlobal(name_const), *span);
                }
                Ok(())
            }
            
            Expr::Binary { op, left, right, span } => {
                self.compile_binary_operation(op, left, right, *span)
            }
            
            Expr::Unary { op, operand, span } => {
                self.compile_unary_operation(op, operand, *span)
            }
            
            Expr::Assignment { left, right, span } => {
                self.compile_assignment(left, right, *span)
            }
            
            Expr::Call { callee, args, span } => {
                self.compile_function_call(callee, args, *span)
            }
            
            Expr::Member { object, property, computed, span } => {
                self.compile_member_access(object, property, *computed, *span)
            }
            
            // TODO: Implement other expressions in next tasks
            _ => {
                // Placeholder: load undefined for unimplemented expressions
                let undefined_const = self.function.constants.add_undefined();
                self.emit(Bytecode::LdaConst(undefined_const));
                Ok(())
            }
        }
    }
    
    /// Compile a literal value
    fn compile_literal(&mut self, literal: &Literal) -> Result<()> {
        match literal {
            Literal::Number(value) => {
                let const_index = self.add_constant_number(*value);
                self.emit(Bytecode::LdaConst(const_index));
            }
            Literal::String(value) => {
                let const_index = self.add_constant_string(value.clone());
                self.emit(Bytecode::LdaConst(const_index));
            }
            Literal::Boolean(value) => {
                let const_index = self.function.constants.add_boolean(*value);
                self.emit(Bytecode::LdaConst(const_index));
            }
            Literal::Null => {
                let const_index = self.function.constants.add_null();
                self.emit(Bytecode::LdaConst(const_index));
            }
            Literal::Undefined => {
                let const_index = self.function.constants.add_undefined();
                self.emit(Bytecode::LdaConst(const_index));
            }
        }
        Ok(())
    }
    
    /// Compile binary operations (e.g., +, -, *, /, ==, <, etc.)
    fn compile_binary_operation(&mut self, op: &BinaryOp, left: &Expr, right: &Expr, _span: Span) -> Result<()> {
        // Compile left operand (result in accumulator)
        self.compile_expression(left)?;
        
        // Push left operand to stack
        self.emit(Bytecode::Push);
        
        // Compile right operand (result in accumulator)  
        self.compile_expression(right)?;
        
        // Emit the appropriate bytecode instruction
        // The operation will be: stack_top op accumulator -> accumulator
        match op {
            BinaryOp::Add => self.emit(Bytecode::Add),
            BinaryOp::Subtract => self.emit(Bytecode::Sub),
            BinaryOp::Multiply => self.emit(Bytecode::Mul),
            BinaryOp::Divide => self.emit(Bytecode::Div),
            BinaryOp::Modulo => self.emit(Bytecode::Mod),
            BinaryOp::Power => self.emit(Bytecode::Pow),
            
            BinaryOp::Equal => self.emit(Bytecode::Eq),
            BinaryOp::NotEqual => self.emit(Bytecode::Ne),
            BinaryOp::StrictEqual => self.emit(Bytecode::StrictEq),
            BinaryOp::StrictNotEqual => self.emit(Bytecode::StrictNe),
            BinaryOp::Less => self.emit(Bytecode::Lt),
            BinaryOp::Greater => self.emit(Bytecode::Gt),
            BinaryOp::LessEqual => self.emit(Bytecode::Le),
            BinaryOp::GreaterEqual => self.emit(Bytecode::Ge),
            
            BinaryOp::LogicalAnd => self.emit(Bytecode::LogicalAnd),
            BinaryOp::LogicalOr => self.emit(Bytecode::LogicalOr),
            
            BinaryOp::BitwiseAnd => self.emit(Bytecode::BitwiseAnd),
            BinaryOp::BitwiseOr => self.emit(Bytecode::BitwiseOr),
            BinaryOp::BitwiseXor => self.emit(Bytecode::BitwiseXor),
            BinaryOp::LeftShift => self.emit(Bytecode::LeftShift),
            BinaryOp::RightShift => self.emit(Bytecode::RightShift),
            BinaryOp::UnsignedRightShift => self.emit(Bytecode::UnsignedRightShift),
            
            // Not implemented yet
            BinaryOp::InstanceOf | BinaryOp::In => {
                return Err(Error::runtime(format!("Operator {:?} not yet implemented", op), None));
            }
        }
        
        Ok(())
    }
    
    /// Compile unary operations (e.g., !, -, +, typeof)
    fn compile_unary_operation(&mut self, op: &UnaryOp, operand: &Expr, _span: Span) -> Result<()> {
        // Compile operand (result in accumulator)
        self.compile_expression(operand)?;
        
        // Emit the appropriate unary instruction
        match op {
            UnaryOp::LogicalNot => self.emit(Bytecode::LogicalNot),
            UnaryOp::Minus => self.emit(Bytecode::UnaryMinus),
            UnaryOp::Plus => self.emit(Bytecode::UnaryPlus),
            UnaryOp::BitwiseNot => self.emit(Bytecode::BitwiseNot),
            UnaryOp::TypeOf => self.emit(Bytecode::TypeOf),
            
            // Not implemented yet
            UnaryOp::Void | UnaryOp::Delete => {
                return Err(Error::runtime(format!("Operator {:?} not yet implemented", op), None));
            }
        }
        
        Ok(())
    }
    
    /// Compile assignment expressions
    fn compile_assignment(&mut self, left: &Expr, right: &Expr, span: Span) -> Result<()> {
        // Compile the right-hand side (value to assign)
        self.compile_expression(right)?;
        
        // Handle different assignment targets
        match left {
            Expr::Identifier { name, .. } => {
                // Simple variable assignment
                if let Some(local) = self.resolve_variable(name) {
                    // Local variable assignment
                    self.emit_with_span(Bytecode::StaLocal(local.index), span);
                } else {
                    // Global variable assignment  
                    let name_const = self.add_constant_property_name(name.clone());
                    self.emit_with_span(Bytecode::StaGlobal(name_const), span);
                }
            }
            
            Expr::Member { .. } => {
                // Property assignment: obj.prop = value or obj[key] = value
                // TODO: Implement in member access section
                return Err(Error::runtime("Member assignment not yet implemented", Some(span)));
            }
            
            _ => {
                return Err(Error::runtime("Invalid assignment target", Some(span)));
            }
        }
        
        Ok(())
    }
    
    /// Compile function calls
    fn compile_function_call(&mut self, _callee: &Expr, _args: &[Expr], span: Span) -> Result<()> {
        // TODO: Implement in function compilation task
        Err(Error::runtime("Function calls not yet implemented", Some(span)))
    }
    
    /// Compile member access (obj.prop or obj[key])
    fn compile_member_access(&mut self, _object: &Expr, _property: &Expr, _computed: bool, span: Span) -> Result<()> {
        // TODO: Implement in next tasks
        Err(Error::runtime("Member access not yet implemented", Some(span)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_compiler_creation() {
        let compiler = Compiler::new_main("test source");
        assert_eq!(compiler.scopes.len(), 1);
        assert_eq!(compiler.next_local_index, 0);
        assert_eq!(compiler.function.name, "<main>");
    }
    
    #[test]
    fn test_function_compiler_creation() {
        let params = vec!["a".to_string(), "b".to_string()];
        let compiler = Compiler::new_function("test".to_string(), &params, "test source");
        
        assert_eq!(compiler.function.name, "test");
        assert_eq!(compiler.function.arity, 2);
        assert_eq!(compiler.next_local_index, 2); // Parameters take slots 0, 1
        
        // Check that parameters are in scope
        assert!(compiler.resolve_variable("a").is_some());
        assert!(compiler.resolve_variable("b").is_some());
    }
    
    #[test]
    fn test_scope_management() {
        let mut compiler = Compiler::new_main("test");
        
        // Declare variable in main scope
        let span = Span::new(0, 0, 1, 1);
        let index1 = compiler.declare_local("x".to_string(), span).unwrap();
        assert_eq!(index1, 0);
        assert!(compiler.resolve_variable("x").is_some());
        
        // Enter block scope
        compiler.enter_scope(ScopeType::Block);
        
        // Declare variable in block scope
        let index2 = compiler.declare_local("y".to_string(), span).unwrap();
        assert_eq!(index2, 1);
        assert!(compiler.resolve_variable("y").is_some());
        assert!(compiler.resolve_variable("x").is_some()); // Still accessible
        
        // Exit block scope
        compiler.exit_scope();
        assert!(compiler.resolve_variable("x").is_some());
        assert!(compiler.resolve_variable("y").is_none()); // No longer accessible
    }
    
    #[test]
    fn test_constant_management() {
        let mut compiler = Compiler::new_main("test");
        
        let _num_const = compiler.add_constant_number(42.0);
        let _str_const = compiler.add_constant_string("hello".to_string());
        let _prop_const = compiler.add_constant_property_name("length".to_string());
        
        assert_eq!(compiler.function.constants.len(), 3);
        
        // Test that constants are properly stored
        // (We would need to expose constant pool access for this)
    }
    
}