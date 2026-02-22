use onu::Session;
use onu::env::StdoutEnvironment;

#[test]
fn test_matrix_parsing_and_evaluation() {
    let mut session = Session::new(Box::new(StdoutEnvironment));
    let script = r#"
the behavior called create-matrix
    with intent: test matrix literal
    takes: nothing
    delivers: a matrix
    as:
        [ 1.0 2.0 : 3.0 4.0 ]
"#;
    session.run_script(script).unwrap();
}

#[test]
fn test_matrix_determinant() {
    let mut session = Session::new(Box::new(StdoutEnvironment));
    let script = r#"
the behavior called test-det
    with intent: test determinant
    takes: nothing
    delivers: a float
    as:
        derivation: m derives-from a matrix [ 1.0 0.0 : 0.0 1.0 ]
        m utilizes determinant
"#;
    session.run_script(script).unwrap();
}

#[test]
fn test_matrix_parsing_error_inconsistent_cols() {
    let mut session = Session::new(Box::new(StdoutEnvironment));
    let script = r#"
the behavior called bad-matrix
    with intent: fail
    takes: nothing
    delivers: a matrix
    as:
        [ 1.0 2.0 : 3.0 ] -- 2 cols then 1 col
"#;
    let result = session.run_script(script);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Matrix Error"));
}
