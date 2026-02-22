use onu::Session;
use onu::env::StdoutEnvironment;

#[test]
fn test_diagnostic_error_format() {
    let mut session = Session::new(Box::new(StdoutEnvironment));
    let script = r#"
the behavior called broken
    with intent: demonstrate error
    takes: nothing
    delivers: nothing
    as:
        [ 1 2 : 3 ] -- Matrix error
"#;
    let result = session.run_script(script);
    assert!(result.is_err());
    let err = result.unwrap_err();
    println!("Error: {}", err);
    // We expect a peer review style error
    assert!(err.contains("PEER REVIEW MEMO"));
    assert!(err.contains("Observation:"));
}
