use onu::Session;
use onu::env::StdoutEnvironment;
use std::env;
use std::fs;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("Usage: onu <file_path>");
        return;
    }

    let file_path = &args[1];
    let input = match fs::read_to_string(file_path) {
        Ok(content) => content,
        Err(e) => {
            println!("Error: Could not read file '{}': {}", file_path, e);
            return;
        }
    };

    let mut session = Session::new(Box::new(StdoutEnvironment));
    if let Err(e) = session.run_script(&input) {
        println!("{}", e);
    }
}
