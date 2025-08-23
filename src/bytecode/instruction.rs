//! Bytecode instruction definitions
//!
//! This module defines all bytecode instructions for the stack-based VM with accumulator.
//! The instruction set is designed to efficiently represent JavaScript operations
//! while maintaining simplicity for the interpreter.

use std::fmt;

/// Type alias for constant pool indices (supports up to 65536 constants)
pub type ConstIndex = u16;

/// Type alias for local variable indices (supports up to 65536 locals)
pub type LocalIndex = u16;

/// Type alias for jump offsets (±32KB jump range)
pub type JumpOffset = i16;

/// Type alias for argument count in function calls
pub type ArgCount = u8;

/// Bytecode instructions for stack-based VM with accumulator
#[derive(Debug, Clone, PartialEq)]
pub enum Bytecode {
    // === Load/Store Operations ===
    /// Load constant into accumulator: acc = constants[index]
    LdaConst(ConstIndex),
    
    /// Load local variable into accumulator: acc = locals[index]
    LdaLocal(LocalIndex),
    
    /// Store accumulator to local variable: locals[index] = acc
    StaLocal(LocalIndex),
    
    /// Load global variable into accumulator: acc = globals[name]
    LdaGlobal(ConstIndex),
    
    /// Store accumulator to global variable: globals[name] = acc
    StaGlobal(ConstIndex),

    // === Stack Operations ===
    /// Push accumulator onto stack: push(acc)
    Push,
    
    /// Pop from stack into accumulator: acc = pop()
    Pop,

    // === Arithmetic Operations ===
    /// Addition: acc = acc + pop()
    Add,
    
    /// Subtraction: acc = acc - pop()
    Sub,
    
    /// Multiplication: acc = acc * pop()
    Mul,
    
    /// Division: acc = acc / pop()
    Div,
    
    /// Modulo: acc = acc % pop()
    Mod,
    
    /// Exponentiation: acc = acc ** pop()
    Pow,

    // === Comparison Operations ===
    /// Equality: acc = (acc == pop())
    Eq,
    
    /// Inequality: acc = (acc != pop())
    Ne,
    
    /// Strict equality: acc = (acc === pop())
    StrictEq,
    
    /// Strict inequality: acc = (acc !== pop())
    StrictNe,
    
    /// Less than: acc = (acc < pop())
    Lt,
    
    /// Greater than: acc = (acc > pop())
    Gt,
    
    /// Less than or equal: acc = (acc <= pop())
    Le,
    
    /// Greater than or equal: acc = (acc >= pop())
    Ge,

    // === Logical Operations ===
    /// Logical AND: acc = acc && pop()
    LogicalAnd,
    
    /// Logical OR: acc = acc || pop()
    LogicalOr,
    
    /// Logical NOT: acc = !acc
    LogicalNot,

    // === Bitwise Operations ===
    /// Bitwise AND: acc = acc & pop()
    BitwiseAnd,
    
    /// Bitwise OR: acc = acc | pop()
    BitwiseOr,
    
    /// Bitwise XOR: acc = acc ^ pop()
    BitwiseXor,
    
    /// Left shift: acc = acc << pop()
    LeftShift,
    
    /// Right shift: acc = acc >> pop()
    RightShift,
    
    /// Unsigned right shift: acc = acc >>> pop()
    UnsignedRightShift,
    
    /// Bitwise NOT: acc = ~acc
    BitwiseNot,

    // === Unary Operations ===
    /// Unary plus: acc = +acc
    UnaryPlus,
    
    /// Unary minus: acc = -acc
    UnaryMinus,
    
    /// Typeof operation: acc = typeof acc
    TypeOf,

    // === Property Access ===
    /// Load named property: acc = pop()[constants[index]]
    LdaNamed(ConstIndex),
    
    /// Store named property: pop()[constants[index]] = acc
    StaNamed(ConstIndex),
    
    /// Load computed property: acc = pop()[pop()]
    LdaKeyed,
    
    /// Store computed property: pop()[pop()] = acc
    StaKeyed,

    // === Function Operations ===
    /// Function call: acc = call(acc, argc_args_from_stack)
    Call(ArgCount),
    
    /// Return from function: return acc
    Return,
    
    /// Return undefined from function
    ReturnUndefined,

    // === Control Flow ===
    /// Unconditional jump: pc += offset
    Jump(JumpOffset),
    
    /// Jump if accumulator is false: if (!acc) pc += offset
    JumpIfFalse(JumpOffset),
    
    /// Jump if accumulator is true: if (acc) pc += offset
    JumpIfTrue(JumpOffset),
    
    /// Jump if accumulator is null or undefined: if (acc == null) pc += offset
    JumpIfNullish(JumpOffset),

    // === Object Creation ===
    /// Create empty object: acc = {}
    CreateObject,
    
    /// Create array with initial size: acc = new Array(size)
    CreateArray(ConstIndex),
    
    /// Create function closure: acc = function(params, bytecode)
    CreateClosure(ConstIndex),

    // === Debugging and Utilities ===
    /// No operation (for padding and debugging)
    Nop,
    
    /// Debugger breakpoint
    Debugger,
}

impl fmt::Display for Bytecode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            // Load/Store
            Bytecode::LdaConst(idx) => write!(f, "LdaConst #{}", idx),
            Bytecode::LdaLocal(idx) => write!(f, "LdaLocal {}", idx),
            Bytecode::StaLocal(idx) => write!(f, "StaLocal {}", idx),
            Bytecode::LdaGlobal(idx) => write!(f, "LdaGlobal #{}", idx),
            Bytecode::StaGlobal(idx) => write!(f, "StaGlobal #{}", idx),
            
            // Stack
            Bytecode::Push => write!(f, "Push"),
            Bytecode::Pop => write!(f, "Pop"),
            
            // Arithmetic
            Bytecode::Add => write!(f, "Add"),
            Bytecode::Sub => write!(f, "Sub"),
            Bytecode::Mul => write!(f, "Mul"),
            Bytecode::Div => write!(f, "Div"),
            Bytecode::Mod => write!(f, "Mod"),
            Bytecode::Pow => write!(f, "Pow"),
            
            // Comparison
            Bytecode::Eq => write!(f, "Eq"),
            Bytecode::Ne => write!(f, "Ne"),
            Bytecode::StrictEq => write!(f, "StrictEq"),
            Bytecode::StrictNe => write!(f, "StrictNe"),
            Bytecode::Lt => write!(f, "Lt"),
            Bytecode::Gt => write!(f, "Gt"),
            Bytecode::Le => write!(f, "Le"),
            Bytecode::Ge => write!(f, "Ge"),
            
            // Logical
            Bytecode::LogicalAnd => write!(f, "LogicalAnd"),
            Bytecode::LogicalOr => write!(f, "LogicalOr"),
            Bytecode::LogicalNot => write!(f, "LogicalNot"),
            
            // Bitwise
            Bytecode::BitwiseAnd => write!(f, "BitwiseAnd"),
            Bytecode::BitwiseOr => write!(f, "BitwiseOr"),
            Bytecode::BitwiseXor => write!(f, "BitwiseXor"),
            Bytecode::LeftShift => write!(f, "LeftShift"),
            Bytecode::RightShift => write!(f, "RightShift"),
            Bytecode::UnsignedRightShift => write!(f, "UnsignedRightShift"),
            Bytecode::BitwiseNot => write!(f, "BitwiseNot"),
            
            // Unary
            Bytecode::UnaryPlus => write!(f, "UnaryPlus"),
            Bytecode::UnaryMinus => write!(f, "UnaryMinus"),
            Bytecode::TypeOf => write!(f, "TypeOf"),
            
            // Property access
            Bytecode::LdaNamed(idx) => write!(f, "LdaNamed #{}", idx),
            Bytecode::StaNamed(idx) => write!(f, "StaNamed #{}", idx),
            Bytecode::LdaKeyed => write!(f, "LdaKeyed"),
            Bytecode::StaKeyed => write!(f, "StaKeyed"),
            
            // Functions
            Bytecode::Call(argc) => write!(f, "Call {}", argc),
            Bytecode::Return => write!(f, "Return"),
            Bytecode::ReturnUndefined => write!(f, "ReturnUndefined"),
            
            // Control flow
            Bytecode::Jump(offset) => write!(f, "Jump {}", offset),
            Bytecode::JumpIfFalse(offset) => write!(f, "JumpIfFalse {}", offset),
            Bytecode::JumpIfTrue(offset) => write!(f, "JumpIfTrue {}", offset),
            Bytecode::JumpIfNullish(offset) => write!(f, "JumpIfNullish {}", offset),
            
            // Object creation
            Bytecode::CreateObject => write!(f, "CreateObject"),
            Bytecode::CreateArray(size) => write!(f, "CreateArray #{}", size),
            Bytecode::CreateClosure(idx) => write!(f, "CreateClosure #{}", idx),
            
            // Debug
            Bytecode::Nop => write!(f, "Nop"),
            Bytecode::Debugger => write!(f, "Debugger"),
        }
    }
}

impl Bytecode {
    /// Returns true if this instruction can change the program counter (branches/calls)
    pub fn is_control_flow(&self) -> bool {
        matches!(self,
            Bytecode::Jump(_) |
            Bytecode::JumpIfFalse(_) |
            Bytecode::JumpIfTrue(_) |
            Bytecode::JumpIfNullish(_) |
            Bytecode::Call(_) |
            Bytecode::Return |
            Bytecode::ReturnUndefined
        )
    }
    
    /// Returns true if this instruction modifies the accumulator
    pub fn modifies_accumulator(&self) -> bool {
        !matches!(self,
            Bytecode::StaLocal(_) |
            Bytecode::StaGlobal(_) |
            Bytecode::Push |
            Bytecode::StaNamed(_) |
            Bytecode::StaKeyed |
            Bytecode::Nop |
            Bytecode::Debugger
        )
    }
    
    /// Returns the number of stack items this instruction pops (not including accumulator)
    pub fn stack_pop_count(&self) -> usize {
        match self {
            Bytecode::Add | Bytecode::Sub | Bytecode::Mul | Bytecode::Div | Bytecode::Mod |
            Bytecode::Pow | Bytecode::Eq | Bytecode::Ne | Bytecode::StrictEq | Bytecode::StrictNe |
            Bytecode::Lt | Bytecode::Gt | Bytecode::Le | Bytecode::Ge | Bytecode::LogicalAnd |
            Bytecode::LogicalOr | Bytecode::BitwiseAnd | Bytecode::BitwiseOr | Bytecode::BitwiseXor |
            Bytecode::LeftShift | Bytecode::RightShift | Bytecode::UnsignedRightShift => 1,
            
            Bytecode::LdaNamed(_) | Bytecode::LdaKeyed => 1,
            Bytecode::StaNamed(_) => 1,
            Bytecode::StaKeyed => 2,
            
            Bytecode::Call(argc) => *argc as usize + 1, // args + function
            
            _ => 0,
        }
    }
    
    /// Returns the number of stack items this instruction pushes
    pub fn stack_push_count(&self) -> usize {
        match self {
            Bytecode::Push => 1,
            _ => 0,
        }
    }
}