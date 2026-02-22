/// Ọ̀nụ Linguistic Validator: The Interface Adapter Layer
///
/// This module implements the grammatical rules of the Ọ̀nụ language
/// that are purely linguistic rather than structural or semantic.
/// Following Clean Architecture, it decouples article validation (a/an)
/// from the core Parser.

use crate::parser::Discourse;
use crate::error::OnuError;
use crate::lexer::Token;

pub struct LinguisticValidator;

impl LinguisticValidator {
    /// Validates all linguistic rules for a discourse unit.
    pub fn validate(discourse: &Discourse) -> Result<(), OnuError> {
        match discourse {
            Discourse::Behavior { header, .. } => {
                for arg in &header.takes {
                    Self::validate_article(&arg.type_info.article, &arg.type_info.display_name)?;
                }
                // ReturnType is always safe because 'nothing' is handled or it uses OnuType
            }
            _ => {}
        }
        Ok(())
    }

    fn validate_article(article: &Token, type_name: &str) -> Result<(), OnuError> {
        let first_char = type_name.chars().next().unwrap_or(' ');
        let is_vowel = matches!(first_char.to_ascii_lowercase(), 'a' | 'e' | 'i' | 'o' | 'u');
        
        match (article, is_vowel) {
            (Token::The, _) => Ok(()), 
            (Token::Nothing, _) => Ok(()), 
            (Token::An, true) => Ok(()),
            (Token::A, false) => Ok(()),
            (Token::An, false) => Err(OnuError::ParseError {
                message: format!("LINGUISTIC VIOLATION: The discourse demands 'a' before '{}' (which initiates with a consonant).", type_name),
                span: Default::default(), // Validator needs spans, but AST currently doesn't store them for types
            }),
            (Token::A, true) => Err(OnuError::ParseError {
                message: format!("LINGUISTIC VIOLATION: The discourse demands 'an' before '{}' (which initiates with a vowel).", type_name),
                span: Default::default(),
            }),
            _ => Ok(()),
        }
    }
}
