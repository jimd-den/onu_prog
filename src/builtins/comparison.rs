use crate::builtins::{BuiltInFunction, expect_two_numbers};
use crate::interpreter::Value;
use crate::env::Environment;
use crate::error::{OnuError, Span};

fn to_value(b: bool) -> Value {
    if b { Value::Number(1) } else { Value::Number(0) }
}

#[derive(Debug)]
pub struct IsEqualTo;
impl BuiltInFunction for IsEqualTo {
    fn call(&self, args: &[Value], _env: &mut dyn Environment) -> Result<Value, OnuError> {
        match (args.get(0), args.get(1)) {
            (Some(v1), Some(v2)) => Ok(to_value(v1 == v2)),
            _ => Err(OnuError::RuntimeError {
                message: "is-equal-to requires two arguments".to_string(),
                span: Span::default(),
            }),
        }
    }
}

#[derive(Debug)]
pub struct IsGreaterThan;
impl BuiltInFunction for IsGreaterThan {
    fn call(&self, args: &[Value], _env: &mut dyn Environment) -> Result<Value, OnuError> {
        let (n1, n2) = expect_two_numbers(args, "is-greater-than")?;
        Ok(to_value(n1 > n2))
    }
}

#[derive(Debug)]
pub struct IsLessThan;
impl BuiltInFunction for IsLessThan {
    fn call(&self, args: &[Value], _env: &mut dyn Environment) -> Result<Value, OnuError> {
        let (n1, n2) = expect_two_numbers(args, "is-less-than")?;
        Ok(to_value(n1 < n2))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::env::MockEnvironment;

    #[test]
    fn test_is_equal_to() {
        let mut env = MockEnvironment::new();
        let is_equal_to = IsEqualTo;
        assert_eq!(is_equal_to.call(&[Value::Number(10), Value::Number(10)], &mut env).unwrap(), Value::Number(1));
        assert_eq!(is_equal_to.call(&[Value::Number(10), Value::Number(20)], &mut env).unwrap(), Value::Number(0));
    }

    #[test]
    fn test_is_greater_than() {
        let mut env = MockEnvironment::new();
        let is_greater_than = IsGreaterThan;
        assert_eq!(is_greater_than.call(&[Value::Number(20), Value::Number(10)], &mut env).unwrap(), Value::Number(1));
        assert_eq!(is_greater_than.call(&[Value::Number(10), Value::Number(20)], &mut env).unwrap(), Value::Number(0));
    }

    #[test]
    fn test_is_less_than() {
        let mut env = MockEnvironment::new();
        let is_less_than = IsLessThan;
        assert_eq!(is_less_than.call(&[Value::Number(10), Value::Number(20)], &mut env).unwrap(), Value::Number(1));
        assert_eq!(is_less_than.call(&[Value::Number(20), Value::Number(10)], &mut env).unwrap(), Value::Number(0));
    }
}
