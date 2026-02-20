use crate::builtins::{BuiltInFunction, expect_text_and_number};
use crate::interpreter::Value;
use crate::env::Environment;
use crate::error::{OnuError, Span};

#[derive(Debug)]
pub struct Join;
impl BuiltInFunction for Join {
    fn call(&self, args: &[Value], _env: &mut dyn Environment) -> Result<Value, OnuError> {
        match (args.get(0), args.get(1)) {
            (Some(v1), Some(v2)) => {
                let s1 = match v1 { Value::Text(s) => s.clone(), Value::Number(n) => n.to_string(), Value::Void => "nothing".to_string() };
                let s2 = match v2 { Value::Text(s) => s.clone(), Value::Number(n) => n.to_string(), Value::Void => "nothing".to_string() };
                Ok(Value::Text(format!("{}{}", s1, s2)))
            }
            _ => Err(OnuError::RuntimeError {
                message: "joined-with requires two arguments".to_string(),
                span: Span::default(),
            }),
        }
    }
}

#[derive(Debug)]
pub struct Len;
impl BuiltInFunction for Len {
    fn call(&self, args: &[Value], _env: &mut dyn Environment) -> Result<Value, OnuError> {
        match args.get(0) {
            Some(Value::Text(s)) => Ok(Value::Number(s.len() as u64)),
            _ => Err(OnuError::RuntimeError {
                message: "len requires a text argument".to_string(),
                span: Span::default(),
            }),
        }
    }
}

#[derive(Debug)]
pub struct CharAt;
impl BuiltInFunction for CharAt {
    fn call(&self, args: &[Value], _env: &mut dyn Environment) -> Result<Value, OnuError> {
        let (s, idx) = expect_text_and_number(args, "char-at")?;
        if let Some(c) = s.chars().nth(idx as usize) {
            Ok(Value::Number(c as u64))
        } else {
            Ok(Value::Number(0))
        }
    }
}

#[derive(Debug)]
pub struct AsText;
impl BuiltInFunction for AsText {
    fn call(&self, args: &[Value], _env: &mut dyn Environment) -> Result<Value, OnuError> {
        match args.get(0) {
            Some(Value::Number(n)) => Ok(Value::Text(n.to_string())),
            Some(Value::Text(s)) => Ok(Value::Text(s.clone())),
            Some(Value::Void) => Ok(Value::Text("nothing".to_string())),
            None => Err(OnuError::RuntimeError {
                message: "as-text requires one argument".to_string(),
                span: Span::default(),
            }),
        }
    }
}

#[derive(Debug)]
pub struct SetChar;
impl BuiltInFunction for SetChar {
    fn call(&self, args: &[Value], _env: &mut dyn Environment) -> Result<Value, OnuError> {
        match (args.get(0), args.get(1), args.get(2)) {
            (Some(Value::Text(s)), Some(Value::Number(idx)), Some(Value::Number(val))) => {
                let mut chars: Vec<char> = s.chars().collect();
                if (*idx as usize) < chars.len() {
                    chars[*idx as usize] = std::char::from_u32(*val as u32).unwrap_or('\0');
                    Ok(Value::Text(chars.into_iter().collect()))
                } else {
                    Ok(Value::Text(s.clone()))
                }
            }
            _ => Err(OnuError::RuntimeError {
                message: "set-char requires text, index, and value".to_string(),
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
    fn test_join() {
        let mut env = MockEnvironment::new();
        let join = Join;
        let args = vec![Value::Text("hello ".to_string()), Value::Text("world".to_string())];
        assert_eq!(join.call(&args, &mut env).unwrap(), Value::Text("hello world".to_string()));
    }

    #[test]
    fn test_len() {
        let mut env = MockEnvironment::new();
        let len = Len;
        let args = vec![Value::Text("abc".to_string())];
        assert_eq!(len.call(&args, &mut env).unwrap(), Value::Number(3));
    }

    #[test]
    fn test_char_at() {
        let mut env = MockEnvironment::new();
        let char_at = CharAt;
        let args = vec![Value::Text("abc".to_string()), Value::Number(1)];
        assert_eq!(char_at.call(&args, &mut env).unwrap(), Value::Number('b' as u64));
    }
}
