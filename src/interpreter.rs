/// Ọ̀nụ Interpreter: The Use Case Layer
///
/// This module implements the core execution engine for Ọ̀nụ.
/// Following Clean Architecture, the Interpreter encapsulates the business rules
/// of the language — how expressions are evaluated and how behaviors are invoked.
///
/// Architectural Features:
/// - Dependency Inversion: I/O is handled via an injected `Environment` trait.
/// - Strategy Pattern: Built-in operations (math, logic, strings) are implemented
///   as atomic strategies, ensuring the interpreter remains Open for extension.
/// - SRP: Argument parsing for built-ins is delegated to specialized helpers.

use crate::parser::{Discourse, Expression};
use crate::env::Environment;
use crate::error::{OnuError, Span};
use crate::builtins::{default_builtins, BuiltInFunction};
use std::collections::HashMap;

/// The Visitor trait defines a generic interface for traversing the Ọ̀nụ AST.
/// This allows for multiple passes (evaluation, static analysis, etc.) without
/// modifying the AST nodes themselves.
pub trait Visitor<T> {
    fn visit_expression(&mut self, expr: &Expression) -> Result<T, OnuError> {
        match expr {
            Expression::Number(n) => self.visit_number(*n),
            Expression::Text(s) => self.visit_text(s),
            Expression::Identifier(name) => self.visit_identifier(name),
            Expression::Nothing => self.visit_nothing(),
            Expression::Emit(inner) => self.visit_emit(inner),
            Expression::Let { name, value, body } => self.visit_let(name, value, body),
            Expression::BehaviorCall { name, args } => self.visit_behavior_call(name, args),
            Expression::If { condition, then_branch, else_branch } => {
                self.visit_if(condition, then_branch, else_branch)
            }
            Expression::Block(exprs) => self.visit_block(exprs),
        }
    }

    fn visit_number(&mut self, n: u64) -> Result<T, OnuError>;
    fn visit_text(&mut self, s: &str) -> Result<T, OnuError>;
    fn visit_identifier(&mut self, name: &str) -> Result<T, OnuError>;
    fn visit_nothing(&mut self) -> Result<T, OnuError>;
    fn visit_emit(&mut self, expr: &Expression) -> Result<T, OnuError>;
    fn visit_let(&mut self, name: &str, value: &Expression, body: &Expression) -> Result<T, OnuError>;
    fn visit_behavior_call(&mut self, name: &str, args: &[Expression]) -> Result<T, OnuError>;
    fn visit_if(&mut self, condition: &Expression, then_branch: &Expression, else_branch: &Expression) -> Result<T, OnuError>;
    fn visit_block(&mut self, exprs: &[Expression]) -> Result<T, OnuError>;
}

/// The Interpreter evaluates the AST within a given Environment.
pub struct Interpreter {
    /// Variable store for the current scope.
    variables: HashMap<String, Value>,
    /// Registry of user-defined behaviors.
    behaviors: HashMap<String, Discourse>,
    /// Map of built-in strategy objects.
    builtins: HashMap<String, Box<dyn BuiltInFunction>>,
    /// Injected I/O dependency.
    env: Box<dyn Environment>,
}

/// EvaluatorVisitor implements the standard evaluation logic for Ọ̀nụ.
pub struct EvaluatorVisitor<'a> {
    interpreter: &'a mut Interpreter,
}

impl<'a> EvaluatorVisitor<'a> {
    pub fn new(interpreter: &'a mut Interpreter) -> Self {
        Self { interpreter }
    }
}

impl<'a> Visitor<Value> for EvaluatorVisitor<'a> {
    fn visit_number(&mut self, n: u64) -> Result<Value, OnuError> {
        Ok(Value::Number(n))
    }

    fn visit_text(&mut self, s: &str) -> Result<Value, OnuError> {
        Ok(Value::Text(s.to_string()))
    }

    fn visit_identifier(&mut self, name: &str) -> Result<Value, OnuError> {
        Ok(self.interpreter.variables.get(name).cloned().unwrap_or(Value::Void))
    }

    fn visit_nothing(&mut self) -> Result<Value, OnuError> {
        Ok(Value::Void)
    }

    fn visit_emit(&mut self, expr: &Expression) -> Result<Value, OnuError> {
        let val = self.visit_expression(expr)?;
        match val {
            Value::Number(n) => self.interpreter.env.emit(&n.to_string()),
            Value::Text(s) => self.interpreter.env.emit(&s),
            Value::Void => self.interpreter.env.emit("nothing"),
        }
        Ok(Value::Void)
    }

    fn visit_let(&mut self, name: &str, value: &Expression, body: &Expression) -> Result<Value, OnuError> {
        let val = self.visit_expression(value)?;
        let old_val = self.interpreter.variables.insert(name.to_string(), val);
        let res = self.visit_expression(body);
        // Restore scope for side-effect-free binding
        if let Some(v) = old_val {
            self.interpreter.variables.insert(name.to_string(), v);
        } else {
            self.interpreter.variables.remove(name);
        }
        res
    }

    fn visit_behavior_call(&mut self, name: &str, args: &[Expression]) -> Result<Value, OnuError> {
        // Eagerly evaluate arguments
        let mut evaluated_args = Vec::new();
        for arg in args {
            evaluated_args.push(self.visit_expression(arg)?);
        }
        self.interpreter.call_behavior(name, &evaluated_args)
    }

    fn visit_if(&mut self, condition: &Expression, then_branch: &Expression, else_branch: &Expression) -> Result<Value, OnuError> {
        let cond_val = self.visit_expression(condition)?;
        // Truthy: Not Zero and Not Void
        if cond_val != Value::Number(0) && cond_val != Value::Void {
            self.visit_expression(then_branch)
        } else {
            self.visit_expression(else_branch)
        }
    }

    fn visit_block(&mut self, exprs: &[Expression]) -> Result<Value, OnuError> {
        let mut last_val = Value::Void;
        for expr in exprs {
            last_val = self.visit_expression(expr)?;
        }
        Ok(last_val)
    }
}

/// DepthBudgetVisitor enforces the KISS principle by limiting AST depth.
pub struct DepthBudgetVisitor {
    max_depth: usize,
    current_depth: usize,
}

impl DepthBudgetVisitor {
    pub fn new(max_depth: usize) -> Self {
        Self { max_depth, current_depth: 0 }
    }

    fn check_depth(&self) -> Result<(), OnuError> {
        if self.current_depth > self.max_depth {
            return Err(OnuError::RuntimeError {
                message: format!("KISS Violation: Depth budget exceeded ({} > {})", self.current_depth, self.max_depth),
                span: Span::default(),
            });
        }
        Ok(())
    }
}

impl Visitor<()> for DepthBudgetVisitor {
    fn visit_expression(&mut self, expr: &Expression) -> Result<(), OnuError> {
        self.current_depth += 1;
        self.check_depth()?;
        let res = match expr {
            Expression::Number(n) => self.visit_number(*n),
            Expression::Text(s) => self.visit_text(s),
            Expression::Identifier(name) => self.visit_identifier(name),
            Expression::Nothing => self.visit_nothing(),
            Expression::Emit(inner) => self.visit_emit(inner),
            Expression::Let { name, value, body } => self.visit_let(name, value, body),
            Expression::BehaviorCall { name, args } => self.visit_behavior_call(name, args),
            Expression::If { condition, then_branch, else_branch } => {
                self.visit_if(condition, then_branch, else_branch)
            }
            Expression::Block(exprs) => self.visit_block(exprs),
        };
        self.current_depth -= 1;
        res
    }

    fn visit_number(&mut self, _n: u64) -> Result<(), OnuError> { Ok(()) }
    fn visit_text(&mut self, _s: &str) -> Result<(), OnuError> { Ok(()) }
    fn visit_identifier(&mut self, _name: &str) -> Result<(), OnuError> { Ok(()) }
    fn visit_nothing(&mut self) -> Result<(), OnuError> { Ok(()) }
    fn visit_emit(&mut self, expr: &Expression) -> Result<(), OnuError> { self.visit_expression(expr) }
    fn visit_let(&mut self, _name: &str, value: &Expression, body: &Expression) -> Result<(), OnuError> {
        self.visit_expression(value)?;
        self.visit_expression(body)
    }
    fn visit_behavior_call(&mut self, _name: &str, args: &[Expression]) -> Result<(), OnuError> {
        for arg in args {
            self.visit_expression(arg)?;
        }
        Ok(())
    }
    fn visit_if(&mut self, condition: &Expression, then_branch: &Expression, else_branch: &Expression) -> Result<(), OnuError> {
        self.visit_expression(condition)?;
        self.visit_expression(then_branch)?;
        self.visit_expression(else_branch)
    }
    fn visit_block(&mut self, exprs: &[Expression]) -> Result<(), OnuError> {
        for expr in exprs {
            self.visit_expression(expr)?;
        }
        Ok(())
    }
}


/// Values represent the data types available in the Ọ̀nụ runtime.
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Number(u64),
    Text(String),
    Void,
}

impl Interpreter {
    /// Creates a new Interpreter with an injected Environment.
    /// Built-in strategies are initialized automatically.
    pub fn new(env: Box<dyn Environment>) -> Self {
        Self {
            variables: HashMap::new(),
            behaviors: HashMap::new(),
            builtins: default_builtins(),
            env,
        }
    }

    /// Registers a user-defined behavior.
    pub fn register_behavior(&mut self, discourse: Discourse) {
        if let Discourse::Behavior { ref header, .. } = discourse {
            self.behaviors.insert(header.name.clone(), discourse);
        }
    }

    /// Executes a top-level discourse unit.
    pub fn execute_discourse(&mut self, discourse: &Discourse) -> Result<Value, OnuError> {
        match discourse {
            Discourse::Behavior { body, .. } => {
                self.evaluate_expression(body)
            }
            _ => Ok(Value::Void),
        }
    }

    /// Recursively evaluates an AST Expression into a Value.
    pub fn evaluate_expression(&mut self, expr: &Expression) -> Result<Value, OnuError> {
        let mut visitor = EvaluatorVisitor::new(self);
        visitor.visit_expression(expr)
    }

    /// Orchestrates behavior invocation, checking built-ins before user-defined behaviors.
    fn call_behavior(&mut self, name: &str, args: &[Value]) -> Result<Value, OnuError> {
        // Attempt built-in strategy first (Open/Closed enforcement)
        if let Some(builtin) = self.builtins.get(name) {
            return builtin.call(args, self.env.as_mut());
        }

        // Fallback to user-defined behavior
        let behavior = self.behaviors.get(name).cloned();
        if let Some(Discourse::Behavior { header, body }) = behavior {
            let old_variables = self.variables.clone();
            self.variables.clear();
            
            // Agglutinative parameter binding
            for (i, (_type_name, arg_name)) in header.receiving.iter().enumerate() {
                if let Some(val) = args.get(i) {
                    self.variables.insert(arg_name.clone(), val.clone());
                }
            }
            
            let last_val = self.evaluate_expression(&body);
            self.variables = old_variables;
            last_val
        } else {
            Err(OnuError::RuntimeError {
                message: format!("Unknown behavior: {}", name),
                span: Span::default(),
            })
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::env::MockEnvironment;

    #[test]
    fn test_interpreter_let_and_identifier() {
        let env = Box::new(MockEnvironment::new());
        let mut interpreter = Interpreter::new(env);
        let expr = Expression::Let {
            name: "x".to_string(),
            value: Box::new(Expression::Number(42)),
            body: Box::new(Expression::Identifier("x".to_string())),
        };
        let val = interpreter.evaluate_expression(&expr).unwrap();
        assert_eq!(val, Value::Number(42));
    }

    #[test]
    fn test_interpreter_emit() {
        let env = Box::new(MockEnvironment::new());
        let mut interpreter = Interpreter::new(env);
        let expr = Expression::Emit(Box::new(Expression::Text("test".to_string())));
        interpreter.evaluate_expression(&expr).unwrap();
    }

    #[test]
    fn test_evaluator_visitor_basic() {
        let env = Box::new(MockEnvironment::new());
        let mut interpreter = Interpreter::new(env);
        let expr = Expression::Number(123);
        let val = interpreter.evaluate_expression(&expr).unwrap();
        assert_eq!(val, Value::Number(123));
    }

    #[test]
    fn test_depth_budget_visitor_pass() {
        let expr = Expression::Let {
            name: "x".to_string(),
            value: Box::new(Expression::Number(42)),
            body: Box::new(Expression::Identifier("x".to_string())),
        };
        let mut visitor = DepthBudgetVisitor::new(10);
        assert!(visitor.visit_expression(&expr).is_ok());
    }

    #[test]
    fn test_depth_budget_visitor_fail() {
        // Depth 3: Let -> value: Number, body: Identifier
        // But let's make it deeper
        let expr = Expression::Let {
            name: "x".to_string(),
            value: Box::new(Expression::Let {
                name: "y".to_string(),
                value: Box::new(Expression::Number(1)),
                body: Box::new(Expression::Number(2)),
            }),
            body: Box::new(Expression::Identifier("x".to_string())),
        };
        let mut visitor = DepthBudgetVisitor::new(2);
        assert!(visitor.visit_expression(&expr).is_err());
    }
}
