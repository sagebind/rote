use error::Error;
use error::RoteError;
use lua;
use lua::ffi;
use modules::{fetch, Module};
use std::mem;


/// A Lua script runtime for parsing and executing build script functions.
pub struct Runtime {
    /// A raw pointer to the heap location of this runtime object.
    ptr: *mut Runtime,

    /// A Lua interpreter state.
    state: lua::State,
}

/// A function that can be bound to be callable inside the Lua runtime.
pub type RuntimeFn = fn(&mut Runtime, Option<usize>) -> i32;

impl Runtime {
    /// Creates a new runtime instance.
    ///
    /// The runtime instance is allocated onto the heap. This allows the runtime object to be passed
    /// around as raw pointers in closure upvalues. The caller will own the box that owns the
    /// runtime instance.
    pub fn new() -> Result<Box<Runtime>, Error> {
        let mut runtime = Box::new(Runtime {
            ptr: 0 as *mut Runtime,
            state: lua::State::new(),
        });

        // Store a raw self pointer to this runtime object.
        runtime.ptr = runtime.as_ptr();

        // Prepare the environment.
        runtime.state.open_libs();

        // Register the module loader.
        runtime.register_loader(loader);

        Ok(runtime)
    }

    /// Gets the runtime as a raw pointer.
    pub fn as_ptr(&self) -> *mut Runtime {
        self as *const Runtime as *mut Runtime
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

    /// Evaluates a Lua string inside the runtime.
    pub fn eval(&mut self, code: &str) -> Result<(), Error> {
        if self.state.do_string(code).is_err() {
            return Err(self.get_last_error().unwrap());
        }

        Ok(())
    }

    /// Registers a global function in the runtime that can be called by Lua scripts.
    pub fn register_fn(&mut self, name: &str, f: RuntimeFn, data: Option<usize>) {
        self.push_fn(f, data);
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

        self.push_fn(f, None);
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

    /// Pushes a safe runtime function onto the stack.
    pub fn push_fn(&mut self, f: RuntimeFn, data: Option<usize>) {
        // First push a pointer to the runtime and a pointer to the given function so that we know
        // what function to delegate to and what runtime to pass.
        unsafe {
            self.state.push_light_userdata(self.ptr);
            self.state.push_light_userdata(f as *mut usize);

            if let Some(data) = data {
                self.state.push_number(1f64);
                self.state.push_light_userdata(data as *mut usize);
            } else {
                self.state.push_number(0f64);
            }
        }

        // Push a wrapper function onto the stack, which delegates to the given function.
        self.state.push_closure(Some(fn_wrapper), if data.is_some() { 4 } else { 3 });

        // Wrapper function for invoking Rust functions from inside Lua.
        unsafe extern fn fn_wrapper(l: *mut ffi::lua_State) -> i32 {
            // Get the runtime from the raw pointer.
            let mut state = lua::State::from_ptr(l);
            let runtime = state.to_userdata(ffi::lua_upvalueindex(1)) as *mut Runtime;

            // Get the raw pointer and turn it back into a Rust function pointer.
            let f_raw_ptr = state.to_userdata(ffi::lua_upvalueindex(2)) as *mut usize;
            let f: RuntimeFn = mem::transmute(f_raw_ptr);

            // Optionally get the closure data value.
            let data = if state.to_number(ffi::lua_upvalueindex(3)) == 1f64 {
                Some(state.to_userdata(ffi::lua_upvalueindex(4)) as usize)
            } else {
                None
            };

            // Invoke the function.
            f(&mut *runtime, data)
        }
    }

    /// Gets the last error pushed on the Lua stack.
    pub fn get_last_error(&mut self) -> Option<Error> {
        if self.state.is_string(-1) {
            Some(Error::new(RoteError::Runtime, self.state.to_str(-1).unwrap()))
        } else {
            None
        }
    }
}

/// A Lua module loader that loads built-in modules.
///
/// # Lua arguments
/// * `name: string`         - The name of the module to load.
fn loader(runtime: &mut Runtime, _: Option<usize>) -> i32 {
    // Get the module name as the first argument.
    let name = runtime.state().check_string(1).to_string();

    if let Some(module) = fetch(&name) {
        match module {
            Module::Builtin(source) => {
                runtime.state().load_string(source);
            },
            Module::Native(_) => {
                runtime.push_fn(loader_native, None);
            },
        };
    } else {
        runtime.state().push_string(&format!("\n\tno builtin module '{}'", name));
    }
    1
}

/// Native module loader callback.
fn loader_native(runtime: &mut Runtime, _: Option<usize>) -> i32 {
    let name = runtime.state().check_string(1).to_string();

    if let Some(Module::Native(mtable)) = fetch(&name) {
        runtime.state().new_table();

        for &(name, func) in mtable.0 {
            runtime.push_fn(func, None);
            runtime.state().set_field(-2, name);
        }

        runtime.state().set_global(&name);

        1
    } else {
        0
    }
}
