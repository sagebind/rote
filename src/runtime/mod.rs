use glob::glob;
use lua;
use lua::ffi;
use lua::wrapper::state;
use lua::ThreadStatus;
use error::Error;
use error::RoteError;
use std::collections::HashMap;
use std::mem;

mod functions;


// Load predefined Lua modules.
const DEFAULT_MODULES: &'static [ &'static str ] = &[
    include_str!("../../modules/core.lua"),
    include_str!("../../modules/table.lua"),
    include_str!("../../modules/cargo.lua"),
    include_str!("../../modules/cpp.lua"),
    include_str!("../../modules/java.lua")
];

/// A Lua script runtime for parsing and executing build script functions.
pub struct Runtime<'r> {
    /// A map of all defined tasks.
    pub tasks: HashMap<&'r str, Task<'r>>,

    /// The name of the default task to run.
    pub default_task: Option<&'r str>,

    /// A raw pointer to the heap location of this runtime object.
    ptr: RuntimePtr<'r>,

    /// A Lua interpreter state.
    state: lua::State
}

/// A raw pointer to a runtime object.
pub type RuntimePtr<'r> = *mut Runtime<'r>;

/// A function that can be bound to be callable inside the Lua runtime.
pub type RuntimeFn<'r> = fn(RuntimePtr<'r>) -> i32;

/// A single build task.
pub struct Task<'t> {
    /// The name of the task.
    pub name: &'t str,

    /// A list of task names that must be ran before this task.
    pub deps: Vec<&'t str>,

    /// A reference to this task's callback function.
    func: state::Reference
}

impl<'r> Runtime<'r> {
    /// Creates a new runtime instance.
    ///
    /// The runtime instance is allocated onto the heap. This allows the runtime object to be passed
    /// around as raw pointers in closure upvalues. The caller will own the box that owns the
    /// runtime instance.
    pub fn new() -> Result<Box<Runtime<'r>>, Error> {
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

        // Register core functions.
        try!(runtime.eval("require_native = require"));
        runtime.register_fn("require", functions::require);
        runtime.register_fn("task", functions::task);
        runtime.register_fn("default", functions::default);
        runtime.register_fn("glob", functions::glob);

        // Load the core Lua module.
        try!(runtime.eval(DEFAULT_MODULES[0]));

        Ok(runtime)
    }

    // Borrows a runtime from a pointer.
    pub fn borrow<'p>(ptr: RuntimePtr<'p>) -> &mut Runtime<'p> {
        assert!(!ptr.is_null());
        unsafe { &mut *ptr }
    }

    // Loads a project script.
    pub fn load(&mut self, filename: &str) -> Result<(), Error> {
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
    pub fn register_fn(&mut self, name: &str, f: RuntimeFn<'r>) {
        unsafe {
            self.state.push_light_userdata(self.ptr);
            self.state.push_light_userdata(f as *mut u8);
        }

        self.state.push_closure(Some(fn_wrapper), 2);
        self.state.set_global(name);

        /// Wrapper function for invoking Rust functions from inside Lua.
        unsafe extern "C" fn fn_wrapper<'r>(l: *mut ffi::lua_State) -> state::Index {
            // Get the runtime from the raw pointer.
            let mut state = lua::State::from_ptr(l);
            let runtime = state.to_userdata(ffi::lua_upvalueindex(1)) as RuntimePtr;

            // Get the raw pointer and turn it back into a Rust function pointer.
            let f_raw_ptr = state.to_userdata(ffi::lua_upvalueindex(2)) as *mut u8;
            let f: RuntimeFn<'r> = mem::transmute(f_raw_ptr);

            // Invoke the function.
            f(&mut *runtime)
        }
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
