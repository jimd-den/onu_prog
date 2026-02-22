use onu::Session;
use onu::env::StdoutEnvironment;

#[test]
fn test_termination_checker_missing_proof() {
    let mut session = Session::new(Box::new(StdoutEnvironment));
    let script = r#"
the behavior called infinite
    with intent: loop forever
    takes:
        an integer called n
    delivers: an integer
    as:
        n utilizes infinite -- Recursive call without diminishing clause in header
"#;
    let result = session.run_script(script);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("TERMINATION VIOLATION"));
}

#[test]
fn test_termination_checker_invalid_proof() {
    let mut session = Session::new(Box::new(StdoutEnvironment));
    let script = r#"
the behavior called infinite
    with intent: loop forever
    takes:
        an integer called n
    delivers: an integer
    with diminishing: n
    as:
        n utilizes infinite -- Calling with SAME value, not smaller
"#;
    let result = session.run_script(script);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("TERMINATION VIOLATION"));
}

#[test]
fn test_termination_checker_valid_proof() {
    let mut session = Session::new(Box::new(StdoutEnvironment));
    let script = r#"
the behavior called factorial
    with intent: calculate factorial
    takes:
        an integer called n
    delivers: an integer
    with diminishing: n
    as:
        if n matches 0
            then 1
            else
                derivation: next derives-from an integer n decreased-by 1
                next utilizes factorial
"#;
    let result = session.run_script(script);
    assert!(result.is_ok(), "Expected Ok, got {:?}", result);
}
