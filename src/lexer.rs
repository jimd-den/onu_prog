/// Ọ̀nụ Lexer: The First Discourse Layer
///
/// This module implements a handwritten lexer for the Ọ̀nụ language.
/// Following the project's aesthetic of a "legal discourse" or "academic whitepaper,"
/// the lexer must handle multi-word keywords (e.g., "the module called") as single
/// semantic tokens while maintaining a simple, predictive SVO topology.
///
/// Design Patterns:
/// - Iterator/Peekable: Uses Rust's standard Peekable interface to look ahead
///   one character without consuming it, allowing for LL(1)-like lexing.

use std::iter::Peekable;
use std::str::Chars;
use std::hash::{Hash, Hasher};
use crate::error::Span;

/// Tokens represent the atomic semantic units of the Ọ̀nụ language.
/// Keywords are derived from Igbo linguistic structures but expressed in English
/// to emphasize agglutinative composition and focus morphology.
#[derive(Debug, Clone)]
pub enum Token {
    // --- Primitives and Bindings ---
    Let,
    Is,
    The,
    Integer,
    Float,
    RealNumber,
    Strings,
    Matrix,
    Identifier(String),
    NumericLiteral(f64),
    IntegerLiteral(i128),
    TextLiteral(String),
    BooleanLiteral(bool),

    // --- Discourse Structures ---
    TheModuleCalled,
    TheShape,
    TheBehaviorCalled,
    TheEffectBehaviorCalled,
    Called,
    
    // --- Clauses and Modifiers ---
    With,
    WithIntent,
    WithConcern,
    WithDiminishing,
    NoGuaranteedTermination, // Composite keyword
    Receiving,
    Returning,
    As,
    Keeps,
    KeepsInternal,
    Exposes,
    Promises,
    Colon,
    A,
    An,
    Via,
    Role,
    
    // --- Evaluation Control ---
    Emit,
    Nothing,
    If,
    Then,
    Else,
    LParen,
    RParen,
    LBracket,
    RBracket,
}

/// TokenWithSpan wraps a token with its location in the source code.
/// This is critical for the "High-Signal Output" mandate, enabling
/// precise error reporting during parsing and runtime.
#[derive(Debug, Clone)]
pub struct TokenWithSpan {
    pub token: Token,
    pub span: Span,
}

impl PartialEq for Token {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Token::Identifier(s1), Token::Identifier(s2)) => s1 == s2,
            (Token::NumericLiteral(n1), Token::NumericLiteral(n2)) => n1.to_bits() == n2.to_bits(),
            (Token::IntegerLiteral(n1), Token::IntegerLiteral(n2)) => n1 == n2,
            (Token::TextLiteral(s1), Token::TextLiteral(s2)) => s1 == s2,
            (Token::BooleanLiteral(b1), Token::BooleanLiteral(b2)) => b1 == b2,
            _ => std::mem::discriminant(self) == std::mem::discriminant(other),
        }
    }
}

impl Eq for Token {}

impl Hash for Token {
    fn hash<H: Hasher>(&self, state: &mut H) {
        std::mem::discriminant(self).hash(state);
        match self {
            Token::Identifier(s) => s.hash(state),
            Token::NumericLiteral(n) => n.to_bits().hash(state),
            Token::IntegerLiteral(n) => n.hash(state),
            Token::TextLiteral(s) => s.hash(state),
            Token::BooleanLiteral(b) => b.hash(state),
            _ => {}
        }
    }
}

/// The Lexer struct maintains the state of the lexing process,
/// specifically tracking the current line and column for Span generation.
pub struct Lexer<'a> {
    input: Peekable<Chars<'a>>,
    line: usize,
    column: usize,
}

impl<'a> Lexer<'a> {
    /// Initializes a new Lexer from a string slice.
    pub fn new(input: &'a str) -> Self {
        Self {
            input: input.chars().peekable(),
            line: 1,
            column: 1,
        }
    }

    /// Peeks at the next character without consuming it.
    fn peek_char(&mut self) -> Option<char> {
        self.input.peek().copied()
    }

    /// Consumes and returns the next character, updating line/column state.
    fn next_char(&mut self) -> Option<char> {
        let c = self.input.next()?;
        if c == '\n' {
            self.line += 1;
            self.column = 1;
        } else {
            self.column += 1;
        }
        Some(c)
    }

    /// The core lexing loop: skip whitespace, determine the start of a token,
    /// and delegate to specialized lexing functions.
    pub fn next_token(&mut self) -> Option<TokenWithSpan> {
        self.skip_whitespace();

        let span = Span {
            line: self.line,
            column: self.column,
        };

        let first_char = self.peek_char()?;
        let token = match first_char {
            '-' => {
                self.next_char();
                if let Some('-') = self.peek_char() {
                    self.skip_comment();
                    return self.next_token();
                } else {
                    return None;
                }
            }
            ':' => {
                self.next_char();
                Token::Colon
            }
            '(' => {
                self.next_char();
                Token::LParen
            }
            ')' => {
                self.next_char();
                Token::RParen
            }
            '[' => {
                self.next_char();
                Token::LBracket
            }
            ']' => {
                self.next_char();
                Token::RBracket
            }
            '"' => self.lex_string()?,
            c if c.is_alphabetic() => self.lex_identifier_or_keyword_multi()?,
            c if c.is_ascii_digit() => self.lex_number()?,
            _ => {
                self.next_char(); // Skip unknown character
                return self.next_token(); // Try next
            }
        };

        Some(TokenWithSpan { token, span })
    }

    fn skip_whitespace(&mut self) {
        while let Some(c) = self.peek_char() {
            if c.is_whitespace() {
                self.next_char();
            } else {
                break;
            }
        }
    }

    fn skip_comment(&mut self) {
        self.next_char(); // Consume second '-'
        while let Some(c) = self.next_char() {
            if c == '\n' {
                break;
            }
        }
    }

    /// Lexes identifiers and multi-word keywords.
    /// This function handles the "The Module Called" style keywords by peeking
    /// ahead and consuming multiple words if they match a known composite token.
    fn lex_identifier_or_keyword_multi(&mut self) -> Option<Token> {
        let first = self.lex_single_identifier_or_keyword();

        match first.as_str() {
            "the" => {
                let saved_line = self.line;
                let saved_column = self.column;
                let saved_input = self.input.clone();

                self.skip_whitespace();
                let second = self.lex_single_identifier_or_keyword();
                match second.as_str() {
                    "module" => {
                        self.skip_whitespace();
                        let third = self.lex_single_identifier_or_keyword();
                        if third == "called" {
                            return Some(Token::TheModuleCalled);
                        }
                    }
                    "shape" => return Some(Token::TheShape),
                    "behavior" => {
                        self.skip_whitespace();
                        let third = self.lex_single_identifier_or_keyword();
                        if third == "called" {
                            return Some(Token::TheBehaviorCalled);
                        }
                    }
                    "effect" => {
                        self.skip_whitespace();
                        let third = self.lex_single_identifier_or_keyword();
                        if third == "behavior" {
                            self.skip_whitespace();
                            let fourth = self.lex_single_identifier_or_keyword();
                            if fourth == "called" {
                                return Some(Token::TheEffectBehaviorCalled);
                            }
                        }
                    }
                    _ => {}
                }
                
                // If no multi-word keyword matched, backtrack and just emit 'The'
                self.line = saved_line;
                self.column = saved_column;
                self.input = saved_input;
                Some(Token::The)
            }
            "with" => {
                let saved_line = self.line;
                let saved_column = self.column;
                let saved_input = self.input.clone();

                self.skip_whitespace();
                let second = self.lex_single_identifier_or_keyword();
                if second == "intent" {
                    Some(Token::WithIntent)
                } else if second == "concern" {
                    Some(Token::WithConcern)
                } else if second == "diminishing" {
                    Some(Token::WithDiminishing)
                } else if second == "no" {
                    self.skip_whitespace();
                    let third = self.lex_single_identifier_or_keyword();
                    if third == "guaranteed" {
                        self.skip_whitespace();
                        let fourth = self.lex_single_identifier_or_keyword();
                        if fourth == "termination" {
                            return Some(Token::NoGuaranteedTermination);
                        }
                    }
                    // Backtrack if not full phrase
                    self.line = saved_line;
                    self.column = saved_column;
                    self.input = saved_input;
                    Some(Token::With)
                } else {
                    self.line = saved_line;
                    self.column = saved_column;
                    self.input = saved_input;
                    Some(Token::With)
                }
            }
            "keeps" => {
                let saved_line = self.line;
                let saved_column = self.column;
                let saved_input = self.input.clone();

                self.skip_whitespace();
                let second = self.lex_single_identifier_or_keyword();
                if second == "internal" {
                    Some(Token::KeepsInternal)
                } else {
                    self.line = saved_line;
                    self.column = saved_column;
                    self.input = saved_input;
                    Some(Token::Keeps)
                }
            }
            "let" => Some(Token::Let),
            "is" => Some(Token::Is),
            "receiving" => Some(Token::Receiving),
            "returning" => Some(Token::Returning),
            "as" => Some(Token::As),
            "exposes" => Some(Token::Exposes),
            "promises" => Some(Token::Promises),
            "emit" => Some(Token::Emit),
            "nothing" => Some(Token::Nothing),
            "if" => Some(Token::If),
            "then" => Some(Token::Then),
            "else" => Some(Token::Else),
            "a" => {
                let saved_line = self.line;
                let saved_column = self.column;
                let saved_input = self.input.clone();

                self.skip_whitespace();
                let second = self.lex_single_identifier_or_keyword();
                if second == "behavior" {
                    self.skip_whitespace();
                    let third = self.lex_single_identifier_or_keyword();
                    if third == "called" {
                        return Some(Token::TheBehaviorCalled); // Reuse same token for now
                    }
                }

                self.line = saved_line;
                self.column = saved_column;
                self.input = saved_input;
                Some(Token::A)
            }
            "an" => {
                let saved_line = self.line;
                let saved_column = self.column;
                let saved_input = self.input.clone();

                self.skip_whitespace();
                let second = self.lex_single_identifier_or_keyword();
                if second == "effect" {
                    self.skip_whitespace();
                    let third = self.lex_single_identifier_or_keyword();
                    if third == "behavior" {
                        self.skip_whitespace();
                        let fourth = self.lex_single_identifier_or_keyword();
                        if fourth == "called" {
                            return Some(Token::TheEffectBehaviorCalled);
                        }
                    }
                }

                self.line = saved_line;
                self.column = saved_column;
                self.input = saved_input;
                Some(Token::An)
            }
            "via" => Some(Token::Via),
            "role" => Some(Token::Role),
            "integer" => Some(Token::Integer),
            "float" => Some(Token::Float),
            "realnumber" => Some(Token::RealNumber),
            "strings" => Some(Token::Strings),
            "matrix" => Some(Token::Matrix),
            "true" => Some(Token::BooleanLiteral(true)),
            "false" => Some(Token::BooleanLiteral(false)),
            _ => Some(Token::Identifier(first)),
        }
    }

    fn lex_single_identifier_or_keyword(&mut self) -> String {
        let mut identifier = String::new();
        while let Some(c) = self.peek_char() {
            if c.is_alphanumeric() || c == '-' {
                identifier.push(c);
                self.next_char();
            } else {
                break;
            }
        }
        identifier
    }

    fn lex_number(&mut self) -> Option<Token> {
        let mut number_str = String::new();
        let mut has_decimal = false;

        while let Some(c) = self.peek_char() {
            if c.is_ascii_digit() {
                number_str.push(c);
                self.next_char();
            } else if c == '.' && !has_decimal {
                has_decimal = true;
                number_str.push(c);
                self.next_char();
            } else {
                break;
            }
        }

        if has_decimal {
            number_str.parse::<f64>().ok().map(Token::NumericLiteral)
        } else {
            number_str.parse::<i128>().ok().map(Token::IntegerLiteral)
        }
    }

    fn lex_string(&mut self) -> Option<Token> {
        self.next_char(); // Consume opening quote
        let mut content = String::new();
        while let Some(c) = self.next_char() {
            if c == '"' {
                break;
            } else {
                content.push(c);
            }
        }
        Some(Token::TextLiteral(content))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lex_let_is_number() {
        let input = "let pi is the number 3.14159";
        let mut lexer = Lexer::new(input);
        assert_eq!(lexer.next_token().unwrap().token, Token::Let);
        assert_eq!(lexer.next_token().unwrap().token, Token::Identifier("pi".to_string()));
        assert_eq!(lexer.next_token().unwrap().token, Token::Is);
        assert_eq!(lexer.next_token().unwrap().token, Token::The);
        assert_eq!(lexer.next_token().unwrap().token, Token::Identifier("number".to_string()));
        assert_eq!(lexer.next_token().unwrap().token, Token::NumericLiteral(3.14159));
        assert!(lexer.next_token().is_none());
    }

    #[test]
    fn test_lex_let_is_text() {
        let input = r#"let label is the text "acceptable""#;
        let mut lexer = Lexer::new(input);
        assert_eq!(lexer.next_token().unwrap().token, Token::Let);
        assert_eq!(lexer.next_token().unwrap().token, Token::Identifier("label".to_string()));
        assert_eq!(lexer.next_token().unwrap().token, Token::Is);
        assert_eq!(lexer.next_token().unwrap().token, Token::The);
        assert_eq!(lexer.next_token().unwrap().token, Token::Identifier("text".to_string()));
        assert_eq!(lexer.next_token().unwrap().token, Token::TextLiteral("acceptable".to_string()));
        assert!(lexer.next_token().is_none());
    }

    #[test]
    fn test_lex_specific_types() {
        let input = "integer float realnumber strings matrix";
        let mut lexer = Lexer::new(input);
        assert_eq!(lexer.next_token().unwrap().token, Token::Integer);
        assert_eq!(lexer.next_token().unwrap().token, Token::Float);
        assert_eq!(lexer.next_token().unwrap().token, Token::RealNumber);
        assert_eq!(lexer.next_token().unwrap().token, Token::Strings);
        assert_eq!(lexer.next_token().unwrap().token, Token::Matrix);
        assert!(lexer.next_token().is_none());
    }

    #[test]
    fn test_lex_discourse_headers() {
        let input = "the module called MeasurementDomain the shape Measurable promises: the behavior called scale-value";
        let mut lexer = Lexer::new(input);
        assert_eq!(lexer.next_token().unwrap().token, Token::TheModuleCalled);
        assert_eq!(lexer.next_token().unwrap().token, Token::Identifier("MeasurementDomain".to_string()));
        assert_eq!(lexer.next_token().unwrap().token, Token::TheShape);
        assert_eq!(lexer.next_token().unwrap().token, Token::Identifier("Measurable".to_string()));
        assert_eq!(lexer.next_token().unwrap().token, Token::Promises);
        assert_eq!(lexer.next_token().unwrap().token, Token::Colon);
        assert_eq!(lexer.next_token().unwrap().token, Token::TheBehaviorCalled);
        assert_eq!(lexer.next_token().unwrap().token, Token::Identifier("scale-value".to_string()));
        assert!(lexer.next_token().is_none());
    }

    #[test]
    fn test_lex_behavior_declaration() {
        let input = "the behavior called scale-value with intent: transform receiving: a number returning: an integer as: result";
        let mut lexer = Lexer::new(input);
        assert_eq!(lexer.next_token().unwrap().token, Token::TheBehaviorCalled);
        assert_eq!(lexer.next_token().unwrap().token, Token::Identifier("scale-value".to_string()));
        assert_eq!(lexer.next_token().unwrap().token, Token::WithIntent);
        assert_eq!(lexer.next_token().unwrap().token, Token::Colon);
        assert_eq!(lexer.next_token().unwrap().token, Token::Identifier("transform".to_string()));
        assert_eq!(lexer.next_token().unwrap().token, Token::Receiving);
        assert_eq!(lexer.next_token().unwrap().token, Token::Colon);
        assert_eq!(lexer.next_token().unwrap().token, Token::A);
        assert_eq!(lexer.next_token().unwrap().token, Token::Identifier("number".to_string()));
        assert_eq!(lexer.next_token().unwrap().token, Token::Returning);
        assert_eq!(lexer.next_token().unwrap().token, Token::Colon);
        assert_eq!(lexer.next_token().unwrap().token, Token::An);
        assert_eq!(lexer.next_token().unwrap().token, Token::Integer);
        assert_eq!(lexer.next_token().unwrap().token, Token::As);
        assert_eq!(lexer.next_token().unwrap().token, Token::Colon);
        assert_eq!(lexer.next_token().unwrap().token, Token::Identifier("result".to_string()));
        assert!(lexer.next_token().is_none());
    }

    #[test]
    fn test_lex_type_specs() {
        let input = "receiving: a number returning nothing via the role Measurable";
        let mut lexer = Lexer::new(input);
        assert_eq!(lexer.next_token().unwrap().token, Token::Receiving);
        assert_eq!(lexer.next_token().unwrap().token, Token::Colon);
        assert_eq!(lexer.next_token().unwrap().token, Token::A);
        assert_eq!(lexer.next_token().unwrap().token, Token::Identifier("number".to_string()));
        assert_eq!(lexer.next_token().unwrap().token, Token::Returning);
        assert_eq!(lexer.next_token().unwrap().token, Token::Nothing);
        assert_eq!(lexer.next_token().unwrap().token, Token::Via);
        assert_eq!(lexer.next_token().unwrap().token, Token::The);
        assert_eq!(lexer.next_token().unwrap().token, Token::Role);
        assert_eq!(lexer.next_token().unwrap().token, Token::Identifier("Measurable".to_string()));
        assert!(lexer.next_token().is_none());
    }

    #[test]
    fn test_lex_an_article() {
        let input = "an integer";
        let mut lexer = Lexer::new(input);
        assert_eq!(lexer.next_token().unwrap().token, Token::An);
        assert_eq!(lexer.next_token().unwrap().token, Token::Integer);
        assert!(lexer.next_token().is_none());
    }

    #[test]
    fn test_lex_comments() {
        let input = "let x is 10 -- this is a comment\nlet y is 20";
        let mut lexer = Lexer::new(input);
        assert_eq!(lexer.next_token().unwrap().token, Token::Let);
        assert_eq!(lexer.next_token().unwrap().token, Token::Identifier("x".to_string()));
        assert_eq!(lexer.next_token().unwrap().token, Token::Is);
        assert_eq!(lexer.next_token().unwrap().token, Token::IntegerLiteral(10));
        assert_eq!(lexer.next_token().unwrap().token, Token::Let);
        assert_eq!(lexer.next_token().unwrap().token, Token::Identifier("y".to_string()));
        assert_eq!(lexer.next_token().unwrap().token, Token::Is);
        assert_eq!(lexer.next_token().unwrap().token, Token::IntegerLiteral(20));
        assert!(lexer.next_token().is_none());
    }

    #[test]
    fn test_lex_spans() {
        let input = "let x\n  is 10";
        let mut lexer = Lexer::new(input);
        
        let t1 = lexer.next_token().unwrap();
        assert_eq!(t1.token, Token::Let);
        assert_eq!(t1.span.line, 1);
        assert_eq!(t1.span.column, 1);
        
        let t2 = lexer.next_token().unwrap();
        assert_eq!(t2.token, Token::Identifier("x".to_string()));
        assert_eq!(t2.span.line, 1);
        assert_eq!(t2.span.column, 5);
        
        let t3 = lexer.next_token().unwrap();
        assert_eq!(t3.token, Token::Is);
        assert_eq!(t3.span.line, 2);
        assert_eq!(t3.span.column, 3);
    }
}
