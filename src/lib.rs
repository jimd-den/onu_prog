use crate::lexer::Lexer;
use crate::registry::Registry;
use crate::parser::{Parser, Discourse};
use crate::interpreter::{Interpreter, Value, DepthBudgetVisitor, Visitor};
use crate::env::Environment;

pub mod env;
pub mod error;
pub mod interpreter;
pub mod lexer;
pub mod parser;
pub mod registry;
pub mod builtins;

pub struct Session {
    registry: Registry,
    interpreter: Interpreter,
}

impl Session {
    pub fn new(env: Box<dyn Environment>) -> Self {
        Self {
            registry: Registry::new(),
            interpreter: Interpreter::new(env),
        }
    }

    pub fn run_script(&mut self, script: &str) -> Result<(), String> {
        let mut lexer = Lexer::new(script);
        let mut tokens = Vec::new();
        while let Some(t_with_span) = lexer.next_token() {
            tokens.push(t_with_span);
        }

        let mut parser = Parser::with_registry(tokens, &mut self.registry);
        let mut behaviors_to_run = Vec::new();

        while !parser.is_eof() {
            match parser.parse_discourse() {
                Ok(discourse) => {
                    match discourse {
                        Discourse::Module { ref name, .. } => {
                            println!("Found module '{}'", name);
                        }
                        Discourse::Behavior { ref header, ref body } => {
                            // KISS Enforcement: Depth Budget Check
                            let mut depth_visitor = DepthBudgetVisitor::new(4); // Strict budget as per specification
                            if let Err(e) = depth_visitor.visit_expression(body) {
                                return Err(format!("KISS Error: {}", e));
                            }

                            println!("Behavior '{}' parsed and registered", header.name);
                            self.interpreter.register_behavior(discourse.clone());
                            
                            if header.name == "run" || header.name == "main" {
                                behaviors_to_run.push(discourse.clone());
                            }
                        }
                        _ => {}
                    }
                }
                Err(e) => {
                    return Err(format!("Parse Error: {}", e));
                }
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
