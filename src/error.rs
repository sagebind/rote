use std::error;
use std::fmt;
use std::io;
use std::io::Write;
use std::process;


#[derive(Debug)]
pub struct Error {
    code: RoteError,
    description: String,
}

#[derive(Debug)]
pub enum RoteError {
    FileNotReadable = 1,
    Runtime,
    TaskNotFound
}

impl Error {
    /// Creates a new error with a given code and description.
    pub fn new(code: RoteError, description: &str) -> Error {
        Error {
            code: code,
            description: String::from(description),
        }
    }

    /// Prints the error and terminates the program, exiting with the given error code.
    pub fn die(self) {
        match writeln!(&mut io::stderr(), "rote: error: {}.", self) {
            Ok(_) => {},
            Err(e) => panic!("Unable to write to stderr: {}", e),
        }

        process::exit(self.code as i32);
    }
}

/// Implement the standard error methods.
impl error::Error for Error {
    /// Gets the error description.
    fn description(&self) -> &str {
        &self.description
    }

    /// Gets a previous error object, if any.
    fn cause(&self) -> Option<&error::Error> {
        None
    }
}

/// Implement display formatting for the error type.
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.description)
    }
}
