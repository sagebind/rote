use lua;
use lua::ThreadStatus;
use error::Error;
use error::RoteError;

// Load predefined Lua modules.
static MODULE_CORE: &'static str = include_str!("modules/core.lua");
static MODULE_CARGO: &'static str = include_str!("modules/cargo.lua");

pub struct Project {
    state: lua::State,
}

impl Project {
    // Creates a new project script handle.
    pub fn new() -> Project {
        let mut project = Project {
            state: lua::State::new(),
        };

        // Prepare the environment.
        project.state.open_libs();
        project
    }

    // Loads a project script.
    pub fn load(&mut self, filename: &str) -> Result<(), Error> {
        // Load the builtin modules.
        try!(self.eval(MODULE_CORE));
        try!(self.eval(MODULE_CARGO));

        // Load the given file.
        match self.state.load_file(filename) {
            ThreadStatus::Ok => { Ok(()) }
            ThreadStatus::FileError => {
                Err(Error::new(RoteError::FileNotReadable, &format!("the file \"{}\" could not be read", filename)))
            }
            ThreadStatus::SyntaxError => {
                let error_message = self.state.to_str(-1).unwrap();
                Err(Error::new(RoteError::Syntax, &error_message))
            }
            _ => {
                Err(Error::new(RoteError::Parse, "parse error"))
            }
        }
    }

    /// Runs the task with the given name.
    pub fn run_task(&mut self, name: &str, args: Vec<String>) -> Result<(), Error> {
        // Parse the loaded file.
        self.state.pcall(0, 0, 0);

        // Get the task function.
        self.state.get_global(name);
        if !self.state.is_fn(-1) {
            return Err(Error::new(RoteError::TaskNotFound, &format!("no such task \"{}\"", name)));
        }

        // Push the given task arguments onto the stack.
        for string in &args {
            self.state.push_string(&string);
        }

        // Invoke the task function.
        if self.state.pcall(args.len() as i32, 0, 0).is_err() {
            if self.state.is_string(-1) {
                let error_message = self.state.to_str(-1).unwrap();
                return Err(Error::new(RoteError::Script, &error_message));
            } else {
                return Err(Error::new(RoteError::Script, "unknown script error"));
            }
        }

        Ok(())
    }

    pub fn eval(&mut self, code: &str) -> Result<(), Error> {
        if self.state.do_string(code).is_err() {
            return Err(self.get_last_error().unwrap());
        }

        Ok(())
    }

    // Closes the project.
    pub fn close(self) {
        self.state.close();
    }

    fn get_last_error(&mut self) -> Option<Error> {
        if self.state.is_string(-1) {
            Some(Error::new(RoteError::Script, self.state.to_str(-1).unwrap()))
        } else {
            None
        }
    }
}
