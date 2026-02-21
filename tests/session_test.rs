use onu::Session;
use onu::env::StdoutEnvironment;

#[test]
fn test_session_run_script_success() {
    let mut session = Session::new(Box::new(StdoutEnvironment));
    let script = r#"
the effect behavior called run
    with intent: test
    receiving:
    returning: nothing
    as:
        emit "Hello Session"
"#;
    let result = session.run_script(script);
    assert!(result.is_ok(), "Expected Ok, got {:?}", result);
}

#[test]
fn test_session_run_script_parse_error() {
    let mut session = Session::new(Box::new(StdoutEnvironment));
    let script = "the behavior called run as: emit"; // Missing mandatory clauses
    let result = session.run_script(script);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Parse Error"));
}

#[test]
fn test_session_run_script_registry_error() {
    let mut session = Session::new(Box::new(StdoutEnvironment));
    let script = r#"
the behavior called foo
    with intent: test
    receiving:
    returning: an integer
    as:
        10

the behavior called bar
    with intent: test
    receiving:
    returning: an integer
    as:
        10
"#;
    let result = session.run_script(script);
    assert!(result.is_err());
    // Should be a BehaviorConflict (DRY Error)
    assert!(result.unwrap_err().contains("identical"));
}

#[test]
fn test_session_run_script_runtime_error() {
    let mut session = Session::new(Box::new(StdoutEnvironment));
    // We use a behavior that calls another behavior that is NOT registered.
    // To pass the parser's arity-based SVO check, we must use a behavior 
    // that IS registered in the Registry but NOT implemented in the Interpreter.
    // However, currently Registry and Interpreter are decoupled.
    
    // Let's use a simple unknown identifier in a context that passes parsing.
    let script = r#"
the behavior called wrapper
    with intent: wrap
    receiving:
    returning: nothing
    as:
        let x is an integer 5
        nothing unknown-behavior -- Treated as behavior call if in registry, but fails at runtime
        nothing

the behavior called run
    with intent: test
    receiving:
    returning: nothing
    as:
        wrapper
"#;
    // We need to manually add 'unknown-behavior' to the registry names to pass Pass 2 (Semantic Pass) 
    // but it won't be in the behaviors map of the interpreter.
    let mut session = Session::new(Box::new(StdoutEnvironment));
    // Actually, run_script uses its OWN registry internally for the script.
    // So we can't easily pre-populate it unless we expose it.
    
    // Alternative: use a name that is in registry (like a builtin) but call it with WRONG arity?
    // Builtins check arity at runtime too.
    
    let script_v2 = r#"
the behavior called run
    with intent: test
    receiving:
    returning: nothing
    as:
        let x is an integer 5
        x multiplied-by -- multiplied-by expects 2 args, but here it gets only 1 (subject)
        nothing
"#;
    let result = session.run_script(script_v2);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.contains("Runtime Error") || err.contains("Arity") || err.contains("argument"));
}
