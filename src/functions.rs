/// This module contains various global functions that are bound and available inside the Lua runtime.

use glob;
use lua;
use modules::{fetch, Module};
use runtime::Runtime;
use std::env;
use term;


/// A Lua module loader that loads built-in modules.
///
/// # Lua arguments
/// * `name: string`         - The name of the module to load.
pub fn loader(runtime: &mut Runtime) -> i32 {
    // Get the module name as the first argument.
    let name = runtime.state().check_string(1).to_string();

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
        runtime.state().push_string(&format!("\n\tno builtin module '{}'", name));
    }
    1
}

/// Native module loader callback.
fn loader_native(runtime: &mut Runtime) -> i32 {
    let name = runtime.state().check_string(1).to_string();

    if let Some(Module::Native(mtable)) = fetch(&name) {
        runtime.state().new_table();

        for &(name, func) in mtable.0 {
            runtime.push_fn(func);
            runtime.state().set_field(-2, name);
        }

        runtime.state().set_global(&name);

        1
    } else {
        0
    }
}

/// Defines a new task.
///
/// # Lua arguments
/// * `name: string`         - The name of the task.
/// * `dependencies: table`  - A list of task names that the task depends on. (Optional)
/// * `func: function`       - A function that should be called when the task is run.
pub fn task(runtime: &mut Runtime) -> i32 {
    let mut arg_index = 1;

    // Get the task name as the first argument.
    let name = runtime.state().check_string(arg_index).to_string();
    arg_index += 1;

    // Second argument is a table of dependent task names (optional).
    let mut deps: Vec<String> = Vec::new();
    if runtime.state().is_table(arg_index) {
        // Read all of the names in the table and add it to the deps vector.
        runtime.state().push_nil();
        while runtime.state().next(arg_index) {
            runtime.state().push_value(-2);
            let dep = runtime.state().to_str(-2).unwrap().to_string();
            runtime.state().pop(1);

            deps.push(dep);
        }

        arg_index += 1;
    }

    // Third argument is the task function.
    runtime.state().check_type(arg_index, lua::Type::Function);

    // Get a portable reference to the task function.
    let func = runtime.state().reference(lua::REGISTRYINDEX);

    // Create the task.
    runtime.create_task(name.to_string(), deps, func);

    0
}

/// Sets the default task.
///
/// # Lua arguments
/// * `name: string` - The name of the task to set as default.
pub fn default(runtime: &mut Runtime) -> i32 {
    // Get the task name as the first argument.
    let name = runtime.state().check_string(1).to_string();

    // Set the default task to the given name.
    runtime.default_task = Some(name);

    0
}

/// Prints text to the console output.
///
/// # Lua arguments
/// * `str: string` - The string to print.
pub fn print(runtime: &mut Runtime) -> i32 {
    let mut out = term::stdout().unwrap();

    if !runtime.stack.is_empty() {
        let cell = runtime.stack.front().unwrap().upgrade().unwrap();
        out.fg(term::color::GREEN).unwrap();
        write!(out, "[{}]\t", cell.borrow().name).unwrap();
        out.reset().unwrap();
    }

    let string = runtime.state().check_string(1).to_string();
    writeln!(out, "{}", &string).unwrap();

    0
}

/// Searches for paths matching a pattern.
///
/// # Lua arguments
/// * `pattern: string` - The glob pattern to match.
pub fn glob(runtime: &mut Runtime) -> i32 {
    // Get the pattern as the first argument.
    let pattern = runtime.state().check_string(1).to_string();

    // Create a table of values to return.
    runtime.state().new_table();

    // Match the pattern and push the results onto the stack.
    let mut index = 1;
    for entry in glob::glob(&pattern).unwrap() {
        match entry {
            Ok(path) => {
                // Push the path onto the return value list.
                runtime.state().push_string(path.to_str().unwrap());
                runtime.state().raw_seti(2, index);
            },

            // if the path matched but was unreadable,
            // thereby preventing its contents from matching
            Err(_) => {},
        }

        index += 1;
    }

    1
}

/// Exports an environment variable.
///
/// # Lua arguments
/// * `key: string` - The variable name.
/// * `value: string` - The value to set.
pub fn export(runtime: &mut Runtime) -> i32 {
    let key = runtime.state().check_string(1).to_string();
    let value = runtime.state().check_string(2).to_string();

    env::set_var(key, value);

    0
}
