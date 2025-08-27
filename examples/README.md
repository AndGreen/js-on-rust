# JavaScript Examples

This directory contains comprehensive example JavaScript programs that demonstrate all capabilities of the V8-like JavaScript engine implementation.

## 📚 Complete Feature Coverage Examples

### 🔤 **01_literals.js** - Literals and Data Types
Demonstrates all supported literal types and basic operations:
- **Numbers**: integers, decimals, scientific notation
- **Strings**: single/double quotes, concatenation
- **Booleans**: true/false values and operations
- **Special values**: null, undefined
- **Type comparisons**: strict vs loose equality

**Expected Output**: `true` (strict equality check result)

### ⚙️ **02_operators.js** - Complete Operator Coverage
Comprehensive demonstration of all supported operators:
- **Arithmetic**: +, -, *, /, %, ** (power)
- **Assignment**: =, +=, -=, *=, /=, %=
- **Comparison**: ==, !=, ===, !==, <, >, <=, >=
- **Logical**: &&, ||, !
- **Bitwise**: &, |, ^, ~, <<, >>, >>>
- **Unary**: +, -, ++, --
- **Special**: typeof, instanceof, in, delete, void

**Expected Output**: `true` (complex logical expression result)

### 🔧 **03_functions.js** - Function Capabilities
Shows all aspects of function support:
- **Declarations**: function statements and expressions
- **Parameters**: single and multiple parameters
- **Return values**: various return scenarios
- **Recursion**: factorial and Fibonacci implementations
- **Scope**: local vs global variable access
- **Higher-order functions**: functions as arguments

**Expected Output**: `120` (factorial of 5)

### 🏗️ **04_objects_arrays.js** - Objects and Arrays
Demonstrates complex data structure operations:
- **Object creation**: literals, property access
- **Property manipulation**: dot and bracket notation
- **Array operations**: creation, indexing, modification
- **Nested structures**: arrays of objects, object properties
- **Method calls**: object methods with `this` context

**Expected Output**: `75000` (calculated average salary)

### 🔄 **05_control_flow.js** - Control Flow Structures
Complete control flow demonstration:
- **Conditionals**: if/else, nested conditions
- **Loops**: while, for, nested loops
- **Flow control**: break, continue, return
- **Complex logic**: multiple conditions and branches
- **Recursive functions**: with conditional returns

**Expected Output**: `6` (GCD calculation result)

### 🎯 **06_advanced.js** - Advanced Algorithms
Sophisticated programming patterns and algorithms:
- **Sorting algorithms**: quicksort implementation
- **Data structures**: stack, queue, memory manager
- **Graph algorithms**: depth-first search
- **Design patterns**: observer pattern, memoization
- **Expression evaluation**: simple interpreter

**Expected Output**: Complex result from combined algorithms

## 🚀 Practical Demonstration Programs

### 🧮 **demo_calculator.js** - Advanced Calculator
Professional calculator implementation featuring:
- **Basic operations**: add, subtract, multiply, divide
- **Advanced math**: power, factorial, Fibonacci, GCD/LCM
- **Memory functions**: store, recall, clear
- **Statistics**: mean, min, max for arrays
- **Equation solving**: quadratic equations with discriminant
- **Geometry**: triangle area using Heron's formula

**Expected Output**: `6` (triangle area calculation)

### 📊 **demo_sorting.js** - Sorting Algorithms Showcase
Comprehensive sorting algorithm implementations:
- **Basic sorts**: bubble, selection, insertion
- **Advanced sorts**: quicksort, mergesort
- **Specialized sorts**: counting sort for small integers
- **Performance testing**: timing and comparison framework
- **Utility functions**: array copying, swap operations
- **Validation**: sorted array verification

**Expected Output**: Sum of first elements from all sorted arrays

### 🗃️ **demo_data_structures.js** - Data Structure Library
Professional implementations of fundamental data structures:
- **Stack**: LIFO operations with fixed capacity
- **Queue**: FIFO operations with circular buffer
- **Linked List**: dynamic node-based structure
- **Hash Table**: key-value storage with collision handling
- **Deque**: double-ended queue operations
- **Practical applications**: balanced parentheses, string reversal

**Expected Output**: Total size of all active data structures

## 📋 Original Test Cases (Legacy)

### Mathematical Computations
- **`demo_factorial.js`** - Calculates factorial using while loop (5! = 120)
- **`demo_fibonacci.js`** - Computes Fibonacci sequence using for loop (F(8) = 21)
- **`demo_collatz.js`** - Demonstrates Collatz conjecture (3n+1 problem)

### Control Flow Tests
- **`test_simple_if.js`** - Basic if statement with comparison
- **`test_simple_while.js`** - Simple while loop with counter
- **`test_for_loop.js`** - For loop with initialization, condition, and update
- **`test_control_flow.js`** - Mixed control structures (if + while)
- **`test_break_continue.js`** - Break and continue statements

### Basic Tests
- **`test_simple.js`** - Simple arithmetic expression (result: 52)
- **`test_vm.js`** - Basic VM functionality test

## 🏃‍♂️ Running Examples

### Individual Examples
```bash
# Run feature coverage examples
cargo run examples/01_literals.js
cargo run examples/02_operators.js
cargo run examples/03_functions.js
cargo run examples/04_objects_arrays.js
cargo run examples/05_control_flow.js
cargo run examples/06_advanced.js

# Run practical demonstrations
cargo run examples/demo_calculator.js
cargo run examples/demo_sorting.js
cargo run examples/demo_data_structures.js
```

### Batch Testing
```bash
# Run all feature coverage examples
for file in examples/0*.js; do
    echo "=== Testing $file ==="
    cargo run "$file"
    echo
done

# Run all demo programs
for file in examples/demo_*.js; do
    echo "=== Running $file ==="
    cargo run "$file"
    echo
done

# Run with bytecode debug output
cargo run -- --debug-bytecode examples/06_advanced.js
```

### Performance Testing
```bash
# Test complex algorithms
time cargo run examples/demo_sorting.js
time cargo run examples/06_advanced.js

# Memory usage analysis
cargo run --bin heap_dump examples/demo_data_structures.js

# Bytecode analysis
cargo run --bin disassembler examples/03_functions.js
```

## 📊 Expected Outputs Reference

| Example | Expected Output | Description |
|---------|----------------|-------------|
| `01_literals.js` | `true` | Strict equality validation |
| `02_operators.js` | `true` | Complex expression result |
| `03_functions.js` | `120` | Factorial calculation |
| `04_objects_arrays.js` | `75000` | Average salary calculation |
| `05_control_flow.js` | `6` | GCD algorithm result |
| `06_advanced.js` | Complex | Multi-algorithm combination |
| `demo_calculator.js` | `6` | Triangle area (Heron's formula) |
| `demo_sorting.js` | Numeric | Sum of sorted array elements |
| `demo_data_structures.js` | Numeric | Total structure sizes |

## 🎓 Educational Value

Each example is designed to teach specific concepts:

1. **Progressive Complexity**: From basic literals to advanced algorithms
2. **Real-world Applications**: Calculator, sorting, data structures
3. **Best Practices**: Proper function design, error handling, modular code
4. **Performance Awareness**: Algorithm complexity and optimization
5. **Code Organization**: Logical structure and documentation

## 🔧 Development Notes

These examples serve multiple purposes:
- **Feature verification**: Ensure all engine capabilities work correctly
- **Performance benchmarking**: Measure execution speed and memory usage
- **Regression testing**: Detect when changes break functionality
- **Documentation**: Show developers how to use the engine
- **Educational material**: Teach JavaScript programming concepts

The examples are designed to be self-contained and well-documented, making them ideal for both testing the engine and learning JavaScript programming fundamentals.