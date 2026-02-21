use crate::lexer::Lexer;
use crate::registry::{Registry, BehaviorSignature};
use crate::parser::{Parser, Discourse};
use crate::interpreter::{Interpreter, Value};
use crate::env::Environment;
use crate::types::OnuType;

pub mod env;
pub mod error;
pub mod interpreter;
pub mod lexer;
pub mod parser;
pub mod registry;
pub mod builtins;
pub mod types;
pub mod linguistics;

pub struct Session {
    registry: Registry,
    interpreter: Interpreter,
}

impl Session {
    pub fn new(env: Box<dyn Environment>) -> Self {
        let mut registry = Registry::new();
        // Register core built-ins
        let core_builtins = vec![
            ("joined-with", BehaviorSignature { input_types: vec![OnuType::Strings, OnuType::Strings], return_type: OnuType::Strings }),
            ("len", BehaviorSignature { input_types: vec![OnuType::Strings], return_type: OnuType::I64 }),
            ("char-at", BehaviorSignature { input_types: vec![OnuType::Strings, OnuType::I64], return_type: OnuType::I64 }),
            ("as-text", BehaviorSignature { input_types: vec![OnuType::I64], return_type: OnuType::Strings }),
            ("set-char", BehaviorSignature { input_types: vec![OnuType::Strings, OnuType::I64, OnuType::I64], return_type: OnuType::Strings }),
        ];
        for (name, sig) in core_builtins {
            registry.add_signature(name, sig);
            registry.mark_implemented(name);
        }

        // Register the Math Library as a Suite
        let math_signatures = vec![
            ("added-to", BehaviorSignature { input_types: vec![OnuType::I64, OnuType::I64], return_type: OnuType::I64 }),
            ("decreased-by", BehaviorSignature { input_types: vec![OnuType::I64, OnuType::I64], return_type: OnuType::I64 }),
            ("subtracted-from", BehaviorSignature { input_types: vec![OnuType::I64, OnuType::I64], return_type: OnuType::I64 }),
            ("multiplied-by", BehaviorSignature { input_types: vec![OnuType::I64, OnuType::I64], return_type: OnuType::I64 }),
            ("divided-by", BehaviorSignature { input_types: vec![OnuType::I64, OnuType::I64], return_type: OnuType::I64 }),
            ("is-zero", BehaviorSignature { input_types: vec![OnuType::I64], return_type: OnuType::I64 }),
            ("is-less", BehaviorSignature { input_types: vec![OnuType::I64, OnuType::I64], return_type: OnuType::I64 }),
            ("is-equal", BehaviorSignature { input_types: vec![OnuType::I64, OnuType::I64], return_type: OnuType::I64 }),
            ("both-true", BehaviorSignature { input_types: vec![OnuType::I64, OnuType::I64], return_type: OnuType::I64 }),
            ("either-true", BehaviorSignature { input_types: vec![OnuType::I64, OnuType::I64], return_type: OnuType::I64 }),
            ("not-true", BehaviorSignature { input_types: vec![OnuType::I64], return_type: OnuType::I64 }),
            ("is-equal-to", BehaviorSignature { input_types: vec![OnuType::I64, OnuType::I64], return_type: OnuType::I64 }),
            ("is-greater-than", BehaviorSignature { input_types: vec![OnuType::I64, OnuType::I64], return_type: OnuType::I64 }),
            ("is-less-than", BehaviorSignature { input_types: vec![OnuType::I64, OnuType::I64], return_type: OnuType::I64 }),
            ("sine", BehaviorSignature { input_types: vec![OnuType::F64], return_type: OnuType::F64 }),
            ("cosine", BehaviorSignature { input_types: vec![OnuType::F64], return_type: OnuType::F64 }),
            ("tangent", BehaviorSignature { input_types: vec![OnuType::F64], return_type: OnuType::F64 }),
            ("arcsin", BehaviorSignature { input_types: vec![OnuType::F64], return_type: OnuType::F64 }),
            ("arccos", BehaviorSignature { input_types: vec![OnuType::F64], return_type: OnuType::F64 }),
            ("arctan", BehaviorSignature { input_types: vec![OnuType::F64], return_type: OnuType::F64 }),
            ("square-root", BehaviorSignature { input_types: vec![OnuType::F64], return_type: OnuType::F64 }),
            ("raised-to", BehaviorSignature { input_types: vec![OnuType::F64, OnuType::F64], return_type: OnuType::F64 }),
            ("natural-log", BehaviorSignature { input_types: vec![OnuType::F64], return_type: OnuType::F64 }),
            ("exponent", BehaviorSignature { input_types: vec![OnuType::F64], return_type: OnuType::F64 }),
            ("dot-product", BehaviorSignature { input_types: vec![OnuType::Tuple(vec![]), OnuType::Tuple(vec![])], return_type: OnuType::F64 }),
            ("cross-product", BehaviorSignature { input_types: vec![OnuType::Tuple(vec![]), OnuType::Tuple(vec![])], return_type: OnuType::Tuple(vec![]) }),
            ("determinant", BehaviorSignature { input_types: vec![OnuType::Matrix], return_type: OnuType::F64 }),
        ];

        let math_shapes = vec![
            ("Addable", vec![
                ("added-to".to_string(), BehaviorSignature { input_types: vec![OnuType::Shape("Addable".to_string()), OnuType::Shape("Addable".to_string())], return_type: OnuType::Shape("Addable".to_string()) }),
                ("subtracted-from".to_string(), BehaviorSignature { input_types: vec![OnuType::Shape("Addable".to_string()), OnuType::Shape("Addable".to_string())], return_type: OnuType::Shape("Addable".to_string()) }),
            ]),
            ("Multiplicable", vec![
                ("multiplied-by".to_string(), BehaviorSignature { input_types: vec![OnuType::Shape("Multiplicable".to_string()), OnuType::Shape("Multiplicable".to_string())], return_type: OnuType::Shape("Multiplicable".to_string()) }),
                ("divided-by".to_string(), BehaviorSignature { input_types: vec![OnuType::Shape("Multiplicable".to_string()), OnuType::Shape("Multiplicable".to_string())], return_type: OnuType::Shape("Multiplicable".to_string()) }),
            ]),
            ("Measurable", vec![
                ("magnitude".to_string(), BehaviorSignature { input_types: vec![OnuType::Shape("Measurable".to_string())], return_type: OnuType::F64 }),
            ]),
        ];

        registry.add_suite("StandardMath", math_signatures, math_shapes);

        Self {
            registry,
            interpreter: Interpreter::new(env),
        }
    }

    pub fn run_script(&mut self, script: &str) -> Result<(), String> {
        let mut lexer = Lexer::new(script);
        let mut tokens = Vec::new();
        while let Some(t_with_span) = lexer.next_token() {
            tokens.push(t_with_span);
        }

        // Pass 1: Structural Pass (Populate Registry Signatures)
        let mut current_pos = 0;
        while current_pos < tokens.len() {
             let discourse = {
                 let mut parser = Parser::new(&tokens[current_pos..]);
                 let d = parser.parse_structural_discourse().map_err(|e| format!("Structural Parse Error: {}", e))?;
                 current_pos += parser.pos;
                 d
             };

             // Linguistic Validation (a/an)
             crate::linguistics::LinguisticValidator::validate(&discourse)
                 .map_err(|e| format!("Linguistic Error: {}", e))?;

             match discourse {
                 Discourse::Behavior { ref header, .. } => {
                     let inputs = header.receiving.iter().map(|a| a.type_info.onu_type.clone()).collect();
                     let ret = header.returning.0.clone();
                     self.registry.add_signature(&header.name, BehaviorSignature {
                         input_types: inputs,
                         return_type: ret,
                     });
                 }
                 Discourse::Shape { ref name, ref behaviors } => {
                     let mut behavior_sigs = Vec::new();
                     for bh in behaviors {
                         let inputs = bh.receiving.iter().map(|a| a.type_info.onu_type.clone()).collect();
                         let ret = bh.returning.0.clone();
                         let sig = BehaviorSignature {
                             input_types: inputs,
                             return_type: ret,
                         };
                         self.registry.add_signature(&bh.name, sig.clone());
                         behavior_sigs.push((bh.name.clone(), sig));
                     }
                     self.registry.add_shape(name, behavior_sigs);
                 }
                 _ => {}
             }
        }

        // Pass 2: Semantic Pass (Full Logic and Disambiguation)
        let mut behaviors_to_run = Vec::new();
        let mut concern_validator = crate::interpreter::ConcernValidator::new();
        current_pos = 0;
        while current_pos < tokens.len() {
             let discourse = {
                let mut parser = Parser::with_registry(&tokens[current_pos..], &self.registry);
                let d = parser.parse_discourse().map_err(|e| format!("Semantic Parse Error: {}", e))?;
                current_pos += parser.pos;
                d
            };

            // Concern Validation (SRP Enforcement)
            concern_validator.check(&discourse).map_err(|e| format!("Semantic Analysis Error: {}", e))?;

            match discourse {
                Discourse::Behavior { ref header, ref body } => {
                    // Termination Check (Proof-Based Structural Recursion)
                    let mut term_checker = crate::interpreter::TerminationChecker::new(&self.registry);
                    term_checker.check(&discourse).map_err(|e| format!("Semantic Analysis Error: {}", e))?;

                    // Shape Verification (Structural Subtyping)
                    let mut shape_validator = crate::interpreter::ShapeValidator::new(&self.registry);
                    shape_validator.check(&discourse).map_err(|e| format!("Semantic Analysis Error: {}", e))?;

                    // DRY Enforcement: Semantic Hashing (including Type Signatures)
                    let signature = self.registry.get_signature(&header.name).cloned().unwrap();
                    let hash = crate::registry::compute_behavior_hash(body, &signature);
                    
                    if let Err(e) = self.registry.register(header.name.clone(), hash) {
                        return Err(format!("DRY Error: {}", e));
                    }

                    println!("Behavior '{}' parsed and registered", header.name);
                    self.interpreter.register_behavior(discourse.clone());
                    
                    if header.name == "run" || header.name == "main" {
                        behaviors_to_run.push(discourse.clone());
                    }
                }
                Discourse::Module { ref name, .. } => {
                    println!("Found module '{}'", name);
                }
                _ => {}
            }
        }

        for behavior in behaviors_to_run {
            match self.interpreter.execute_discourse(&behavior) {
                Ok(result) => {
                    if result != Value::Void {
                        // In a real session, we might want to return these values
                    }
                }
                Err(e) => {
                    return Err(format!("Runtime Error: {}", e));
                }
            }
        }

        Ok(())
    }
}
