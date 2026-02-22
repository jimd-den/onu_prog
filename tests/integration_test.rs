use std::process::Command;
use std::fs;

#[test]
fn test_cli_lex_and_register_success() {
    let script = r#"
the behavior called foo
    with intent: test
    takes: nothing
    delivers: a number
    as:
        derivation: x derives-from the number 10
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
    takes:
        an integer called arg
    delivers: an integer
    as:
        derivation: x derives-from an integer 10
        x

the behavior called bar
    with intent: test
    takes:
        an integer called arg
    delivers: an integer
    as:
        derivation: x derives-from an integer 10
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
    takes: nothing
    delivers: nothing
    as:
        broadcasts "Hello, World!"
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
    takes:
        an integer called the-count
    delivers: an integer
    with diminishing: the-count
    as:
        if the-count matches 0
            then 1
            else
                derivation: the-previous-count derives-from an integer the-count decreased-by 1
                derivation: the-accumulated-value derives-from an integer the-previous-count utilizes factorial
                the-count scales-by the-accumulated-value
        
the effect behavior called main
    with intent: demonstrate factorial
    takes: nothing
    delivers: nothing
    as:
        derivation: result derives-from an integer 5 utilizes factorial
        broadcasts result
"#;
    let file_path = "factorial_int.onu";
    fs::write(file_path, script).unwrap();

    let output = Command::new("cargo")
        .args(["run", "--", file_path])
        .output()
        .expect("Failed to execute cargo run");

    assert!(output.status.success());
    let _stdout = String::from_utf8_lossy(&output.stdout);
    fs::remove_file(file_path).unwrap();
}

#[test]
fn test_cli_fibonacci() {
    let script = r#"
the module called PopulationDynamics
    with concern: modeling sequential growth

the behavior called calculate-population
    with intent: determine population size for a generation
    takes:
        an integer called the-generation
    delivers: an integer
    with diminishing: the-generation
    as:
        if the-generation matches 0
            then 0
            else if (the-generation decreased-by 1) matches 0
                then 1
                else
                    derivation: the-previous-generation derives-from an integer the-generation decreased-by 1
                    derivation: the-ancestral-generation derives-from an integer the-generation decreased-by 2
                    
                    derivation: parent-population derives-from an integer the-previous-generation utilizes calculate-population
                    derivation: grandparent-population derives-from an integer the-ancestral-generation utilizes calculate-population
                    
                    parent-population added-to grandparent-population

the effect behavior called main
    with intent: run simulation
    takes: nothing
    delivers: nothing
    as:
        derivation: final-population derives-from an integer 10 utilizes calculate-population
        broadcasts final-population
"#;
    let file_path = "fibonacci_int.onu";
    fs::write(file_path, script).unwrap();

    let output = Command::new("cargo")
        .args(["run", "--", file_path])
        .output()
        .expect("Failed to execute cargo run");

    assert!(output.status.success());
    let _stdout = String::from_utf8_lossy(&output.stdout);
    fs::remove_file(file_path).unwrap();
}

#[test]
fn test_cli_hanoi() {
    let script = r#"
the module called Logistics
    with concern: moving inventory recursively

the effect behavior called move-stack
    with intent: relocate a stack of items between piles
    takes:
        an integer called the-item-count
        a string called source-pile
        a string called destination-pile
        a string called transit-pile
    delivers: nothing
    with diminishing: the-item-count
    as:
        if the-item-count matches 0
            then nothing
            else
                derivation: the-remaining-stack derives-from an integer the-item-count decreased-by 1
                
                -- Move n-1 disks from Source to Transit
                derivation: step-one derives-from nothing the-remaining-stack utilizes move-stack source-pile transit-pile destination-pile
                
                -- Move the biggest disk from Source to Dest
                derivation: msg1 derives-from a string "Move item " joined-with (the-item-count utilizes as-text)
                derivation: msg2 derives-from a string msg1 joined-with " from "
                derivation: msg3 derives-from a string msg2 joined-with source-pile
                derivation: msg4 derives-from a string msg3 joined-with " to "
                derivation: msg5 derives-from a string msg4 joined-with destination-pile
                derivation: step-two derives-from nothing broadcasts msg5
                
                -- Move n-1 disks from Transit to Dest
                the-remaining-stack utilizes move-stack transit-pile destination-pile source-pile

the effect behavior called main
    with intent: execute logistics plan
    takes: nothing
    delivers: nothing
    as:
        3 utilizes move-stack "Depot-A" "Depot-C" "Depot-B"
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
