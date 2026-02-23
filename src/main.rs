use onu::CompilerSession;
use std::env;
use std::fs;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("Usage: onu <file_path> [--run] [--ir]");
        return;
    }

    let file_path = &args[1];
    let do_run = args.iter().any(|arg| arg == "--run");
    let show_ir = args.iter().any(|arg| arg == "--ir");
    let do_native = args.iter().any(|arg| arg == "--native");

    let input = match fs::read_to_string(file_path) {
        Ok(content) => content,
        Err(e) => {
            println!("Error: Could not read file '{}': {}", file_path, e);
            return;
        }
    };

    let mut session = match CompilerSession::new() {
        Ok(s) => s,
        Err(e) => {
            println!("Compiler Initialization Error: {}", e);
            return;
        }
    };

    if show_ir {
        match session.get_llvm_ir(&input) {
            Ok(ir) => {
                println!("--- LLVM IR ---");
                println!("{}", ir);
                println!("---------------");
            }
            Err(e) => {
                println!("{}", e);
                return;
            }
        }
    }

    match session.compile(&input) {
        Ok(binary) => {
            if let Err(e) = fs::write("output.bc", binary) {
                println!("Error writing output.bc: {}", e);
                return;
            }
            
            if do_run {
                // Automate: clang runtime.c -> llvm-link -> lli
                let status = std::process::Command::new("clang-14")
                    .args(&["-emit-llvm", "-c", "runtime.c", "-o", "runtime.bc"])
                    .status();
                
                if status.is_err() || !status.unwrap().success() {
                    println!("Error: Failed to compile runtime.c. Ensure clang-14 is installed.");
                    return;
                }

                let link_status = std::process::Command::new("llvm-link-14")
                    .args(&["output.bc", "runtime.bc", "-o", "final.bc"])
                    .status();

                if link_status.is_err() || !link_status.unwrap().success() {
                    println!("Error: Failed to link bitcode. Ensure llvm-link-14 is installed.");
                    return;
                }

                let run_status = std::process::Command::new("lli-14")
                    .arg("final.bc")
                    .status();

                if run_status.is_err() || !run_status.unwrap().success() {
                    println!("Error: Execution failed via lli-14.");
                }
            } else if do_native {
                // Automate: clang runtime.c output.bc -O3 -o onu_prog
                println!("Compiling to native binary...");
                let status = std::process::Command::new("clang-14")
                    .args(&["runtime.c", "output.bc", "-O3", "-o", "onu_prog"])
                    .status();
                
                if status.is_err() || !status.unwrap().success() {
                    println!("Error: Failed to link native binary.");
                } else {
                    println!("Native binary generated: ./onu_prog");
                }
            } else {
                println!("Successfully compiled {} to output.bc.", file_path);
                println!("To run (JIT): onu {} --run", file_path);
                println!("To compile (Native): onu {} --native", file_path);
            }
        }
        Err(e) => {
            println!("{}", e);
        }
    }
}
