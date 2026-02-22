use crate::builtins::BuiltInFunction;
use crate::interpreter::Value;
use crate::env::Environment;
use crate::error::OnuError;

/// Helper to perform numeric operations across different Value variants.
/// Enforces that both operands are of the same specific type category (Integer vs Float)
/// or handles promotion if explicitly desired (currently strict for professional safety).
fn bin_op<FI, FF>(args: &[Value], op_name: &str, int_op: FI, float_op: FF) -> Result<Value, OnuError>
where
    FI: Fn(i128, i128) -> i128,
    FF: Fn(f64, f64) -> f64,
{
    match (args.get(0), args.get(1)) {
        // Handle all integer variants (promoting to i128 for intermediate calculation)
        (Some(v1), Some(v2)) if v1.is_integer() && v2.is_integer() => {
            let n1 = v1.as_i128().unwrap();
            let n2 = v2.as_i128().unwrap();
            // Return I64 as the standard large integer for now, or match v1's type?
            // For Phase 1 simplified: return Value::I64
            Ok(Value::I64(int_op(n1, n2) as i64))
        }
        // Handle float variants
        (Some(v1), Some(v2)) if v1.is_float() && v2.is_float() => {
            let f1 = v1.as_f64().unwrap();
            let f2 = v2.as_f64().unwrap();
            Ok(Value::F64(float_op(f1, f2)))
        }
        (Some(v1), Some(v2)) => Err(OnuError::RuntimeError {
            message: format!("Type Mismatch: '{}' requires consistent numeric types (found {} and {})", 
                op_name, v1.get_type_name(), v2.get_type_name()),
            span: Default::default(),
        }),
        _ => Err(OnuError::RuntimeError {
            message: format!("'{}' requires two arguments", op_name),
            span: Default::default(),
        }),
    }
}

#[derive(Debug)]
pub struct Add;
impl BuiltInFunction for Add {
    fn call(&self, args: &[Value], _env: &mut dyn Environment) -> Result<Value, OnuError> {
        bin_op(args, "added-to", |a, b| a + b, |a, b| a + b)
    }
}

#[derive(Debug)]
pub struct Sub;
impl BuiltInFunction for Sub {
    fn call(&self, args: &[Value], _env: &mut dyn Environment) -> Result<Value, OnuError> {
        bin_op(args, "decreased-by", |a, b| a - b, |a, b| a - b)
    }
}

#[derive(Debug)]
pub struct SubtractedFrom;
impl BuiltInFunction for SubtractedFrom {
    fn call(&self, args: &[Value], _env: &mut dyn Environment) -> Result<Value, OnuError> {
        bin_op(args, "subtracted-from", |a, b| b - a, |a, b| b - a)
    }
}

#[derive(Debug)]
pub struct Mul;
impl BuiltInFunction for Mul {
    fn call(&self, args: &[Value], _env: &mut dyn Environment) -> Result<Value, OnuError> {
        bin_op(args, "scales-by", |a, b| a * b, |a, b| a * b)
    }
}

#[derive(Debug)]
pub struct Div;
impl BuiltInFunction for Div {
    fn call(&self, args: &[Value], _env: &mut dyn Environment) -> Result<Value, OnuError> {
        match (args.get(0), args.get(1)) {
            (Some(v1), Some(v2)) if v1.is_float() && v2.is_float() => {
                let f2 = v2.as_f64().unwrap();
                if f2 == 0.0 {
                    return Err(OnuError::RuntimeError { message: "Division by zero".to_string(), span: Default::default() });
                }
                let f1 = v1.as_f64().unwrap();
                Ok(Value::F64(f1 / f2))
            }
            (Some(v1), Some(v2)) if v1.is_integer() && v2.is_integer() => {
                let n2 = v2.as_i128().unwrap();
                if n2 == 0 {
                    return Err(OnuError::RuntimeError { message: "Division by zero".to_string(), span: Default::default() });
                }
                let n1 = v1.as_i128().unwrap();
                Ok(Value::I64((n1 / n2) as i64))
            }
            _ => Err(OnuError::RuntimeError {
                message: "'partitions-by' requires consistent numeric arguments".to_string(),
                span: Default::default(),
            }),
        }
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
        let args = vec![Value::I64(10), Value::I64(20)];
        assert_eq!(add.call(&args, &mut env).unwrap(), Value::I64(30));
        
        let args_f = vec![Value::F64(10.5), Value::F64(20.0)];
        assert_eq!(add.call(&args_f, &mut env).unwrap(), Value::F64(30.5));
    }
}
