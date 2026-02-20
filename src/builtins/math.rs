use crate::builtins::{BuiltInFunction, expect_two_numbers};
use crate::interpreter::Value;
use crate::env::Environment;
use crate::error::OnuError;

#[derive(Debug)]
pub struct Add;
impl BuiltInFunction for Add {
    fn call(&self, args: &[Value], _env: &mut dyn Environment) -> Result<Value, OnuError> {
        let (n1, n2) = expect_two_numbers(args, "added-to")?;
        Ok(Value::Number(n1 + n2))
    }
}

#[derive(Debug)]
pub struct Sub;
impl BuiltInFunction for Sub {
    fn call(&self, args: &[Value], _env: &mut dyn Environment) -> Result<Value, OnuError> {
        let (n1, n2) = expect_two_numbers(args, "decreased-by")?;
        Ok(Value::Number(n1.saturating_sub(n2)))
    }
}

#[derive(Debug)]
pub struct SubtractedFrom;
impl BuiltInFunction for SubtractedFrom {
    fn call(&self, args: &[Value], _env: &mut dyn Environment) -> Result<Value, OnuError> {
        let (n1, n2) = expect_two_numbers(args, "subtracted-from")?;
        Ok(Value::Number(n2.saturating_sub(n1)))
    }
}

#[derive(Debug)]
pub struct Mul;
impl BuiltInFunction for Mul {
    fn call(&self, args: &[Value], _env: &mut dyn Environment) -> Result<Value, OnuError> {
        let (n1, n2) = expect_two_numbers(args, "multiplied-by")?;
        Ok(Value::Number(n1 * n2))
    }
}

#[derive(Debug)]
pub struct Div;
impl BuiltInFunction for Div {
    fn call(&self, args: &[Value], _env: &mut dyn Environment) -> Result<Value, OnuError> {
        let (n1, n2) = expect_two_numbers(args, "divided-by")?;
        if n2 == 0 {
            return Err(OnuError::RuntimeError {
                message: "Division by zero".to_string(),
                span: crate::error::Span::default(),
            });
        }
        Ok(Value::Number(n1 / n2))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::env::MockEnvironment;

    #[test]
    fn test_add() {
        let mut env = MockEnvironment::new();
        let add = Add;
        let args = vec![Value::Number(10), Value::Number(20)];
        let res = add.call(&args, &mut env).unwrap();
        assert_eq!(res, Value::Number(30));
    }

    #[test]
    fn test_sub() {
        let mut env = MockEnvironment::new();
        let sub = Sub;
        let args = vec![Value::Number(20), Value::Number(5)];
        let res = sub.call(&args, &mut env).unwrap();
        assert_eq!(res, Value::Number(15));
    }

    #[test]
    fn test_subtracted_from() {
        let mut env = MockEnvironment::new();
        let sub = SubtractedFrom;
        let args = vec![Value::Number(5), Value::Number(20)];
        let res = sub.call(&args, &mut env).unwrap();
        assert_eq!(res, Value::Number(15));
    }

    #[test]
    fn test_mul() {
        let mut env = MockEnvironment::new();
        let mul = Mul;
        let args = vec![Value::Number(5), Value::Number(6)];
        let res = mul.call(&args, &mut env).unwrap();
        assert_eq!(res, Value::Number(30));
    }

    #[test]
    fn test_div() {
        let mut env = MockEnvironment::new();
        let div = Div;
        let args = vec![Value::Number(20), Value::Number(5)];
        let res = div.call(&args, &mut env).unwrap();
        assert_eq!(res, Value::Number(4));
    }
}
