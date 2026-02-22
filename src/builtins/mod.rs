use crate::interpreter::Value;
use crate::env::Environment;
use crate::error::OnuError;
use std::fmt;
use std::collections::HashMap;

pub mod math;
pub mod logic;
pub mod strings;
pub mod comparison;
pub mod math_adv;

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
    builtins.insert("scales-by".to_string(), Box::new(math::Mul));
    builtins.insert("partitions-by".to_string(), Box::new(math::Div));
    
    builtins.insert("unites-with".to_string(), Box::new(logic::BothTrue));
    builtins.insert("joins-with".to_string(), Box::new(logic::EitherTrue));
    builtins.insert("opposes".to_string(), Box::new(logic::NotTrue));

    builtins.insert("matches".to_string(), Box::new(comparison::IsEqualTo));
    builtins.insert("exceeds".to_string(), Box::new(comparison::IsGreaterThan));
    builtins.insert("falls-short-of".to_string(), Box::new(comparison::IsLessThan));
    
    builtins.insert("joined-with".to_string(), Box::new(strings::Join));
    builtins.insert("len".to_string(), Box::new(strings::Len));
    builtins.insert("char-at".to_string(), Box::new(strings::CharAt));
    builtins.insert("as-text".to_string(), Box::new(strings::AsText));
    builtins.insert("set-char".to_string(), Box::new(strings::SetChar));
    builtins.insert("tail-of".to_string(), Box::new(strings::TailOf));
    builtins.insert("init-of".to_string(), Box::new(strings::InitOf));
    builtins.insert("char-from-code".to_string(), Box::new(strings::CharFromCode));

    // --- Advanced Math ---
    builtins.insert("sine".to_string(), Box::new(math_adv::Sine));
    builtins.insert("cosine".to_string(), Box::new(math_adv::Cosine));
    builtins.insert("tangent".to_string(), Box::new(math_adv::Tangent));
    builtins.insert("arcsin".to_string(), Box::new(math_adv::ArcSin));
    builtins.insert("arccos".to_string(), Box::new(math_adv::ArcCos));
    builtins.insert("arctan".to_string(), Box::new(math_adv::ArcTan));
    
    builtins.insert("square-root".to_string(), Box::new(math_adv::SquareRoot));
    builtins.insert("raised-to".to_string(), Box::new(math_adv::Power));
    builtins.insert("natural-log".to_string(), Box::new(math_adv::NaturalLog));
    builtins.insert("exponent".to_string(), Box::new(math_adv::Exp));
    
    builtins.insert("dot-product".to_string(), Box::new(math_adv::DotProduct));
    builtins.insert("cross-product".to_string(), Box::new(math_adv::CrossProduct));
    builtins.insert("determinant".to_string(), Box::new(math_adv::Determinant));
    
    builtins
}

// --- Helper Functions for DRY Built-in Implementation ---

pub fn expect_one_number(args: &[Value], op_name: &str) -> Result<f64, OnuError> {
    match args.get(0) {
        Some(v) => v.as_f64().ok_or_else(|| OnuError::RuntimeError {
            message: format!("'{}' requires one number", op_name),
            span: crate::error::Span::default(),
        }),
        _ => Err(OnuError::RuntimeError {
            message: format!("'{}' requires one number", op_name),
            span: crate::error::Span::default(),
        }),
    }
}

pub fn expect_two_numbers(args: &[Value], op_name: &str) -> Result<(f64, f64), OnuError> {
    match (args.get(0), args.get(1)) {
        (Some(v1), Some(v2)) => {
            let f1 = v1.as_f64().ok_or_else(|| OnuError::RuntimeError {
                message: format!("'{}' requires two numbers", op_name),
                span: crate::error::Span::default(),
            })?;
            let f2 = v2.as_f64().ok_or_else(|| OnuError::RuntimeError {
                message: format!("'{}' requires two numbers", op_name),
                span: crate::error::Span::default(),
            })?;
            Ok((f1, f2))
        },
        _ => Err(OnuError::RuntimeError {
            message: format!("'{}' requires two numbers", op_name),
            span: crate::error::Span::default(),
        }),
    }
}

pub fn expect_text_and_number(args: &[Value], op_name: &str) -> Result<(String, f64), OnuError> {
    match (args.get(0), args.get(1)) {
        (Some(Value::Text(s)), Some(v2)) => {
            let n = v2.as_f64().ok_or_else(|| OnuError::RuntimeError {
                message: format!("'{}' requires text and a number", op_name),
                span: crate::error::Span::default(),
            })?;
            Ok((s.clone(), n))
        },
        _ => Err(OnuError::RuntimeError {
            message: format!("'{}' requires text and a number", op_name),
            span: crate::error::Span::default(),
        }),
    }
}
