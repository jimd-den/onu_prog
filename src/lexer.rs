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
    Number,
    Text,
    Identifier(String),
    NumericLiteral(f64),
    TextLiteral(String),

    // --- Discourse Structures ---
    TheModuleCalled,
    TheShape,
    TheBehaviorCalled,
    Called,
    
    // --- Clauses and Modifiers ---
    With,
    WithIntent,
    WithConcern,
    WithDiminishing,
    Receiving,
    Returning,
    As,
    Keeps,
    KeepsInternal,
    Exposes,
    Promises,
    Colon,
    
    // --- Evaluation Control ---
    Emit,
    Nothing,
    If,
    Then,
    Else,
    LParen,
    RParen,
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
            (Token::TextLiteral(s1), Token::TextLiteral(s2)) => s1 == s2,
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
            Token::TextLiteral(s) => s.hash(state),
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
                self.skip_whitespace();
                let second = self.lex_single_identifier_or_keyword();
                match second.as_str() {
                    "module" => {
                        self.skip_whitespace();
                        let third = self.lex_single_identifier_or_keyword();
                        if third == "called" {
                            Some(Token::TheModuleCalled)
                        } else {
                            Some(Token::TheModuleCalled)
                        }
                    }
                    "shape" => Some(Token::TheShape),
                    "behavior" => {
                        self.skip_whitespace();
                        let third = self.lex_single_identifier_or_keyword();
                        if third == "called" {
                            Some(Token::TheBehaviorCalled)
                        } else {
                            Some(Token::TheBehaviorCalled)
                        }
                    }
                    "number" => Some(Token::Number),
                    "text" => Some(Token::Text),
                    _ => Some(Token::The),
                }
            }
            "with" => {
                self.skip_whitespace();
                let second = self.lex_single_identifier_or_keyword();
                if second == "intent" {
                    Some(Token::WithIntent)
                } else if second == "concern" {
                    Some(Token::WithConcern)
                } else if second == "diminishing" {
                    Some(Token::WithDiminishing)
                } else {
                    Some(Token::With)
                }
            }
            "keeps" => {
                self.skip_whitespace();
                let second = self.lex_single_identifier_or_keyword();
                if second == "internal" {
                    Some(Token::KeepsInternal)
                } else {
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
            "number" => Some(Token::Number),
            "text" => Some(Token::Text),
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

        number_str.parse::<f64>().ok().map(Token::NumericLiteral)
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
        assert_eq!(lexer.next_token().unwrap().token, Token::Number);
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
        assert_eq!(lexer.next_token().unwrap().token, Token::Text);
        assert_eq!(lexer.next_token().unwrap().token, Token::TextLiteral("acceptable".to_string()));
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
        let input = "the behavior called scale-value with intent: transform receiving: a number returning: a number as: result";
        let mut lexer = Lexer::new(input);
        assert_eq!(lexer.next_token().unwrap().token, Token::TheBehaviorCalled);
        assert_eq!(lexer.next_token().unwrap().token, Token::Identifier("scale-value".to_string()));
        assert_eq!(lexer.next_token().unwrap().token, Token::WithIntent);
        assert_eq!(lexer.next_token().unwrap().token, Token::Colon);
        assert_eq!(lexer.next_token().unwrap().token, Token::Identifier("transform".to_string()));
        assert_eq!(lexer.next_token().unwrap().token, Token::Receiving);
        assert_eq!(lexer.next_token().unwrap().token, Token::Colon);
        assert_eq!(lexer.next_token().unwrap().token, Token::Identifier("a".to_string()));
        assert_eq!(lexer.next_token().unwrap().token, Token::Number);
        assert_eq!(lexer.next_token().unwrap().token, Token::Returning);
        assert_eq!(lexer.next_token().unwrap().token, Token::Colon);
        assert_eq!(lexer.next_token().unwrap().token, Token::Identifier("a".to_string()));
        assert_eq!(lexer.next_token().unwrap().token, Token::Number);
        assert_eq!(lexer.next_token().unwrap().token, Token::As);
        assert_eq!(lexer.next_token().unwrap().token, Token::Colon);
        assert_eq!(lexer.next_token().unwrap().token, Token::Identifier("result".to_string()));
        assert!(lexer.next_token().is_none());
    }

    #[test]
    fn test_lex_comments() {
        let input = "let x is 10 -- this is a comment\nlet y is 20";
        let mut lexer = Lexer::new(input);
        assert_eq!(lexer.next_token().unwrap().token, Token::Let);
        assert_eq!(lexer.next_token().unwrap().token, Token::Identifier("x".to_string()));
        assert_eq!(lexer.next_token().unwrap().token, Token::Is);
        assert_eq!(lexer.next_token().unwrap().token, Token::NumericLiteral(10.0));
        assert_eq!(lexer.next_token().unwrap().token, Token::Let);
        assert_eq!(lexer.next_token().unwrap().token, Token::Identifier("y".to_string()));
        assert_eq!(lexer.next_token().unwrap().token, Token::Is);
        assert_eq!(lexer.next_token().unwrap().token, Token::NumericLiteral(20.0));
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
