use crate::builtins::BuiltInFunction;
use crate::interpreter::Value;
use crate::env::Environment;
use crate::error::{OnuError, Span};

fn to_value(b: bool) -> Value {
    Value::Boolean(b)
}

#[derive(Debug)]
pub struct IsEqualTo;
impl BuiltInFunction for IsEqualTo {
    fn call(&self, args: &[Value], _env: &mut dyn Environment) -> Result<Value, OnuError> {
        match (args.get(0), args.get(1)) {
            (Some(v1), Some(v2)) => Ok(to_value(v1 == v2)),
            _ => Err(OnuError::RuntimeError {
                message: "matches requires two arguments".to_string(),
                span: Span::default(),
            }),
        }
    }
}

#[derive(Debug)]
pub struct IsGreaterThan;
impl BuiltInFunction for IsGreaterThan {
    fn call(&self, args: &[Value], _env: &mut dyn Environment) -> Result<Value, OnuError> {
        match (args.get(0), args.get(1)) {
            (Some(v1), Some(v2)) => {
                let f1 = v1.as_f64().ok_or_else(|| OnuError::RuntimeError { message: "exceeds requires numbers".to_string(), span: Span::default() })?;
                let f2 = v2.as_f64().ok_or_else(|| OnuError::RuntimeError { message: "exceeds requires numbers".to_string(), span: Span::default() })?;
                Ok(to_value(f1 > f2))
            }
            _ => Err(OnuError::RuntimeError {
                message: "exceeds requires two arguments".to_string(),
                span: Span::default(),
            }),
        }
    }
}

#[derive(Debug)]
pub struct IsLessThan;
impl BuiltInFunction for IsLessThan {
    fn call(&self, args: &[Value], _env: &mut dyn Environment) -> Result<Value, OnuError> {
        match (args.get(0), args.get(1)) {
            (Some(v1), Some(v2)) => {
                let f1 = v1.as_f64().ok_or_else(|| OnuError::RuntimeError { message: "falls-short-of requires numbers".to_string(), span: Span::default() })?;
                let f2 = v2.as_f64().ok_or_else(|| OnuError::RuntimeError { message: "falls-short-of requires numbers".to_string(), span: Span::default() })?;
                Ok(to_value(f1 < f2))
            }
            _ => Err(OnuError::RuntimeError {
                message: "falls-short-of requires two arguments".to_string(),
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
    fn test_is_equal_to() {
        let mut env = MockEnvironment::new();
        let is_equal_to = IsEqualTo;
        assert_eq!(is_equal_to.call(&[Value::I64(10), Value::I64(10)], &mut env).unwrap(), Value::Boolean(true));
        assert_eq!(is_equal_to.call(&[Value::I64(10), Value::I64(20)], &mut env).unwrap(), Value::Boolean(false));
    }
}
