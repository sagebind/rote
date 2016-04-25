use iter::TableIterator;
use lua::{self, ffi};
use std::clone;
use std::error::Error;
use std::mem;


/// Results that are returned by functions callable from Lua.
pub type RuntimeResult = Result<i32, Box<Error>>;

/// A function that can be bound to be callable inside the Lua runtime.
pub type Function = fn(Runtime) -> RuntimeResult;

/// A Lua script runtime for parsing and executing build script functions.
pub struct Runtime {
    /// A Lua interpreter state.
    state: lua::State
}


/// A runtime instance that wraps a Lua state with convenience methods.
///
/// The entirety of a runtime's state is contained within the Lua state that it wraps; this allows multiple runtime
/// objects to coexist safely at different memory locations and refer to the same runtime data.
impl Runtime {
    /// Creates a new runtime instance.
    ///
    /// A new Lua state is created, which is owned by the returned runtime.
    pub fn new() -> Runtime {
        // Create a new Lua state.
        let mut state = lua::State::new();

        // Initialize the standard Lua libraries.
        state.open_libs();

        Runtime {
            state: state
        }
    }

    /// Gets a runtime instance from a pointer.
    ///
    /// The state is not owned by the returned runtime instance.
    pub unsafe fn from_ptr<'r>(ptr: *mut ffi::lua_State) -> Runtime {
        assert!(!ptr.is_null());

        Runtime {
            state: lua::State::from_ptr(ptr)
        }
    }

    /// Gets the runtime as a raw pointer.
    pub fn as_ptr(&self) -> *mut ffi::lua_State {
        self.state.as_ptr()
    }

    /// Gets a mutable instance of the Lua interpreter state.
    ///
    /// This function uses the direct lua_State pointer, so multiple owners can all mutate the same
    /// state simultaneously. Obviously this is a little unsafe, so use responsibly.
    pub fn state(&self) -> lua::State {
        unsafe {
            lua::State::from_ptr(self.as_ptr())
        }
    }

    /// Loads a project script.
    pub fn load(&mut self, filename: &str) -> Result<(), Box<Error>> {
        // Load the given file.
        let result = self.state.do_file(filename);

        match result {
            lua::ThreadStatus::Ok => { }
            lua::ThreadStatus::FileError => {
                return Err(format!("the file \"{}\" could not be read", filename.to_string()).into());
            }
            _ => {
                return Err(self.state.to_str(-1).unwrap().into());
            }
        };

        Ok(())
    }

    /// Evaluates a Lua string inside the runtime.
    pub fn eval(&mut self, code: &str) -> Result<(), Box<Error>> {
        if self.state.do_string(code).is_err() {
            return Err(self.state.to_str(-1).unwrap().into());
        }

        Ok(())
    }

    /// Registers a global function in the runtime that can be called by Lua scripts.
    pub fn register_fn(&mut self, name: &str, f: Function) {
        self.push_fn(f);
        self.state.set_global(name);
    }

    /// Registers a new dynamic module library.
    pub fn register_lib(&mut self, mtable: &[(&str, Function)]) {
        self.state.create_table(0, mtable.len() as i32);

        for &(name, func) in mtable {
            self.push_fn(func);
            self.state.set_field(-2, name);
        }
    }

    /// Adds a path to Lua's require path for modules.
    pub fn add_path(&mut self, path: &str) {
        self.state.get_global("package");
        self.state.get_field(-1, "path");

        let current_path = self.state.to_str(-1).unwrap().to_string();

        let mut new_path = String::from(path);
        new_path.push(';');
        new_path.push_str(&current_path);

        self.state.push_string(&new_path);
        self.state.set_field(-4, "path");
        self.state.pop(3);
    }

    /// Adds a path to Lua's require path for native modules.
    pub fn add_cpath(&mut self, path: &str) {
        self.state.get_global("package");
        self.state.get_field(-1, "cpath");

        let current_path = self.state.to_str(-1).unwrap().to_string();
        let mut new_path = String::from(path);
        new_path.push(';');
        new_path.push_str(&current_path);

        self.state.push_string(&new_path);
        self.state.set_field(-4, "cpath");
        self.state.pop(3);
    }

    /// Pushes a safe Rust function onto the stack.
    pub fn push_fn(&mut self, f: Function) {
        // First push a pointer to the given function so that we know what function to delegate to.
        unsafe {
            self.state.push_light_userdata(f as *mut usize);
        }

        // Push a wrapper function onto the stack, which delegates to the given function.
        self.state.push_closure(Some(fn_wrapper), 1);

        // Wrapper function for invoking Rust functions from inside Lua.
        unsafe extern fn fn_wrapper(ptr: *mut ffi::lua_State) -> i32 {
            // Get the runtime from the raw pointer.
            let mut runtime = Runtime::from_ptr(ptr);

            // Get the raw pointer and turn it back into a Rust function pointer.
            let f_raw_ptr = runtime.state.to_userdata(ffi::lua_upvalueindex(1)) as *mut usize;
            let f: Function = mem::transmute(f_raw_ptr);

            // Invoke the function.
            f(runtime.clone()).unwrap_or_else(|err: Box<Error>| {
                runtime.state.location(1);
                runtime.state.push_string(err.description());
                runtime.state.concat(2);
                runtime.state.error();
            })
        }
    }

    pub fn iter(&mut self, index: lua::Index) -> TableIterator {
        TableIterator::new(self.clone(), index)
    }

    /// Wrapper around `lua_pcall()` that catches errors as a result.
    pub fn call(&mut self, nargs: i32, nresults: i32, msgh: i32) -> Result<lua::ThreadStatus, Box<Error>> {
        let status = self.state.pcall(nargs, nresults, msgh);

        if status.is_err() {
            if self.state.is_string(-1) {
                Err(self.state.to_str(-1).unwrap().to_string().into())
            } else {
                Err("unknown error".into())
            }
        } else {
            Ok(status)
        }
    }

    /// Gets a stored pointer value from the runtime state registry.
    pub fn reg_get<'a, T>(&mut self, name: &str) -> Option<&'a mut T> {
        self.state.push_string(name);
        self.state.get_table(lua::REGISTRYINDEX);

        if !self.state.is_userdata(-1) {
            return None;
        }

        unsafe {
            let pointer = self.state.to_userdata(-1);
            self.state.pop(1);
            Some(mem::transmute(pointer))
        }
    }

    /// Stores a pointer value into the runtime state registry.
    pub fn reg_set<T>(&mut self, name: &str, pointer: *mut T) {
        self.state.push_string(name);
        unsafe {
            self.state.push_light_userdata(pointer);
        }
        self.state.set_table(lua::REGISTRYINDEX);
    }
}

// Runtime objects can be cloned; this just creates a new object pointing to the same Lua state.
impl clone::Clone for Runtime {
    fn clone(&self) -> Runtime {
        unsafe {
            Runtime::from_ptr(self.as_ptr())
        }
    }
}
