use lua;
use lua::ffi;
use lua::wrapper::state;
use lua::ThreadStatus;
use error::Error;
use error::RoteError;
use std::collections::HashMap;

// Load predefined Lua modules.
static MODULE_CORE: &'static str = include_str!("../modules/core.lua");
static MODULE_CARGO: &'static str = include_str!("../modules/cargo.lua");

/// A Lua script runtime for parsing and executing build script functions.
pub struct Runtime<'r> {
    /// A raw pointer to the heap location of this runtime object.
    ptr: RuntimePtr<'r>,

    /// A Lua interpreter state.
    state: lua::State,

    /// A map of all defined tasks.
    tasks: HashMap<&'r str, Task<'r>>,

    /// The name of the default task to run.
    default_task: Option<&'r str>
}

/// A raw pointer to a runtime object.
pub type RuntimePtr<'r> = *mut Runtime<'r>;

/// A single build task.
pub struct Task<'t> {
    /// The name of the task.
    name: &'t str,

    /// A list of task names that must be ran before this task.
    deps: Vec<&'t str>,

    /// A reference to this task's callback function.
    func: state::Reference
}

impl<'r> Runtime<'r> {
    /// Creates a new runtime instance.
    ///
    /// The runtime instance is allocated onto the heap. This allows the runtime object to be passed
    /// around as raw pointers in cloure upvalues.
    pub fn new() -> Box<Runtime<'r>> {
        let mut runtime = Box::new(Runtime {
            ptr: 0 as RuntimePtr,
            state: lua::State::new(),
            tasks: HashMap::new(),
            default_task: None
        });

        // Store a raw self pointer to this runtime object.
        runtime.ptr = &mut *runtime as RuntimePtr;

        // Prepare the environment.
        runtime.state.open_libs();
        runtime.register_fn("task", task_callback);
        runtime.register_fn("default", default_callback);

        runtime
    }

    // Gets a runtime pointer from a Lua state pointer inside a closure.
    pub unsafe fn from_upvalue<'p>(l: *mut ffi::lua_State) -> RuntimePtr<'p> {
        let mut state = lua::State::from_ptr(l);
        state.to_userdata(ffi::lua_upvalueindex(1)) as RuntimePtr
    }

    // Loads a project script.
    pub fn load(&mut self, filename: &str) -> Result<(), Error> {
        // Load the builtin modules.
        try!(self.eval(MODULE_CORE));
        try!(self.eval(MODULE_CARGO));

        // Load the given file.
        match self.state.do_file(filename) {
            ThreadStatus::Ok => { }
            ThreadStatus::FileError => {
                return Err(Error::new(RoteError::FileNotReadable, &format!("the file \"{}\" could not be read", filename)));
            }
            _ => {
                return Err(self.get_last_error().unwrap());
            }
        };

        Ok(())
    }

    /// Creates a new task.
    pub fn create_task(&mut self, name: &'r str, deps: Vec<&'r str>, func: state::Reference) {
        // Create a task object.
        let task = Task {
            name: name,
            deps: deps,
            func: func
        };

        // Add it to the master list of tasks.
        self.tasks.insert(name, task);
    }

    /// Runs the task with the given name.
    pub fn run_task(&mut self, name: &str, args: Vec<String>) -> Result<(), Error> {
        let task_name = if name == "default" {
            if self.default_task.is_none() {
                return Err(Error::new(RoteError::TaskNotFound, "no default task defined"));
            } else {
                self.default_task.unwrap()
            }
        } else {
            name
        };

        if !self.tasks.contains_key(task_name) {
            return Err(Error::new(RoteError::TaskNotFound, &format!("no such task \"{}\"", name)));
        }

        {
            let task = self.tasks.get(task_name).unwrap();

            // Push the task function onto the stack.
            self.state.raw_geti(ffi::LUA_REGISTRYINDEX, task.func.value() as i64);
        }

        // Push the given task arguments onto the stack.
        for string in &args {
            self.state.push_string(&string);
        }

        // Invoke the task function.
        if self.state.pcall(args.len() as i32, 0, 0).is_err() {
            return Err(self.get_last_error().unwrap());
        }

        Ok(())
    }

    /// Evaluates a Lua string inside the runtime.
    pub fn eval(&mut self, code: &str) -> Result<(), Error> {
        if self.state.do_string(code).is_err() {
            return Err(self.get_last_error().unwrap());
        }

        Ok(())
    }

    // Registers a global function in the runtime that can be called by Lua scripts.
    pub fn register_fn(&mut self, name: &str, func: unsafe extern "C" fn(l: *mut ffi::lua_State) -> state::Index) {
        unsafe {
            self.state.push_light_userdata(self.ptr);
        }
        self.state.push_closure(Some(func), 1);
        self.state.set_global(name);
    }

    // Closes the runtime.
    pub fn close(self) {
        self.state.close();
    }

    fn get_last_error(&mut self) -> Option<Error> {
        if self.state.is_string(-1) {
            Some(Error::new(RoteError::Runtime, self.state.to_str(-1).unwrap()))
        } else {
            None
        }
    }
}

unsafe extern "C" fn task_callback(l: *mut ffi::lua_State) -> state::Index {
    let runtime = Runtime::from_upvalue(l);

    // Get the task name as the first argument.
    let name = (*runtime).state.check_string(1);

    // Second argument is a table of dependent task names.
    let mut deps: Vec<&str> = Vec::new();
    (*runtime).state.check_type(2, state::Type::Table);

    // Read all of the names in the table and add it to the deps vector.
    (*runtime).state.push_nil();
    while (*runtime).state.next(2) {
        let dep = (*runtime).state.check_string(-1);
        (*runtime).state.pop(1);

        deps.push(dep);
    }

    // Third argument is the task function.
    (*runtime).state.check_type(3, state::Type::Function);

    // Get a portable reference to the task function.
    let func = (*runtime).state.reference(ffi::LUA_REGISTRYINDEX);

    // Create the task.
    (*runtime).create_task(name, deps, func);

    0
}

unsafe extern "C" fn default_callback(l: *mut ffi::lua_State) -> state::Index {
    let runtime = Runtime::from_upvalue(l);

    // Get the task name as the first argument.
    let name = (*runtime).state.check_string(1);

    // Set the default task to the given name.
    (*runtime).default_task = Some(name);

    0
}
