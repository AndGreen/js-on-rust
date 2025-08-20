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
    
    #[test]
    fn test_unicode_comments() {
        // Test with Russian comments that previously caused panic
        let source = r##"
function getRandomColor() {
    const letters = "0123456789ABCDEF";
    let color = "#";
    return color;
}

// Находим кнопку по id
const btn = document.getElementById("test");

// Вешаем обработчик на клик
btn.addEventListener("click", () => {
    document.body.style.backgroundColor = getRandomColor();
});
"##;
        
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize().unwrap();
        
        // Should not panic and should produce tokens
        assert!(tokens.len() > 20);
        
        // Verify we can find identifiers after the Unicode comments
        let has_btn_identifier = tokens.iter().any(|token| {
            matches!(&token.kind, TokenKind::Identifier(name) if name == "btn")
        });
        assert!(has_btn_identifier, "Should tokenize identifier after Unicode comments");
    }
    
    #[test]
    fn test_unicode_identifiers() {
        // Test with Unicode identifiers (though JS typically uses ASCII)
        let source = "let café = 42; let naïve = true;";
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize().unwrap();
        
        // Should not panic
        assert!(tokens.len() > 5);
        
        let has_cafe = tokens.iter().any(|token| {
            matches!(&token.kind, TokenKind::Identifier(name) if name == "café")
        });
        assert!(has_cafe, "Should handle Unicode identifiers");
    }
    
    #[test]
    fn test_unicode_strings() {
        // Test with Unicode content in strings
        let source = r##"let greeting = "Привет мир!"; let emoji = "🚀🎉";"##;
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize().unwrap();
        
        // Should not panic
        assert!(tokens.len() > 5);
        
        let has_russian_string = tokens.iter().any(|token| {
            matches!(&token.kind, TokenKind::String(content) if content == "Привет мир!")
        });
        assert!(has_russian_string, "Should handle Russian strings");
        
        let has_emoji_string = tokens.iter().any(|token| {
            matches!(&token.kind, TokenKind::String(content) if content == "🚀🎉")
        });
        assert!(has_emoji_string, "Should handle emoji strings");
    }
    
    #[test]
    fn test_mixed_ascii_unicode() {
        // Test mixing ASCII and Unicode throughout the source
        let source = r##"
function test() {
    // English comment
    let x = 42;
    // Русский комментарий
    let message = "Hello мир!";
    return message + "🎯";
}
"##;
        
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize().unwrap();
        
        // Should not panic and should tokenize correctly
        assert!(tokens.len() > 10);
        
        // Should have function identifier
        let has_function_name = tokens.iter().any(|token| {
            matches!(&token.kind, TokenKind::Identifier(name) if name == "test")
        });
        assert!(has_function_name, "Should handle mixed Unicode content");
        
        // Should have the Unicode string
        let has_unicode_string = tokens.iter().any(|token| {
            matches!(&token.kind, TokenKind::String(content) if content == "Hello мир!")
        });
        assert!(has_unicode_string, "Should handle Unicode strings");
    }
    
    #[test]
    fn test_unicode_edge_cases() {
        // Test edge cases with Unicode at string boundaries
        let source = "🚀"; // Single emoji
        let mut lexer = Lexer::new(source);
        let result = lexer.tokenize();
        
        // This might be an unexpected character, but shouldn't panic
        assert!(result.is_err() || result.is_ok());
        
        // Test empty string with Unicode comment
        let source2 = "// 测试";
        let mut lexer2 = Lexer::new(source2);
        let tokens2 = lexer2.tokenize().unwrap();
        
        // Should just have EOF token
        assert_eq!(tokens2.len(), 1);
        matches!(&tokens2[0].kind, TokenKind::Eof);
    }
}