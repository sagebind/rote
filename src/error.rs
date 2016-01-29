use std::error;
use std::fmt;
use std::io::Write;
use std::process;


#[macro_export]
macro_rules! throw {
    ($e:expr) => (return Err($e))
}

/// Syntax sugar for pattern matching against multiple results.
#[macro_export]
macro_rules! try {
    {
        $body:expr;
        $(catch $pattern:pat => $catch:block)*
    } => {
        match (|| {
            $body;
            Ok(())
        })() {
            Ok(val) => val,
            Err(err) => match err {
                $($pattern => $catch),*
                _ => return Err(err)
            }
        }
    };

    ($result:expr) => {
        match $result {
            Ok(val) => val,
            Err(err) => return Err(err),
        }
    };
}

#[derive(Debug)]
pub enum Error {
    Chained(&'static error::Error),
    FileNotReadable(String),
    OptionMissing(String),
    RuntimeError(String),
    TaskNotFound(String),
}

impl Error {
    pub fn description(&self) -> &str {
        match *self {
            Error::Chained(ref err) => err.description(),
            Error::FileNotReadable(_) => "file could not be read",
            Error::OptionMissing(ref m) => m,
            Error::RuntimeError(ref message) => &message,
            Error::TaskNotFound(_) => "task not found",
        }
    }

    /// Gets the error code.
    pub fn code(&self) -> i32 {
        match *self {
            Error::Chained(_) => 1,
            Error::OptionMissing(_) => 4,
            Error::FileNotReadable(_) => 66,
            Error::RuntimeError(_) => 3,
            Error::TaskNotFound(_) => 2,
        }
    }

    /// Prints the error and terminates the program, exiting with the given error code.
    pub fn die(&self) {
        error!("{}", self);
        process::exit(self.code());
    }
}

/// Implements display formatting for the error type.
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::FileNotReadable(ref file) => write!(f, "the file \"{}\" could not be read", &file),
            Error::TaskNotFound(ref name) => write!(f, "no task or rule found for \"{}\"", &name),
            _ => write!(f, "{}", self.description())
        }
    }
}

/// And finally, Rote errors are standard errors as well.
impl error::Error for Error {
    fn description(&self) -> &str {
        (*self).description()
    }

    /// Gets a previous error object, if any.
    fn cause(&self) -> Option<&error::Error> {
        match *self {
            Error::Chained(cause) => Some(cause),
            _ => None,
        }
    }
}

/// Implement casting from other error types.
impl From<&'static error::Error> for Error {
    fn from(err: &'static error::Error) -> Error {
        Error::Chained(err)
    }
}
