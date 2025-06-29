// SAPF Lexer Implementation
//
// Tokenizes SAPF source code according to the language specification

use crate::core::{error::SapfError, symbol::get_symbol};
use crate::parser::token::{Position, Token, TokenType};

/// SAPF Lexer for tokenizing source code
pub struct Lexer {
    input: String,
    position: usize,
    line: usize,
    column: usize,
}

impl Lexer {
    pub fn new(input: &str) -> Self {
        Self {
            input: input.to_string(),
            position: 0,
            line: 1,
            column: 1,
        }
    }

    /// Get current position in the input
    fn current_position(&self) -> Position {
        Position::new(self.line, self.column, self.position)
    }

    /// Peek at the current character without consuming it
    fn peek_char(&self) -> Option<char> {
        self.input.chars().nth(self.position)
    }

    /// Peek ahead n characters
    fn peek_ahead(&self, n: usize) -> Option<char> {
        self.input.chars().nth(self.position + n)
    }

    /// Consume and return the next character
    fn next_char(&mut self) -> Option<char> {
        if let Some(ch) = self.peek_char() {
            self.position += 1;
            if ch == '\n' {
                self.line += 1;
                self.column = 1;
            } else {
                self.column += 1;
            }
            Some(ch)
        } else {
            None
        }
    }

    /// Skip whitespace and comments
    fn skip_whitespace_and_comments(&mut self) {
        while let Some(ch) = self.peek_char() {
            if ch.is_whitespace() {
                self.next_char();
            } else if ch == ';' {
                // Skip comment to end of line
                self.next_char(); // consume ';'
                while let Some(ch) = self.peek_char() {
                    self.next_char();
                    if ch == '\n' {
                        break;
                    }
                }
            } else {
                break;
            }
        }
    }

    /// Check if input starts with a specific string at current position
    fn starts_with(&self, s: &str) -> bool {
        self.input[self.position..].starts_with(s)
    }

    /// Tokenize the entire input into a vector of tokens
    pub fn tokenize(&mut self) -> Result<Vec<Token>, SapfError> {
        let mut tokens = Vec::new();

        loop {
            self.skip_whitespace_and_comments();

            let pos = self.current_position();
            
            match self.peek_char() {
                None => {
                    tokens.push(Token::eof(pos));
                    break;
                }
                Some(ch) => {
                    let token = self.next_token(pos, ch)?;
                    tokens.push(token);
                }
            }
        }

        Ok(tokens)
    }

    /// Parse the next token based on the current character
    fn next_token(&mut self, pos: Position, first_char: char) -> Result<Token, SapfError> {
        match first_char {
            // Hex numbers (0x...) - check first
            '0' if self.starts_with("0x") => self.parse_hex_number(pos),
            
            // Check for 'pi' constant
            'p' if self.starts_with("pi") => self.parse_pi_constant(pos),
            
            // Numbers (including other cases)
            '0'..='9' | '+' | '-' => self.parse_number(pos),
            
            // Strings
            '"' => self.parse_string(pos),
            
            // Special operators and quotes
            '\'' => {
                self.next_char();
                Ok(Token::new(TokenType::Quote, "'".to_string(), pos))
            }
            '`' => {
                self.next_char();
                Ok(Token::new(TokenType::Backquote, "`".to_string(), pos))
            }
            ',' => {
                self.next_char();
                Ok(Token::new(TokenType::Comma, ",".to_string(), pos))
            }
            ':' => {
                self.next_char();
                Ok(Token::new(TokenType::Colon, ":".to_string(), pos))
            }
            '=' => {
                self.next_char();
                Ok(Token::new(TokenType::Equal, "=".to_string(), pos))
            }
            
            // Delimiters
            '(' => {
                self.next_char();
                Ok(Token::new(TokenType::LeftParen, "(".to_string(), pos))
            }
            ')' => {
                self.next_char();
                Ok(Token::new(TokenType::RightParen, ")".to_string(), pos))
            }
            '[' => {
                self.next_char();
                Ok(Token::new(TokenType::LeftBracket, "[".to_string(), pos))
            }
            ']' => {
                self.next_char();
                Ok(Token::new(TokenType::RightBracket, "]".to_string(), pos))
            }
            '{' => {
                self.next_char();
                Ok(Token::new(TokenType::LeftBrace, "{".to_string(), pos))
            }
            '}' => {
                self.next_char();
                Ok(Token::new(TokenType::RightBrace, "}".to_string(), pos))
            }
            
            // Special syntax
            '\\' => {
                self.next_char();
                Ok(Token::new(TokenType::Lambda, "\\".to_string(), pos))
            }
            '#' if self.starts_with("#[") => {
                self.next_char(); // consume '#'
                self.next_char(); // consume '['
                Ok(Token::new(TokenType::HashArray, "#[".to_string(), pos))
            }
            
            // Dot (could be number or operator)
            '.' => {
                if self.peek_ahead(1).map_or(false, |c| c.is_ascii_digit()) {
                    self.parse_number(pos)
                } else {
                    self.next_char();
                    Ok(Token::new(TokenType::Dot, ".".to_string(), pos))
                }
            }
            
            // Symbols (identifiers)
            _ => self.parse_symbol(pos),
        }
    }

    /// Parse pi constant
    fn parse_pi_constant(&mut self, pos: Position) -> Result<Token, SapfError> {
        let start_pos = self.position;
        self.next_char(); // consume 'p'
        self.next_char(); // consume 'i'
        
        // Check if this is actually just part of a larger symbol
        if let Some(ch) = self.peek_char() {
            if self.is_symbol_char(ch) {
                // Reset position and parse as symbol instead
                self.position = start_pos;
                self.column -= 2;
                return self.parse_symbol(pos);
            }
        }
        
        Ok(Token::new(
            TokenType::Number(std::f64::consts::PI),
            "pi".to_string(),
            pos,
        ))
    }

    /// Parse hexadecimal number
    fn parse_hex_number(&mut self, pos: Position) -> Result<Token, SapfError> {
        let mut value = 0i64;
        let start_pos = self.position;
        
        self.next_char(); // consume '0'
        self.next_char(); // consume 'x'
        
        while let Some(ch) = self.peek_char() {
            if ch.is_ascii_hexdigit() {
                self.next_char();
                let digit = ch.to_digit(16).unwrap() as i64;
                value = value * 16 + digit;
            } else {
                break;
            }
        }
        
        let lexeme = &self.input[start_pos..self.position];
        Ok(Token::new(
            TokenType::Number(value as f64),
            lexeme.to_string(),
            pos,
        ))
    }

    /// Parse numeric literal with optional suffixes
    fn parse_number(&mut self, pos: Position) -> Result<Token, SapfError> {
        let start_pos = self.position;
        let mut has_digits = false;
        
        // Handle optional sign
        if matches!(self.peek_char(), Some('+') | Some('-')) {
            self.next_char();
        }
        
        // Parse integer part
        while let Some(ch) = self.peek_char() {
            if ch.is_ascii_digit() {
                has_digits = true;
                self.next_char();
            } else {
                break;
            }
        }
        
        // Parse decimal part
        if self.peek_char() == Some('.') {
            self.next_char(); // consume '.'
            
            while let Some(ch) = self.peek_char() {
                if ch.is_ascii_digit() {
                    has_digits = true;
                    self.next_char();
                } else {
                    break;
                }
            }
        }
        
        // Parse exponent
        if matches!(self.peek_char(), Some('e') | Some('E')) {
            self.next_char(); // consume 'e' or 'E'
            
            if matches!(self.peek_char(), Some('+') | Some('-')) {
                self.next_char();
            }
            
            while let Some(ch) = self.peek_char() {
                if ch.is_ascii_digit() {
                    self.next_char();
                } else {
                    break;
                }
            }
        }
        
        // Parse optional suffix
        let suffix = match self.peek_char() {
            Some('p') if self.peek_ahead(1) == Some('i') => {
                self.next_char(); // consume 'p'
                self.next_char(); // consume 'i'
                Some("pi")
            }
            Some('M') => {
                self.next_char();
                Some("M")
            }
            Some('k') => {
                self.next_char();
                Some("k")
            }
            Some('h') => {
                self.next_char();
                Some("h")
            }
            Some('c') => {
                self.next_char();
                Some("c")
            }
            Some('m') => {
                self.next_char();
                Some("m")
            }
            Some('u') => {
                self.next_char();
                Some("u")
            }
            _ => None,
        };
        
        // If we didn't find any digits, this isn't a number - reset and parse as symbol
        if !has_digits {
            self.position = start_pos;
            self.column = pos.column;
            return self.parse_symbol(pos);
        }
        
        let number_str = &self.input[start_pos..self.position];
        let base_str = if let Some(suffix) = suffix {
            &number_str[..number_str.len() - suffix.len()]
        } else {
            number_str
        };
        
        let mut value = base_str.parse::<f64>()
            .map_err(|_| SapfError::ParseError(format!("Invalid number: {}", base_str)))?;
        
        // Apply suffix multiplier
        if let Some(suffix) = suffix {
            value *= match suffix {
                "pi" => std::f64::consts::PI,
                "M" => 1e6,
                "k" => 1e3,
                "h" => 1e2,
                "c" => 1e-2,
                "m" => 1e-3,
                "u" => 1e-6,
                _ => 1.0,
            };
        }
        
        Ok(Token::new(
            TokenType::Number(value),
            number_str.to_string(),
            pos,
        ))
    }

    /// Parse string literal with escape sequences
    fn parse_string(&mut self, pos: Position) -> Result<Token, SapfError> {
        let start_pos = self.position;
        self.next_char(); // consume opening quote
        
        let mut result = String::new();
        
        while let Some(ch) = self.peek_char() {
            match ch {
                '"' => {
                    self.next_char(); // consume closing quote
                    break;
                }
                '\\' => {
                    self.next_char(); // consume backslash
                    match self.peek_char() {
                        Some('n') => {
                            self.next_char();
                            result.push('\n');
                        }
                        Some('r') => {
                            self.next_char();
                            result.push('\r');
                        }
                        Some('t') => {
                            self.next_char();
                            result.push('\t');
                        }
                        Some('0') => {
                            self.next_char();
                            result.push('\0');
                        }
                        Some('\\') => {
                            self.next_char();
                            result.push('\\');
                        }
                        Some('"') => {
                            self.next_char();
                            result.push('"');
                        }
                        Some(c) => {
                            self.next_char();
                            result.push(c);
                        }
                        None => {
                            return Err(SapfError::ParseError("Unterminated string".to_string()));
                        }
                    }
                }
                c => {
                    self.next_char();
                    result.push(c);
                }
            }
        }
        
        let lexeme = &self.input[start_pos..self.position];
        Ok(Token::new(
            TokenType::String(result),
            lexeme.to_string(),
            pos,
        ))
    }

    /// Check if character is valid in a symbol
    fn is_symbol_char(&self, ch: char) -> bool {
        !ch.is_whitespace() && !"();[]{}.`,:\"\n".contains(ch)
    }

    /// Parse symbol (identifier)
    fn parse_symbol(&mut self, pos: Position) -> Result<Token, SapfError> {
        let start_pos = self.position;
        
        while let Some(ch) = self.peek_char() {
            if self.is_symbol_char(ch) {
                self.next_char();
            } else {
                break;
            }
        }
        
        if start_pos == self.position {
            return Err(SapfError::ParseError(format!(
                "Unexpected character: '{}'",
                self.peek_char().unwrap_or('\0')
            )));
        }
        
        let symbol_str = &self.input[start_pos..self.position];
        let symbol = get_symbol(symbol_str);
        
        Ok(Token::new(
            TokenType::Symbol(symbol),
            symbol_str.to_string(),
            pos,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenize_numbers() {
        let mut lexer = Lexer::new("42 3.14 -5 +10 1e6 0xff pi 440k");
        let tokens = lexer.tokenize().unwrap();
        
        // Filter out EOF token for easier testing
        let tokens: Vec<_> = tokens.into_iter().filter(|t| !t.is_eof()).collect();
        
        assert_eq!(tokens.len(), 8);
        
        match &tokens[0].token_type {
            TokenType::Number(n) => assert_eq!(*n, 42.0),
            _ => panic!("Expected number"),
        }
        
        match &tokens[1].token_type {
            TokenType::Number(n) => assert_eq!(*n, 3.14),
            _ => panic!("Expected number"),
        }
        
        match &tokens[2].token_type {
            TokenType::Number(n) => assert_eq!(*n, -5.0),
            _ => panic!("Expected number"),
        }
        
        match &tokens[6].token_type {
            TokenType::Number(n) => assert!((n - std::f64::consts::PI).abs() < 1e-10),
            _ => panic!("Expected pi constant"),
        }
        
        match &tokens[7].token_type {
            TokenType::Number(n) => assert_eq!(*n, 440000.0), // 440k
            _ => panic!("Expected number with k suffix"),
        }
    }

    #[test]
    fn test_tokenize_strings() {
        let mut lexer = Lexer::new(r#""hello" "world\n""#);
        let tokens = lexer.tokenize().unwrap();
        
        let tokens: Vec<_> = tokens.into_iter().filter(|t| !t.is_eof()).collect();
        assert_eq!(tokens.len(), 2);
        
        match &tokens[0].token_type {
            TokenType::String(s) => assert_eq!(s, "hello"),
            _ => panic!("Expected string"),
        }
        
        match &tokens[1].token_type {
            TokenType::String(s) => assert_eq!(s, "world\n"),
            _ => panic!("Expected string with escape"),
        }
    }

    #[test]
    fn test_tokenize_symbols() {
        let mut lexer = Lexer::new("foo bar + sin");
        let tokens = lexer.tokenize().unwrap();
        
        let tokens: Vec<_> = tokens.into_iter().filter(|t| !t.is_eof()).collect();
        assert_eq!(tokens.len(), 4);
        
        for token in &tokens {
            match &token.token_type {
                TokenType::Symbol(_) => {} // Expected
                _ => panic!("Expected symbol"),
            }
        }
    }

    #[test]
    fn test_tokenize_delimiters() {
        let mut lexer = Lexer::new("()[]{}");
        let tokens = lexer.tokenize().unwrap();
        
        let tokens: Vec<_> = tokens.into_iter().filter(|t| !t.is_eof()).collect();
        assert_eq!(tokens.len(), 6);
        
        let expected = vec![
            TokenType::LeftParen,
            TokenType::RightParen,
            TokenType::LeftBracket,
            TokenType::RightBracket,
            TokenType::LeftBrace,
            TokenType::RightBrace,
        ];
        
        for (i, expected_type) in expected.iter().enumerate() {
            assert!(std::mem::discriminant(&tokens[i].token_type) == std::mem::discriminant(expected_type));
        }
    }

    #[test]
    fn test_tokenize_quotes() {
        let mut lexer = Lexer::new("'foo `bar ,baz .qux");
        let tokens = lexer.tokenize().unwrap();
        
        let tokens: Vec<_> = tokens.into_iter().filter(|t| !t.is_eof()).collect();
        assert_eq!(tokens.len(), 8); // quote + symbol for each
        
        assert!(matches!(tokens[0].token_type, TokenType::Quote));
        assert!(matches!(tokens[2].token_type, TokenType::Backquote));
        assert!(matches!(tokens[4].token_type, TokenType::Comma));
        assert!(matches!(tokens[6].token_type, TokenType::Dot));
    }

    #[test]
    fn test_tokenize_comments() {
        let mut lexer = Lexer::new("; this is a comment\n42");
        let tokens = lexer.tokenize().unwrap();
        
        // Comments are filtered out during tokenization
        let tokens: Vec<_> = tokens.into_iter().filter(|t| !t.is_eof()).collect();
        assert_eq!(tokens.len(), 1);
        
        match &tokens[0].token_type {
            TokenType::Number(n) => assert_eq!(*n, 42.0),
            _ => panic!("Expected number after comment"),
        }
    }

    #[test]
    fn test_tokenize_lambda() {
        let mut lexer = Lexer::new(r"\x [x 2 *]");
        let tokens = lexer.tokenize().unwrap();
        
        let tokens: Vec<_> = tokens.into_iter().filter(|t| !t.is_eof()).collect();
        
        
        assert_eq!(tokens.len(), 7);
        
        assert!(matches!(tokens[0].token_type, TokenType::Lambda));
        assert!(matches!(tokens[1].token_type, TokenType::Symbol(_)));
        assert!(matches!(tokens[2].token_type, TokenType::LeftBracket));
    }

    #[test]
    fn test_hex_numbers() {
        let mut lexer = Lexer::new("0xff 0x10 0xDEADBEEF");
        let tokens = lexer.tokenize().unwrap();
        
        let tokens: Vec<_> = tokens.into_iter().filter(|t| !t.is_eof()).collect();
        assert_eq!(tokens.len(), 3);
        
        match &tokens[0].token_type {
            TokenType::Number(n) => assert_eq!(*n, 255.0),
            _ => panic!("Expected hex number"),
        }
        
        match &tokens[1].token_type {
            TokenType::Number(n) => assert_eq!(*n, 16.0),
            _ => panic!("Expected hex number"),
        }
    }
}