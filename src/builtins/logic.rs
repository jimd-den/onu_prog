use crate::builtins::{BuiltInFunction, expect_one_number, expect_two_numbers};
use crate::interpreter::Value;
use crate::env::Environment;
use crate::error::{OnuError, Span};

fn is_truthy(v: &Value) -> bool {
    match v {
        Value::Number(0) => false,
        Value::Void => false,
        _ => true,
    }
}

fn to_value(b: bool) -> Value {
    if b { Value::Number(1) } else { Value::Number(0) }
}

#[derive(Debug)]
pub struct IsZero;
impl BuiltInFunction for IsZero {
    fn call(&self, args: &[Value], _env: &mut dyn Environment) -> Result<Value, OnuError> {
        let n = expect_one_number(args, "is-zero")?;
        Ok(to_value(n == 0))
    }
}

#[derive(Debug)]
pub struct IsLess;
impl BuiltInFunction for IsLess {
    fn call(&self, args: &[Value], _env: &mut dyn Environment) -> Result<Value, OnuError> {
        let (n1, n2) = expect_two_numbers(args, "is-less")?;
        Ok(to_value(n1 < n2))
    }
}

#[derive(Debug)]
pub struct IsEqual;
impl BuiltInFunction for IsEqual {
    fn call(&self, args: &[Value], _env: &mut dyn Environment) -> Result<Value, OnuError> {
        match (args.get(0), args.get(1)) {
            (Some(v1), Some(v2)) => Ok(to_value(v1 == v2)),
            _ => Err(OnuError::RuntimeError {
                message: "is-equal requires two arguments".to_string(),
                span: Span::default(),
            }),
        }
    }
}

#[derive(Debug)]
pub struct BothTrue;
impl BuiltInFunction for BothTrue {
    fn call(&self, args: &[Value], _env: &mut dyn Environment) -> Result<Value, OnuError> {
        match (args.get(0), args.get(1)) {
            (Some(v1), Some(v2)) => Ok(to_value(is_truthy(v1) && is_truthy(v2))),
            _ => Err(OnuError::RuntimeError {
                message: "both-true requires two arguments".to_string(),
                span: Span::default(),
            }),
        }
    }
}

#[derive(Debug)]
pub struct EitherTrue;
impl BuiltInFunction for EitherTrue {
    fn call(&self, args: &[Value], _env: &mut dyn Environment) -> Result<Value, OnuError> {
        match (args.get(0), args.get(1)) {
            (Some(v1), Some(v2)) => Ok(to_value(is_truthy(v1) || is_truthy(v2))),
            _ => Err(OnuError::RuntimeError {
                message: "either-true requires two arguments".to_string(),
                span: Span::default(),
            }),
        }
    }
}

#[derive(Debug)]
pub struct NotTrue;
impl BuiltInFunction for NotTrue {
    fn call(&self, args: &[Value], _env: &mut dyn Environment) -> Result<Value, OnuError> {
        match args.get(0) {
            Some(v) => Ok(to_value(!is_truthy(v))),
            None => Err(OnuError::RuntimeError {
                message: "not-true requires one argument".to_string(),
                span: Span::default(),
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::env::MockEnvironment;

    #[test]
    fn test_is_zero() {
        let mut env = MockEnvironment::new();
        let is_zero = IsZero;
        assert_eq!(is_zero.call(&[Value::Number(0)], &mut env).unwrap(), Value::Number(1));
        assert_eq!(is_zero.call(&[Value::Number(10)], &mut env).unwrap(), Value::Number(0));
    }

    #[test]
    fn test_is_less() {
        let mut env = MockEnvironment::new();
        let is_less = IsLess;
        assert_eq!(is_less.call(&[Value::Number(5), Value::Number(10)], &mut env).unwrap(), Value::Number(1));
        assert_eq!(is_less.call(&[Value::Number(15), Value::Number(10)], &mut env).unwrap(), Value::Number(0));
    }

    #[test]
    fn test_both_true() {
        let mut env = MockEnvironment::new();
        let both_true = BothTrue;
        assert_eq!(both_true.call(&[Value::Number(1), Value::Number(1)], &mut env).unwrap(), Value::Number(1));
        assert_eq!(both_true.call(&[Value::Number(1), Value::Number(0)], &mut env).unwrap(), Value::Number(0));
    }

    #[test]
    fn test_either_true() {
        let mut env = MockEnvironment::new();
        let either_true = EitherTrue;
        assert_eq!(either_true.call(&[Value::Number(1), Value::Number(0)], &mut env).unwrap(), Value::Number(1));
        assert_eq!(either_true.call(&[Value::Number(0), Value::Number(0)], &mut env).unwrap(), Value::Number(0));
    }

    #[test]
    fn test_not_true() {
        let mut env = MockEnvironment::new();
        let not_true = NotTrue;
        assert_eq!(not_true.call(&[Value::Number(1)], &mut env).unwrap(), Value::Number(0));
        assert_eq!(not_true.call(&[Value::Number(0)], &mut env).unwrap(), Value::Number(1));
    }
}
