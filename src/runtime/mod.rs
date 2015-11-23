use error::Error;
use error::RoteError;
use glob::glob;
use lua;
use lua::ffi;
use lua::ThreadStatus;
use lua::wrapper::state;
use modules;
use std::cell::RefCell;
use std::collections::{HashMap, LinkedList};
use std::mem;
use std::rc::{Rc, Weak};

mod functions;


/// A Lua script runtime for parsing and executing build script functions.
pub struct Runtime {
    /// A map of all defined tasks.
    pub tasks: HashMap<String, Rc<RefCell<Task>>>,

    /// The name of the default task to run.
    pub default_task: Option<String>,

    /// Task execution stack.
    stack: LinkedList<Weak<RefCell<Task>>>,

    /// A raw pointer to the heap location of this runtime object.
    ptr: RuntimePtr,

    /// A Lua interpreter state.
    state: lua::State,
}

/// A raw pointer to a runtime object.
pub type RuntimePtr = *mut Runtime;

/// A function that can be bound to be callable inside the Lua runtime.
pub type RuntimeFn = fn(RuntimePtr) -> i32;

/// A single build task.
pub struct Task {
    /// The name of the task.
    pub name: String,

    /// A list of task names that must be ran before this task.
    pub deps: Vec<String>,

    /// A reference to this task's callback function.
    func: state::Reference,
}

impl Runtime {
    /// Creates a new runtime instance.
    ///
    /// The runtime instance is allocated onto the heap. This allows the runtime object to be passed
    /// around as raw pointers in closure upvalues. The caller will own the box that owns the
    /// runtime instance.
    pub fn new() -> Result<Box<Runtime>, Error> {
        let mut runtime = Box::new(Runtime {
            tasks: HashMap::new(),
            default_task: None,
            stack: LinkedList::new(),
            ptr: 0 as RuntimePtr,
            state: lua::State::new(),
        });

        // Store a raw self pointer to this runtime object.
        runtime.ptr = runtime.as_ptr();

        // Prepare the environment.
        runtime.state.open_libs();

        // Register core functions.
        try!(runtime.eval("require_native = require"));
        runtime.register_fn("require", functions::require);
        runtime.register_fn("task", functions::task);
        runtime.register_fn("default", functions::default);
        runtime.register_fn("print", functions::print);
        runtime.register_fn("glob", functions::glob);

        // Load the core Lua module.
        try!(runtime.eval(modules::fetch("core").unwrap()));

        Ok(runtime)
    }

    // Gets the runtime as a raw pointer.
    pub fn as_ptr(&self) -> RuntimePtr {
        self as *const Runtime as RuntimePtr
    }

    // Borrows a runtime from a pointer.
    pub fn borrow<'p>(ptr: RuntimePtr) -> &'p mut Runtime {
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
    pub fn create_task(&mut self, name: String, deps: Vec<String>, func: state::Reference) {
        // Create a task object.
        let task = Task {
            name: name,
            deps: deps,
            func: func,
        };

        // Add it to the master list of tasks.
        self.tasks.insert(task.name.clone(), Rc::new(RefCell::new(task)));
    }

    /// Runs the task with the given name.
    pub fn run_task(&mut self, name: &str, args: Vec<String>) -> Result<(), Error> {
        // Determine the name of the task to run.
        let task_name = if name == "default" {
            if let Some(ref default_name) = self.default_task {
                default_name.clone()
            } else {
                return Err(Error::new(RoteError::TaskNotFound, "no default task defined"));
            }
        } else {
            name.to_string()
        };

        // Get the task object from the given name.
        let task = if let Some(task) = self.tasks.get(&task_name) {
            task.clone()
        } else {
            return Err(Error::new(RoteError::TaskNotFound, &format!("no such task \"{}\"", name)));
        };

        // Set the active task.
        self.stack.push_front(Rc::downgrade(&task));

        // Run all dependencies first.
        for dep_name in &task.borrow().deps {
            try!(self.run_task(&dep_name, Vec::new()));
        }

        // Call the task itself.
        // Push the task function onto the Lua stack.
        self.state.raw_geti(ffi::LUA_REGISTRYINDEX, task.borrow().func.value() as i64);

        // Push the given task arguments onto the stack.
        for string in &args {
            self.state.push_string(&string);
        }

        // Invoke the task function.
        if self.state.pcall(args.len() as i32, 0, 0).is_err() {
            return Err(self.get_last_error().unwrap());
        }
        self.state.pop(1);

        // Pop the task off the call stack.
        self.stack.pop_front();

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
    pub fn register_fn(&mut self, name: &str, f: RuntimeFn) {
        unsafe {
            self.state.push_light_userdata(self.ptr);
            self.state.push_light_userdata(f as *mut usize);
        }

        self.state.push_closure(Some(fn_wrapper), 2);
        self.state.set_global(name);

        /// Wrapper function for invoking Rust functions from inside Lua.
        unsafe extern fn fn_wrapper<'r>(l: *mut ffi::lua_State) -> state::Index {
            // Get the runtime from the raw pointer.
            let mut state = lua::State::from_ptr(l);
            let runtime = state.to_userdata(ffi::lua_upvalueindex(1)) as RuntimePtr;

            // Get the raw pointer and turn it back into a Rust function pointer.
            let f_raw_ptr = state.to_userdata(ffi::lua_upvalueindex(2)) as *mut usize;
            let f: RuntimeFn = mem::transmute(f_raw_ptr);

            // Invoke the function.
            f(runtime)
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
