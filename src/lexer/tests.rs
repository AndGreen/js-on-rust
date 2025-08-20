#[cfg(test)]
mod tests {
    use crate::lexer::{Lexer, TokenKind, Keyword};
    
    #[test]
    fn test_basic_tokenization() {
        let source = "let x = 42;";
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize().unwrap();
        
        assert_eq!(tokens.len(), 6); // let, x, =, 42, ;, EOF
        
        match &tokens[0].kind {
            TokenKind::Keyword(Keyword::Let) => {},
            _ => panic!("Expected 'let' keyword"),
        }
        
        match &tokens[1].kind {
            TokenKind::Identifier(name) => assert_eq!(name, "x"),
            _ => panic!("Expected identifier 'x'"),
        }
        
        match &tokens[2].kind {
            TokenKind::Equal => {},
            _ => panic!("Expected '=' token"),
        }
        
        match &tokens[3].kind {
            TokenKind::Number(n) => assert_eq!(*n, 42.0),
            _ => panic!("Expected number 42"),
        }
    }
    
    #[test]
    fn test_function_tokenization() {
        let source = "function test() {}";
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize().unwrap();
        
        assert_eq!(tokens.len(), 7); // function, test, (, ), {, }, EOF
        
        match &tokens[0].kind {
            TokenKind::Keyword(Keyword::Function) => {},
            _ => panic!("Expected 'function' keyword"),
        }
    }
}