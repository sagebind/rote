#[macro_use] extern crate log;
extern crate lua;

use error::Error;
use lua::ffi;
use std::clone;
use std::mem;

#[macro_use] pub mod error;
mod iter;

pub use self::iter::{
    TableIterator,
    TableItem
};

pub use lua::ffi::lua_State as LuaState;


/// A Lua script runtime for parsing and executing build script functions.
pub struct Runtime {
    /// A Lua interpreter state.
    state: lua::State,
}

/// A function that can be bound to be callable inside the Lua runtime.
pub type RuntimeFn = fn(Runtime) -> i32;


/// A runtime instance that wraps a Lua state with convenience methods.
///
/// The entirety of a runtime's state is contained within the Lua state that it wraps; this allows multiple runtime
/// objects to coexist safely at different memory locations and refer to the same runtime data.
impl Runtime {
    /// Creates a new runtime instance.
    ///
    /// Creates a new Lua state, which is owned by the returned runtime.
    pub fn new() -> Runtime {
        Runtime {
            state: lua::State::new(),
        }
    }

    /// Gets a runtime instance from a pointer.
    ///
    /// The state is not owned by the returned runtime instance.
    pub fn from_ptr<'r>(ptr: *mut LuaState) -> Runtime {
        Runtime {
            state: unsafe { lua::State::from_ptr(ptr) }
        }
    }

    /// Initializes the runtime environment.
    ///
    /// Please call this *after* you've put the runtime in a stable memory location. Thank you.
    pub fn init(&mut self) {
        // Prepare the environment.
        self.state.open_libs();

        // Register the module loader.
        //self.register_module_searcher(searcher);
    }

    /// Gets the runtime as a raw pointer.
    pub fn as_ptr(&self) -> *mut LuaState {
        self.state.as_ptr()
    }

    /// Gets a mutable instance of the Lua interpreter state.
    ///
    /// This function uses the direct lua_State pointer, so multiple owners can all mutate the same
    /// state simultaneously. Obviously this is a little unsafe, so use responsibly.
    pub fn state(&self) -> lua::State {
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
                throw!(Error::FileNotReadable(filename.to_string()));
            }
            _ => {
                throw!(self.get_last_error().unwrap());
            }
        };

        Ok(())
    }

    /// Evaluates a Lua string inside the runtime.
    pub fn eval(&mut self, code: &str) -> Result<(), Error> {
        if self.state.do_string(code).is_err() {
            //throw!(self.get_last_error().unwrap());
        }

        Ok(())
    }

    /// Registers a global function in the runtime that can be called by Lua scripts.
    pub fn register_fn(&mut self, name: &str, f: RuntimeFn) {
        self.push_fn(f);
        self.state.set_global(name);
    }

    /// Registers a new dynamic module library.
    pub fn register_lib(&mut self, mtable: &[(&str, RuntimeFn)]) {
        self.state().create_table(0, mtable.len() as i32);

        for &(name, func) in mtable {
            self.push_fn(func);
            self.state().set_field(-2, name);
        }
    }

    /// Registers a function as a new package searcher.
    ///
    /// The given loader is inserted into the list of module searchers before other loaders. This
    /// is to ensure that built-in and native modules are loaded quickly and cannot be shadowed by
    /// other modules of the same name.
    pub fn register_module_searcher(&mut self, f: RuntimeFn) {
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

    /// Raises an error message.
    pub fn throw_error(&mut self, message: &str) {
        self.state.location(1);
        self.state.push_string(message);
        self.state.concat(2);
        self.state.error();
    }

    /// Pushes a safe runtime function onto the stack.
    pub fn push_fn(&mut self, f: RuntimeFn) {
        // First push a pointer to the runtime and a pointer to the given function so that we know
        // what function to delegate to and what runtime to pass.
        unsafe {
            self.state.push_light_userdata(f as *mut usize);
        }

        // Push a wrapper function onto the stack, which delegates to the given function.
        self.state.push_closure(Some(fn_wrapper), 1);

        // Wrapper function for invoking Rust functions from inside Lua.
        unsafe extern fn fn_wrapper(ptr: *mut LuaState) -> i32 {
            // Get the runtime from the raw pointer.
            let mut runtime = Runtime::from_ptr(ptr);

            // Get the raw pointer and turn it back into a Rust function pointer.
            let f_raw_ptr = runtime.state.to_userdata(ffi::lua_upvalueindex(1)) as *mut usize;
            let f: RuntimeFn = mem::transmute(f_raw_ptr);

            // Invoke the function.
            f(runtime)
        }
    }

    pub fn iter(&mut self, index: lua::Index) -> TableIterator {
        TableIterator::new(self.clone(), index)
    }

    /// Gets the last error pushed on the Lua stack.
    pub fn get_last_error(&mut self) -> Option<Error> {
        if self.state.is_string(-1) {
            Some(Error::RuntimeError(self.state.to_str(-1).unwrap().to_string()))
        } else {
            None
        }
    }

    /// Gets a stored pointer value from the runtime state registry.
    pub fn reg_get<'a, T>(&mut self, name: &str) -> Option<&'a mut T> {
        self.state().push_string(name);
        self.state().get_table(lua::REGISTRYINDEX);

        if !self.state().is_userdata(-1) {
            return None;
        }

        unsafe {
            let pointer = self.state().to_userdata(-1);
            self.state().pop(1);
            Some(mem::transmute(pointer))
        }
    }

    /// Stores a pointer value into the runtime state registry.
    pub fn reg_set(&mut self, name: &str, pointer: *mut usize) {
        self.state().push_string(name);
        unsafe {
            self.state().push_light_userdata(pointer);
        }
        self.state().set_table(lua::REGISTRYINDEX);
    }
}

// Runtime objects can be cloned; this just creates a new object pointing to the same Lua state.
impl clone::Clone for Runtime {
    fn clone(&self) -> Runtime {
        Runtime::from_ptr(self.as_ptr())
    }
}


/*
/// A Lua module loader that loads built-in modules.
///
/// # Lua arguments
/// * `name: string`         - The name of the module to load.
fn searcher(runtime: &mut Runtime) -> i32 {
    // Get the module name as the first argument.
    let name = runtime.state().check_string(1).to_string();
    info!("loading module \"{}\"", name);

    if let Some(module) = fetch(&name) {
        match module {
            Module::Builtin(source) => {
                runtime.state().load_string(source);
            },
            Module::Native(_) => {
                runtime.push_fn(loader_native);
            },
        };
    } else {
        info!("no builtin module '{}'", name);
        runtime.state().push_string(&format!("\n\tno builtin module '{}'", name));
    }
    1
}

/// Native module loader callback.
fn loader_native(runtime: &mut Runtime) -> i32 {
    let name = runtime.state().check_string(1).to_string();

    if let Some(Module::Native(mtable)) = fetch(&name) {
        runtime.state().create_table(mtable.0.len() as i32, 0);

        for &(name, func) in mtable.0 {
            runtime.push_fn(func);
            runtime.state().set_field(-2, name);
        }

        1
    } else {
        0
    }
}
*/
