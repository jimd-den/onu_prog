use crate::interpreter::Value;
use crate::env::Environment;
use crate::error::OnuError;
use std::fmt;
use std::collections::HashMap;

pub mod math;
pub mod logic;
pub mod strings;
pub mod comparison;

/// BuiltInFunction represents the Strategy pattern for core language operations.
pub trait BuiltInFunction: fmt::Debug + Send + Sync {
    fn call(&self, args: &[Value], env: &mut dyn Environment) -> Result<Value, OnuError>;
}

/// Returns a map of all default built-in strategies.
pub fn default_builtins() -> HashMap<String, Box<dyn BuiltInFunction>> {
    let mut builtins: HashMap<String, Box<dyn BuiltInFunction>> = HashMap::new();
    
    builtins.insert("added-to".to_string(), Box::new(math::Add));
    builtins.insert("decreased-by".to_string(), Box::new(math::Sub));
    builtins.insert("subtracted-from".to_string(), Box::new(math::SubtractedFrom));
    builtins.insert("multiplied-by".to_string(), Box::new(math::Mul));
    builtins.insert("divided-by".to_string(), Box::new(math::Div));
    
    builtins.insert("is-zero".to_string(), Box::new(logic::IsZero));
    builtins.insert("is-less".to_string(), Box::new(logic::IsLess));
    builtins.insert("is-equal".to_string(), Box::new(logic::IsEqual));
    builtins.insert("both-true".to_string(), Box::new(logic::BothTrue));
    builtins.insert("either-true".to_string(), Box::new(logic::EitherTrue));
    builtins.insert("not-true".to_string(), Box::new(logic::NotTrue));

    builtins.insert("is-equal-to".to_string(), Box::new(comparison::IsEqualTo));
    builtins.insert("is-greater-than".to_string(), Box::new(comparison::IsGreaterThan));
    builtins.insert("is-less-than".to_string(), Box::new(comparison::IsLessThan));
    
    builtins.insert("joined-with".to_string(), Box::new(strings::Join));
    builtins.insert("len".to_string(), Box::new(strings::Len));
    builtins.insert("char-at".to_string(), Box::new(strings::CharAt));
    builtins.insert("as-text".to_string(), Box::new(strings::AsText));
    builtins.insert("set-char".to_string(), Box::new(strings::SetChar));
    
    builtins
}

// --- Helper Functions for DRY Built-in Implementation ---

pub fn expect_one_number(args: &[Value], op_name: &str) -> Result<u64, OnuError> {
    match args.get(0) {
        Some(Value::Number(n)) => Ok(*n),
        _ => Err(OnuError::RuntimeError {
            message: format!("'{}' requires one number", op_name),
            span: crate::error::Span::default(),
        }),
    }
}

pub fn expect_two_numbers(args: &[Value], op_name: &str) -> Result<(u64, u64), OnuError> {
    match (args.get(0), args.get(1)) {
        (Some(Value::Number(n1)), Some(Value::Number(n2))) => Ok((*n1, *n2)),
        _ => Err(OnuError::RuntimeError {
            message: format!("'{}' requires two numbers", op_name),
            span: crate::error::Span::default(),
        }),
    }
}

pub fn expect_text_and_number(args: &[Value], op_name: &str) -> Result<(String, u64), OnuError> {
    match (args.get(0), args.get(1)) {
        (Some(Value::Text(s)), Some(Value::Number(n))) => Ok((s.clone(), *n)),
        _ => Err(OnuError::RuntimeError {
            message: format!("'{}' requires text and a number", op_name),
            span: crate::error::Span::default(),
        }),
    }
}
