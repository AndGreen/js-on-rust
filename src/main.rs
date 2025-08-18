use v8::{Engine, Result};
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
            // Execute file
            let filename = &args[1];
            execute_file(filename)
        },
        _ => {
            eprintln!("Usage: {} [file.js]", args[0]);
            eprintln!("  {} - Start REPL", args[0]);
            eprintln!("  {} file.js - Execute JavaScript file", args[0]);
            std::process::exit(1);
        }
    }
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
                            Ok(()) => {
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
    engine.execute(&source)
}
