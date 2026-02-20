use std::process::Command;
use std::fs;

#[test]
fn test_cli_lex_and_register_success() {
    let script = r#"
the behavior called foo
    with intent: test
    returning: a number
    as:
        let x is 10
        x
"#;
    let file_path = "test_script_success.onu";
    fs::write(file_path, script).unwrap();

    let output = Command::new("cargo")
        .args(["run", "--", file_path])
        .output()
        .expect("Failed to execute cargo run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Behavior 'foo' parsed and registered"));
    
    fs::remove_file(file_path).unwrap();
}

#[test]
fn test_cli_lex_and_register_duplicate() {
    let script = r#"
the behavior called foo
    with intent: test
    returning: a number
    as:
        let x is 10
        x

the behavior called bar
    with intent: test
    returning: a number
    as:
        let x is 10
        x
"#;
    let file_path = "test_script_duplicate.onu";
    fs::write(file_path, script).unwrap();

    let output = Command::new("cargo")
        .args(["run", "--", file_path])
        .output()
        .expect("Failed to execute cargo run");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Conflict: Behavior 'bar' is semantically identical to 'foo'"));
    
    fs::remove_file(file_path).unwrap();
}

#[test]
fn test_cli_hello_world() {
    let script = r#"
the module called HelloWorld
    with concern: greeting

the behavior called run
    with intent: say-hello
    returning: nothing
    as:
        emit "Hello, World!"
"#;
    let file_path = "hello_world.onu";
    fs::write(file_path, script).unwrap();

    let output = Command::new("cargo")
        .args(["run", "--", file_path])
        .output()
        .expect("Failed to execute cargo run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Hello, World!"));
    
    fs::remove_file(file_path).unwrap();
}

#[test]
fn test_cli_factorial() {
    let script = r#"
the module called Math
    with concern: arithmetic

the behavior called factorial
    with intent: calculate-factorial
    receiving:
        a number called n
    returning: a number
    with diminishing: n
    as:
        if is-zero receiving n
            then 1
            else multiplied-by receiving n (factorial receiving (decreased-by receiving n 1))
        
the behavior called main
    as:
        let res is factorial receiving 5
        emit res
"#;
    let file_path = "factorial.onu";
    fs::write(file_path, script).unwrap();

    let output = Command::new("cargo")
        .args(["run", "--", file_path])
        .output()
        .expect("Failed to execute cargo run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("120")); // 5! = 120
    
    fs::remove_file(file_path).unwrap();
}
