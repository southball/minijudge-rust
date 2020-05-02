#[derive(Debug, Clone)]
pub struct ExecuteError;

impl std::fmt::Display for ExecuteError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Execution error.")
    }
}

impl std::error::Error for ExecuteError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}

#[derive(Debug, Clone)]
pub struct CompileError;

impl std::fmt::Display for CompileError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Compile error.")
    }
}

impl std::error::Error for CompileError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}

/// This is an error in the user's option passed to the program. The process cannot be continued in this case.
#[derive(Debug, Clone)]
pub struct OptionError {
    pub message: String,
}

impl std::fmt::Display for OptionError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Option error: {}", self.message)
    }
}

impl std::error::Error for OptionError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}
