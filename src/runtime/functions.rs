use glob;
use lua::ffi;
use lua::wrapper::state;
use modules;
use runtime::{Runtime, RuntimePtr};


/// Imports a Lua module.
///
/// # Lua arguments
/// * `name: string` - The name of the module to require.
pub fn require<'r>(runtime: RuntimePtr<'r>) -> i32 {
    // Get the module name as the first argument.
    let name = Runtime::borrow(runtime).state.check_string(1);

    // Check if the name matches a built-in module first.
    if let Some(module) = modules::fetch(name) {
        // Execute the module.
        if let Err(e) = Runtime::borrow(runtime).eval(module) {
            e.die();
        }

        // Return a module reference.
        Runtime::borrow(runtime).state.get_global(name);
        return 1;
    }

    // Call default require()
    Runtime::borrow(runtime).state.get_global("require_native");
    Runtime::borrow(runtime).state.push_string(name);
    if Runtime::borrow(runtime).state.pcall(1, 1, 0).is_err() {
        Runtime::borrow(runtime).get_last_error().unwrap().die();
    }

    1
}

/// Below are various global functions that are bound and available inside the Lua runtime.

/// Defines a new task.
///
/// # Lua arguments
/// * `name: string`         - The name of the task.
/// * `dependencies: table`  - A list of task names that the task depends on. (Optional)
/// * `func: function`       - A function that should be called when the task is run.
pub fn task<'r>(runtime: RuntimePtr<'r>) -> i32 {
    let mut arg_index = 1;

    // Get the task name as the first argument.
    let name = Runtime::borrow(runtime).state.check_string(arg_index);
    arg_index += 1;

    // Second argument is a table of dependent task names (optional).
    let mut deps: Vec<&str> = Vec::new();
    if Runtime::borrow(runtime).state.type_of(arg_index).unwrap() == state::Type::Table {
        // Read all of the names in the table and add it to the deps vector.
        Runtime::borrow(runtime).state.push_nil();
        while Runtime::borrow(runtime).state.next(arg_index) {
            Runtime::borrow(runtime).state.push_value(-2);
            let dep = Runtime::borrow(runtime).state.to_str(-2).unwrap();
            Runtime::borrow(runtime).state.pop(1);

            deps.push(dep);
        }

        arg_index += 1;
    }

    // Third argument is the task function.
    Runtime::borrow(runtime).state.check_type(arg_index, state::Type::Function);

    // Get a portable reference to the task function.
    let func = Runtime::borrow(runtime).state.reference(ffi::LUA_REGISTRYINDEX);

    // Create the task.
    Runtime::borrow(runtime).create_task(name, deps, func);

    0
}

/// Sets the default task.
///
/// # Lua arguments
/// * `name: string` - The name of the task to set as default.
pub fn default<'r>(runtime: RuntimePtr<'r>) -> i32 {
    // Get the task name as the first argument.
    let name = Runtime::borrow(runtime).state.check_string(1);

    // Set the default task to the given name.
    Runtime::borrow(runtime).default_task = Some(name.to_string());

    0
}

/// Searches for paths matching a pattern.
///
/// # Lua arguments
/// * `pattern: string` - The glob pattern to match.
pub fn glob<'r>(runtime: RuntimePtr<'r>) -> i32 {
    // Get the pattern as the first argument.
    let pattern = Runtime::borrow(runtime).state.check_string(1);

    // Match the pattern and push the results onto the stack.
    let mut count = 0;
    for entry in glob::glob(pattern).unwrap() {
        match entry {
            Ok(path) => {
                // Push the path onto the return value list.
                Runtime::borrow(runtime).state.push_string(path.to_str().unwrap());
            },

            // if the path matched but was unreadable,
            // thereby preventing its contents from matching
            Err(_) => {},
        }

        count += 1;
    }

    count
}
