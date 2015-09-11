use lua;
use error::Error;
use error::RoteError::{FileNotReadableError, ParseError};

pub struct Project {
    state: lua::State,
}

impl Project {
    pub fn new() -> Project {
        let mut project = Project {
            state: lua::State::new(),
        };

        project.state.open_libs();
        project
    }

    pub fn load(&mut self, filename: &str) -> Result<(), Error> {
        match self.state.do_file(filename) {
            lua::ThreadStatus::Ok => { Ok(()) }
            lua::ThreadStatus::FileError => {
                Err(Error::new(FileNotReadableError, &format!("the file \"{}\" could not be read", filename)))
            }
            _ => { Err(Error::new(ParseError, "parse error")) }
        }
    }

    pub fn close(self) {
        self.state.close();
    }
}
