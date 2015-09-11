use lua;
use lua::ThreadStatus;
use error::Error;
use error::RoteError::{FileNotReadableError, ParseError, TaskError};

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
            ThreadStatus::Ok => { Ok(()) }
            ThreadStatus::FileError => {
                Err(Error::new(FileNotReadableError, &format!("the file \"{}\" could not be read", filename)))
            }
            _ => { Err(Error::new(ParseError, "parse error")) }
        }
    }

    /// Runs the task with the given name.
    pub fn run_task(&mut self, name: &str, args: Vec<String>) -> Result<(), Error> {
        // Get the task function.
        self.state.get_global(name);
        if !self.state.is_fn(-1) {
            return Err(Error::new(TaskError, &format!("no such task \"{}\"", name)));
        }

        // Push the given task arguments onto the stack.
        for string in &args {
            self.state.push_string(&string);
        }

        // Invoke the task function.
        if self.state.pcall(args.len() as i32, 0, 0) != ThreadStatus::Ok {
            return Err(Error::new(ParseError, "function error"));
        }

        Ok(())
    }

    pub fn close(self) {
        self.state.close();
    }
}
