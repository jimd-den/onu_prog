use crate::builtins::BuiltInFunction;
use crate::interpreter::Value;
use crate::env::Environment;
use crate::error::OnuError;

/// Helper for single-argument math functions
fn unary_math_op<F>(args: &[Value], op_name: &str, op: F) -> Result<Value, OnuError>
where
    F: Fn(f64) -> f64,
{
    match args.get(0) {
        Some(v) => {
            let f = v.as_f64().ok_or_else(|| OnuError::RuntimeError {
                message: format!("'{}' requires a numeric argument", op_name),
                span: Default::default(),
            })?;
            Ok(Value::F64(op(f)))
        }
        _ => Err(OnuError::RuntimeError {
            message: format!("'{}' requires one argument", op_name),
            span: Default::default(),
        }),
    }
}

/// Helper for binary-argument math functions
fn binary_math_op<F>(args: &[Value], op_name: &str, op: F) -> Result<Value, OnuError>
where
    F: Fn(f64, f64) -> f64,
{
    match (args.get(0), args.get(1)) {
        (Some(v1), Some(v2)) => {
            let f1 = v1.as_f64().ok_or_else(|| OnuError::RuntimeError {
                message: format!("'{}' requires numeric arguments", op_name),
                span: Default::default(),
            })?;
            let f2 = v2.as_f64().ok_or_else(|| OnuError::RuntimeError {
                message: format!("'{}' requires numeric arguments", op_name),
                span: Default::default(),
            })?;
            Ok(Value::F64(op(f1, f2)))
        }
        _ => Err(OnuError::RuntimeError {
            message: format!("'{}' requires two arguments", op_name),
            span: Default::default(),
        }),
    }
}

// --- Trigonometry ---

#[derive(Debug)]
pub struct Sine;
impl BuiltInFunction for Sine {
    fn call(&self, args: &[Value], _env: &mut dyn Environment) -> Result<Value, OnuError> {
        unary_math_op(args, "sine", |a| a.sin())
    }
}

#[derive(Debug)]
pub struct Cosine;
impl BuiltInFunction for Cosine {
    fn call(&self, args: &[Value], _env: &mut dyn Environment) -> Result<Value, OnuError> {
        unary_math_op(args, "cosine", |a| a.cos())
    }
}

#[derive(Debug)]
pub struct Tangent;
impl BuiltInFunction for Tangent {
    fn call(&self, args: &[Value], _env: &mut dyn Environment) -> Result<Value, OnuError> {
        unary_math_op(args, "tangent", |a| a.tan())
    }
}

#[derive(Debug)]
pub struct ArcSin;
impl BuiltInFunction for ArcSin {
    fn call(&self, args: &[Value], _env: &mut dyn Environment) -> Result<Value, OnuError> {
        unary_math_op(args, "arcsin", |a| a.asin())
    }
}

#[derive(Debug)]
pub struct ArcCos;
impl BuiltInFunction for ArcCos {
    fn call(&self, args: &[Value], _env: &mut dyn Environment) -> Result<Value, OnuError> {
        unary_math_op(args, "arccos", |a| a.acos())
    }
}

#[derive(Debug)]
pub struct ArcTan;
impl BuiltInFunction for ArcTan {
    fn call(&self, args: &[Value], _env: &mut dyn Environment) -> Result<Value, OnuError> {
        unary_math_op(args, "arctan", |a| a.atan())
    }
}

// --- Calculus/Exponential ---

#[derive(Debug)]
pub struct SquareRoot;
impl BuiltInFunction for SquareRoot {
    fn call(&self, args: &[Value], _env: &mut dyn Environment) -> Result<Value, OnuError> {
        unary_math_op(args, "square-root", |a| a.sqrt())
    }
}

#[derive(Debug)]
pub struct Power;
impl BuiltInFunction for Power {
    fn call(&self, args: &[Value], _env: &mut dyn Environment) -> Result<Value, OnuError> {
        binary_math_op(args, "raised-to", |a, b| a.powf(b))
    }
}

#[derive(Debug)]
pub struct NaturalLog;
impl BuiltInFunction for NaturalLog {
    fn call(&self, args: &[Value], _env: &mut dyn Environment) -> Result<Value, OnuError> {
        unary_math_op(args, "natural-log", |a| a.ln())
    }
}

#[derive(Debug)]
pub struct Exp;
impl BuiltInFunction for Exp {
    fn call(&self, args: &[Value], _env: &mut dyn Environment) -> Result<Value, OnuError> {
        unary_math_op(args, "exponent", |a| a.exp())
    }
}

// --- Linear Algebra ---

#[derive(Debug)]
pub struct DotProduct;
impl BuiltInFunction for DotProduct {
    fn call(&self, args: &[Value], _env: &mut dyn Environment) -> Result<Value, OnuError> {
        match (args.get(0), args.get(1)) {
            (Some(Value::Tuple(v1)), Some(Value::Tuple(v2))) => {
                if v1.len() != v2.len() {
                    return Err(OnuError::RuntimeError { message: "dot-product requires vectors of same length".to_string(), span: Default::default() });
                }
                let mut sum = 0.0;
                for (a, b) in v1.iter().zip(v2.iter()) {
                    let fa = a.as_f64().ok_or_else(|| OnuError::RuntimeError { message: "dot-product requires numeric components".to_string(), span: Default::default() })?;
                    let fb = b.as_f64().ok_or_else(|| OnuError::RuntimeError { message: "dot-product requires numeric components".to_string(), span: Default::default() })?;
                    sum += fa * fb;
                }
                Ok(Value::F64(sum))
            }
            _ => Err(OnuError::RuntimeError {
                message: "dot-product requires two tuples (vectors)".to_string(),
                span: Default::default(),
            }),
        }
    }
}

#[derive(Debug)]
pub struct CrossProduct;
impl BuiltInFunction for CrossProduct {
    fn call(&self, args: &[Value], _env: &mut dyn Environment) -> Result<Value, OnuError> {
        match (args.get(0), args.get(1)) {
            (Some(Value::Tuple(v1)), Some(Value::Tuple(v2))) => {
                if v1.len() != 3 || v2.len() != 3 {
                    return Err(OnuError::RuntimeError { message: "cross-product requires 3D vectors".to_string(), span: Default::default() });
                }
                let f1: Vec<f64> = v1.iter().map(|v| v.as_f64().unwrap_or(0.0)).collect();
                let f2: Vec<f64> = v2.iter().map(|v| v.as_f64().unwrap_or(0.0)).collect();
                
                let res = vec![
                    Value::F64(f1[1] * f2[2] - f1[2] * f2[1]),
                    Value::F64(f1[2] * f2[0] - f1[0] * f2[2]),
                    Value::F64(f1[0] * f2[1] - f1[1] * f2[0]),
                ];
                Ok(Value::Tuple(res))
            }
            _ => Err(OnuError::RuntimeError {
                message: "cross-product requires two 3D tuples".to_string(),
                span: Default::default(),
            }),
        }
    }
}

#[derive(Debug)]
pub struct Determinant;
impl BuiltInFunction for Determinant {
    fn call(&self, args: &[Value], _env: &mut dyn Environment) -> Result<Value, OnuError> {
        match args.get(0) {
            Some(Value::Matrix(m)) => {
                if m.rows != m.cols {
                    return Err(OnuError::RuntimeError { message: "determinant requires square matrix".to_string(), span: Default::default() });
                }
                if m.rows == 2 {
                    Ok(Value::F64(m.data[0] * m.data[3] - m.data[1] * m.data[2]))
                } else {
                    Err(OnuError::RuntimeError { message: "determinant currently only supports 2x2".to_string(), span: Default::default() })
                }
            }
            _ => Err(OnuError::RuntimeError {
                message: "determinant requires a matrix".to_string(),
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
    fn test_sine() {
        let mut env = MockEnvironment::new();
        let s = Sine;
        let res = s.call(&[Value::F64(0.0)], &mut env).unwrap();
        assert_eq!(res, Value::F64(0.0));
        let res = s.call(&[Value::F64(std::f64::consts::PI / 2.0)], &mut env).unwrap();
        assert_eq!(res, Value::F64(1.0));
    }

    #[test]
    fn test_cosine() {
        let mut env = MockEnvironment::new();
        let c = Cosine;
        let res = c.call(&[Value::F64(0.0)], &mut env).unwrap();
        assert_eq!(res, Value::F64(1.0));
    }

    #[test]
    fn test_sqrt() {
        let mut env = MockEnvironment::new();
        let s = SquareRoot;
        let res = s.call(&[Value::I64(16)], &mut env).unwrap();
        assert_eq!(res, Value::F64(4.0));
    }

    #[test]
    fn test_power() {
        let mut env = MockEnvironment::new();
        let p = Power;
        let res = p.call(&[Value::F64(2.0), Value::F64(3.0)], &mut env).unwrap();
        assert_eq!(res, Value::F64(8.0));
    }

    #[test]
    fn test_log() {
        let mut env = MockEnvironment::new();
        let l = NaturalLog;
        let res = l.call(&[Value::F64(std::f64::consts::E)], &mut env).unwrap();
        assert_eq!(res, Value::F64(1.0));
    }

    #[test]
    fn test_dot_product() {
        let mut env = MockEnvironment::new();
        let d = DotProduct;
        let v1 = Value::Tuple(vec![Value::F64(1.0), Value::F64(2.0)]);
        let v2 = Value::Tuple(vec![Value::F64(3.0), Value::F64(4.0)]);
        let res = d.call(&[v1, v2], &mut env).unwrap();
        assert_eq!(res, Value::F64(11.0));
    }

    #[test]
    fn test_cross_product() {
        let mut env = MockEnvironment::new();
        let c = CrossProduct;
        let v1 = Value::Tuple(vec![Value::F64(1.0), Value::F64(0.0), Value::F64(0.0)]);
        let v2 = Value::Tuple(vec![Value::F64(0.0), Value::F64(1.0), Value::F64(0.0)]);
        let res = c.call(&[v1, v2], &mut env).unwrap();
        assert_eq!(res, Value::Tuple(vec![Value::F64(0.0), Value::F64(0.0), Value::F64(1.0)]));
    }

    #[test]
    fn test_determinant() {
        let mut env = MockEnvironment::new();
        let d = Determinant;
        let m = crate::interpreter::Matrix::new(2, 2, vec![1.0, 2.0, 3.0, 4.0]);
        let res = d.call(&[Value::Matrix(m)], &mut env).unwrap();
        assert_eq!(res, Value::F64(-2.0));
    }
}
