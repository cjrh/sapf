// SAPF Parser Implementation
//
// Parses SAPF tokens into expressions and generates code

use crate::core::error::{SapfError, Result};
use crate::core::value::{Value, StringObject};
use crate::core::symbol::get_symbol;
use crate::core::list::{List, Array};
use crate::core::form::{Form, Table};
use crate::core::function::{Function, FunctionDef};
use crate::parser::token::{Token, TokenType, Position};
use crate::vm::thread::Thread;
use crate::vm::compile_scope::{CompileScope, TopCompileScope, InnerCompileScope, ScopeType};
use std::sync::Arc;


/// AST Node representing a parsed expression
#[derive(Debug, Clone)]
pub enum ASTNode {
    // Literals
    Number(f64),
    String(String),
    Symbol(Arc<StringObject>),
    
    // Collections
    List(Vec<ASTNode>),
    ZList(Vec<ASTNode>),  // Typed numeric list #[...]
    Form(Vec<ASTNode>),
    
    // Functions
    Lambda {
        args: Vec<Arc<StringObject>>,
        help: Option<String>,
        body: Vec<ASTNode>,
    },
    
    // Special operators
    Quote(Arc<StringObject>),
    Backquote(Arc<StringObject>),
    Dot(Arc<StringObject>),
    Comma(Arc<StringObject>),
    
    // Variable operations  
    Assignment {
        targets: Vec<Arc<StringObject>>,
        value: Box<ASTNode>,
        is_from_list: bool,
    },
    
    // Call/execution
    Call(Arc<StringObject>),
}

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

    /// Get the current token
    fn current_token(&self) -> &Token {
        if self.position < self.tokens.len() {
            &self.tokens[self.position]
        } else {
            // Return EOF token if past end
            static EOF_TOKEN: std::sync::LazyLock<Token> = std::sync::LazyLock::new(|| {
                Token::eof(Position::new(0, 0, 0))
            });
            &EOF_TOKEN
        }
    }

    /// Peek at the next token without consuming it
    fn peek_token(&self) -> &Token {
        if self.position + 1 < self.tokens.len() {
            &self.tokens[self.position + 1]
        } else {
            static EOF_TOKEN: std::sync::LazyLock<Token> = std::sync::LazyLock::new(|| {
                Token::eof(Position::new(0, 0, 0))
            });
            &EOF_TOKEN
        }
    }

    /// Advance to the next token
    fn advance(&mut self) -> &Token {
        if self.position < self.tokens.len() {
            self.position += 1;
        }
        self.current_token()
    }

    /// Check if current token matches expected type
    fn expect(&mut self, expected: &TokenType) -> Result<()> {
        if std::mem::discriminant(&self.current_token().token_type) == std::mem::discriminant(expected) {
            self.advance();
            Ok(())
        } else {
            Err(SapfError::ParseError(format!(
                "Expected {:?}, found {:?}",
                expected, self.current_token().token_type
            )))
        }
    }

    /// Parse the tokens into a list of expressions
    pub fn parse(&mut self) -> Result<Vec<ASTNode>> {
        let mut expressions = Vec::new();
        
        while !self.current_token().is_eof() {
            let expr = self.parse_expression()?;
            expressions.push(expr);
        }
        
        Ok(expressions)
    }

    /// Parse a single expression
    fn parse_expression(&mut self) -> Result<ASTNode> {
        match &self.current_token().token_type {
            TokenType::Number(n) => {
                let value = *n;
                self.advance();
                Ok(ASTNode::Number(value))
            }

            TokenType::String(s) => {
                let value = s.clone();
                self.advance();
                Ok(ASTNode::String(value))
            }

            TokenType::Symbol(sym) => {
                let symbol = sym.clone();
                self.advance();
                Ok(ASTNode::Symbol(symbol))
            }

            TokenType::Quote => {
                self.advance(); // consume '
                if let TokenType::Symbol(sym) = &self.current_token().token_type {
                    let symbol = sym.clone();
                    self.advance();
                    Ok(ASTNode::Quote(symbol))
                } else {
                    Err(SapfError::ParseError("Expected symbol after quote".to_string()))
                }
            }

            TokenType::Backquote => {
                self.advance(); // consume `
                // Check if this is an assignment expression
                if let TokenType::Symbol(sym) = &self.current_token().token_type {
                    let symbol = sym.clone();
                    self.advance();
                    
                    // Check for assignment
                    if matches!(self.current_token().token_type, TokenType::Equal) {
                        self.advance(); // consume =
                        let value_expr = self.parse_expression()?;
                        Ok(ASTNode::Assignment {
                            targets: vec![symbol],
                            value: Box::new(value_expr),
                            is_from_list: false,
                        })
                    } else {
                        Ok(ASTNode::Backquote(symbol))
                    }
                } else {
                    Err(SapfError::ParseError("Expected symbol after backquote".to_string()))
                }
            }

            TokenType::Dot => {
                self.advance(); // consume .
                if let TokenType::Symbol(sym) = &self.current_token().token_type {
                    let symbol = sym.clone();
                    self.advance();
                    Ok(ASTNode::Dot(symbol))
                } else {
                    Err(SapfError::ParseError("Expected symbol after dot".to_string()))
                }
            }

            TokenType::Comma => {
                self.advance(); // consume ,
                if let TokenType::Symbol(sym) = &self.current_token().token_type {
                    let symbol = sym.clone();
                    self.advance();
                    Ok(ASTNode::Comma(symbol))
                } else {
                    Err(SapfError::ParseError("Expected symbol after comma".to_string()))
                }
            }

            TokenType::LeftBracket => {
                self.parse_list()
            }

            TokenType::HashArray => {
                self.parse_zlist()
            }

            TokenType::LeftBrace => {
                self.parse_form()
            }

            TokenType::Lambda => {
                self.parse_lambda()
            }

            _ => Err(SapfError::ParseError(format!(
                "Unexpected token: {:?}",
                self.current_token().token_type
            ))),
        }
    }

    /// Parse a list [...]
    fn parse_list(&mut self) -> Result<ASTNode> {
        self.expect(&TokenType::LeftBracket)?;
        let mut elements = Vec::new();
        
        while !matches!(self.current_token().token_type, TokenType::RightBracket | TokenType::Eof) {
            elements.push(self.parse_expression()?);
        }
        
        self.expect(&TokenType::RightBracket)?;
        Ok(ASTNode::List(elements))
    }

    /// Parse a typed list #[...]
    fn parse_zlist(&mut self) -> Result<ASTNode> {
        self.advance(); // consume #[
        let mut elements = Vec::new();
        
        while !matches!(self.current_token().token_type, TokenType::RightBracket | TokenType::Eof) {
            elements.push(self.parse_expression()?);
        }
        
        self.expect(&TokenType::RightBracket)?;
        Ok(ASTNode::ZList(elements))
    }

    /// Parse a form {...}
    fn parse_form(&mut self) -> Result<ASTNode> {
        self.expect(&TokenType::LeftBrace)?;
        let mut elements = Vec::new();
        
        while !matches!(self.current_token().token_type, TokenType::RightBrace | TokenType::Eof) {
            elements.push(self.parse_expression()?);
        }
        
        self.expect(&TokenType::RightBrace)?;
        Ok(ASTNode::Form(elements))
    }

    /// Parse a lambda function \args [body]
    fn parse_lambda(&mut self) -> Result<ASTNode> {
        self.advance(); // consume \
        
        let mut args = Vec::new();
        let mut help = None;
        
        // Parse arguments - symbols before the [
        while !matches!(self.current_token().token_type, TokenType::LeftBracket | TokenType::Eof) {
            if let TokenType::Symbol(sym) = &self.current_token().token_type {
                args.push(sym.clone());
                self.advance();
            } else if let TokenType::String(s) = &self.current_token().token_type {
                // Help string
                help = Some(s.clone());
                self.advance();
            } else {
                return Err(SapfError::ParseError(format!(
                    "Expected symbol or string in lambda arguments, found {:?}",
                    self.current_token().token_type
                )));
            }
        }
        
        // Parse body [...]
        if !matches!(self.current_token().token_type, TokenType::LeftBracket) {
            return Err(SapfError::ParseError("Expected '[' for lambda body".to_string()));
        }
        
        let body_ast = self.parse_list()?;
        let body = if let ASTNode::List(elements) = body_ast {
            elements
        } else {
            return Err(SapfError::ParseError("Expected list for lambda body".to_string()));
        };
        
        Ok(ASTNode::Lambda { args, help, body })
    }

    /// Compile AST nodes to bytecode
    pub fn compile(&self, nodes: &[ASTNode]) -> Result<crate::vm::opcode::Bytecode> {
        use crate::parser::codegen::CodeGenerator;
        
        let mut codegen = CodeGenerator::new();
        codegen.generate_program(nodes)
    }
    
    /// Compile and execute AST nodes using bytecode
    pub fn compile_and_execute(&self, nodes: &[ASTNode], thread: &mut Thread) -> Result<()> {
        let bytecode = self.compile(nodes)?;
        thread.execute_bytecode(&bytecode)
    }

    /// Execute AST nodes using the VM
    pub fn execute(&self, nodes: &[ASTNode], thread: &mut Thread) -> Result<()> {
        for node in nodes {
            self.execute_node(node, thread)?;
        }
        Ok(())
    }

    /// Execute a single AST node
    fn execute_node(&self, node: &ASTNode, thread: &mut Thread) -> Result<()> {
        match node {
            ASTNode::Number(n) => {
                thread.push(Value::Real(*n));
                Ok(())
            }

            ASTNode::String(s) => {
                let string_obj = Arc::new(StringObject::from_str(s));
                thread.push(Value::Object(string_obj));
                Ok(())
            }

            ASTNode::Symbol(sym) => {
                // Try to lookup symbol in VM builtins first
                use crate::vm::vm::VM;
                let vm = VM::instance();
                
                // Try to lookup symbol by string name
                if let Some(builtin_value) = vm.lookup_by_name(sym.as_str()) {
                    // If it's a callable function/primitive, execute it
                    if builtin_value.is_callable() {
                        builtin_value.apply(thread)?;
                    } else {
                        // Not callable - push the value
                        thread.push(builtin_value);
                    }
                } else {
                    // Symbol not found in builtins - push as literal symbol for late binding
                    thread.push(Value::Object(sym.clone()));
                }
                Ok(())
            }

            ASTNode::Quote(sym) => {
                // Quote prevents evaluation - just push the symbol
                thread.push(Value::Object(sym.clone()));
                Ok(())
            }

            ASTNode::List(elements) => {
                // Create a new list by executing elements
                let mut list_values = Vec::new();
                for elem in elements {
                    self.execute_node(elem, thread)?;
                    let value = thread.pop().map_err(|_| {
                        SapfError::ParseError("Stack underflow during list creation".to_string())
                    })?;
                    list_values.push(value);
                }
                list_values.reverse(); // Restore original order
                
                let array = Array::from_values(list_values);
                let list = List::from_array(array);
                thread.push(Value::Object(Arc::new(list)));
                Ok(())
            }

            ASTNode::Form(elements) => {
                // Create a new form by executing pairs
                let mut pairs = Vec::new();
                let mut i = 0;
                while i + 1 < elements.len() {
                    // Execute key
                    self.execute_node(&elements[i], thread)?;
                    let key = thread.pop().map_err(|_| {
                        SapfError::ParseError("Missing key in form".to_string())
                    })?;
                    
                    // Execute value  
                    self.execute_node(&elements[i + 1], thread)?;
                    let value = thread.pop().map_err(|_| {
                        SapfError::ParseError("Missing value in form".to_string())
                    })?;
                    
                    pairs.push((key, value));
                    i += 2;
                }
                
                let table = Table::from_pairs(pairs);
                let form = Form::from_table(table);
                thread.push(Value::Object(Arc::new(form)));
                Ok(())
            }

            ASTNode::Lambda { args: _, help: _, body: _ } => {
                // Create a function definition
                // TODO: Implement function creation when bytecode generation is ready
                // TODO: Create function when bytecode generation is implemented
                Ok(())
            }

            ASTNode::Assignment { targets, value, is_from_list: _ } => {
                // Execute the value expression
                self.execute_node(value, thread)?;
                let value_result = thread.pop().map_err(|_| {
                    SapfError::ParseError("Stack underflow during assignment".to_string())
                })?;
                
                // Assign to all target variables
                use crate::vm::vm::VM;
                let vm = VM::instance();
                for target in targets {
                    vm.def_by_name(target.as_str(), value_result.clone())?;
                }
                Ok(())
            }

            ASTNode::Backquote(sym) => {
                // Backquote without assignment - just push the symbol
                thread.push(Value::Object(sym.clone()));
                Ok(())
            }

            ASTNode::Dot(sym) => {
                // Dot operator - push the symbol (for now, same as backquote)
                thread.push(Value::Object(sym.clone()));
                Ok(())
            }

            ASTNode::Comma(sym) => {
                // Comma operator - push the symbol (for now, same as backquote)
                thread.push(Value::Object(sym.clone()));
                Ok(())
            }

            ASTNode::Call(sym) => {
                // Call a symbol - similar to Symbol execution
                use crate::vm::vm::VM;
                let vm = VM::instance();
                
                if let Some(builtin_value) = vm.lookup_by_name(sym.as_str()) {
                    if builtin_value.is_callable() {
                        builtin_value.apply(thread)?;
                    } else {
                        thread.push(builtin_value);
                    }
                } else {
                    thread.push(Value::Object(sym.clone()));
                }
                Ok(())
            }

            ASTNode::ZList(elements) => {
                // Create a numeric list by executing elements
                let mut list_values = Vec::new();
                for elem in elements {
                    self.execute_node(elem, thread)?;
                    let value = thread.pop().map_err(|_| {
                        SapfError::ParseError("Stack underflow during zlist creation".to_string())
                    })?;
                    list_values.push(value);
                }
                list_values.reverse(); // Restore original order
                
                let array = Array::from_values(list_values);
                let list = List::from_array(array);
                thread.push(Value::Object(Arc::new(list)));
                Ok(())
            }

            _ => {
                // For now, just ignore other node types
                Ok(())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::lexer::Lexer;

    #[test]
    fn test_parser_creation() {
        let parser = Parser::new(vec![]);
        assert_eq!(parser.position, 0);
    }

    #[test]
    fn test_parse_number() {
        let mut lexer = Lexer::new("42");
        let tokens = lexer.tokenize().unwrap();
        let mut parser = Parser::new(tokens);
        
        let expressions = parser.parse().unwrap();
        assert_eq!(expressions.len(), 1);
        
        match &expressions[0] {
            ASTNode::Number(n) => assert_eq!(*n, 42.0),
            _ => panic!("Expected number"),
        }
    }

    #[test]
    fn test_parse_symbol() {
        let mut lexer = Lexer::new("symbol");
        let tokens = lexer.tokenize().unwrap();
        let mut parser = Parser::new(tokens);
        
        let expressions = parser.parse().unwrap();
        assert_eq!(expressions.len(), 1);
        
        match &expressions[0] {
            ASTNode::Symbol(sym) => assert_eq!(sym.value(), "symbol"),
            _ => panic!("Expected symbol"),
        }
    }

    #[test]
    fn test_parse_list() {
        let mut lexer = Lexer::new("[1 2 3]");
        let tokens = lexer.tokenize().unwrap();
        let mut parser = Parser::new(tokens);
        
        let expressions = parser.parse().unwrap();
        assert_eq!(expressions.len(), 1);
        
        match &expressions[0] {
            ASTNode::List(elements) => {
                assert_eq!(elements.len(), 3);
                match &elements[0] {
                    ASTNode::Number(n) => assert_eq!(*n, 1.0),
                    _ => panic!("Expected number in list"),
                }
            }
            _ => panic!("Expected list"),
        }
    }

    #[test]
    fn test_parse_lambda() {
        let mut lexer = Lexer::new(r"\x [x 2 *]");
        let tokens = lexer.tokenize().unwrap();
        let mut parser = Parser::new(tokens);
        
        let expressions = parser.parse().unwrap();
        assert_eq!(expressions.len(), 1);
        
        match &expressions[0] {
            ASTNode::Lambda { args, body, .. } => {
                assert_eq!(args.len(), 1);
                assert_eq!(args[0].value(), "x");
                assert_eq!(body.len(), 3);
            }
            _ => panic!("Expected lambda"),
        }
    }
    
    #[test]
    fn test_bytecode_compilation() {
        let mut lexer = Lexer::new("1 2 [3 4]");
        let tokens = lexer.tokenize().unwrap();
        let mut parser = Parser::new(tokens);
        let nodes = parser.parse().unwrap();
        
        // Test bytecode compilation
        let bytecode = parser.compile(&nodes).unwrap();
        assert_eq!(bytecode.len(), 5); // 1, 2, 3, 4, newlist
        
        // Test bytecode execution
        let mut thread = Thread::new();
        parser.compile_and_execute(&nodes, &mut thread).unwrap();
        
        assert_eq!(thread.stack_depth(), 3); // 1, 2, and the list [3, 4]
        
        // Verify the list was created correctly
        let list = thread.pop().unwrap();
        if let Value::Object(obj) = list {
            let _list_ref = obj.as_any().downcast_ref::<List>().expect("Expected List");
            // List was successfully created and has correct type
        } else {
            panic!("Expected list object");
        }
    }
}
