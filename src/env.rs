use std::io;

/// The Environment trait defines the abstract interface for Ọ̀nụ's interaction with the external world.
///
/// In Clean Architecture, this acts as the interface for the Output Port (and Input Port).
/// By decoupling the core interpreter from concrete I/O (like stdout or stdin), we ensure
/// the language core remains portable, testable, and side-effect free from the perspective
/// of the business logic (the evaluation engine).
pub trait Environment {
    /// Emits a message to the external world (e.g., printing to a console).
    fn emit(&mut self, text: &str);

    /// Reads a message from the external world (e.g., reading from stdin).
    fn read(&mut self) -> String;
}

pub struct StdoutEnvironment;

impl Environment for StdoutEnvironment {
    fn emit(&mut self, text: &str) {
        println!("{}", text);
    }

    fn read(&mut self) -> String {
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap_or(0);
        input.trim().to_string()
    }
}

/// A simple Mock environment for testing purposes.
#[cfg(test)]
pub struct MockEnvironment {
    pub emitted: Vec<String>,
    pub input_queue: Vec<String>,
}

#[cfg(test)]
impl MockEnvironment {
    pub fn new() -> Self {
        Self {
            emitted: Vec::new(),
            input_queue: Vec::new(),
        }
    }
}

#[cfg(test)]
impl Environment for MockEnvironment {
    fn emit(&mut self, text: &str) {
        self.emitted.push(text.to_string());
    }

    fn read(&mut self) -> String {
        self.input_queue.pop().unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock_environment_emit() {
        let mut env = MockEnvironment::new();
        env.emit("hello world");
        assert_eq!(env.emitted.len(), 1);
        assert_eq!(env.emitted[0], "hello world");
    }

    #[test]
    fn test_mock_environment_read() {
        let mut env = MockEnvironment::new();
        env.input_queue.push("test input".to_string());
        let input = env.read();
        assert_eq!(input, "test input");
    }
}
