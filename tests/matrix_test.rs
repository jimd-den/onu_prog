use onu::Session;
use onu::env::StdoutEnvironment;
use onu::interpreter::Value;

#[test]
fn test_matrix_parsing_and_evaluation() {
    let mut session = Session::new(Box::new(StdoutEnvironment));
    let script = r#"
the behavior called create-matrix
    with intent: test matrix literal
    receiving: nothing
    returning: a matrix
    as:
        [ 1.0 2.0 : 3.0 4.0 ]
"#;
    session.run_script(script).unwrap();
    // In a real integration, we'd inspect the return value, 
    // but run_script returns (), so we rely on it not panicking/erroring.
}

#[test]
fn test_matrix_determinant() {
    let mut session = Session::new(Box::new(StdoutEnvironment));
    let script = r#"
the behavior called test-det
    with intent: test determinant
    receiving: nothing
    returning: a float
    as:
        let m is a matrix [ 1.0 0.0 : 0.0 1.0 ]
        m determinant
"#;
    session.run_script(script).unwrap();
}

#[test]
fn test_matrix_parsing_error_inconsistent_cols() {
    let mut session = Session::new(Box::new(StdoutEnvironment));
    let script = r#"
the behavior called bad-matrix
    with intent: fail
    receiving: nothing
    returning: a matrix
    as:
        [ 1.0 2.0 : 3.0 ] -- 2 cols then 1 col
"#;
    let result = session.run_script(script);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Matrix Error"));
}
