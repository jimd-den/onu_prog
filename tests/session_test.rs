use onu::Session;
use onu::env::StdoutEnvironment;

#[test]
fn test_session_run_script_success() {
    let mut session = Session::new(Box::new(StdoutEnvironment));
    let script = r#"
the effect behavior called run
    with intent: test
    takes: nothing
    delivers: nothing
    as:
        broadcasts "Hello Session"
"#;
    let result = session.run_script(script);
    assert!(result.is_ok(), "Expected Ok, got {:?}", result);
}

#[test]
fn test_session_run_script_parse_error() {
    let mut session = Session::new(Box::new(StdoutEnvironment));
    let script = "the behavior called run as: broadcasts"; // Missing mandatory clauses
    let result = session.run_script(script);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.contains("Parse Error") || err.contains("refuse"));
}

#[test]
fn test_session_run_script_registry_error() {
    let mut session = Session::new(Box::new(StdoutEnvironment));
    let script = r#"
the behavior called foo
    with intent: test
    takes: nothing
    delivers: an integer
    as:
        10

the behavior called bar
    with intent: test
    takes: nothing
    delivers: an integer
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
    
    let script = r#"
the behavior called run
    with intent: test
    takes: nothing
    delivers: nothing
    as:
        derivation: x derives-from an integer 5
        x scales-by -- scales-by expects 2 args, but here it gets only 1 (subject)
        nothing
"#;
    let result = session.run_script(script);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.contains("Runtime Error") || err.contains("requires two numbers") || err.contains("argument"));
}
