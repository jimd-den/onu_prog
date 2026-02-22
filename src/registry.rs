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
use crate::types::OnuType;
use std::collections::hash_map::DefaultHasher;
use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};

/// A semantic hash represents the structural uniqueness of an AST node.
pub type SemanticHash = u64;

/// BehaviorSignature defines the contract of a behavior.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BehaviorSignature {
    pub input_types: Vec<OnuType>,
    pub return_type: OnuType,
}

/// Computes a structural hash for any hashable item (usually an Expression AST node).
pub fn compute_hash<T: Hash>(item: &T) -> SemanticHash {
    let mut hasher = DefaultHasher::new();
    item.hash(&mut hasher);
    hasher.finish()
}

/// Computes a semantic hash for a behavior, incorporating both its implementation
/// (body) and its type signature. This ensures that DRY enforcement respects
/// type-based differences.
pub fn compute_behavior_hash(body: &crate::parser::Expression, signature: &BehaviorSignature) -> SemanticHash {
    let mut hasher = DefaultHasher::new();
    body.hash(&mut hasher);
    signature.hash(&mut hasher);
    hasher.finish()
}

/// The Registry maintains a map of semantic hashes to behavior names.
#[derive(Debug, Clone)]
pub struct Registry {
    /// A map from semantic hash to the first name associated with that implementation.
    entries: HashMap<SemanticHash, String>, // Hash -> Name
    /// A set of all registered behavior names (built-ins and user-defined).
    names: HashSet<String>,
    /// A set of behavior names that have been implemented (built-ins or user-defined).
    implemented_names: HashSet<String>,
    /// A map from behavior name to its arity (number of parameters).
    arities: HashMap<String, usize>,
    /// A map from behavior name to its full type signature.
    signatures: HashMap<String, BehaviorSignature>,
    /// A map from shape name to its list of required behavior signatures.
    shapes: HashMap<String, Vec<(String, BehaviorSignature)>>,
    /// A set of registered suite names to track dynamic loading.
    suites: HashSet<String>,
}

impl Registry {
    /// Creates a new, empty Registry.
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
            names: HashSet::new(),
            implemented_names: HashSet::new(),
            arities: HashMap::new(),
            signatures: HashMap::new(),
            shapes: HashMap::new(),
            suites: HashSet::new(),
        }
    }

    /// Registers a suite of behaviors and shapes.
    pub fn add_suite(&mut self, name: &str, signatures: Vec<(&str, BehaviorSignature)>, shapes: Vec<(&str, Vec<(String, BehaviorSignature)>)>) {
        if self.suites.insert(name.to_string()) {
            for (bh_name, sig) in signatures {
                self.add_signature(bh_name, sig);
                self.mark_implemented(bh_name);
            }
            for (sh_name, behaviors) in shapes {
                self.add_shape(sh_name, behaviors);
            }
        }
    }

    /// Registers a new shape definition.
    pub fn add_shape(&mut self, name: &str, behaviors: Vec<(String, BehaviorSignature)>) {
        self.shapes.insert(name.to_string(), behaviors);
    }

    /// Returns the behavior requirements for a shape.
    pub fn get_shape(&self, name: &str) -> Option<&Vec<(String, BehaviorSignature)>> {
        self.shapes.get(name)
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
        self.names.insert(name.clone());
        self.implemented_names.insert(name.clone());
        self.entries.insert(hash, name);
        Ok(())
    }

    /// Checks if a name is already registered as a behavior.
    pub fn is_registered(&self, name: &str) -> bool {
        self.names.contains(name)
    }

    /// Checks if a behavior has been implemented.
    pub fn is_implemented(&self, name: &str) -> bool {
        self.implemented_names.contains(name)
    }

    /// Marks a behavior name as implemented (e.g. for built-ins).
    pub fn mark_implemented(&mut self, name: &str) {
        self.implemented_names.insert(name.to_string());
    }

    /// Returns the arity of a registered behavior.
    pub fn get_arity(&self, name: &str) -> Option<usize> {
        self.arities.get(name).copied()
    }

    /// Adds a behavior name and its arity to the registry.
    pub fn add_name(&mut self, name: &str, arity: usize) {
        self.names.insert(name.to_string());
        self.arities.insert(name.to_string(), arity);
    }

    /// Adds a full behavior signature to the registry.
    pub fn add_signature(&mut self, name: &str, signature: BehaviorSignature) {
        self.names.insert(name.to_string());
        self.arities.insert(name.to_string(), signature.input_types.len());
        self.signatures.insert(name.to_string(), signature);
    }

    /// Returns the signature of a registered behavior.
    pub fn get_signature(&self, name: &str) -> Option<&BehaviorSignature> {
        self.signatures.get(name)
    }

    /// Verifies if a type satisfies a specific shape (interface).
    /// Currently, this is a structural check: does the registry contain all 
    /// behaviors promised by the shape for this type?
    ///
    /// Logic: When the parser encounters `acts-as`, the Registry must perform 
    /// a deep comparison of the Subject's AST against the Shape's Promises.
    pub fn satisfies(&self, _type_name: &str, shape_name: &str) -> bool {
        if let Some(required_behaviors) = self.shapes.get(shape_name) {
            for (bh_name, required_sig) in required_behaviors {
                // Must be implemented to satisfy a shape
                if !self.implemented_names.contains(bh_name) {
                    return false;
                }
                
                if let Some(existing_sig) = self.signatures.get(bh_name) {
                    if existing_sig != required_sig {
                        return false;
                    }
                }
            }
            true
        } else {
            false
        }
    }

    /// High-level satisfaction check that returns a Result with a descriptive error.
    pub fn verify_acts_as(&self, subject_name: &str, shape_name: &str) -> Result<(), OnuError> {
        if let Some(required_behaviors) = self.shapes.get(shape_name) {
            for (bh_name, required_sig) in required_behaviors {
                let matched = if let Some(existing_sig) = self.signatures.get(bh_name) {
                    existing_sig == required_sig
                } else {
                    self.names.contains(bh_name)
                };

                if !matched {
                    return Err(OnuError::ParseError {
                        message: format!("VIOLATION: [{}] refuses to act-as [{}] because it lacks the [{}] action", 
                            subject_name, shape_name, bh_name),
                        span: Default::default(),
                    });
                }
            }
            Ok(())
        } else {
            Err(OnuError::ParseError {
                message: format!("VIOLATION: Shape [{}] is not defined in the registry", shape_name),
                span: Default::default(),
            })
        }
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

    #[test]
    fn test_registry_tracks_names_and_arities() {
        let mut registry = Registry::new();
        registry.add_name("add", 2);
        assert!(registry.is_registered("add"));
        assert_eq!(registry.get_arity("add"), Some(2));
        assert!(!registry.is_registered("sub"));
        
        let hash = compute_hash(&10u64);
        registry.register("foo".to_string(), hash).unwrap();
        assert!(registry.is_registered("foo"));
    }

    #[test]
    fn test_behavior_collision_detection() {
        use crate::parser::Expression;
        use crate::types::OnuType;

        let mut registry = Registry::new();
        let body = Expression::I64(10);
        let sig = BehaviorSignature {
            input_types: vec![],
            return_type: OnuType::I64,
        };

        let hash = compute_behavior_hash(&body, &sig);
        registry.register("foo".to_string(), hash).unwrap();

        // Same body, same signature -> conflict
        let result = registry.register("bar".to_string(), hash);
        assert!(result.is_err());

        // Same body, different signature -> no conflict
        let sig2 = BehaviorSignature {
            input_types: vec![],
            return_type: OnuType::F64,
        };
        let hash2 = compute_behavior_hash(&body, &sig2);
        registry.register("baz".to_string(), hash2).unwrap();
    }
}
