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
use crate::registry::Registry;
use crate::types::OnuType;

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

/// TypeInfo contains the grammatical metadata for a type declaration.
#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct TypeInfo {
    pub onu_type: OnuType,
    pub display_name: String, // Original name used in discourse (e.g. "integer")
    pub article: Token,       // Token::A or Token::An
    pub via_role: Option<String>,
}

/// Argument represents a named provision in a behavior's receiving clause.
#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct Argument {
    pub name: String,
    pub type_info: TypeInfo,
}

/// ReturnType is a type-safe wrapper for the declared output of a behavior.
#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct ReturnType(pub OnuType);

/// Expression represents the executable logic within a behavior's body.
/// Expressions are strictly pure and side-effect free, except for 'Emit' 
/// which is handled via the injected Environment.
#[derive(Debug, Clone)]
pub enum Expression {
    I8(i8), I16(i16), I32(i32), I64(i64), I128(i128),
    U8(u8), U16(u16), U32(u32), U64(u64), U128(u128),
    F32(f32), F64(f64),
    Boolean(bool),
    Text(String),
    Identifier(String),
    Nothing,
    Tuple(Vec<Expression>),
    Array(Vec<Expression>),
    Matrix { rows: usize, cols: usize, data: Vec<Expression> },
    Emit(Box<Expression>),
    Broadcasts(Box<Expression>), // Active-tense alias for Emit
    Derivation { 
        name: String, 
        type_info: Option<TypeInfo>,
        value: Box<Expression>, 
        body: Box<Expression> 
    },
    ActsAs {
        subject: Box<Expression>,
        shape: String,
    },
    BehaviorCall { name: String, args: Vec<Expression> },
    If {
        condition: Box<Expression>,
        then_branch: Box<Expression>,
        else_branch: Box<Expression>,
    },
    Block(Vec<Expression>),
}

impl PartialEq for Expression {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Expression::I8(n1), Expression::I8(n2)) => n1 == n2,
            (Expression::I16(n1), Expression::I16(n2)) => n1 == n2,
            (Expression::I32(n1), Expression::I32(n2)) => n1 == n2,
            (Expression::I64(n1), Expression::I64(n2)) => n1 == n2,
            (Expression::I128(n1), Expression::I128(n2)) => n1 == n2,
            (Expression::U8(n1), Expression::U8(n2)) => n1 == n2,
            (Expression::U16(n1), Expression::U16(n2)) => n1 == n2,
            (Expression::U32(n1), Expression::U32(n2)) => n1 == n2,
            (Expression::U64(n1), Expression::U64(n2)) => n1 == n2,
            (Expression::U128(n1), Expression::U128(n2)) => n1 == n2,
            (Expression::F32(n1), Expression::F32(n2)) => n1.to_bits() == n2.to_bits(),
            (Expression::F64(n1), Expression::F64(n2)) => n1.to_bits() == n2.to_bits(),
            (Expression::Boolean(b1), Expression::Boolean(b2)) => b1 == b2,
            (Expression::Text(s1), Expression::Text(s2)) => s1 == s2,
            (Expression::Identifier(s1), Expression::Identifier(s2)) => s1 == s2,
            (Expression::Nothing, Expression::Nothing) => true,
            (Expression::Tuple(v1), Expression::Tuple(v2)) => v1 == v2,
            (Expression::Array(v1), Expression::Array(v2)) => v1 == v2,
            (Expression::Matrix { rows: r1, cols: c1, data: d1 }, Expression::Matrix { rows: r2, cols: c2, data: d2 }) => {
                r1 == r2 && c1 == c2 && d1 == d2
            }
            (Expression::Emit(e1), Expression::Emit(e2)) => e1 == e2,
            (Expression::Broadcasts(e1), Expression::Broadcasts(e2)) => e1 == e2,
            (Expression::Derivation { name: n1, value: v1, body: b1, .. }, Expression::Derivation { name: n2, value: v2, body: b2, .. }) => {
                n1 == n2 && v1 == v2 && b1 == b2
            }
            (Expression::ActsAs { subject: s1, shape: sh1 }, Expression::ActsAs { subject: s2, shape: sh2 }) => {
                s1 == s2 && sh1 == sh2
            }
            (Expression::BehaviorCall { name: n1, args: a1 }, Expression::BehaviorCall { name: n2, args: a2 }) => {
                n1 == n2 && a1 == a2
            }
            (Expression::If { condition: c1, then_branch: t1, else_branch: e1 }, Expression::If { condition: c2, then_branch: t2, else_branch: e2 }) => {
                c1 == c2 && t1 == t2 && e1 == e2
            }
            (Expression::Block(b1), Expression::Block(b2)) => b1 == b2,
            _ => false,
        }
    }
}

impl Eq for Expression {}

impl std::hash::Hash for Expression {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        std::mem::discriminant(self).hash(state);
        match self {
            Expression::I8(n) => n.hash(state),
            Expression::I16(n) => n.hash(state),
            Expression::I32(n) => n.hash(state),
            Expression::I64(n) => n.hash(state),
            Expression::I128(n) => n.hash(state),
            Expression::U8(n) => n.hash(state),
            Expression::U16(n) => n.hash(state),
            Expression::U32(n) => n.hash(state),
            Expression::U64(n) => n.hash(state),
            Expression::U128(n) => n.hash(state),
            Expression::F32(n) => n.to_bits().hash(state),
            Expression::F64(n) => n.to_bits().hash(state),
            Expression::Boolean(b) => b.hash(state),
            Expression::Text(s) => s.hash(state),
            Expression::Identifier(s) => s.hash(state),
            Expression::Nothing => {}.hash(state),
            Expression::Tuple(v) => v.hash(state),
            Expression::Array(v) => v.hash(state),
            Expression::Matrix { rows, cols, data } => {
                rows.hash(state);
                cols.hash(state);
                data.hash(state);
            }
            Expression::Emit(e) => e.hash(state),
            Expression::Broadcasts(e) => e.hash(state),
            Expression::Derivation { name, value, body, .. } => {
                name.hash(state);
                value.hash(state);
                body.hash(state);
            }
            Expression::ActsAs { subject, shape } => {
                subject.hash(state);
                shape.hash(state);
            }
            Expression::BehaviorCall { name, args } => {
                name.hash(state);
                args.hash(state);
            }
            Expression::If { condition, then_branch, else_branch } => {
                condition.hash(state);
                then_branch.hash(state);
                else_branch.hash(state);
            }
            Expression::Block(b) => b.hash(state),
        }
    }
}

/// BehaviorHeader contains the metadata for a behavior, including its intent,
/// arguments (receiving), and return type.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct BehaviorHeader {
    pub name: String,
    pub is_effect: bool,
    pub intent: String,
    pub takes: Vec<Argument>,
    pub delivers: ReturnType,
    pub diminishing: Option<String>, // name of the proof/variable that is smaller
    pub skip_termination_check: bool,
}

/// The Parser maintains a position in the token stream and builds the AST.
pub struct Parser<'a, 'b> {
    tokens: &'a [TokenWithSpan],
    pub pos: usize,
    pub registry: Option<&'b Registry>,
    is_pure_context: bool,
    current_depth: usize,
    max_depth: usize,
}

impl<'a, 'b> Parser<'a, 'b> {
    /// Creates a new Parser from a slice of tokens.
    pub fn new(tokens: &'a [TokenWithSpan]) -> Self {
        Self { tokens, pos: 0, registry: None, is_pure_context: false, current_depth: 0, max_depth: 16 }
    }

    /// Creates a new Parser with a Registry for semantic enforcement.
    pub fn with_registry(tokens: &'a [TokenWithSpan], registry: &'b Registry) -> Self {
        Self { tokens, pos: 0, registry: Some(registry), is_pure_context: false, current_depth: 0, max_depth: 16 }
    }

    fn enter_expression(&mut self) -> Result<(), OnuError> {
        self.current_depth += 1;
        if self.current_depth > self.max_depth {
            return Err(OnuError::ParseError {
                message: format!("KISS VIOLATION: The discourse is too deep ({} > {}). Please derive intermediate values.", self.current_depth, self.max_depth),
                span: self.current_span(),
            });
        }
        Ok(())
    }

    fn exit_expression(&mut self) {
        self.current_depth -= 1;
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
            Token::TheBehaviorCalled | Token::TheEffectBehaviorCalled => self.parse_behavior(),
            _ => Err(OnuError::ParseError {
                message: format!("Unexpected token: {:?}", token),
                span: self.current_span(),
            }),
        }
    }

    /// Parses a discourse unit structurally (skipping function bodies) to bootstrap the Registry.
    pub fn parse_structural_discourse(&mut self) -> Result<Discourse, OnuError> {
        let token = self.peek_token().ok_or_else(|| OnuError::ParseError {
            message: "Expected token, found EOF".to_string(),
            span: self.current_span(),
        })?;

        match token {
            Token::TheModuleCalled => self.parse_module(),
            Token::TheShape => self.parse_shape(),
            Token::TheBehaviorCalled | Token::TheEffectBehaviorCalled => {
                let header = self.parse_behavior_header()?;
                // Skip tokens until the next discourse marker or EOF
                while let Some(t) = self.peek_token() {
                    if matches!(t, Token::TheModuleCalled | Token::TheShape | Token::TheBehaviorCalled | Token::TheEffectBehaviorCalled) {
                        break;
                    }
                    self.pos += 1;
                }
                Ok(Discourse::Behavior { header, body: Expression::Nothing })
            },
            _ => Err(OnuError::ParseError {
                message: format!("Unexpected token: {:?}", token),
                span: self.current_span(),
            }),
        }
    }

    fn parse_module(&mut self) -> Result<Discourse, OnuError> {
        self.consume(Token::TheModuleCalled)?;
        let name = self.consume_identifier(false)?;
        self.consume(Token::WithConcern)?;
        self.consume(Token::Colon)?;
        
        let mut concern = String::new();
        while let Some(token) = self.peek_token() {
            if matches!(token, Token::TheModuleCalled | Token::TheShape | Token::TheBehaviorCalled | Token::TheEffectBehaviorCalled) {
                break;
            }
            if !concern.is_empty() {
                concern.push(' ');
            }
            concern.push_str(&self.consume_identifier(false)?);
        }
        
        Ok(Discourse::Module { name, concern })
    }

    fn parse_shape(&mut self) -> Result<Discourse, OnuError> {
        self.consume(Token::TheShape)?;
        let name = self.consume_identifier(false)?;
        self.consume(Token::Promises)?;
        self.consume(Token::Colon)?;
        let mut behaviors = Vec::new();
        while let Some(token) = self.peek_token() {
            if matches!(token, Token::TheBehaviorCalled | Token::TheEffectBehaviorCalled) {
                // Peek ahead to see if this behavior has an 'as' clause.
                // If it does, it's a top-level behavior, not part of the shape.
                if self.header_has_as_clause() {
                    break;
                }
                behaviors.push(self.parse_behavior_header()?);
            } else {
                break;
            }
        }
        Ok(Discourse::Shape { name, behaviors })
    }

    /// Peeks ahead to see if the current behavior header is followed by an 'as' clause.
    fn header_has_as_clause(&self) -> bool {
        let mut offset = 1; // Start after the discourse marker
        while let Some(t) = self.peek_ahead(offset) {
            if matches!(t, Token::TheModuleCalled | Token::TheShape | Token::TheBehaviorCalled | Token::TheEffectBehaviorCalled) {
                return false; // Found next discourse unit before 'as'
            }
            if matches!(t, Token::As) {
                return true;
            }
            offset += 1;
        }
        false
    }

    fn parse_behavior(&mut self) -> Result<Discourse, OnuError> {
        let start_span = self.current_span();
        let header = self.parse_behavior_header()?;
        self.is_pure_context = !header.is_effect;
        
        self.consume(Token::As)?;
        self.consume(Token::Colon)?;
        
        let mut expressions = Vec::new();
        while let Some(token) = self.peek_token() {
            if matches!(token, Token::TheModuleCalled | Token::TheShape | Token::TheBehaviorCalled | Token::TheEffectBehaviorCalled) {
                break;
            }
            if matches!(token, Token::Derivation | Token::Let | Token::If) {
                expressions.push(self.parse_primary()?);
            } else {
                expressions.push(self.parse_expression()?);
            }
        }
        
        let body = if expressions.len() == 1 {
            expressions.pop().unwrap()
        } else {
            Expression::Block(expressions)
        };

        if header.delivers.0 == OnuType::Nothing {
            let is_yielding = match body {
                Expression::I8(_) | Expression::I16(_) | Expression::I32(_) | Expression::I64(_) | Expression::I128(_) |
                Expression::U8(_) | Expression::U16(_) | Expression::U32(_) | Expression::U64(_) | Expression::U128(_) |
                Expression::F32(_) | Expression::F64(_) | Expression::Text(_) | Expression::Boolean(_) | Expression::Identifier(_) => true,
                Expression::Block(ref exprs) => {
                    if let Some(last) = exprs.last() {
                        matches!(last, Expression::I8(_) | Expression::I16(_) | Expression::I32(_) | Expression::I64(_) | Expression::I128(_) |
                                          Expression::U8(_) | Expression::U16(_) | Expression::U32(_) | Expression::U64(_) | Expression::U128(_) |
                                          Expression::F32(_) | Expression::F64(_) | Expression::Text(_) | Expression::Boolean(_) | Expression::Identifier(_))
                    } else {
                        false
                    }
                }
                _ => false,
            };

            if is_yielding {
                return Err(OnuError::ParseError {
                    message: "Behavior body yields a value but 'delivers nothing' was specified.".to_string(),
                    span: start_span,
                });
            }
        }
        
        Ok(Discourse::Behavior { header, body })
    }

    /// Parses an expression using SVO (Subject-Verb-Object) Infix topology.
    pub fn parse_expression(&mut self) -> Result<Expression, OnuError> {
        self.enter_expression()?;

        let mut left = self.parse_primary()?;
        
        while let Some(token) = self.peek_token() {
            match token {
                Token::Utilizes | Token::Identifier(_) | 
                Token::Matches | Token::Exceeds | Token::FallsShortOf | 
                Token::ScalesBy | Token::PartitionsBy | 
                Token::UnitesWith | Token::JoinsWith | Token::Opposes | 
                Token::DecreasedBy | Token::InitOf | Token::TailOf => {
                    let name = match token {
                        Token::Utilizes => {
                            self.pos += 1;
                            self.consume_identifier(false)?
                        }
                        Token::Identifier(ref n) => n.clone(),
                        Token::Matches => "matches".to_string(),
                        Token::Exceeds => "exceeds".to_string(),
                        Token::FallsShortOf => "falls-short-of".to_string(),
                        Token::ScalesBy => "scales-by".to_string(),
                        Token::PartitionsBy => "partitions-by".to_string(),
                        Token::UnitesWith => "unites-with".to_string(),
                        Token::JoinsWith => "joins-with".to_string(),
                        Token::Opposes => "opposes".to_string(),
                        Token::DecreasedBy => "decreased-by".to_string(),
                        Token::InitOf => "init-of".to_string(),
                        Token::TailOf => "tail-of".to_string(),
                        _ => unreachable!(),
                    };

                    if let Some(registry) = self.registry {
                        if registry.is_registered(&name) {
                            if !matches!(token, Token::Utilizes) {
                                self.pos += 1;
                            }
                            let arity = registry.get_arity(&name).unwrap_or(0);
                            let mut args = Vec::new();
                            args.push(left);
                            
                            for _ in 0..(arity.saturating_sub(1)) {
                                args.push(self.parse_primary()?);
                            }
                            left = Expression::BehaviorCall { name, args };
                            continue;
                        }
                    }
                }
                Token::ActsAs => {
                    self.pos += 1; // Consume ActsAs
                    // Optional article: a, an, the
                    if let Some(Token::A | Token::An | Token::The) = self.peek_token() {
                        self.pos += 1;
                    }
                    let shape = self.consume_identifier(false)?;
                    left = Expression::ActsAs { subject: Box::new(left), shape };
                    continue;
                }
                _ => {}
            }
            break;
        }
        self.exit_expression();
        Ok(left)
    }

    /// Parses primary expressions (literals, variables, keywords with markers, parenthesized expressions).
    fn parse_primary(&mut self) -> Result<Expression, OnuError> {
        let span = self.current_span();
        match self.peek_token() {
            Some(Token::NumericLiteral(n)) => {
                self.pos += 1;
                Ok(Expression::F64(n))
            }
            Some(Token::IntegerLiteral(n)) => {
                self.pos += 1;
                // Default parsing of generic integer literal to I64 for now.
                // Cast safety: i128 to i64 might overflow, but for literals in most samples it's safe.
                Ok(Expression::I64(n as i64))
            }
            Some(Token::BooleanLiteral(b)) => {
                self.pos += 1;
                Ok(Expression::Boolean(b))
            }
            Some(Token::TextLiteral(s)) => {
                self.pos += 1;
                Ok(Expression::Text(s))
            }
            Some(Token::Nothing) => {
                self.pos += 1;
                Ok(Expression::Nothing)
            }
            Some(Token::LParen) => {
                self.pos += 1;
                // Handle Tuples: (expr, expr, ...) or just (expr)
                let mut exprs = Vec::new();
                if let Some(Token::RParen) = self.peek_token() {
                    self.pos += 1;
                    return Ok(Expression::Tuple(exprs));
                }
                
                exprs.push(self.parse_expression()?);
                
                let mut is_tuple = false;
                while let Some(Token::Colon) = self.peek_token() { // Using Colon as separator for now if it makes sense in discourse
                    self.pos += 1;
                    exprs.push(self.parse_expression()?);
                    is_tuple = true;
                }
                
                self.consume(Token::RParen)?;
                
                if is_tuple {
                    Ok(Expression::Tuple(exprs))
                } else {
                    Ok(exprs.pop().unwrap())
                }
            }
            Some(Token::LBracket) => {
                self.pos += 1;
                let mut data = Vec::new();
                let mut rows = 1;
                let mut cols = 0;
                let mut current_row_cols = 0;
                let mut is_matrix = false;

                while let Some(token) = self.peek_token() {
                    if token == Token::RBracket { break; }
                    
                    if token == Token::Colon {
                        self.pos += 1;
                        is_matrix = true;
                        if cols == 0 {
                            cols = current_row_cols;
                        } else if current_row_cols != cols {
                            return Err(OnuError::ParseError {
                                message: format!("Matrix Error: Inconsistent column count. Row 1 has {} columns, but Row {} has {}.", cols, rows, current_row_cols),
                                span: self.current_span(),
                            });
                        }
                        rows += 1;
                        current_row_cols = 0;
                        continue;
                    }

                    data.push(self.parse_expression()?);
                    current_row_cols += 1;
                }
                self.consume(Token::RBracket)?;

                if is_matrix {
                    if cols == 0 { cols = current_row_cols; }
                    else if current_row_cols != cols {
                         return Err(OnuError::ParseError {
                            message: format!("Matrix Error: Inconsistent column count in final row. Expected {}, found {}.", cols, current_row_cols),
                            span: self.current_span(),
                        });
                    }
                    Ok(Expression::Matrix { rows, cols, data })
                } else {
                    Ok(Expression::Array(data))
                }
            }
            Some(Token::Emit) | Some(Token::Broadcasts) => {
                let token = self.peek_token().unwrap();
                if self.is_pure_context {
                    let name = if token == Token::Emit { "emit" } else { "broadcasts" };
                    return Err(OnuError::ParseError {
                        message: format!("Side-effect '{}' is not allowed in a pure behavior. Use 'the effect behavior called...'.", name),
                        span: span,
                    });
                }
                self.pos += 1;
                let value = Box::new(self.parse_expression()?);
                if token == Token::Emit {
                    Ok(Expression::Emit(value))
                } else {
                    Ok(Expression::Broadcasts(value))
                }
            }
            Some(Token::Derivation) => {
                self.consume(Token::Derivation)?;
                self.consume(Token::Colon)?;
                let name = self.consume_identifier(true)?;
                self.consume(Token::DerivesFrom)?;

                // Optional type info
                let type_info = match self.peek_token() {
                    Some(Token::A) | Some(Token::An) | Some(Token::The) | Some(Token::Nothing) => Some(self.parse_type_info()?),
                    _ => None,
                };

                let value = Box::new(self.parse_expression()?); 
                
                // KISS Principle: extracting a named intermediate should not penalize the depth of subsequent logic.
                let saved_depth = self.current_depth;
                self.current_depth = 1; // Fresh start for the body

                let mut body_exprs = Vec::new();
                while let Some(token) = self.peek_token() {
                    if self.is_terminator(&token) { break; }
                    body_exprs.push(self.parse_expression()?);
                }
                
                self.current_depth = saved_depth; // Restore depth for parent context

                let body = if body_exprs.is_empty() {
                    Box::new(Expression::Nothing)
                } else if body_exprs.len() == 1 {
                    Box::new(body_exprs.pop().unwrap())
                } else {
                    Box::new(Expression::Block(body_exprs))
                };
                
                Ok(Expression::Derivation { name, type_info, value, body })
            }
            Some(Token::Let) => {
                // Map 'let' to Derivation AST
                self.consume(Token::Let)?;
                let name = self.consume_identifier(true)?;
                self.consume(Token::Is)?;
                let type_info = Some(self.parse_type_info()?);
                let value = Box::new(self.parse_expression()?); 
                
                let saved_depth = self.current_depth;
                self.current_depth = 1;
                let mut body_exprs = Vec::new();
                while let Some(token) = self.peek_token() {
                    if self.is_terminator(&token) { break; }
                    body_exprs.push(self.parse_expression()?);
                }
                self.current_depth = saved_depth;
                let body = if body_exprs.is_empty() { Box::new(Expression::Nothing) } 
                           else if body_exprs.len() == 1 { Box::new(body_exprs.pop().unwrap()) } 
                           else { Box::new(Expression::Block(body_exprs)) };
                
                Ok(Expression::Derivation { name, type_info, value, body })
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
            Some(Token::Identifier(s)) => {
                // SVO Enforcement: Prefix usage of registered behaviors is forbidden,
                // UNLESS they take zero arguments (act as constants/propositions).
                if let Some(registry) = self.registry {
                    if registry.is_registered(&s) {
                        let arity = registry.get_arity(&s).unwrap_or(0);
                        if arity > 0 {
                            return Err(OnuError::ParseError {
                                message: format!("The behavior '{}' refuses to be used as a prefix. Please utilize Subject-Verb-Object (SVO) grammar.", s),
                                span,
                            });
                        } else {
                            // Arity 0: Treat as an immediate call
                            self.pos += 1;
                            return Ok(Expression::BehaviorCall { name: s, args: vec![] });
                        }
                    }
                }
                self.pos += 1;
                Ok(Expression::Identifier(s))
            }
            Some(Token::A) => {
                self.pos += 1;
                Ok(Expression::Identifier("a".to_string()))
            }
            Some(Token::An) => {
                self.pos += 1;
                Ok(Expression::Identifier("an".to_string()))
            }
            Some(Token::Integer) | Some(Token::Float) | Some(Token::RealNumber) | Some(Token::Strings) | Some(Token::Matrix) => {
                let token = self.peek_token().unwrap();
                self.pos += 1;
                Err(OnuError::ParseError {
                    message: format!("Unexpected keyword in primary expression: {:?}. Specific types cannot be used as variable names.", token),
                    span,
                })
            }
            Some(token) => Err(OnuError::ParseError {
                message: format!("Expected primary expression, found {:?}", token),
                span,
            }),
            None => Err(OnuError::ParseError {
                message: "Expected primary expression, found EOF".to_string(),
                span,
            }),
        }
    }

    fn is_terminator(&self, token: &Token) -> bool {
        matches!(token, Token::RParen | Token::RBracket | Token::Returning | Token::Delivers | Token::As | Token::Then | Token::Else | 
                       Token::TheModuleCalled | Token::TheShape | Token::TheBehaviorCalled | Token::TheEffectBehaviorCalled |
                       Token::WithIntent | Token::Receiving | Token::Takes | Token::WithDiminishing | Token::NoGuaranteedTermination |
                       Token::Promises | Token::WithConcern)
    }

    pub fn parse_behavior_header(&mut self) -> Result<BehaviorHeader, OnuError> {
        let is_effect = if let Some(Token::TheEffectBehaviorCalled) = self.peek_token() {
            self.consume(Token::TheEffectBehaviorCalled)?;
            true
        } else {
            self.consume(Token::TheBehaviorCalled)?;
            false
        };

        let name = self.consume_identifier(false)?;
        
        let mut intent = String::new();
        if let Some(Token::WithIntent) = self.peek_token() {
            self.consume(Token::WithIntent)?;
            self.consume(Token::Colon)?;
            while let Some(token) = self.peek_token() {
                if matches!(token, Token::Receiving | Token::Takes | Token::Returning | Token::Delivers | Token::WithDiminishing | Token::NoGuaranteedTermination | Token::As) {
                    break;
                }
                if !intent.is_empty() {
                    intent.push(' ');
                }
                intent.push_str(&self.consume_identifier(false)?);
            }
        }
        
        let mut takes = Vec::new();
        if let Some(Token::Takes) = self.peek_token() {
            self.consume(Token::Takes)?;
            if let Some(Token::Colon) = self.peek_token() {
                self.consume(Token::Colon)?;
            }
        } else {
            self.consume(Token::Receiving)?;
            self.consume(Token::Colon)?;
        }
        
        // Handle explicit 'receiving: nothing' or 'takes: nothing'
        if let Some(Token::Nothing) = self.peek_token() {
            self.consume(Token::Nothing)?;
        } else {
            while let Some(token) = self.peek_token() {
                if matches!(token, Token::Returning | Token::Delivers | Token::As | Token::WithDiminishing | Token::NoGuaranteedTermination) {
                    break;
                }
                
                let mut type_info = self.parse_type_info()?;
                
                if let Some(Token::Called) = self.peek_token() {
                    self.consume(Token::Called)?;
                } else if let Some(Token::Identifier(ref s)) = self.peek_token() {
                    if s == "called" {
                        self.pos += 1;
                    }
                }
                
                let var_name = self.consume_identifier(true)?;

                if let Some(Token::Via) = self.peek_token() {
                    self.consume(Token::Via)?;
                    self.consume(Token::The)?;
                    self.consume(Token::Role)?;
                    type_info.via_role = Some(self.consume_identifier(false)?);
                }

                takes.push(Argument {
                    name: var_name,
                    type_info,
                });
            }
        }

        if let Some(Token::Delivers) = self.peek_token() {
            self.consume(Token::Delivers)?;
        } else {
            self.consume(Token::Returning)?;
        }

        if let Some(Token::Colon) = self.peek_token() {
            self.consume(Token::Colon)?;
        }
        
        let type_info = self.parse_type_info()?;
        let returning = ReturnType(type_info.onu_type);

        let mut diminishing = None;
        let mut skip_termination_check = false;

        if let Some(Token::WithDiminishing) = self.peek_token() {
            self.consume(Token::WithDiminishing)?;
            self.consume(Token::Colon)?;
            diminishing = Some(self.consume_identifier(true)?);
        } else if let Some(Token::NoGuaranteedTermination) = self.peek_token() {
            self.consume(Token::NoGuaranteedTermination)?;
            skip_termination_check = true;
        }

        Ok(BehaviorHeader {
            name,
            is_effect,
            intent,
            takes,
            delivers: returning,
            diminishing,
            skip_termination_check,
        })
    }

    fn peek_token(&self) -> Option<Token> {
        self.tokens.get(self.pos).map(|t| t.token.clone())
    }

    fn peek_ahead(&self, offset: usize) -> Option<Token> {
        self.tokens.get(self.pos + offset).map(|t| t.token.clone())
    }

    fn parse_type_info(&mut self) -> Result<TypeInfo, OnuError> {
        // Check for explicit 'nothing' first
        if self.peek_token() == Some(Token::Nothing) {
            self.consume(Token::Nothing)?;
            return Ok(TypeInfo {
                onu_type: OnuType::Nothing,
                display_name: "nothing".to_string(),
                article: Token::Nothing,
                via_role: None,
            });
        }

        let article = match self.peek_token() {
            Some(Token::A) => { self.consume(Token::A)?; Token::A }
            Some(Token::An) => { self.consume(Token::An)?; Token::An }
            Some(Token::The) => { self.consume(Token::The)?; Token::The }
            _ => Token::Nothing, // Optional
        };

        let type_name = self.consume_identifier(false)?;
        
        // Optional 'of' clause: a tuple of (string, string)
        if let Some(Token::Of) = self.peek_token() {
            self.consume(Token::Of)?;
        }

        let onu_type = match type_name.as_str() {
            "tuple" => {
                self.consume(Token::LParen)?;
                let mut types = Vec::new();
                loop {
                    types.push(self.parse_type_info()?.onu_type);
                    if let Some(Token::Colon) = self.peek_token() {
                        self.consume(Token::Colon)?;
                    } else if let Some(Token::Identifier(ref s)) = self.peek_token() {
                        if s == "," { self.pos += 1; }
                        else { break; }
                    } else {
                        break;
                    }
                }
                self.consume(Token::RParen)?;
                OnuType::Tuple(types)
            }
            "array" => {
                self.consume(Token::Of)?;
                let inner = self.parse_type_info()?;
                OnuType::Array(Box::new(inner.onu_type))
            }
            _ => OnuType::from_name(&type_name).unwrap_or(OnuType::Shape(type_name.clone())),
        };

        Ok(TypeInfo {
            onu_type,
            display_name: type_name,
            article,
            via_role: None,
        })
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

    fn consume_identifier(&mut self, restricted: bool) -> Result<String, OnuError> {
        let span = self.current_span();
        match self.tokens.get(self.pos) {
            Some(t) => {
                let res = match t.token {
                    Token::Identifier(ref name) => {
                        if restricted {
                            if let Some(registry) = self.registry {
                                if registry.is_registered(name) {
                                    return Err(OnuError::ParseError {
                                        message: format!("Ambiguous identifier '{}': Name is already used by a registered behavior.", name),
                                        span,
                                    });
                                }
                            }
                        }
                        name.clone()
                    },
                    Token::Integer => "integer".to_string(),
                    Token::Float => "float".to_string(),
                    Token::RealNumber => "realnumber".to_string(),
                    Token::Strings => "string".to_string(),
                    Token::Matrix => "matrix".to_string(),
                    Token::Nothing => "nothing".to_string(),
                    Token::The => "the".to_string(),
                    Token::With => "with".to_string(),
                    Token::If => "if".to_string(),
                    Token::Is => "is".to_string(),
                    Token::Called => "called".to_string(),
                    Token::As => "as".to_string(),
                    Token::Emit => "emit".to_string(),
                    Token::Broadcasts => "broadcasts".to_string(),
                    Token::A => "a".to_string(),
                    Token::An => "an".to_string(),
                    Token::Of => "of".to_string(),
                    Token::Derivation => "derivation".to_string(),
                    Token::DerivesFrom => "derives-from".to_string(),
                    Token::Takes => "takes".to_string(),
                    Token::Delivers => "delivers".to_string(),
                    Token::Utilizes => "utilizes".to_string(),
                    Token::ActsAs => "acts-as".to_string(),
                    Token::Matches => "matches".to_string(),
                    Token::Exceeds => "exceeds".to_string(),
                    Token::FallsShortOf => "falls-short-of".to_string(),
                    Token::ScalesBy => "scales-by".to_string(),
                    Token::PartitionsBy => "partitions-by".to_string(),
                    Token::UnitesWith => "unites-with".to_string(),
                    Token::JoinsWith => "joins-with".to_string(),
                    Token::Opposes => "opposes".to_string(),
                    Token::DecreasedBy => "decreased-by".to_string(),
                    Token::InitOf => "init-of".to_string(),
                    Token::TailOf => "tail-of".to_string(),
                    Token::NumericLiteral(n) => n.to_string(),
                    Token::IntegerLiteral(n) => n.to_string(),
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
            t(Token::Identifier("recursion".to_string())),
        ];
        let mut parser = Parser::new(&tokens);
        let result = parser.parse_discourse().unwrap();
        assert_eq!(
            result,
            Discourse::Module {
                name: "MeasurementDomain".to_string(),
                concern: "recursion".to_string()
            }
        );
    }

    #[test]
    fn test_parser_accesses_registry() {
        let tokens = vec![
            t(Token::Identifier("foo".to_string())),
        ];
        let mut registry = Registry::new();
        registry.add_name("foo", 1);
        let parser = Parser::with_registry(&tokens, &registry);
        assert!(parser.registry.unwrap().is_registered("foo"));
    }

    #[test]
    fn test_parse_svo_infix() {
        let tokens = vec![
            t(Token::IntegerLiteral(5)),
            t(Token::Identifier("multiplied-by".to_string())),
            t(Token::IntegerLiteral(2)),
        ];
        let mut registry = Registry::new();
        registry.add_name("multiplied-by", 2);
        let mut parser = Parser::with_registry(&tokens, &registry);
        let result = parser.parse_expression().unwrap();
        
        assert_eq!(
            result,
            Expression::BehaviorCall {
                name: "multiplied-by".to_string(),
                args: vec![Expression::I64(5), Expression::I64(2)],
            }
        );
    }

    #[test]
    fn test_parse_single_arg_verb() {
        let tokens = vec![
            t(Token::Identifier("angle".to_string())),
            t(Token::Identifier("sine".to_string())),
        ];
        let mut registry = Registry::new();
        registry.add_name("sine", 1);
        let mut parser = Parser::with_registry(&tokens, &registry);
        let result = parser.parse_expression().unwrap();
        
        assert_eq!(
            result,
            Expression::BehaviorCall {
                name: "sine".to_string(),
                args: vec![Expression::Identifier("angle".to_string())],
            }
        );
    }

    #[test]
    fn test_parse_prefix_fail() {
        let tokens = vec![
            t(Token::Identifier("scales-by".to_string())),
            t(Token::IntegerLiteral(5)),
            t(Token::IntegerLiteral(2)),
        ];
        let mut registry = Registry::new();
        registry.add_name("scales-by", 2);
        let mut parser = Parser::with_registry(&tokens, &registry);
        let result = parser.parse_expression();
        
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("refuses to be used as a prefix"));
    }

    #[test]
    fn test_parse_nested_infix() {
        let tokens = vec![
            t(Token::IntegerLiteral(5)),
            t(Token::Identifier("added-to".to_string())),
            t(Token::IntegerLiteral(2)),
            t(Token::Identifier("multiplied-by".to_string())),
            t(Token::IntegerLiteral(3)),
        ];
        let mut registry = Registry::new();
        registry.add_name("added-to", 2);
        registry.add_name("multiplied-by", 2);
        let mut parser = Parser::with_registry(&tokens, &registry);
        let result = parser.parse_expression().unwrap();
        
        assert_eq!(
            result,
            Expression::BehaviorCall {
                name: "multiplied-by".to_string(),
                args: vec![
                    Expression::BehaviorCall {
                        name: "added-to".to_string(),
                        args: vec![Expression::I64(5), Expression::I64(2)],
                    },
                    Expression::I64(3)
                ],
            }
        );
    }

    #[test]
    fn test_parse_shadowing_fail() {
        let tokens = vec![
            t(Token::Let),
            t(Token::Identifier("multiplied-by".to_string())),
            t(Token::Is),
            t(Token::IntegerLiteral(42)),
        ];
        let mut registry = Registry::new();
        registry.add_name("multiplied-by", 2);
        let mut parser = Parser::with_registry(&tokens, &registry);
        let result = parser.parse_expression();
        
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Ambiguous identifier 'multiplied-by'"));
    }

    #[test]
    fn test_parse_takes_shadowing_fail() {
        let tokens = vec![
            t(Token::TheBehaviorCalled), t(Token::Identifier("factorial".to_string())),
            t(Token::Takes), t(Token::Colon),
            t(Token::An), t(Token::Integer), t(Token::Identifier("multiplied-by".to_string())),
            t(Token::Delivers), t(Token::Nothing),
            t(Token::As), t(Token::Colon),
            t(Token::Nothing),
        ];
        let mut registry = Registry::new();
        registry.add_name("multiplied-by", 2);
        let mut parser = Parser::with_registry(&tokens, &registry);
        let result = parser.parse_discourse();
        
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Ambiguous identifier 'multiplied-by'"));
    }

    #[test]
    fn test_parse_emit_in_pure_fail() {
        let tokens = vec![
            t(Token::TheBehaviorCalled), t(Token::Identifier("test".to_string())),
            t(Token::Receiving), t(Token::Colon),
            t(Token::Returning), t(Token::Nothing),
            t(Token::As), t(Token::Colon),
            t(Token::Emit), t(Token::IntegerLiteral(1)),
        ];
        let mut parser = Parser::new(&tokens);
        let result = parser.parse_discourse();
        
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Side-effect 'emit' is not allowed"));
    }

    #[test]
    fn test_parse_emit_in_effect_pass() {
        let tokens = vec![
            t(Token::TheEffectBehaviorCalled), t(Token::Identifier("test".to_string())),
            t(Token::Receiving), t(Token::Colon),
            t(Token::Returning), t(Token::Nothing),
            t(Token::As), t(Token::Colon),
            t(Token::Emit), t(Token::IntegerLiteral(1)),
        ];
        let mut parser = Parser::new(&tokens);
        let result = parser.parse_discourse();
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_active_tense_derivation() {
        let tokens = vec![
            t(Token::Derivation),
            t(Token::Colon),
            t(Token::Identifier("x".to_string())),
            t(Token::DerivesFrom),
            t(Token::IntegerLiteral(10)),
            t(Token::Identifier("x".to_string())),
        ];
        let mut parser = Parser::new(&tokens);
        let result = parser.parse_expression().unwrap();
        
        if let Expression::Derivation { name, value, .. } = result {
            assert_eq!(name, "x");
            assert_eq!(*value, Expression::I64(10));
        } else {
            panic!("Expected Derivation, found {:?}", result);
        }
    }

    #[test]
    fn test_parse_acts_as_with_article() {
        let tokens = vec![
            t(Token::Identifier("x".to_string())),
            t(Token::ActsAs),
            t(Token::A),
            t(Token::Identifier("Measurable".to_string())),
        ];
        let mut parser = Parser::new(&tokens);
        let result = parser.parse_expression().unwrap();
        
        if let Expression::ActsAs { subject, shape } = result {
            assert_eq!(*subject, Expression::Identifier("x".to_string()));
            assert_eq!(shape, "Measurable");
        } else {
            panic!("Expected ActsAs, found {:?}", result);
        }
    }

    #[test]
    fn test_parse_utilizes_call() {
        let tokens = vec![
            t(Token::Identifier("m".to_string())),
            t(Token::Utilizes),
            t(Token::Identifier("ackermann".to_string())),
            t(Token::Identifier("n".to_string())),
        ];
        let mut registry = Registry::new();
        registry.add_name("ackermann", 2);
        let mut parser = Parser::with_registry(&tokens, &registry);
        let result = parser.parse_expression().unwrap();
        
        assert_eq!(
            result,
            Expression::BehaviorCall {
                name: "ackermann".to_string(),
                args: vec![Expression::Identifier("m".to_string()), Expression::Identifier("n".to_string())],
            }
        );
    }

    #[test]
    fn test_parse_active_behavior_header() {
        let tokens = vec![
            t(Token::TheBehaviorCalled),
            t(Token::Identifier("compute".to_string())),
            t(Token::WithIntent),
            t(Token::Colon),
            t(Token::Identifier("do".to_string())),
            t(Token::Identifier("work".to_string())),
            t(Token::Takes),
            t(Token::Colon),
            t(Token::A),
            t(Token::Integer),
            t(Token::Called),
            t(Token::Identifier("input".to_string())),
            t(Token::Delivers),
            t(Token::An),
            t(Token::Integer),
            t(Token::As),
            t(Token::Colon),
            t(Token::Identifier("input".to_string())),
        ];
        let mut parser = Parser::new(&tokens);
        let result = parser.parse_discourse().unwrap();
        
        if let Discourse::Behavior { header, .. } = result {
            assert_eq!(header.name, "compute");
            assert_eq!(header.intent, "do work");
            assert_eq!(header.takes.len(), 1);
            assert_eq!(header.takes[0].name, "input");
        } else {
            panic!("Expected Behavior, found {:?}", result);
        }
    }
}
