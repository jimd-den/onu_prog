use crate::types::OnuType;
use crate::parser::{Discourse, Expression, BehaviorHeader, Argument};

#[derive(Debug, Clone, PartialEq)]
pub enum HirDiscourse {
    Module { name: String, concern: String },
    Shape { name: String, behaviors: Vec<HirBehaviorHeader> },
    Behavior { header: HirBehaviorHeader, body: HirExpression },
}

#[derive(Debug, Clone, PartialEq)]
pub struct HirBehaviorHeader {
    pub name: String,
    pub is_effect: bool,
    pub args: Vec<HirArgument>,
    pub return_type: OnuType,
}

#[derive(Debug, Clone, PartialEq)]
pub struct HirArgument {
    pub name: String,
    pub typ: OnuType,
}

#[derive(Debug, Clone, PartialEq)]
pub enum HirExpression {
    Literal(HirLiteral),
    Variable(String),
    Call { name: String, args: Vec<HirExpression> },
    Derivation { 
        name: String, 
        typ: OnuType, 
        value: Box<HirExpression>, 
        body: Box<HirExpression> 
    },
    If { 
        condition: Box<HirExpression>, 
        then_branch: Box<HirExpression>, 
        else_branch: Box<HirExpression> 
    },
    ActsAs { 
        subject: Box<HirExpression>, 
        shape: String 
    },
    Tuple(Vec<HirExpression>),
    Index { 
        subject: Box<HirExpression>, 
        index: usize 
    },
    Block(Vec<HirExpression>),
    Emit(Box<HirExpression>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum HirLiteral {
    I64(i64),
    F64(f64),
    Boolean(bool),
    Text(String),
    Nothing,
}

pub struct LoweringVisitor;

impl LoweringVisitor {
    pub fn lower_discourse(discourse: &Discourse) -> HirDiscourse {
        match discourse {
            Discourse::Module { name, concern } => HirDiscourse::Module {
                name: name.clone(),
                concern: concern.clone(),
            },
            Discourse::Shape { name, behaviors } => HirDiscourse::Shape {
                name: name.clone(),
                behaviors: behaviors.iter().map(Self::lower_header).collect(),
            },
            Discourse::Behavior { header, body } => HirDiscourse::Behavior {
                header: Self::lower_header(header),
                body: Self::lower_expression(body),
            },
        }
    }

    fn lower_header(header: &BehaviorHeader) -> HirBehaviorHeader {
        HirBehaviorHeader {
            name: header.name.clone(),
            is_effect: header.is_effect,
            args: header.takes.iter().map(Self::lower_argument).collect(),
            return_type: header.delivers.0.clone(),
        }
    }

    fn lower_argument(arg: &Argument) -> HirArgument {
        HirArgument {
            name: arg.name.clone(),
            typ: arg.type_info.onu_type.clone(),
        }
    }

    fn lower_expression(expr: &Expression) -> HirExpression {
        match expr {
            Expression::I64(n) => HirExpression::Literal(HirLiteral::I64(*n)),
            Expression::F64(n) => HirExpression::Literal(HirLiteral::F64(*n)),
            Expression::Boolean(b) => HirExpression::Literal(HirLiteral::Boolean(*b)),
            Expression::Text(s) => HirExpression::Literal(HirLiteral::Text(s.clone())),
            Expression::Nothing => HirExpression::Literal(HirLiteral::Nothing),
            Expression::Identifier(s) => HirExpression::Variable(s.clone()),
            Expression::BehaviorCall { name, args } => {
                // Heuristic: identify linguistic indexing (char-at)
                if name == "char-at" && args.len() == 2 {
                    if let Expression::I64(idx) = args[1] {
                        return HirExpression::Index {
                            subject: Box::new(Self::lower_expression(&args[0])),
                            index: idx as usize,
                        };
                    }
                }
                HirExpression::Call {
                    name: name.clone(),
                    args: args.iter().map(Self::lower_expression).collect(),
                }
            }
            Expression::Derivation { name, type_info, value, body } => HirExpression::Derivation {
                name: name.clone(),
                typ: type_info.as_ref().map(|ti| ti.onu_type.clone()).unwrap_or(OnuType::Nothing), // Default to nothing if unknown, though type checker should handle it
                value: Box::new(Self::lower_expression(value)),
                body: Box::new(Self::lower_expression(body)),
            },
            Expression::If { condition, then_branch, else_branch } => HirExpression::If {
                condition: Box::new(Self::lower_expression(condition)),
                then_branch: Box::new(Self::lower_expression(then_branch)),
                else_branch: Box::new(Self::lower_expression(else_branch)),
            },
            Expression::ActsAs { subject, shape } => HirExpression::ActsAs {
                subject: Box::new(Self::lower_expression(subject)),
                shape: shape.clone(),
            },
            Expression::Block(exprs) => HirExpression::Block(
                exprs.iter().map(Self::lower_expression).collect()
            ),
            Expression::Emit(e) | Expression::Broadcasts(e) => HirExpression::Emit(
                Box::new(Self::lower_expression(e))
            ),
            // Handle other literal types by mapping them to I64/F64 for now
            Expression::I8(n) => HirExpression::Literal(HirLiteral::I64(*n as i64)),
            Expression::I16(n) => HirExpression::Literal(HirLiteral::I64(*n as i64)),
            Expression::I32(n) => HirExpression::Literal(HirLiteral::I64(*n as i64)),
            Expression::I128(n) => HirExpression::Literal(HirLiteral::I64(*n as i64)),
            Expression::U8(n) => HirExpression::Literal(HirLiteral::I64(*n as i64)),
            Expression::U16(n) => HirExpression::Literal(HirLiteral::I64(*n as i64)),
            Expression::U32(n) => HirExpression::Literal(HirLiteral::I64(*n as i64)),
            Expression::U64(n) => HirExpression::Literal(HirLiteral::I64(*n as i64)),
            Expression::U128(n) => HirExpression::Literal(HirLiteral::I64(*n as i64)),
            Expression::F32(n) => HirExpression::Literal(HirLiteral::F64(*n as f64)),
            
            Expression::Tuple(v) => HirExpression::Tuple(
                 v.iter().map(Self::lower_expression).collect()
            ),
            Expression::Array(v) => HirExpression::Call {
                 name: "array".to_string(),
                 args: v.iter().map(Self::lower_expression).collect()
            },
            Expression::Matrix { rows, cols, data } => HirExpression::Call {
                 name: format!("matrix-{}x{}", rows, cols),
                 args: data.iter().map(Self::lower_expression).collect()
            },
        }
    }
}
