use crate::builtins::BuiltInFunction;
use crate::interpreter::Value;
use crate::env::Environment;
use crate::error::{OnuError, Span};

#[derive(Debug)]
pub struct Join;
impl BuiltInFunction for Join {
    fn call(&self, args: &[Value], _env: &mut dyn Environment) -> Result<Value, OnuError> {
        match (args.get(0), args.get(1)) {
            (Some(v1), Some(v2)) => {
                Ok(Value::Text(format!("{}{}", v1, v2)))
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
            Some(Value::Text(s)) => Ok(Value::I64(s.len() as i64)),
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
        match (args.get(0), args.get(1)) {
            (Some(Value::Text(s)), Some(v2)) => {
                let idx = v2.as_f64().ok_or_else(|| OnuError::RuntimeError { message: "char-at requires a numeric index".to_string(), span: Span::default() })? as usize;
                if let Some(c) = s.chars().nth(idx) {
                    Ok(Value::I64(c as u32 as i64))
                } else {
                    Ok(Value::I64(0))
                }
            }
            _ => Err(OnuError::RuntimeError {
                message: "char-at requires text and an index".to_string(),
                span: Span::default(),
            }),
        }
    }
}

#[derive(Debug)]
pub struct AsText;
impl BuiltInFunction for AsText {
    fn call(&self, args: &[Value], _env: &mut dyn Environment) -> Result<Value, OnuError> {
        match args.get(0) {
            Some(v) => Ok(Value::Text(v.to_string())),
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
            (Some(Value::Text(s)), Some(v_idx), Some(v_val)) => {
                let idx = v_idx.as_f64().ok_or_else(|| OnuError::RuntimeError { message: "set-char index must be numeric".to_string(), span: Span::default() })? as usize;
                let val = v_val.as_f64().ok_or_else(|| OnuError::RuntimeError { message: "set-char value must be numeric".to_string(), span: Span::default() })? as u32;
                let mut chars: Vec<char> = s.chars().collect();
                if idx < chars.len() {
                    chars[idx] = std::char::from_u32(val).unwrap_or('\0');
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
        assert_eq!(len.call(&args, &mut env).unwrap(), Value::I64(3));
    }

    #[test]
    fn test_char_at() {
        let mut env = MockEnvironment::new();
        let char_at = CharAt;
        let args = vec![Value::Text("abc".to_string()), Value::I64(1)];
        assert_eq!(char_at.call(&args, &mut env).unwrap(), Value::I64('b' as u32 as i64));
    }
}
