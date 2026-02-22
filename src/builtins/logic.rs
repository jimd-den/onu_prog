use crate::builtins::BuiltInFunction;
use crate::interpreter::Value;
use crate::env::Environment;
use crate::error::{OnuError, Span};

fn to_value(b: bool) -> Value {
    Value::Boolean(b)
}

#[derive(Debug)]
pub struct IsZero;
impl BuiltInFunction for IsZero {
    fn call(&self, args: &[Value], _env: &mut dyn Environment) -> Result<Value, OnuError> {
        match args.get(0) {
            Some(v) => Ok(to_value(!v.is_truthy())),
            None => Err(OnuError::RuntimeError {
                message: "is-zero requires one argument".to_string(),
                span: Span::default(),
            }),
        }
    }
}

#[derive(Debug)]
pub struct IsLess;
impl BuiltInFunction for IsLess {
    fn call(&self, args: &[Value], _env: &mut dyn Environment) -> Result<Value, OnuError> {
        match (args.get(0), args.get(1)) {
            (Some(v1), Some(v2)) => {
                let f1 = v1.as_f64().ok_or_else(|| OnuError::RuntimeError { message: "is-less requires numbers".to_string(), span: Span::default() })?;
                let f2 = v2.as_f64().ok_or_else(|| OnuError::RuntimeError { message: "is-less requires numbers".to_string(), span: Span::default() })?;
                Ok(to_value(f1 < f2))
            }
            _ => Err(OnuError::RuntimeError {
                message: "is-less requires two arguments".to_string(),
                span: Span::default(),
            }),
        }
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
            (Some(v1), Some(v2)) => Ok(to_value(v1.is_truthy() && v2.is_truthy())),
            _ => Err(OnuError::RuntimeError {
                message: "unites-with requires two arguments".to_string(),
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
            (Some(v1), Some(v2)) => Ok(to_value(v1.is_truthy() || v2.is_truthy())),
            _ => Err(OnuError::RuntimeError {
                message: "joins-with requires two arguments".to_string(),
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
            Some(v) => Ok(to_value(!v.is_truthy())),
            None => Err(OnuError::RuntimeError {
                message: "opposes requires one argument".to_string(),
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
        assert_eq!(is_zero.call(&[Value::I64(0)], &mut env).unwrap(), Value::Boolean(true));
        assert_eq!(is_zero.call(&[Value::I64(10)], &mut env).unwrap(), Value::Boolean(false));
    }
}
