use std::error::Error as StdError;
use std::fmt;
use std::io::Write;
use std::process;



/// Defines a method that prints out the error description and exits with an error code. Safer than
/// using `unwrap()`, which panics the thread.
pub trait Die: fmt::Display {
    /// Prints the error and terminates the program, exiting with the given error code.
    fn die(&self) {
        error!("{}", self);
        process::exit(1);
    }
}

/// Implement Die for all errors.
impl<T: ?Sized> Die for Box<T> where T: StdError {
}


#[derive(Debug)]
pub enum Error {
    Chained(Box<StdError>),
    FileNotReadable(String),
    OptionMissing(String),
    RuntimeError(String),
    TaskNotFound(String),
}

/// Implements display formatting for the error type.
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.description())
    }
}

/// And finally, Rote errors are standard errors as well.
impl StdError for Error {
    fn description(&self) -> &str {
        match *self {
            Error::Chained(ref err) => err.description(),
            Error::FileNotReadable(_) => "file could not be read",
            Error::OptionMissing(ref m) => m,
            Error::RuntimeError(ref message) => &message,
            Error::TaskNotFound(_) => "task not found",
        }
    }

    /// Gets a previous error object, if any.
    fn cause(&self) -> Option<&StdError> {
        match *self {
            Error::Chained(ref cause) => Some(&**cause),
            _ => None,
        }
    }
}

/// Implement casting from other error types.
impl From<Box<StdError>> for Error {
    fn from(err: Box<StdError>) -> Error {
        Error::Chained(err)
    }
}

impl From<Box<StdError>> for Box<Error> {
    fn from(err: Box<StdError>) -> Box<Error> {
        Box::new(Error::Chained(err))
    }
}
