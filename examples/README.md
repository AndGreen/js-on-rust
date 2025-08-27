# JavaScript Examples

This directory contains example JavaScript programs that demonstrate the capabilities of the V8-like JavaScript engine.

## Demo Programs

### Mathematical Computations
- **`demo_factorial.js`** - Calculates factorial using while loop (5! = 120)
- **`demo_fibonacci.js`** - Computes Fibonacci sequence using for loop (F(8) = 21)
- **`demo_collatz.js`** - Demonstrates Collatz conjecture (3n+1 problem)

### Test Cases

#### Control Flow Tests
- **`test_simple_if.js`** - Basic if statement with comparison
- **`test_simple_while.js`** - Simple while loop with counter
- **`test_for_loop.js`** - For loop with initialization, condition, and update

#### Complex Tests  
- **`test_control_flow.js`** - Mixed control structures (if + while)
- **`test_break_continue.js`** - Break and continue statements (parser limitation)

#### Basic Tests
- **`test_simple.js`** - Simple arithmetic expression
- **`test_vm.js`** - Basic VM functionality test

## Running Examples

```bash
# Run a specific example
cargo run examples/demo_factorial.js

# Run with debug output  
cargo run -- --debug-bytecode examples/demo_fibonacci.js

# Run all demo programs
for file in examples/demo_*.js; do
    echo "=== Running $file ==="
    cargo run "$file"
    echo
done
```

## Expected Outputs

- **demo_factorial.js**: `120` (5! = 5×4×3×2×1)
- **demo_fibonacci.js**: `21` (8th Fibonacci number)  
- **demo_collatz.js**: `16` (steps for n=7 to reach 1)
- **test_simple_if.js**: `10` (conditional assignment)
- **test_for_loop.js**: `6` (1×2×3 factorial)