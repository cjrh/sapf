// SAPF Token Types
//
// Defines the token types for SAPF lexical analysis

use crate::core::value::{StringObject, Value};
use std::sync::Arc;

/// Position information for tokens
#[derive(Debug, Clone, PartialEq)]
pub struct Position {
    pub line: usize,
    pub column: usize,
    pub offset: usize,
}

impl Position {
    pub fn new(line: usize, column: usize, offset: usize) -> Self {
        Self {
            line,
            column,
            offset,
        }
    }
}

/// Token types for SAPF lexical analysis
#[derive(Debug, Clone, PartialEq)]
pub enum TokenType {
    // Literals
    Number(f64),
    String(String),
    Symbol(Arc<StringObject>),
    
    // Quotes and special operators
    Quote,           // '
    Backquote,       // `
    Comma,           // ,
    Dot,             // .
    Colon,           // :
    Equal,           // =
    
    // Delimiters
    LeftParen,       // (
    RightParen,      // )
    LeftBracket,     // [
    RightBracket,    // ]
    LeftBrace,       // {
    RightBrace,      // }
    
    // Special syntax
    Lambda,          // \
    HashArray,       // #[
    
    // Comments and whitespace
    Comment(String), 
    Whitespace,
    Newline,
    
    // End of input
    Eof,
}

/// A token with position information
#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub token_type: TokenType,
    pub lexeme: String,
    pub position: Position,
}

impl Token {
    pub fn new(token_type: TokenType, lexeme: String, position: Position) -> Self {
        Self {
            token_type,
            lexeme,
            position,
        }
    }
    
    pub fn eof(position: Position) -> Self {
        Self::new(TokenType::Eof, String::new(), position)
    }
    
    pub fn is_eof(&self) -> bool {
        matches!(self.token_type, TokenType::Eof)
    }
    
    pub fn is_whitespace(&self) -> bool {
        matches!(
            self.token_type,
            TokenType::Whitespace | TokenType::Newline | TokenType::Comment(_)
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_position() {
        let pos = Position::new(1, 5, 10);
        assert_eq!(pos.line, 1);
        assert_eq!(pos.column, 5);
        assert_eq!(pos.offset, 10);
    }

    #[test]
    fn test_token_creation() {
        let pos = Position::new(1, 1, 0);
        let token = Token::new(TokenType::Number(42.0), "42".to_string(), pos.clone());
        
        assert_eq!(token.lexeme, "42");
        assert_eq!(token.position, pos);
        assert!(matches!(token.token_type, TokenType::Number(42.0)));
    }

    #[test]
    fn test_eof_token() {
        let pos = Position::new(1, 1, 0);
        let token = Token::eof(pos);
        
        assert!(token.is_eof());
        assert!(!token.is_whitespace());
    }

    #[test]
    fn test_whitespace_tokens() {
        let pos = Position::new(1, 1, 0);
        
        let ws = Token::new(TokenType::Whitespace, " ".to_string(), pos.clone());
        assert!(ws.is_whitespace());
        
        let nl = Token::new(TokenType::Newline, "\n".to_string(), pos.clone());
        assert!(nl.is_whitespace());
        
        let comment = Token::new(TokenType::Comment("test".to_string()), "; test".to_string(), pos);
        assert!(comment.is_whitespace());
    }
}