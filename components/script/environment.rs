use ::ScriptResult;
use iter::TableIterator;
use libc::{c_int, c_void};
use lua::{self, ffi};
use rule::Rule;
use std::cell::RefCell;
use std::clone::Clone;
use std::collections::HashMap;
use std::env;
use std::error::Error;
use std::mem;
use std::path::{Path, PathBuf};
use std::ptr;
use std::rc::{Rc, Weak};
use task::{Task, NamedTask};


// Just a tiny key used to obtain a unique registry location to store the environment pointer.
static KEY: f64 = 264299.0;

/// A function that can be bound to be callable inside the Lua runtime.
pub type Function = fn(Environment) -> ScriptResult;
pub type Closure = Box<FnMut(Environment) -> ScriptResult>;


/// Stores the state of an entire task execution environment.
struct EnvironmentStruct {
    /// A map of all named tasks.
    tasks: RefCell<HashMap<String, Rc<NamedTask>>>,

    /// A vector of all defined file rules.
    rules: RefCell<Vec<Rc<Rule>>>,

    /// The default task to run.
    default_task: RefCell<Option<String>>,

    path: PathBuf,

    directory: PathBuf,

    /// A Lua interpreter state.
    state: lua::State,
}

pub struct Environment(Rc<EnvironmentStruct>);

impl Environment {
    /// Creates a new environment for a given script file.
    ///
    /// The instance is placed inside a box to ensure the runner has a constant location in memory
    /// so that it can be referenced by native closures in the runtime.
    pub fn new<P: Into<PathBuf>>(script: P) -> Result<Environment, Box<Error>> {
        let script = script.into();
        let directory = match script.parent() {
            Some(path) => path.into(),
            None => {
                return Err("failed to parse script directory".into());
            }
        };

        let environment = Environment(Rc::new(EnvironmentStruct {
            tasks: RefCell::new(HashMap::new()),
            rules: RefCell::new(Vec::new()),
            default_task: RefCell::new(None),
            path: script,
            directory: directory,
            state: lua::State::new(),
        }));

        // Initialize the standard Lua libraries.
        environment.state().open_libs();

        // Set up the environment.
        environment.add_path("./components/?.lua");
        environment.add_cpath("/usr/lib/rote/plugins/?.so");

        if let Some(mut path) = env::current_exe().ok()
            .and_then(|path| path.parent()
                .map(|p| p.to_path_buf())
            ) {
            path.push("lib?.so");
            environment.add_cpath(path.to_str().unwrap());
        }

        // Create a weak pointer to the environment and push it into the registry so that you can
        // access the environment object by its Lua state.
        environment.state().push(KEY);
        let ptr = environment.state().new_userdata_typed();
        let weak = Rc::downgrade(&environment.0);
        unsafe {
            ptr::write(ptr, weak);
        }
        environment.state().set_table(lua::REGISTRYINDEX);

        // Set the OS
        environment.state().push_string(if cfg!(windows) {
            "windows"
        } else {
            "unix"
        });
        environment.state().set_global("OS");

        Ok(environment)
    }

    /// Fetches the environment by its Lua pointer.
    ///
    /// This function is far from safe, and should be used with care.
    pub unsafe fn from_ptr(ptr: ::StatePtr) -> Environment {
        // First, get a State object from the raw pointer.
        let mut state = lua::State::from_ptr(ptr);

        // Fetch the weak environment pointer from the registry.
        //state.push_light_userdata(&KEY as *const _ as *mut u8);
        state.push(KEY);
        state.get_table(lua::REGISTRYINDEX);

        // Read the weak pointer.
        let weak = match state.to_userdata_typed::<Weak<EnvironmentStruct>>(-1) {
            Some(weak) => weak,
            None => panic!("unable to read environment pointer")
        };

        // Upgrade the pointer and convert it to an environment.
        Environment(match Weak::upgrade(&weak) {
            Some(rc) => rc,
            None => panic!("unable to upgrade environment pointer")
        })
    }

    /// Executes the script.
    pub fn load(&self) -> Result<(), Box<Error>> {
        let path_str = if let Some(s) = self.0.path.to_str() {
            s
        } else {
            return Err("path contains invalid characters".into());
        };

        // Load the given file.
        match self.state().do_file(path_str) {
            lua::ThreadStatus::Ok => { }
            lua::ThreadStatus::FileError => {
                return Err(format!("the file \"{}\" could not be read", path_str).into());
            }
            _ => {
                return Err(self.state().to_str(-1).unwrap().into());
            }
        };

        Ok(())
    }

    /// Gets the full path of the script file.
    pub fn path(&self) -> &Path {
        &self.0.path
    }

    /// Gets the full path of the directory containing the script file.
    pub fn directory(&self) -> &Path {
        &self.0.directory
    }

    /// Gets a list of all registered tasks.
    pub fn tasks(&self) -> Vec<Rc<NamedTask>> {
        self.0.tasks.borrow().values().map(|rc| rc.clone()).collect()
    }

    /// Gets a list of all registered rules.
    pub fn rules(&self) -> Vec<Rc<Rule>> {
        self.0.rules.borrow().iter().map(|rc| rc.clone()).collect()
    }

    /// Creates a new task.
    pub fn create_task(&self, task: Rc<NamedTask>) {
        // Add it to the master list of tasks.
        self.0.tasks.borrow_mut().insert(task.name().into(), task.clone());
    }

    /// Creates a new rule.
    pub fn create_rule(&self, rule: Rc<Rule>) {
        self.0.rules.borrow_mut().push(rule);
    }

    /// Gets a task by name.
    pub fn get_task<S: AsRef<str>>(&self, name: S) -> Option<Rc<NamedTask>> {
        self.0.tasks.borrow().get(name.as_ref()).map(|rc| rc.clone())
    }

    /// Gets the default task to run.
    pub fn default_task<'a>(&'a self) -> Option<String> {
        match *self.0.default_task.borrow() {
            Some(ref task) => Some(task.clone()),
            None => None,
        }
    }

    /// Sets the default task.
    pub fn set_default_task<S: Into<String>>(&self, name: S) {
        *self.0.default_task.borrow_mut() = Some(name.into());
    }

    /// Gets a mutable instance of the Lua interpreter state.
    ///
    /// This function uses the direct lua_State pointer, so multiple owners can all mutate the same
    /// state simultaneously. Obviously this is a little unsafe, so use responsibly.
    pub fn state(&self) -> lua::State {
        unsafe {
            lua::State::from_ptr(self.0.state.as_ptr())
        }
    }

    /// Evaluates a Lua string inside the runtime.
    pub fn eval<S: AsRef<str>>(&self, code: S) -> Result<(), Box<Error>> {
        if self.state().do_string(code.as_ref()).is_err() {
            return Err(self.state().to_str(-1).unwrap().into());
        }

        Ok(())
    }

    /// Registers a global function in the runtime that can be called by Lua scripts.
    pub fn register_fn(&self, name: &str, f: Function) {
        self.push_fn(f);
        self.state().set_global(name);
    }

    /// Registers a new dynamic module library.
    pub fn register_lib(&self, mtable: &[(&str, Function)]) {
        self.state().create_table(0, mtable.len() as i32);

        for &(name, func) in mtable {
            self.push_fn(func);
            self.state().set_field(-2, name);
        }
    }

    /// Pushes a safe Rust function onto the stack.
    pub fn push_fn(&self, function: Function) {
        unsafe {
            // Push a pointer to the given function so that we know what function to delegate to.
            self.state().push_light_userdata(function as *mut c_void);
        }

        // Push a wrapper function onto the stack, which delegates to the given function.
        self.state().push_closure(Some(fn_wrapper), 1);

        // Wrapper function for invoking Rust functions from inside Lua.
        unsafe extern fn fn_wrapper(ptr: *mut ffi::lua_State) -> c_int {
            // Get the environment from the raw pointer.
            let environment = Environment::from_ptr(ptr);

            // Get the raw pointer and turn it back into a Rust function pointer.
            let raw_ptr = environment.state().to_userdata(ffi::lua_upvalueindex(1));
            let function: Function = mem::transmute(raw_ptr);

            // Invoke the function.
            function(environment).unwrap_or_else(|err: Box<Error>| {
                let mut state = lua::State::from_ptr(ptr);

                state.location(1);
                state.push_string(err.description());
                state.concat(2);
                state.error();
            }) as c_int
        }
    }

    /// Pushes a safe Rust closure onto the stack.
    ///
    /// Note that this currently leaks memory as there is no way to cleanup resources, so it should be used sparingly.
    pub fn push_closure(&self, closure: Closure) {
        let cb = Box::new(closure);

        unsafe {
            // Push a pointer to the closure so that we know what to delegate to.
            self.state().push_light_userdata(Box::into_raw(cb) as *mut c_void);
        }

        // Push a wrapper function onto the stack, which delegates to the given function.
        self.state().push_closure(Some(fn_wrapper), 1);

        // Wrapper function for invoking Rust functions from inside Lua.
        unsafe extern fn fn_wrapper(ptr: *mut ffi::lua_State) -> c_int {
            // Get the environment from the raw pointer.
            let environment = Environment::from_ptr(ptr);

            // Get the raw pointer and turn it back into a Rust closure pointer.
            let raw_ptr = environment.state().to_userdata(ffi::lua_upvalueindex(1));
            let closure: &mut Closure = mem::transmute(raw_ptr);

            // Invoke the closure.
            closure(environment).unwrap_or_else(|err: Box<Error>| {
                let mut state = lua::State::from_ptr(ptr);

                state.location(1);
                state.push_string(err.description());
                state.concat(2);
                state.error();
            }) as c_int
        }
    }

    /// Returns an iterator for iterating over the table at the top of the stack.
    pub fn iter(&self, index: lua::Index) -> TableIterator {
        TableIterator::new(self.state(), index)
    }

    /// Wrapper around `lua_pcall()` that catches errors as a result.
    pub fn call(&self, nargs: i32, nresults: i32, msgh: i32) -> Result<lua::ThreadStatus, Box<Error>> {
        let status = self.state().pcall(nargs, nresults, msgh);

        if status.is_err() {
            if self.state().is_string(-1) {
                Err(self.state().to_str(-1).unwrap().to_string().into())
            } else {
                Err("unknown error".into())
            }
        } else {
            Ok(status)
        }
    }

    /// Adds a path to Lua's require path for modules.
    pub fn add_path(&self, path: &str) {
        self.state().get_global("package");
        self.state().get_field(-1, "path");

        let current_path = self.state().to_str(-1).unwrap().to_string();

        let mut new_path = String::from(path);
        new_path.push(';');
        new_path.push_str(&current_path);

        self.state().push_string(&new_path);
        self.state().set_field(-4, "path");
        self.state().pop(3);
    }

    /// Adds a path to Lua's require path for native modules.
    pub fn add_cpath(&self, path: &str) {
        self.state().get_global("package");
        self.state().get_field(-1, "cpath");

        let current_path = self.state().to_str(-1).unwrap().to_string();
        let mut new_path = String::from(path);
        new_path.push(';');
        new_path.push_str(&current_path);

        self.state().push_string(&new_path);
        self.state().set_field(-4, "cpath");
        self.state().pop(3);
    }

    /// Pushes the value of a registry key onto the stack.
    pub fn reg_get(&self, name: &str) {
        self.state().push(name);
        self.state().get_table(lua::REGISTRYINDEX);
    }

    /// Sets a registry key to the value at the top of the stack.
    pub fn reg_set(&self, name: &str) {
        self.state().push(name);
        self.state().push_value(-2);
        self.state().set_table(lua::REGISTRYINDEX);
    }
}

/// Implement cloning for environment references, with the same semantics as an Rc.
impl Clone for Environment {
    fn clone(&self) -> Self {
        Environment(self.0.clone())
    }
}
