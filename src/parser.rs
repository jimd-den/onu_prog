/// Ọ̀nụ Parser: The Structural Discourse Layer
///
/// This module implements a recursive descent parser for the Ọ̀nụ language.
/// Its responsibility is to transform a flat stream of tokens into a 
/// hierarchical Abstract Syntax Tree (AST) composed of 'Discourse' units
/// and 'Expression' nodes.
///
/// Clean Architecture:
/// This parser is an Interface Adapter. It translates the external language
/// representation (tokens) into the internal representation (AST) that the
/// Use Case layer (Interpreter) can understand.

use crate::lexer::{Token, TokenWithSpan};
use crate::error::{OnuError, Span};
use crate::registry::{Registry, compute_hash};
use std::hash::Hash;

/// Discourse represents the top-level semantic units of an Ọ̀nụ program.
/// Each unit represents a 'proposition' in the academic discourse.
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Discourse {
    /// A module defines a namespace with a single concern (SRP enforcement).
    Module { name: String, concern: String },
    /// A shape defines a contract (interface) that other things promise to fulfill.
    Shape { name: String, behaviors: Vec<BehaviorHeader> },
    /// A behavior is a pure function that fulfills an intent.
    Behavior { header: BehaviorHeader, body: Expression },
}

/// Expression represents the executable logic within a behavior's body.
/// Expressions are strictly pure and side-effect free, except for 'Emit' 
/// which is handled via the injected Environment.
#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub enum Expression {
    Number(u64),
    Text(String),
    Identifier(String),
    Nothing,
    Emit(Box<Expression>),
    Let { name: String, value: Box<Expression>, body: Box<Expression> },
    BehaviorCall { name: String, args: Vec<Expression> },
    If {
        condition: Box<Expression>,
        then_branch: Box<Expression>,
        else_branch: Box<Expression>,
    },
    Block(Vec<Expression>),
}

/// BehaviorHeader contains the metadata for a behavior, including its intent,
/// arguments (receiving), and return type.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct BehaviorHeader {
    pub name: String,
    pub intent: String,
    pub receiving: Vec<(String, String)>, // (type, name)
    pub returning: String,
    pub diminishing: Option<String>, // name of the proof/variable that is smaller
}

/// The Parser maintains a position in the token stream and builds the AST.
pub struct Parser<'a> {
    tokens: Vec<TokenWithSpan>,
    pos: usize,
    registry: Option<&'a mut Registry>,
}

impl<'a> Parser<'a> {
    /// Creates a new Parser from a vector of tokens.
    pub fn new(tokens: Vec<TokenWithSpan>) -> Self {
        Self { tokens, pos: 0, registry: None }
    }

    /// Creates a new Parser with a Registry for semantic enforcement.
    pub fn with_registry(tokens: Vec<TokenWithSpan>, registry: &'a mut Registry) -> Self {
        Self { tokens, pos: 0, registry: Some(registry) }
    }

    /// Returns true if the parser has consumed all tokens.
    pub fn is_eof(&self) -> bool {
        self.pos >= self.tokens.len()
    }

    /// Returns the current span (location) for error reporting.
    fn current_span(&self) -> Span {
        self.tokens.get(self.pos).map(|t| t.span).unwrap_or_else(|| {
            self.tokens.last().map(|t| t.span).unwrap_or_default()
        })
    }

    /// Parses a single discourse unit.
    pub fn parse_discourse(&mut self) -> Result<Discourse, OnuError> {
        let token = self.peek_token().ok_or_else(|| OnuError::ParseError {
            message: "Expected token, found EOF".to_string(),
            span: self.current_span(),
        })?;

        match token {
            Token::TheModuleCalled => self.parse_module(),
            Token::TheShape => self.parse_shape(),
            Token::TheBehaviorCalled => self.parse_behavior(),
            _ => Err(OnuError::ParseError {
                message: format!("Unexpected token: {:?}", token),
                span: self.current_span(),
            }),
        }
    }

    fn parse_module(&mut self) -> Result<Discourse, OnuError> {
        self.consume(Token::TheModuleCalled)?;
        let name = self.consume_identifier()?;
        self.consume(Token::WithConcern)?;
        self.consume(Token::Colon)?;
        
        let mut concern = String::new();
        while let Some(token) = self.peek_token() {
            if matches!(token, Token::TheModuleCalled | Token::TheShape | Token::TheBehaviorCalled) {
                break;
            }
            if !concern.is_empty() {
                concern.push(' ');
            }
            concern.push_str(&self.consume_identifier()?);
        }
        
        Ok(Discourse::Module { name, concern })
    }

    fn parse_shape(&mut self) -> Result<Discourse, OnuError> {
        self.consume(Token::TheShape)?;
        let name = self.consume_identifier()?;
        self.consume(Token::Promises)?;
        self.consume(Token::Colon)?;
        let mut behaviors = Vec::new();
        while let Some(Token::TheBehaviorCalled) = self.peek_token() {
            behaviors.push(self.parse_behavior_header()?);
        }
        Ok(Discourse::Shape { name, behaviors })
    }

    fn parse_behavior(&mut self) -> Result<Discourse, OnuError> {
        let header = self.parse_behavior_header()?;
        self.consume(Token::As)?;
        self.consume(Token::Colon)?;
        
        let mut expressions = Vec::new();
        while let Some(token) = self.peek_token() {
            if matches!(token, Token::TheModuleCalled | Token::TheShape | Token::TheBehaviorCalled) {
                break;
            }
            expressions.push(self.parse_expression()?);
        }
        
        let body = if expressions.len() == 1 {
            expressions.pop().unwrap()
        } else {
            Expression::Block(expressions)
        };

        // Semantic Enforcement (DRY): Semantic Hashing and Registry Check
        if let Some(registry) = self.registry.as_mut() {
            let hash = compute_hash(&body);
            registry.register(header.name.clone(), hash)?;
        }
        
        Ok(Discourse::Behavior { header, body })
    }

    /// Parses an expression. This is the heart of the recursive descent engine.
    /// Parses an expression. This is the heart of the recursive descent engine.
    fn parse_expression(&mut self) -> Result<Expression, OnuError> {
        let span = self.current_span();
        match self.peek_token() {
            Some(Token::Emit) => {
                self.consume(Token::Emit)?;
                let value = Box::new(self.parse_expression()?);
                Ok(Expression::Emit(value))
            }
            Some(Token::Let) => {
                self.consume(Token::Let)?;
                let name = self.consume_identifier()?;
                self.consume(Token::Is)?;
                let value = Box::new(self.parse_expression()?);
                
                let mut body_exprs = Vec::new();
                while let Some(token) = self.peek_token() {
                    if matches!(token, Token::TheModuleCalled | Token::TheShape | Token::TheBehaviorCalled | Token::RParen | Token::Else | Token::Then) {
                        break;
                    }
                    body_exprs.push(self.parse_expression()?);
                }
                
                let body = if body_exprs.is_empty() {
                    Box::new(Expression::Nothing)
                } else if body_exprs.len() == 1 {
                    Box::new(body_exprs.pop().unwrap())
                } else {
                    Box::new(Expression::Block(body_exprs))
                };
                
                Ok(Expression::Let { name, value, body })
            }
            Some(Token::LParen) => {
                self.consume(Token::LParen)?;
                let expr = self.parse_expression()?;
                self.consume(Token::RParen)?;
                Ok(expr)
            }
            Some(Token::If) => {
                self.consume(Token::If)?;
                let condition = Box::new(self.parse_expression()?);
                self.consume(Token::Then)?;
                let then_branch = Box::new(self.parse_expression()?);
                self.consume(Token::Else)?;
                let else_branch = Box::new(self.parse_expression()?);
                Ok(Expression::If {
                    condition,
                    then_branch,
                    else_branch,
                })
            }
            Some(Token::NumericLiteral(n)) => {
                let val = n;
                self.pos += 1;
                Ok(Expression::Number(val as u64))
            }
            Some(Token::TextLiteral(s)) => {
                let val = s.clone();
                self.pos += 1;
                Ok(Expression::Text(val))
            }
            Some(Token::Nothing) => {
                self.pos += 1;
                Ok(Expression::Nothing)
            }
            Some(Token::Identifier(s)) => {
                let name = s.clone();
                self.pos += 1;
                if let Some(Token::Receiving) = self.peek_token() {
                    self.consume(Token::Receiving)?;
                    let mut args = Vec::new();
                    while let Some(token) = self.peek_token() {
                        if matches!(token, Token::Returning | Token::As | Token::Let | Token::Emit | Token::If | Token::TheModuleCalled | Token::TheShape | Token::TheBehaviorCalled | Token::Colon | Token::Then | Token::Else | Token::RParen) {
                            break;
                        }
                        
                        // If it's an identifier followed by 'receiving', it's likely a subsequent call
                        // unless it's wrapped in parentheses (which parse_expression handles).
                        if let Token::Identifier(_) = token {
                            if let Some(Token::Receiving) = self.peek_ahead(1) {
                                break;
                            }
                        }

                        args.push(self.parse_expression()?);
                    }
                    Ok(Expression::BehaviorCall { name, args })
                } else {
                    Ok(Expression::Identifier(name))
                }
            }
            Some(Token::Number) => {
                self.pos += 1;
                let n = self.consume_numeric_literal()?;
                Ok(Expression::Number(n as u64))
            }
            Some(Token::Text) => {
                self.pos += 1;
                let s = self.consume_text_literal()?;
                Ok(Expression::Text(s))
            }
            Some(token) => Err(OnuError::ParseError {
                message: format!("Expected expression, found {:?}", token),
                span,
            }),
            None => Err(OnuError::ParseError {
                message: "Expected expression, found EOF".to_string(),
                span,
            }),
        }
    }

    fn consume_numeric_literal(&mut self) -> Result<f64, OnuError> {
        let span = self.current_span();
        match self.tokens.get(self.pos) {
            Some(t) => {
                if let Token::NumericLiteral(n) = t.token {
                    self.pos += 1;
                    Ok(n)
                } else {
                    Err(OnuError::ParseError {
                        message: format!("Expected NumericLiteral, found {:?}", t.token),
                        span,
                    })
                }
            }
            None => Err(OnuError::ParseError {
                message: "Expected NumericLiteral, found EOF".to_string(),
                span,
            }),
        }
    }

    fn consume_text_literal(&mut self) -> Result<String, OnuError> {
        let span = self.current_span();
        match self.tokens.get(self.pos) {
            Some(t) => {
                if let Token::TextLiteral(ref s) = t.token {
                    let res = s.clone();
                    self.pos += 1;
                    Ok(res)
                } else {
                    Err(OnuError::ParseError {
                        message: format!("Expected TextLiteral, found {:?}", t.token),
                        span,
                    })
                }
            }
            None => Err(OnuError::ParseError {
                message: "Expected TextLiteral, found EOF".to_string(),
                span,
            }),
        }
    }

    fn parse_behavior_header(&mut self) -> Result<BehaviorHeader, OnuError> {
        self.consume(Token::TheBehaviorCalled)?;
        let name = self.consume_identifier()?;
        
        let mut intent = String::new();
        if let Some(Token::WithIntent) = self.peek_token() {
            self.consume(Token::WithIntent)?;
            self.consume(Token::Colon)?;
            while let Some(token) = self.peek_token() {
                if matches!(token, Token::Receiving | Token::Returning | Token::WithDiminishing | Token::As) {
                    break;
                }
                if !intent.is_empty() {
                    intent.push(' ');
                }
                intent.push_str(&self.consume_identifier()?);
            }
        }
        
        let mut receiving = Vec::new();
        if let Some(Token::Receiving) = self.peek_token() {
            self.consume(Token::Receiving)?;
            self.consume(Token::Colon)?;
            while let Some(token) = self.peek_token() {
                if matches!(token, Token::Returning | Token::As | Token::WithDiminishing) {
                    break;
                }
                
                if let Some(Token::Identifier(ref s)) = self.peek_token() {
                    if s == "a" || s == "an" {
                        self.pos += 1;
                    }
                }
                
                let type_name = self.consume_identifier()?;
                
                if let Some(Token::Identifier(ref s)) = self.peek_token() {
                    if s == "called" {
                        self.pos += 1;
                    }
                }
                
                let var_name = self.consume_identifier()?;
                receiving.push((type_name, var_name));
            }
        }

        let mut returning = "nothing".to_string();
        if let Some(Token::Returning) = self.peek_token() {
            self.consume(Token::Returning)?;
            if let Some(Token::Colon) = self.peek_token() {
                self.consume(Token::Colon)?;
            }
            
            if let Some(Token::Identifier(ref s)) = self.peek_token() {
                if s == "a" || s == "an" {
                    self.pos += 1;
                }
            }
            
            returning = self.consume_identifier()?;
        }

        let mut diminishing = None;
        if let Some(Token::WithDiminishing) = self.peek_token() {
            self.consume(Token::WithDiminishing)?;
            self.consume(Token::Colon)?;
            diminishing = Some(self.consume_identifier()?);
        }

        Ok(BehaviorHeader {
            name,
            intent,
            receiving,
            returning,
            diminishing,
        })
    }

    fn peek_token(&self) -> Option<Token> {
        self.tokens.get(self.pos).map(|t| t.token.clone())
    }

    fn peek_ahead(&self, offset: usize) -> Option<Token> {
        self.tokens.get(self.pos + offset).map(|t| t.token.clone())
    }

    fn consume(&mut self, expected: Token) -> Result<(), OnuError> {
        let span = self.current_span();
        match self.tokens.get(self.pos) {
            Some(t) if t.token == expected => {
                self.pos += 1;
                Ok(())
            }
            Some(t) => Err(OnuError::ParseError {
                message: format!("Expected {:?}, found {:?}", expected, t.token),
                span,
            }),
            None => Err(OnuError::ParseError {
                message: format!("Expected {:?}, found EOF", expected),
                span,
            }),
        }
    }

    fn consume_identifier(&mut self) -> Result<String, OnuError> {
        let span = self.current_span();
        match self.tokens.get(self.pos) {
            Some(t) => {
                let res = match t.token {
                    Token::Identifier(ref name) => name.clone(),
                    Token::Number => "number".to_string(),
                    Token::Text => "text".to_string(),
                    Token::Nothing => "nothing".to_string(),
                    Token::The => "the".to_string(),
                    Token::With => "with".to_string(),
                    Token::If => "if".to_string(),
                    Token::Is => "is".to_string(),
                    Token::Called => "called".to_string(),
                    Token::As => "as".to_string(),
                    Token::Emit => "emit".to_string(),
                    Token::NumericLiteral(n) => n.to_string(),
                    Token::TextLiteral(ref s) => s.clone(),
                    ref other => return Err(OnuError::ParseError {
                        message: format!("Expected Identifier, found {:?}", other),
                        span,
                    }),
                };
                self.pos += 1;
                Ok(res)
            }
            None => Err(OnuError::ParseError {
                message: "Expected Identifier, found EOF".to_string(),
                span,
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::Token;

    fn t(token: Token) -> TokenWithSpan {
        TokenWithSpan { token, span: Span::default() }
    }

    #[test]
    fn test_parse_module_header() {
        let tokens = vec![
            t(Token::TheModuleCalled),
            t(Token::Identifier("MeasurementDomain".to_string())),
            t(Token::WithConcern),
            t(Token::Colon),
            t(Token::Identifier("scaling".to_string())),
        ];
        let mut parser = Parser::new(tokens);
        let result = parser.parse_discourse().unwrap();
        assert_eq!(
            result,
            Discourse::Module {
                name: "MeasurementDomain".to_string(),
                concern: "scaling".to_string()
            }
        );
    }

    #[test]
    fn test_parse_behavior_conflict() {
        let tokens = vec![
            t(Token::TheBehaviorCalled), t(Token::Identifier("foo".to_string())),
            t(Token::As), t(Token::Colon),
            t(Token::NumericLiteral(42.0)),
            
            t(Token::TheBehaviorCalled), t(Token::Identifier("bar".to_string())),
            t(Token::As), t(Token::Colon),
            t(Token::NumericLiteral(42.0)),
        ];
        let mut registry = Registry::new();
        let mut parser = Parser::with_registry(tokens, &mut registry);
        
        parser.parse_discourse().unwrap();
        let result = parser.parse_discourse();
        
        assert!(result.is_err());
        match result.unwrap_err() {
            OnuError::BehaviorConflict { name, other_name } => {
                assert_eq!(name, "bar");
                assert_eq!(other_name, "foo");
            }
            _ => panic!("Expected BehaviorConflict error"),
        }
    }
}
