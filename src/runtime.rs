use error::Error;
use error::RoteError;
use functions;
use lua;
use lua::ffi;
use std::cell::RefCell;
use std::collections::{HashMap, LinkedList};
use std::mem;
use std::rc::{Rc, Weak};


/// A Lua script runtime for parsing and executing build script functions.
pub struct Runtime {
    /// A map of all defined tasks.
    pub tasks: HashMap<String, Rc<RefCell<Task>>>,

    /// The name of the default task to run.
    pub default_task: Option<String>,

    /// Task execution stack.
    pub stack: LinkedList<Weak<RefCell<Task>>>,

    /// A raw pointer to the heap location of this runtime object.
    ptr: *mut Runtime,

    /// A Lua interpreter state.
    state: lua::State,
}

/// A function that can be bound to be callable inside the Lua runtime.
pub type RuntimeFn = fn(&mut Runtime) -> i32;

/// A single build task.
pub struct Task {
    /// The name of the task.
    pub name: String,

    /// A list of task names that must be ran before this task.
    pub deps: Vec<String>,

    /// A reference to this task's callback function.
    func: lua::Reference,
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
            ptr: 0 as *mut Runtime,
            state: lua::State::new(),
        });

        // Store a raw self pointer to this runtime object.
        runtime.ptr = runtime.as_ptr();

        // Prepare the environment.
        runtime.state.open_libs();

        // Register core functions.
        runtime.register_fn("task", functions::task);
        runtime.register_fn("default", functions::default);
        runtime.register_fn("print", functions::print);
        runtime.register_fn("glob", functions::glob);
        runtime.register_fn("export", functions::export);

        // Register the module loader.
        runtime.register_loader(functions::loader);

        // Load the core Lua module.
        try!(runtime.eval("require 'core'"));

        Ok(runtime)
    }

    /// Gets a mutable instance of the Lua interpreter state.
    ///
    /// This function uses the direct lua_State pointer, so multiple owners can all mutate the same
    /// state simultaneously. Obviously this is a little unsafe, so use responsibly.
    pub fn state(&mut self) -> lua::State {
        let ptr = self.state.as_ptr();
        unsafe { lua::State::from_ptr(ptr) }
    }

    /// Loads a project script.
    pub fn load(&mut self, filename: &str) -> Result<(), Error> {
        // Load the given file.
        let result = self.state().do_file(filename);

        match result {
            lua::ThreadStatus::Ok => { }
            lua::ThreadStatus::FileError => {
                return Err(Error::new(RoteError::FileNotReadable, &format!("the file \"{}\" could not be read", filename)));
            }
            _ => {
                return Err(self.get_last_error().unwrap());
            }
        };

        Ok(())
    }

    /// Creates a new task.
    pub fn create_task(&mut self, name: String, deps: Vec<String>, func: lua::Reference) {
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
        self.state.raw_geti(lua::REGISTRYINDEX, task.borrow().func.value() as i64);

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

    /// Registers a global function in the runtime that can be called by Lua scripts.
    pub fn register_fn(&mut self, name: &str, f: RuntimeFn) {
        self.push_fn(f);
        self.state.set_global(name);
    }

    /// Registers a function as a new package loader.
    ///
    /// The given loader is inserted into the list of module searchers before other loaders. This
    /// is to ensure that built-in and native modules are loaded quickly and cannot be shadowed by
    /// other modules of the same name.
    pub fn register_loader(&mut self, f: RuntimeFn) {
        self.state.get_global("package");
        self.state.get_field(-1, "searchers");
        let count = self.state.raw_len(-1) as i64;

        // Shift the existing loaders after [1] to the right.
        for i in 0..(count - 1) {
            self.state.raw_geti(-1, count - i);
            self.state.raw_seti(-2, count - i + 1);
        }

        self.push_fn(f);
        self.state.raw_seti(-2, 2);
        self.state.pop(2);
    }

    /// Raises an error message.
    pub fn throw_error(&mut self, message: &str) {
        self.state.location(1);
        self.state.push_string(message);
        self.state.concat(2);
        self.state.error();
    }

    /// Closes the runtime.
    pub fn close(self) {
        self.state.close();
    }

    /// Pushes a safe runtime function onto the stack.
    pub fn push_fn(&mut self, f: RuntimeFn) {
        // First push a pointer to the runtime and a pointer to the given function so that we know
        // what function to delegate to and what runtime to pass.
        unsafe {
            self.state.push_light_userdata(self.ptr);
            self.state.push_light_userdata(f as *mut usize);
        }

        // Push a wrapper function onto the stack, which delegates to the given function.
        self.state.push_closure(Some(fn_wrapper), 2);

        // Wrapper function for invoking Rust functions from inside Lua.
        unsafe extern fn fn_wrapper(l: *mut ffi::lua_State) -> i32 {
            // Get the runtime from the raw pointer.
            let mut state = lua::State::from_ptr(l);
            let runtime = state.to_userdata(ffi::lua_upvalueindex(1)) as *mut Runtime;

            // Get the raw pointer and turn it back into a Rust function pointer.
            let f_raw_ptr = state.to_userdata(ffi::lua_upvalueindex(2)) as *mut usize;
            let f: RuntimeFn = mem::transmute(f_raw_ptr);

            // Invoke the function.
            f(&mut *runtime)
        }
    }

    /// Gets the runtime as a raw pointer.
    fn as_ptr(&self) -> *mut Runtime {
        self as *const Runtime as *mut Runtime
    }

    /// Gets the last error pushed on the Lua stack.
    fn get_last_error(&mut self) -> Option<Error> {
        if self.state.is_string(-1) {
            Some(Error::new(RoteError::Runtime, self.state.to_str(-1).unwrap()))
        } else {
            None
        }
    }
}
