// SAPF Parser Implementation
//
// Parses SAPF tokens into expressions and generates code

use crate::core::error::SapfError;
use crate::parser::token::Token;

/// SAPF Parser for parsing tokenized source code
pub struct Parser {
    tokens: Vec<Token>,
    position: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens,
            position: 0,
        }
    }

    /// Parse the tokens into expressions
    pub fn parse(&mut self) -> Result<(), SapfError> {
        // TODO: Implement parser
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parser_creation() {
        let parser = Parser::new(vec![]);
        assert_eq!(parser.position, 0);
    }
}