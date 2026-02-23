use onu::CompilerSession;
use onu::hir::HirDiscourse;

#[test]
fn test_compiler_session_initialization() {
    let session = CompilerSession::new();
    assert!(session.is_ok());
}

#[test]
fn test_monomorphization() {
    let mut session = CompilerSession::new().unwrap();
    let source = "
the shape Measurable promises:
    a behavior called magnitude
        receiving: nothing
        returning: a float

the behavior called magnitude
    receiving:
        a float called input
    returning:
        a float
    as:
        input

the behavior called get-size
    receiving:
        a Measurable via the role Measurable called item
    returning:
        a float
    as:
        item magnitude

the behavior called main
    as:
        let x is a float 10.5
        (x acts-as a Measurable) get-size
";
    let result = session.compile(source);
    if let Err(e) = &result {
        eprintln!("Compile Error in test_monomorphization: {}", e);
    }
    assert!(result.is_ok());
    
    eprintln!("HIR after compile in test_monomorphization: {}", session.hir.len());
    for item in &session.hir {
        if let HirDiscourse::Behavior { header, .. } = item {
            eprintln!("Found behavior in HIR: {}", header.name);
        }
    }

    // In a monomorphized world, we expect a specialized 'get-size' for 'float'
    // This is a simplified check for the presence of specialized HIR
    let has_specialized = session.hir.iter().any(|d| {
        if let HirDiscourse::Behavior { header, .. } = d {
            header.name.contains("get-size") && header.args.iter().any(|a| a.typ == onu::types::OnuType::F64)
        } else {
            false
        }
    });
    
    assert!(has_specialized, "Should have a specialized 'get-size' behavior for float");

    // Check if main was rewritten to use get-size_float
    let main_rewritten = session.hir.iter().any(|d| {
        if let HirDiscourse::Behavior { header, body } = d {
            if header.name == "main" {
                // Check if body contains a call to get-size_float
                let body_str = format!("{:?}", body);
                body_str.contains("get-size_float")
            } else {
                false
            }
        } else {
            false
        }
    });
    
    assert!(main_rewritten, "Main behavior call sites should be rewritten for static dispatch");
}

#[test]
fn test_mir_lowering() {
    let mut session = CompilerSession::new().unwrap();
    session.registry.add_name("added-to", 2);

    let source = "
the behavior called add-five
    receiving:
        an integer called input
    returning:
        an integer
    as:
        input added-to 5
";
    let result = session.compile(source);
    if let Err(e) = &result {
        eprintln!("Compile Error: {}", e);
    }
    assert!(result.is_ok());
    
    assert!(session.mir.is_some());
    let mir = session.mir.unwrap();
    
    let func = mir.functions.iter().find(|f| f.name == "add-five").expect("Should find add-five function in MIR");
    assert_eq!(func.args.len(), 1);
    assert_eq!(func.blocks.len(), 1);
    
    let block = &func.blocks[0];
    // Should have one BinaryOperation instruction (Add)
    let has_add = block.instructions.iter().any(|i| matches!(i, onu::mir::MirInstruction::BinaryOperation { op: onu::mir::MirBinOp::Add, .. }));
    assert!(has_add, "MIR should contain an Add operation");
}

#[test]
fn test_llvm_codegen() {
    let mut session = CompilerSession::new().unwrap();
    // StandardMath should be in registry by default now
    let source = "
the behavior called multiply
    receiving:
        an integer called a
        an integer called b
    returning:
        an integer
    as:
        a scales-by b
";
    let result = session.compile(source);
    if let Err(e) = &result {
        eprintln!("Compile Error in test_llvm_codegen: {}", e);
    }
    let binary = result.unwrap();
    assert!(!binary.is_empty(), "Binary output should not be empty");
}

#[test]
fn test_pedagogical_diagnostics() {
    let mut session = CompilerSession::new().unwrap();
    let source = "the behavior called invalid as let x is 5"; // Missing colon after as
    let result = session.compile(source);
    
    assert!(result.is_err());
    let err = result.unwrap_err();
    let err_str = format!("{}", err);
    
    assert!(err_str.contains("PEER REVIEW MEMO"));
    assert!(err_str.contains("Observation:"));
    assert!(err_str.contains("Assessment:"));
    assert!(err_str.contains("Conclusion:"));
    assert!(err_str.contains("violates the grammatical covenant"));
}
