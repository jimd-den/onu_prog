use onu::Session;
use onu::env::StdoutEnvironment;

#[test]
fn test_shape_verification_failure() {
    let mut session = Session::new(Box::new(StdoutEnvironment));
    let script = r#"
the shape Measurable promises:
    a behavior called measure
        takes:
            an integer called input
        delivers: an integer

the behavior called process
    with intent: process measurable
    takes:
        an integer called val via the role Measurable
    delivers: an integer
    as:
        val utilizes measure -- 'measure' is defined in the shape, but NO implementation exists for 'integer'
"#;
    let result = session.run_script(script);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.contains("SHAPE VIOLATION"));
}

#[test]
fn test_shape_verification_success() {
    let mut session = Session::new(Box::new(StdoutEnvironment));
    let script = r#"
the shape Measurable promises:
    a behavior called measure
        takes:
            an integer called input
        delivers: an integer

the behavior called measure
    with intent: implement measure for integer
    takes:
        an integer called n
    delivers: an integer
    as:
        n

the behavior called process
    with intent: process measurable
    takes:
        an integer called val via the role Measurable
    delivers: an integer
    as:
        val utilizes measure
"#;
    let result = session.run_script(script);
    assert!(result.is_ok(), "Expected Ok, got {:?}", result);
}
