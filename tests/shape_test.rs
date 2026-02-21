use onu::Session;
use onu::env::StdoutEnvironment;

#[test]
fn test_shape_verification_failure() {
    let mut session = Session::new(Box::new(StdoutEnvironment));
    let script = r#"
the shape Measurable promises:
    a behavior called measure
        receiving:
            an integer called input
        returning: an integer

the behavior called process
    with intent: process measurable
    receiving:
        an integer called val via the role Measurable
    returning: an integer
    as:
        val measure -- 'measure' is defined in the shape, but NO implementation exists for 'integer'
"#;
    let result = session.run_script(script);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.contains("Shape Error") || err.contains("Structural Error") || err.contains("Semantic Error"));
}

#[test]
fn test_shape_verification_success() {
    let mut session = Session::new(Box::new(StdoutEnvironment));
    let script = r#"
the shape Measurable promises:
    a behavior called measure
        receiving:
            an integer called input
        returning: an integer

the behavior called measure
    with intent: implement measure for integer
    receiving:
        an integer called n
    returning: an integer
    as:
        n

the behavior called process
    with intent: process measurable
    receiving:
        an integer called val via the role Measurable
    returning: an integer
    as:
        val measure
"#;
    let result = session.run_script(script);
    assert!(result.is_ok(), "Expected Ok, got {:?}", result);
}
