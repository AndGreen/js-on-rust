use v8::{Engine, Result, Lexer, Parser, ast::PrettyPrint};
use std::env;
use std::fs;

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    
    match args.len() {
        1 => {
            // Start REPL mode
            repl_mode()
        },
        2 => {
            // Check if we should debug tokenization, AST, or bytecode
            let filename = &args[1];
            if filename == "--debug-tokens" {
                debug_tokens_mode()
            } else if filename == "--debug-ast" {
                debug_ast_mode()
            } else if filename == "--debug-bytecode" {
                debug_bytecode_mode()
            } else {
                execute_file(filename)
            }
        },
        3 => {
            if &args[1] == "--debug-tokens" {
                let filename = &args[2];
                debug_tokens_for_file(filename)
            } else if &args[1] == "--debug-ast" {
                let filename = &args[2];
                debug_ast_for_file(filename)
            } else if &args[1] == "--debug-bytecode" {
                let filename = &args[2];
                debug_bytecode_for_file(filename)
            } else {
                eprintln!("Usage: {} [file.js] or {} --debug-tokens [file.js] or {} --debug-ast [file.js] or {} --debug-bytecode [file.js]", args[0], args[0], args[0], args[0]);
                std::process::exit(1);
            }
        },
        _ => {
            eprintln!("Usage: {} [file.js]", args[0]);
            eprintln!("  {} - Start REPL", args[0]);
            eprintln!("  {} file.js - Execute JavaScript file", args[0]);
            eprintln!("  {} --debug-tokens [file.js] - Show tokens for input", args[0]);
            eprintln!("  {} --debug-ast [file.js] - Show detailed AST tree", args[0]);
            eprintln!("  {} --debug-bytecode [file.js] - Show compiled bytecode", args[0]);
            std::process::exit(1);
        }
    }
}

fn debug_tokens_mode() -> Result<()> {
    println!("Enter JavaScript code to see tokens (Ctrl+D to exit):");
    
    loop {
        print!("tokens> ");
        use std::io::{self, Write};
        io::stdout().flush().unwrap();
        
        let mut input = String::new();
        match io::stdin().read_line(&mut input) {
            Ok(0) => break, // EOF
            Ok(_) => {
                let input = input.trim();
                if input.is_empty() { continue; }
                
                let mut lexer = Lexer::new(input);
                match lexer.tokenize() {
                    Ok(tokens) => {
                        for token in tokens {
                            println!("{:?}", token);
                        }
                    },
                    Err(e) => eprintln!("Lexer error: {}", e),
                }
            },
            Err(e) => {
                eprintln!("Error reading input: {}", e);
                break;
            }
        }
    }
    Ok(())
}

fn debug_tokens_for_file(filename: &str) -> Result<()> {
    let source = fs::read_to_string(filename)?;
    println!("Tokens for file '{}':", filename);
    println!("Source: {}", source);
    println!();
    
    let mut lexer = Lexer::new(&source);
    match lexer.tokenize() {
        Ok(tokens) => {
            for (i, token) in tokens.iter().enumerate() {
                println!("{}: {:?}", i, token);
            }
        },
        Err(e) => eprintln!("Lexer error: {}", e),
    }
    Ok(())
}

fn repl_mode() -> Result<()> {
    println!("V8-like JavaScript Engine v{}", v8::VERSION);
    println!("Type 'exit' or '.exit' to quit, '.help' for help");
    
    let mut engine = Engine::new();
    
    loop {
        print!("js> ");
        use std::io::{self, Write};
        io::stdout().flush().unwrap();
        
        let mut input = String::new();
        match io::stdin().read_line(&mut input) {
            Ok(0) => {
                // EOF (Ctrl+D) - graceful exit
                println!("\nGoodbye!");
                break;
            }
            Ok(_) => {
                // Successfully read input
                let input = input.trim();
                
                // Handle REPL commands
                match input {
                    "exit" | ".exit" => break,
                    ".help" => {
                        println!("Available commands:");
                        println!("  .exit   - Exit the REPL");
                        println!("  .help   - Show this help message");
                        println!("  <expr>  - Evaluate JavaScript expression");
                        continue;
                    }
                    "" => continue,
                    _ => {
                        // Execute JavaScript code
                        match engine.execute(input) {
                            Ok(_result) => {
                                // Success - result already printed by engine
                            },
                            Err(e) => {
                                eprintln!("Error: {}", e);
                            }
                        }
                    }
                }
            }
            Err(e) => {
                eprintln!("Error reading input: {}", e);
                break;
            }
        }
    }
    
    Ok(())
}

fn execute_file(filename: &str) -> Result<()> {
    let source = fs::read_to_string(filename)?;
    let mut engine = Engine::new();
    engine.execute(&source)?;
    Ok(())
}

fn debug_ast_mode() -> Result<()> {
    println!("Enter JavaScript code to see AST tree (Ctrl+D to exit):");
    
    loop {
        print!("ast> ");
        use std::io::{self, Write};
        io::stdout().flush().unwrap();
        
        let mut input = String::new();
        match io::stdin().read_line(&mut input) {
            Ok(0) => break, // EOF
            Ok(_) => {
                let input = input.trim();
                if input.is_empty() { continue; }
                
                let mut lexer = Lexer::new(input);
                match lexer.tokenize() {
                    Ok(tokens) => {
                        let mut parser = Parser::new(tokens);
                        match parser.parse() {
                            Ok(ast) => {
                                println!("{}", ast.pretty_print(0));
                            },
                            Err(e) => eprintln!("Parser error: {}", e),
                        }
                    },
                    Err(e) => eprintln!("Lexer error: {}", e),
                }
            },
            Err(e) => {
                eprintln!("Error reading input: {}", e);
                break;
            }
        }
    }
    Ok(())
}

fn debug_ast_for_file(filename: &str) -> Result<()> {
    let source = fs::read_to_string(filename)?;
    println!("AST tree for file '{}':", filename);
    println!("Source: {}", source);
    println!();
    
    let mut lexer = Lexer::new(&source);
    match lexer.tokenize() {
        Ok(tokens) => {
            let mut parser = Parser::new(tokens);
            match parser.parse() {
                Ok(ast) => {
                    println!("{}", ast.pretty_print(0));
                },
                Err(e) => eprintln!("Parser error: {}", e),
            }
        },
        Err(e) => eprintln!("Lexer error: {}", e),
    }
    Ok(())
}

fn debug_bytecode_mode() -> Result<()> {
    println!("Enter JavaScript code to see bytecode (Ctrl+D to exit):");
    
    loop {
        print!("bytecode> ");
        use std::io::{self, Write};
        io::stdout().flush().unwrap();
        
        let mut input = String::new();
        match io::stdin().read_line(&mut input) {
            Ok(0) => break, // EOF
            Ok(_) => {
                let input = input.trim();
                if input.is_empty() { continue; }
                
                let mut engine = Engine::new_with_bytecode_debug();
                match engine.execute(input) {
                    Ok(_result) => {
                        // Success - bytecode already printed by engine
                    },
                    Err(e) => eprintln!("Error: {}", e),
                }
            },
            Err(e) => {
                eprintln!("Error reading input: {}", e);
                break;
            }
        }
    }
    Ok(())
}

fn debug_bytecode_for_file(filename: &str) -> Result<()> {
    let source = fs::read_to_string(filename)?;
    println!("Bytecode for file '{}':", filename);
    println!("Source: {}", source);
    println!();
    
    let mut engine = Engine::new_with_bytecode_debug();
    engine.execute(&source)?;
    Ok(())
}