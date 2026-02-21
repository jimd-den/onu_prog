use std::process::Command;
use std::fs;

#[test]
fn test_cli_lex_and_register_success() {
    let script = r#"
the behavior called foo
    with intent: test
    receiving:
    returning: a number
    as:
        let x is the number 10
        x
"#;
    let file_path = "test_script_success_int.onu";
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
    receiving:
    returning: an integer
    as:
        let x is an integer 10
        x

the behavior called bar
    with intent: test
    receiving:
    returning: an integer
    as:
        let x is an integer 10
        x
"#;
    let file_path = "test_script_duplicate_int.onu";
    fs::write(file_path, script).unwrap();

    let output = Command::new("cargo")
        .args(["run", "--", file_path])
        .output()
        .expect("Failed to execute cargo run");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("PEER REVIEW MEMO"));
    assert!(stdout.contains("Duplicate semantic implementation detected"));
    
    fs::remove_file(file_path).unwrap();
}

#[test]
fn test_cli_hello_world() {
    let script = r#"
the module called HelloWorld
    with concern: greeting

the effect behavior called run
    with intent: say-hello
    receiving:
    returning: nothing
    as:
        emit "Hello, World!"
"#;
    let file_path = "hello_world_int.onu";
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
    with concern: arithmetic and accumulation

the behavior called factorial
    with intent: calculate the product of a sequence
    receiving:
        an integer called the-count
    returning: an integer
    with diminishing: the-count
    as:
        if the-count is-zero
            then 1
            else
                let the-previous-count is an integer the-count decreased-by 1
                let the-accumulated-value is an integer the-previous-count factorial
                the-count multiplied-by the-accumulated-value
        
the effect behavior called main
    with intent: demonstrate factorial
    receiving:
    returning: nothing
    as:
        let result is an integer 5 factorial
        emit result
"#;
    let file_path = "factorial_int.onu";
    fs::write(file_path, script).unwrap();

    let output = Command::new("cargo")
        .args(["run", "--", file_path])
        .output()
        .expect("Failed to execute cargo run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    fs::remove_file(file_path).unwrap();
}

#[test]
fn test_cli_fibonacci() {
    let script = r#"
the module called PopulationDynamics
    with concern: modeling sequential growth

the behavior called calculate-population
    with intent: determine population size for a generation
    receiving:
        an integer called the-generation
    returning: an integer
    with diminishing: the-generation
    as:
        if the-generation is-zero
            then 0
            else if (the-generation decreased-by 1) is-zero
                then 1
                else
                    let the-previous-generation  is an integer the-generation decreased-by 1
                    let the-ancestral-generation is an integer the-generation decreased-by 2
                    
                    let parent-population        is an integer the-previous-generation calculate-population
                    let grandparent-population   is an integer the-ancestral-generation calculate-population
                    
                    parent-population added-to grandparent-population

the effect behavior called main
    with intent: run simulation
    receiving:
    returning: nothing
    as:
        let final-population is an integer 10 calculate-population
        emit final-population
"#;
    let file_path = "fibonacci_int.onu";
    fs::write(file_path, script).unwrap();

    let output = Command::new("cargo")
        .args(["run", "--", file_path])
        .output()
        .expect("Failed to execute cargo run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    fs::remove_file(file_path).unwrap();
}

#[test]
fn test_cli_hanoi() {
    let script = r#"
the module called Logistics
    with concern: moving inventory recursively

the effect behavior called move-stack
    with intent: relocate a stack of items between piles
    receiving:
        an integer called the-item-count
        a strings called source-pile
        a strings called destination-pile
        a strings called transit-pile
    returning: nothing
    with diminishing: the-item-count
    as:
        if the-item-count is-zero
            then nothing
            else
                let the-remaining-stack is an integer the-item-count decreased-by 1
                
                -- Move n-1 disks from Source to Transit
                let step-one is nothing the-remaining-stack move-stack source-pile transit-pile destination-pile
                
                -- Move the biggest disk from Source to Dest
                let msg1 is the strings "Move item " joined-with (the-item-count as-text)
                let msg2 is the strings msg1 joined-with " from "
                let msg3 is the strings msg2 joined-with source-pile
                let msg4 is the strings msg3 joined-with " to "
                let msg5 is the strings msg4 joined-with destination-pile
                let step-two is nothing emit msg5
                
                -- Move n-1 disks from Transit to Dest
                the-remaining-stack move-stack transit-pile destination-pile source-pile

the effect behavior called main
    with intent: execute logistics plan
    receiving:
    returning: nothing
    as:
        3 move-stack "Depot-A" "Depot-C" "Depot-B"
"#;
    let file_path = "hanoi_int.onu";
    fs::write(file_path, script).unwrap();

    let output = Command::new("cargo")
        .args(["run", "--", file_path])
        .output()
        .expect("Failed to execute cargo run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Move item 1 from Depot-A to Depot-C"));
    assert!(stdout.contains("Move item 3 from Depot-A to Depot-C"));
    
    fs::remove_file(file_path).unwrap();
}
