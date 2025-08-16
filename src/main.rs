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
    println!("Type 'exit' to quit");
    
    let mut engine = Engine::new();
    
    loop {
        print!("js> ");
        use std::io::{self, Write};
        io::stdout().flush().unwrap();
        
        let mut input = String::new();
        io::stdin().read_line(&mut input).expect("Failed to read input");
        
        let input = input.trim();
        if input == "exit" {
            break;
        }
        
        if input.is_empty() {
            continue;
        }
        
        match engine.execute(input) {
            Ok(()) => {
                // Success - result already printed by engine
            },
            Err(e) => {
                eprintln!("{}", e);
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
