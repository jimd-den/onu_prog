use onu::Session;
use onu::env::StdoutEnvironment;

#[test]
fn test_multiple_modules_fail() {
    let mut session = Session::new(Box::new(StdoutEnvironment));
    let script = r#"
the module called Math
    with concern: arithmetic

the module called IO
    with concern: logging
"#;
    let result = session.run_script(script);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.contains("Concern Error") || err.contains("SRP Violation"));
}

#[test]
fn test_behavior_alignment_pass() {
    let mut session = Session::new(Box::new(StdoutEnvironment));
    let script = r#"
the module called Math
    with concern: arithmetic and calculation

the behavior called add
    with intent: arithmetic addition
    receiving:
        an integer called a
        an integer called b
    returning: an integer
    as:
        a added-to b
"#;
    let result = session.run_script(script);
    assert!(result.is_ok(), "Expected Ok, got {:?}", result);
}
