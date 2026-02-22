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
        write!(f, "\n═══════════════════════════════════════════\n")?;
        write!(f, "           PEER REVIEW MEMO\n")?;
        write!(f, "═══════════════════════════════════════════\n\n")?;
        
        match self {
            OnuError::LexicalError { message, span } => {
                write!(f, "Observation: An illegal character attempts to enter the discourse at {}.\n", span)?;
                write!(f, "Assessment:  {}\n", message)?;
                write!(f, "Conclusion:  The discourse refuses to be tokenized.\n")
            }
            OnuError::ParseError { message, span } => {
                write!(f, "Observation: The proposition at {} violates the grammatical covenant.\n", span)?;
                write!(f, "Assessment:  {}\n", message)?;
                write!(f, "Conclusion:  The proposition refuses to comply with the grammar.\n")
            }
            OnuError::RuntimeError { message, span } => {
                write!(f, "Observation: An evaluation event failed at {}.\n", span)?;
                write!(f, "Assessment:  {}\n", message)?;
                write!(f, "Conclusion:  The derivation refuses to evaluate.\n")
            }
            OnuError::BehaviorConflict { name, other_name } => {
                write!(f, "Observation: Duplicate semantic implementation detected.\n")?;
                write!(f, "Assessment:  The behavior '{}' is semantically identical to '{}'.\n", name, other_name)?;
                write!(f, "Conclusion:  This violates the Principle of Non-Repetition (DRY).\n")
            }
        }
    }
}

impl std::error::Error for OnuError {}
