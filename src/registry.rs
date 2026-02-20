/// Ọ̀nụ Semantic Registry: The DRY Enforcement Layer
///
/// This module provides a mechanism to ensure that every behavior's implementation 
/// is unique within a program. It implements the Single-Source Registry mandate.
///
/// Enforcement Mechanism:
/// The parser hashes the semantic body (the AST) of every behavior declaration.
/// If two declarations produce the same hash, the compiler refuses to parse 
/// the second, preventing duplicate logic across the codebase.

use crate::error::OnuError;
use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};

/// A semantic hash represents the structural uniqueness of an AST node.
pub type SemanticHash = u64;

/// Computes a structural hash for any hashable item (usually an Expression AST node).
pub fn compute_hash<T: Hash>(item: &T) -> SemanticHash {
    let mut hasher = DefaultHasher::new();
    item.hash(&mut hasher);
    hasher.finish()
}

/// The Registry maintains a map of semantic hashes to behavior names.
pub struct Registry {
    /// A map from semantic hash to the first name associated with that implementation.
    entries: HashMap<SemanticHash, String>, // Hash -> Name
}

impl Registry {
    /// Creates a new, empty Registry.
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
        }
    }

    /// Registers a new behavior implementation by its name and semantic hash.
    /// If the hash already exists, it returns a BehaviorConflict error (DRY enforcement).
    pub fn register(&mut self, name: String, hash: SemanticHash) -> Result<(), OnuError> {
        if let Some(existing_name) = self.entries.get(&hash) {
            return Err(OnuError::BehaviorConflict {
                name,
                other_name: existing_name.clone(),
            });
        }
        self.entries.insert(hash, name);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_identical_produce_identical_hash() {
        let val1 = 42u64;
        let val2 = 42u64;
        assert_eq!(compute_hash(&val1), compute_hash(&val2));
    }

    #[test]
    fn test_different_produce_different_hash() {
        let val1 = 42u64;
        let val2 = 43u64;
        assert_ne!(compute_hash(&val1), compute_hash(&val2));
    }

    #[test]
    fn test_registry_rejects_duplicate_hash() {
        let mut registry = Registry::new();
        let hash = compute_hash(&10u64);

        registry.register("foo".to_string(), hash).unwrap();
        let result = registry.register("bar".to_string(), hash);

        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            OnuError::BehaviorConflict {
                name: "bar".to_string(),
                other_name: "foo".to_string()
            }
        );
    }
}
