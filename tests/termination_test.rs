use onu::Session;
use onu::env::StdoutEnvironment;

#[test]
fn test_termination_checker_missing_proof() {
    let mut session = Session::new(Box::new(StdoutEnvironment));
    let script = r#"
the behavior called infinite
    with intent: loop forever
    receiving:
        an integer called n
    returning: an integer
    as:
        n infinite -- Recursive call without diminishing clause in header
"#;
    let result = session.run_script(script);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Termination Error"));
}

#[test]
fn test_termination_checker_invalid_proof() {
    let mut session = Session::new(Box::new(StdoutEnvironment));
    let script = r#"
the behavior called infinite
    with intent: loop forever
    receiving:
        an integer called n
    returning: an integer
    with diminishing: n
    as:
        n infinite -- Calling with SAME value, not smaller
"#;
    let result = session.run_script(script);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Termination Error"));
}

#[test]
fn test_termination_checker_valid_proof() {
    let mut session = Session::new(Box::new(StdoutEnvironment));
    let script = r#"
the behavior called factorial
    with intent: calculate factorial
    receiving:
        an integer called n
    returning: an integer
    with diminishing: n
    as:
        if n is-zero
            then 1
            else
                let next is an integer n decreased-by 1
                next factorial
"#;
    let result = session.run_script(script);
    assert!(result.is_ok(), "Expected Ok, got {:?}", result);
}
