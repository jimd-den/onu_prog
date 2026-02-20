use onu::Session;
use onu::env::StdoutEnvironment;

#[test]
fn test_session_run_script_success() {
    let mut session = Session::new(Box::new(StdoutEnvironment));
    let script = r#"
the behavior called run
    with intent: test
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
    let script = "the behavior called run as: emit"; // Missing expression
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
    as: 10

the behavior called bar
    with intent: test
    as: 10
"#;
    let result = session.run_script(script);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Parse Error"));
}

#[test]
fn test_session_run_script_runtime_error() {
    let mut session = Session::new(Box::new(StdoutEnvironment));
    let script = r#"
the behavior called run
    as:
        unknown-behavior receiving 10
"#;
    let result = session.run_script(script);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Runtime Error"));
}
