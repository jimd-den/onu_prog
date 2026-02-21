/// Ọ̀nụ Core Types: The Domain Logic Layer
///
/// This module defines the formal type system of Ọ̀nụ.
/// Following Clean Architecture, types are first-class domain entities
/// that govern the rules of static analysis and runtime evaluation.

use std::fmt;

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub enum OnuType {
    // --- Integers ---
    I8, I16, I32, I64, I128,
    U8, U16, U32, U64, U128,
    
    // --- Floats ---
    F32, F64,
    
    // --- Boolean ---
    Boolean,

    // --- Other Primitives ---
    Strings,
    Matrix,
    Nothing,       // The void type
    
    // --- Structural ---
    Tuple(Vec<OnuType>), // Fixed-size collection of potentially different types
    Array(Box<OnuType>), // Variable-size collection of the same type
    
    // --- Abstract ---
    Shape(String), // Reference to a Shape (Interface)
}

impl fmt::Display for OnuType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OnuType::I8 => write!(f, "i8"),
            OnuType::I16 => write!(f, "i16"),
            OnuType::I32 => write!(f, "i32"),
            OnuType::I64 => write!(f, "i64"),
            OnuType::I128 => write!(f, "i128"),
            OnuType::U8 => write!(f, "u8"),
            OnuType::U16 => write!(f, "u16"),
            OnuType::U32 => write!(f, "u32"),
            OnuType::U64 => write!(f, "u64"),
            OnuType::U128 => write!(f, "u128"),
            OnuType::F32 => write!(f, "f32"),
            OnuType::F64 => write!(f, "f64"),
            OnuType::Boolean => write!(f, "boolean"),
            OnuType::Strings => write!(f, "strings"),
            OnuType::Matrix => write!(f, "matrix"),
            OnuType::Nothing => write!(f, "nothing"),
            OnuType::Tuple(types) => {
                write!(f, "tuple of (")?;
                for (i, t) in types.iter().enumerate() {
                    if i > 0 { write!(f, ", ")?; }
                    write!(f, "{}", t)?;
                }
                write!(f, ")")
            },
            OnuType::Array(inner) => write!(f, "array of {}", inner),
            OnuType::Shape(name) => write!(f, "role {}", name),
        }
    }
}

impl OnuType {
    /// Maps a discourse type name string to an OnuType.
    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "i8" => Some(OnuType::I8),
            "i16" => Some(OnuType::I16),
            "i32" => Some(OnuType::I32),
            "i64" => Some(OnuType::I64),
            "i128" => Some(OnuType::I128),
            "u8" => Some(OnuType::U8),
            "u16" => Some(OnuType::U16),
            "u32" => Some(OnuType::U32),
            "u64" => Some(OnuType::U64),
            "u128" => Some(OnuType::U128),
            "f32" => Some(OnuType::F32),
            "f64" => Some(OnuType::F64),
            "boolean" => Some(OnuType::Boolean),
            "strings" => Some(OnuType::Strings),
            "matrix" => Some(OnuType::Matrix),
            "nothing" => Some(OnuType::Nothing),
            // Legacy/Alias support if needed
            "integer" => Some(OnuType::I64),
            "float" => Some(OnuType::F64),
            _ => None, 
        }
    }
}
