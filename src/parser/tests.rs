#[cfg(test)]
mod tests {
    use crate::lexer::Lexer;
    use crate::error::Result;
    use crate::parser::{Parser, Program, Stmt, Expr, BinaryOp};
    
    fn parse_source(source: &str) -> Result<Program> {
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize()?;
        let mut parser = Parser::new(tokens);
        parser.parse()
    }
    
    #[test]
    fn test_function_declaration() {
        let source = "function add(a, b) { return a + b; }";
        let program = parse_source(source).unwrap();
        
        assert_eq!(program.statements.len(), 1);
        match &program.statements[0] {
            Stmt::FunctionDecl { name, params, body, .. } => {
                assert_eq!(name, "add");
                assert_eq!(params, &["a", "b"]);
                assert_eq!(body.len(), 1);
                
                // Check that body contains a return statement
                match &body[0] {
                    Stmt::Return { value, .. } => {
                        assert!(value.is_some());
                    }
                    _ => panic!("Expected return statement in function body"),
                }
            }
            _ => panic!("Expected function declaration"),
        }
    }
    
    #[test]
    fn test_empty_function() {
        let source = "function test() {}";
        let program = parse_source(source).unwrap();
        
        assert_eq!(program.statements.len(), 1);
        match &program.statements[0] {
            Stmt::FunctionDecl { name, params, body, .. } => {
                assert_eq!(name, "test");
                assert_eq!(params.len(), 0);
                assert_eq!(body.len(), 0);
            }
            _ => panic!("Expected function declaration"),
        }
    }
    
    #[test]
    fn test_if_statement() {
        let source = "if (x > 0) { return x; }";
        let program = parse_source(source).unwrap();
        
        assert_eq!(program.statements.len(), 1);
        match &program.statements[0] {
            Stmt::If { test, then_stmt, else_stmt, .. } => {
                // Test condition should be a binary expression
                match test {
                    Expr::Binary { op: BinaryOp::Greater, .. } => {},
                    _ => panic!("Expected binary expression in if condition"),
                }
                
                // Then branch should be a block
                match then_stmt.as_ref() {
                    Stmt::Block { .. } => {},
                    _ => panic!("Expected block statement in then branch"),
                }
                
                // No else branch
                assert!(else_stmt.is_none());
            }
            _ => panic!("Expected if statement"),
        }
    }
    
    #[test]
    fn test_if_else_statement() {
        let source = "if (x > 0) { return x; } else { return 0; }";
        let program = parse_source(source).unwrap();
        
        assert_eq!(program.statements.len(), 1);
        match &program.statements[0] {
            Stmt::If { else_stmt, .. } => {
                assert!(else_stmt.is_some());
            }
            _ => panic!("Expected if statement"),
        }
    }
    
    #[test]
    fn test_while_statement() {
        let source = "while (x > 0) { x = x - 1; }";
        let program = parse_source(source).unwrap();
        
        assert_eq!(program.statements.len(), 1);
        match &program.statements[0] {
            Stmt::While { test, body, .. } => {
                match test {
                    Expr::Binary { op: BinaryOp::Greater, .. } => {},
                    _ => panic!("Expected binary expression in while condition"),
                }
                
                match body.as_ref() {
                    Stmt::Block { statements, .. } => {
                        assert_eq!(statements.len(), 1);
                    }
                    _ => panic!("Expected block statement in while body"),
                }
            }
            _ => panic!("Expected while statement"),
        }
    }
    
    #[test]
    fn test_variable_declarations() {
        let source = "let x = 42; const y = 'hello'; var z;";
        let program = parse_source(source).unwrap();
        
        assert_eq!(program.statements.len(), 3);
        
        // Test let declaration
        match &program.statements[0] {
            Stmt::VarDecl { name, init, .. } => {
                assert_eq!(name, "x");
                assert!(init.is_some());
            }
            _ => panic!("Expected variable declaration"),
        }
        
        // Test const declaration  
        match &program.statements[1] {
            Stmt::VarDecl { name, init, .. } => {
                assert_eq!(name, "y");
                assert!(init.is_some());
            }
            _ => panic!("Expected variable declaration"),
        }
        
        // Test var declaration without initializer
        match &program.statements[2] {
            Stmt::VarDecl { name, init, .. } => {
                assert_eq!(name, "z");
                assert!(init.is_none());
            }
            _ => panic!("Expected variable declaration"),
        }
    }
    
    #[test]
    fn test_assignment_expression() {
        let source = "x = y + 1;";
        let program = parse_source(source).unwrap();
        
        assert_eq!(program.statements.len(), 1);
        match &program.statements[0] {
            Stmt::Expression(Expr::Assignment { left, right, .. }) => {
                match left.as_ref() {
                    Expr::Identifier { name, .. } => assert_eq!(name, "x"),
                    _ => panic!("Expected identifier on left side of assignment"),
                }
                
                match right.as_ref() {
                    Expr::Binary { op: BinaryOp::Add, .. } => {},
                    _ => panic!("Expected binary expression on right side of assignment"),
                }
            }
            _ => panic!("Expected assignment expression"),
        }
    }
    
    #[test]
    fn test_binary_expressions() {
        let source = "x + y * z;";
        let program = parse_source(source).unwrap();
        
        assert_eq!(program.statements.len(), 1);
        match &program.statements[0] {
            Stmt::Expression(expr) => {
                // Should parse as x + (y * z) due to precedence
                match expr {
                    Expr::Binary { op: BinaryOp::Add, left, right, .. } => {
                        match left.as_ref() {
                            Expr::Identifier { name, .. } => assert_eq!(name, "x"),
                            _ => panic!("Expected identifier x"),
                        }
                        
                        match right.as_ref() {
                            Expr::Binary { op: BinaryOp::Multiply, .. } => {},
                            _ => panic!("Expected multiplication expression"),
                        }
                    }
                    _ => panic!("Expected addition expression"),
                }
            }
            _ => panic!("Expected expression statement"),
        }
    }
    
    #[test]
    fn test_call_expression() {
        let source = "add(1, 2);";
        let program = parse_source(source).unwrap();
        
        assert_eq!(program.statements.len(), 1);
        match &program.statements[0] {
            Stmt::Expression(Expr::Call { callee, args, .. }) => {
                match callee.as_ref() {
                    Expr::Identifier { name, .. } => assert_eq!(name, "add"),
                    _ => panic!("Expected identifier as callee"),
                }
                
                assert_eq!(args.len(), 2);
            }
            _ => panic!("Expected call expression"),
        }
    }
    
    #[test]
    fn test_member_access() {
        let source = "obj.prop; obj[key];";
        let program = parse_source(source).unwrap();
        
        assert_eq!(program.statements.len(), 2);
        
        // Test dot notation
        match &program.statements[0] {
            Stmt::Expression(Expr::Member { object, property: _, computed, .. }) => {
                assert!(!computed);
                match object.as_ref() {
                    Expr::Identifier { name, .. } => assert_eq!(name, "obj"),
                    _ => panic!("Expected identifier as object"),
                }
            }
            _ => panic!("Expected member expression"),
        }
        
        // Test bracket notation
        match &program.statements[1] {
            Stmt::Expression(Expr::Member { computed, .. }) => {
                assert!(*computed);
            }
            _ => panic!("Expected member expression"),
        }
    }
    
    #[test]
    fn test_complex_program() {
        let source = r#"
            function factorial(n) {
                if (n <= 1) {
                    return 1;
                } else {
                    return n * factorial(n - 1);
                }
            }
            
            let result = factorial(5);
        "#;
        
        let program = parse_source(source).unwrap();
        
        // Should contain function declaration and variable declaration
        assert_eq!(program.statements.len(), 2);
        
        match &program.statements[0] {
            Stmt::FunctionDecl { name, .. } => assert_eq!(name, "factorial"),
            _ => panic!("Expected function declaration"),
        }
        
        match &program.statements[1] {
            Stmt::VarDecl { name, .. } => assert_eq!(name, "result"),
            _ => panic!("Expected variable declaration"),
        }
    }
}