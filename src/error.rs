use std::fmt;

/// A Span represents a range of characters in the source code.
/// This provides the necessary metadata for high-quality error messages,
/// allowing the user to pinpoint exactly where an issue occurred.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Span {
    pub line: usize,
    pub column: usize,
}

impl fmt::Display for Span {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.line, self.column)
    }
}

/// OnuError captures all possible failure states within the Ọ̀nụ pipeline.
/// By using an enum, we adhere to the Single Responsibility Principle:
/// the error system is the one place where failure logic is defined.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OnuError {
    LexicalError { message: String, span: Span },
    ParseError { message: String, span: Span },
    RuntimeError { message: String, span: Span },
    BehaviorConflict { name: String, other_name: String },
}

impl fmt::Display for OnuError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OnuError::LexicalError { message, span } => {
                write!(f, "Lexical Error at [{}]: {}", span, message)
            }
            OnuError::ParseError { message, span } => {
                write!(f, "Parse Error at [{}]: {}", span, message)
            }
            OnuError::RuntimeError { message, span } => {
                write!(f, "Runtime Error at [{}]: {}", span, message)
            }
            OnuError::BehaviorConflict { name, other_name } => {
                write!(f, "Conflict: Behavior '{}' is semantically identical to '{}'", name, other_name)
            }
        }
    }
}

impl std::error::Error for OnuError {}
