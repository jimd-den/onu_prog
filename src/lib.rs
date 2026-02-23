use crate::lexer::Lexer;
use crate::registry::{Registry, BehaviorSignature};
use crate::parser::{Parser, Discourse};
use crate::types::OnuType;
use crate::error::OnuError;

pub mod env;
pub mod error;
pub mod lexer;
pub mod parser;
pub mod registry;
pub mod types;
pub mod linguistics;
pub mod hir;
pub mod monomorphize;
pub mod mir;
pub mod codegen;

pub struct CompilerSession {
    pub registry: Registry,
    pub ast: Vec<Discourse>,
    pub hir: Vec<crate::hir::HirDiscourse>,
    pub mir: Option<crate::mir::MirProgram>,
}

impl CompilerSession {
    pub fn new() -> Result<Self, String> {
        let mut registry = Registry::new();
        let core_builtins = vec![
            ("joined-with", BehaviorSignature { input_types: vec![OnuType::Strings, OnuType::Strings], return_type: OnuType::Strings }),
            ("len", BehaviorSignature { input_types: vec![OnuType::Strings], return_type: OnuType::I64 }),
            ("char-at", BehaviorSignature { input_types: vec![OnuType::Strings, OnuType::I64], return_type: OnuType::I64 }),
            ("as-text", BehaviorSignature { input_types: vec![OnuType::I64], return_type: OnuType::Strings }),
            ("set-char", BehaviorSignature { input_types: vec![OnuType::Strings, OnuType::I64, OnuType::I64], return_type: OnuType::Strings }),
            ("broadcasts", BehaviorSignature { input_types: vec![OnuType::Strings], return_type: OnuType::Nothing }),
            ("tail-of", BehaviorSignature { input_types: vec![OnuType::Strings], return_type: OnuType::Strings }),
            ("init-of", BehaviorSignature { input_types: vec![OnuType::Strings], return_type: OnuType::Strings }),
            ("char-from-code", BehaviorSignature { input_types: vec![OnuType::I64], return_type: OnuType::Strings }),
        ];
        for (name, sig) in core_builtins {
            registry.add_signature(name, sig);
            registry.mark_implemented(name);
        }

        let math_signatures = vec![
            ("added-to", BehaviorSignature { input_types: vec![OnuType::I64, OnuType::I64], return_type: OnuType::I64 }),
            ("decreased-by", BehaviorSignature { input_types: vec![OnuType::I64, OnuType::I64], return_type: OnuType::I64 }),
            ("scales-by", BehaviorSignature { input_types: vec![OnuType::I64, OnuType::I64], return_type: OnuType::I64 }),
            ("partitions-by", BehaviorSignature { input_types: vec![OnuType::I64, OnuType::I64], return_type: OnuType::I64 }),
            ("matches", BehaviorSignature { input_types: vec![OnuType::I64, OnuType::I64], return_type: OnuType::I64 }),
            ("exceeds", BehaviorSignature { input_types: vec![OnuType::I64, OnuType::I64], return_type: OnuType::I64 }),
            ("falls-short-of", BehaviorSignature { input_types: vec![OnuType::I64, OnuType::I64], return_type: OnuType::I64 }),
        ];
        
        let math_shapes = vec![
            ("Addable", vec![
                ("added-to".to_string(), BehaviorSignature { input_types: vec![OnuType::Shape("Addable".to_string()), OnuType::Shape("Addable".to_string())], return_type: OnuType::Shape("Addable".to_string()) }),
                ("decreased-by".to_string(), BehaviorSignature { input_types: vec![OnuType::Shape("Addable".to_string()), OnuType::Shape("Addable".to_string())], return_type: OnuType::Shape("Addable".to_string()) }),
            ]),
            ("Multiplicable", vec![
                ("scales-by".to_string(), BehaviorSignature { input_types: vec![OnuType::Shape("Multiplicable".to_string()), OnuType::Shape("Multiplicable".to_string())], return_type: OnuType::Shape("Multiplicable".to_string()) }),
                ("partitions-by".to_string(), BehaviorSignature { input_types: vec![OnuType::Shape("Multiplicable".to_string()), OnuType::Shape("Multiplicable".to_string())], return_type: OnuType::Shape("Multiplicable".to_string()) }),
            ]),
        ];

        registry.add_suite("StandardMath", math_signatures, math_shapes);

        Ok(Self {
            registry,
            ast: Vec::new(),
            hir: Vec::new(),
            mir: None,
        })
    }

    pub fn compile(&mut self, _source: &str) -> Result<Vec<u8>, OnuError> {
        if _source.is_empty() {
             return Ok(Vec::new());
        }
        let tokens = self.lex(_source).map_err(|e| OnuError::LexicalError { message: e, span: Default::default() })?;
        
        let mut current_pos = 0;
        while current_pos < tokens.len() {
             let mut parser = Parser::new(&tokens[current_pos..]);
             if let Ok(discourse) = parser.parse_structural_discourse() {
                 match discourse {
                     Discourse::Behavior { ref header, .. } => {
                         let inputs = header.takes.iter().map(|a| a.type_info.onu_type.clone()).collect();
                         let ret = header.delivers.0.clone();
                         self.registry.add_signature(&header.name, BehaviorSignature {
                             input_types: inputs,
                             return_type: ret,
                         });
                     }
                     Discourse::Shape { ref name, ref behaviors } => {
                         let mut behavior_sigs = Vec::new();
                         for bh in behaviors {
                             let inputs = bh.takes.iter().map(|a| a.type_info.onu_type.clone()).collect();
                             let ret = bh.delivers.0.clone();
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
                 current_pos += parser.pos;
             } else {
                 break;
             }
        }

        self.ast = self.parse(&tokens)?;
        self.hir = self.lower(&self.ast).map_err(|e| OnuError::MonomorphizationError { message: e })?;
        let mir = Self::analyze(&mut self.hir, &self.registry).map_err(|e| OnuError::MonomorphizationError { message: e })?;
        self.mir = Some(mir.clone());
        let binary = self.emit(&mir).map_err(|e| OnuError::CodeGenError { message: e })?;
        
        Ok(binary)
    }

    pub fn get_llvm_ir(&self, _source: &str) -> Result<String, OnuError> {
        let mut session = Self::new().unwrap();
        session.compile(_source)?;
        let context = inkwell::context::Context::create();
        let generator = crate::codegen::LlvmGenerator::new(&context, "onu_module", Some(session.registry.clone()));
        use crate::codegen::CodeGenerator;
        generator.generate(session.mir.as_ref().unwrap()).map_err(|e| OnuError::CodeGenError { message: e })?;
        Ok(generator.get_ir_string())
    }

    fn lex(&self, _source: &str) -> Result<Vec<crate::lexer::TokenWithSpan>, String> {
        let mut lexer = Lexer::new(_source);
        let mut tokens = Vec::new();
        while let Some(t) = lexer.next_token() {
            tokens.push(t);
        }
        Ok(tokens)
    }

    fn parse(&self, _tokens: &[crate::lexer::TokenWithSpan]) -> Result<Vec<Discourse>, OnuError> {
        let mut current_pos = 0;
        let mut ast = Vec::new();
        while current_pos < _tokens.len() {
             let mut parser = Parser::with_registry(&_tokens[current_pos..], &self.registry);
             match parser.parse_discourse() {
                 Ok(discourse) => {
                     current_pos += parser.pos;
                     ast.push(discourse);
                 }
                 Err(e) => {
                     return Err(e);
                 }
             }
        }
        Ok(ast)
    }

    fn lower(&self, _ast: &[Discourse]) -> Result<Vec<crate::hir::HirDiscourse>, String> {
        Ok(_ast.iter().map(crate::hir::LoweringVisitor::lower_discourse).collect())
    }

    fn analyze(hir: &mut Vec<crate::hir::HirDiscourse>, registry: &Registry) -> Result<crate::mir::MirProgram, String> {
        crate::monomorphize::Monomorphizer::run(hir);
        let mut builder = crate::mir::MirBuilder::new();
        // Pass registry info if needed for builder
        Ok(builder.build_program_with_registry(hir, registry))
    }

    fn emit(&self, _mir: &crate::mir::MirProgram) -> Result<Vec<u8>, String> {
        use crate::codegen::CodeGenerator;
        let context = inkwell::context::Context::create();
        let generator = crate::codegen::LlvmGenerator::new(&context, "onu_module", Some(self.registry.clone()));
        generator.generate(_mir)
    }
}
