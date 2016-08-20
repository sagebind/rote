use glob;
use lua;
use regex::{Captures, Regex};
use rule::Rule;
use runtime::{Runtime, ScriptResult};
use std::env;
use std::io::prelude::*;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::str;
use task::NamedTask;


/// Expands global and environment variables inside a given string.
pub fn expand_string(input: &str, runtime: Runtime) -> String {
    // Replace anything that looks like a variable expansion.
    let pattern = Regex::new(r"\$(\w+)").unwrap();

    pattern.replace_all(input, |caps: &Captures| {
        let name = caps.at(1).unwrap_or("");

        // Attempt to match a variable definition, or fallback to an empty string.
        runtime.state().get_global(name);
        let value = runtime.state().to_str_in_place(-1).unwrap_or("").to_string();
        runtime.state().pop(1);

        value
    })
}

fn get_next_description(runtime: Runtime) -> Option<String> {
    runtime.reg_get("rote.nextDescription");

    let result = if runtime.state().is_string(-1) {
        Some(runtime.state().check_string(-1).to_string())
    } else {
        None
    };

    runtime.state().pop(1);
    runtime.state().push_nil();
    runtime.reg_set("rote.nextDescription");

    result
}


/// Sets the current working directory.
fn change_dir(runtime: Runtime) -> ScriptResult {
    let path = runtime.state().check_string(1).to_string();

    if env::set_current_dir(path).is_err() {
        Err("failed to change directory".into())
    } else {
        Ok(0)
    }
}

/// Defines a new rule.
///
/// # Lua arguments
/// * `pattern: string`      - The name of the task.
/// * `description: string`  - A description of the task. (Optional)
/// * `dependencies: table`  - A list of task names that the rule depends on. (Optional)
/// * `func: function`       - A function that should be called when the rule is run. (Optional)
fn create_rule(runtime: Runtime) -> ScriptResult {
    let pattern = runtime.state().check_string(1).to_string();
    let mut func_index = 3;

    // Get the list of dependencies if given.
    let deps = if runtime.state().type_of(2) == Some(lua::Type::Table) {
        runtime.iter(2)
            .map(|(_, value)| runtime.state().to_str_in_place(value).unwrap().to_string())
            .collect()
    } else {
        func_index -= 1;
        Vec::new()
    };

    // Get the task function if given.
    runtime.state().push_value(func_index);
    let func = if runtime.state().type_of(-1) == Some(lua::Type::Function) {
        // Get a portable reference to the task function.
        Some(runtime.state().reference(lua::REGISTRYINDEX))
    } else {
        runtime.state().pop(1);
        None
    };

    let closure_env = runtime.clone();
    let callback = func.map(|func| {
        move |name: &str| {
            // Get the function reference onto the Lua stack.
            closure_env.state().raw_geti(lua::REGISTRYINDEX, func.value() as i64);

            // Push the synthesized name onto the stack.
            closure_env.state().push(name);

            // Invoke the task function.
            closure_env.environment().set_current_task(name);
            let result = closure_env.call(1, 0, 0).map(|_| ()).map_err(|e| e.into());
            closure_env.environment().clear_current_task();

            result
        }
    });

    runtime.environment().create_rule(Rule::new(pattern, deps, callback));
    Ok(0)
}

/// Creates a new task.
///
/// # Lua arguments
/// * `name: string`         - The name of the task.
/// * `description: string`  - A description of the task. (Optional)
/// * `dependencies: table`  - A list of task names that the task depends on. (Optional)
/// * `func: function`       - A function that should be called when the task is run.
fn create_task(runtime: Runtime) -> ScriptResult {
    let name = runtime.state().check_string(1).to_string();
    let desc = get_next_description(runtime.clone());
    let mut func_index = 3;

    // Get the list of dependencies if given.
    let deps = if runtime.state().type_of(2) == Some(lua::Type::Table) {
        runtime.iter(2)
            .map(|(_, value)| runtime.state().to_str_in_place(value).unwrap().to_string())
            .collect()
    } else {
        func_index -= 1;
        Vec::new()
    };

    // Get the task function if given.
    runtime.state().push_value(func_index);
    let func = if runtime.state().type_of(-1) == Some(lua::Type::Function) {
        // Get a portable reference to the task function.
        Some(runtime.state().reference(lua::REGISTRYINDEX))
    } else {
        runtime.state().pop(1);
        None
    };

    let closure_env = runtime.clone();
    let later_name = name.clone();
    let callback = func.map(|func| {
        move || {
            // Get the function reference onto the Lua stack.
            closure_env.state().raw_geti(lua::REGISTRYINDEX, func.value() as i64);

            // Invoke the task function.
            let name = name.clone();
            closure_env.environment().set_current_task(name);
            let result = closure_env.call(0, 0, 0).map(|_| ()).map_err(|e| e.into());
            closure_env.environment().clear_current_task();

            result
        }
    });

    runtime.environment().create_task(NamedTask::new(later_name, desc, deps, callback));
    Ok(0)
}

/// Gets the current working directory.
fn current_dir(runtime: Runtime) -> ScriptResult {
    Ok(env::current_dir()
        .map(|dir| {
            runtime.state().push(dir.to_str());
            1
        })
        .unwrap_or(0))
}

/// Executes a shell command with a given list of arguments.
fn execute(runtime: Runtime) -> ScriptResult {
    // Create a command for the given program name.
    let mut command = Command::new(runtime.state().check_string(1));

    // Set the current directory.
    if let Ok(dir) = env::current_dir() {
        command.current_dir(dir);
    }

    // For each other parameter given, add it as a shell argument.
    for i in 2..runtime.state().get_top()+1 {
        // Expand each argument as we go.
        command.arg(expand_string(runtime.state().check_string(i), runtime.clone()));
    }

    // Spawn the command, capturing its status.
    command.status().map_err(|e| {
        format!("failed to execute process: {}", e).into()
    }).and_then(|status| {
        let status = status.code().unwrap_or(1);

        if status > 0 {
            Err("command returned nonzero exit code".into())
        } else {
            runtime.state().push_number(status as f64);
            Ok(1)
        }
    })
}

/// Pipes a string into a shell command with a given list of arguments.
fn pipe(runtime: Runtime) -> ScriptResult {
    // Create a command for the given program name.
    let mut command = Command::new(runtime.state().check_string(2));

    // Set the current directory.
    if let Ok(dir) = env::current_dir() {
        command.current_dir(dir);
    }

    // For each other parameter given, add it as a shell argument.
    for i in 3..runtime.state().get_top()+1 {
        // Expand each argument as we go.
        command.arg(expand_string(runtime.state().check_string(i), runtime.clone()));
    }

    // Get the input buffer string, if given.
    let input = if runtime.state().type_of(1) == Some(lua::Type::Nil) {
        command.stdin(Stdio::null());
        None
    } else {
        command.stdin(Stdio::piped());
        Some(runtime.state().check_string(1).to_string())
    };

    command.stdout(Stdio::piped());
    command.stderr(Stdio::piped());

    // Start running the command process.
    let mut child = match command.spawn() {
        Ok(child) => child,
        Err(e) => return Err(format!("failed to execute process: {}", e).into()),
    };

    // Write the input string to the pipe if given.
    if let Some(input) = input {
        if let Err(e) = child.stdin.as_mut().unwrap().write_all(input.as_bytes()) {
            return Err(format!("failed to execute process: {}", e).into());
        }
    }

    // Wait for the program to finish and collect the output.
    child.wait_with_output().map_err(|e| {
        format!("failed to execute process: {}", e).into()
    }).and_then(|output| {
        unsafe {
            runtime.state().push_string(str::from_utf8_unchecked(&output.stdout));
            runtime.state().push_string(str::from_utf8_unchecked(&output.stderr));
        }
        runtime.state().push_number(output.status.code().unwrap_or(1) as f64);

        Ok(3)
    })
}

/// Expands global and environment variables inside a given string.
fn expand(runtime: Runtime) -> ScriptResult {
    // Get the input string.
    let input = runtime.state().check_string(1).to_string();

    // Expand the string.
    let result = expand_string(&input, runtime.clone());

    // Return the result.
    runtime.state().push_string(&result);
    Ok(1)
}

/// Gets the value of an environment variable.
///
/// # Lua arguments
/// * `key: string` - The variable name.
fn env(runtime: Runtime) -> ScriptResult {
    let key = runtime.state().check_string(1).to_string();

    if let Ok(value) = env::var(key) {
        runtime.state().push(value);
    } else {
        runtime.state().push_nil();
    }

    Ok(1)
}

/// Exports an environment variable.
///
/// # Lua arguments
/// * `key: string` - The variable name.
/// * `value: string` - The value to set.
fn export(runtime: Runtime) -> ScriptResult {
    let key = runtime.state().check_string(1).to_string();
    let value = runtime.state().check_string(2).to_string();
    let expanded = expand_string(&value, runtime.clone());

    env::set_var(key, expanded);
    Ok(0)
}

/// Searches for paths matching a pattern.
///
/// # Lua arguments
/// * `pattern: string` - The glob pattern to match.
fn glob(runtime: Runtime) -> ScriptResult {
    // Get the pattern as the first argument.
    let pattern = runtime.state().check_string(1).to_string();

    // Make the pattern absolute if it isn't already.
    let mut full_path = PathBuf::from(pattern);
    if full_path.is_relative() {
        let path = full_path;
        full_path = env::current_dir().unwrap();
        full_path.push(&path);
    }

    // Create a table of values to return.
    runtime.state().new_table();

    // Get an iterator for the glob result and return a Lua iterator.
    if let Ok(mut iter) = glob::glob(&full_path.to_str().unwrap()) {
        runtime.push_closure(Box::new(move |runtime: Runtime| {
            loop {
                match iter.next() {
                    Some(Ok(path)) => {
                        // Turn the path into a string.
                        if let Some(path) = path.to_str() {
                            // Push the path onto the return value list.
                            runtime.state().push(path);
                            return Ok(1);
                        } else {
                            warn!("bad characters in path from glob");
                        }
                    }
                    Some(Err(_)) => {
                        // if the path matched but was unreadable,
                        // thereby preventing its contents from matching
                        warn!("unreadable path in glob");
                        // just continue the loop until we find a path we care about
                    }
                    None => {
                        trace!("reached end of iterator");
                        runtime.state().push_nil();
                        return Ok(1);
                    }
                }
            }
        }));
        Ok(1)
    } else {
        warn!("invalid glob pattern");
        Ok(0)
    }
}

/// Creates a new table produced by merging all tables given as arguments.
///
/// This makes a deep copy of all tables given into a new table. No tables are modified.
fn merge(runtime: Runtime) -> ScriptResult {
    let args_count = runtime.state().get_top();

    // merged = {}
    runtime.state().new_table();

    // Walk through the arguments, merging as we go.
    for i in 1..args_count+1 {
        if runtime.state().is_table(i) {
            do_merge(runtime.clone(), i, args_count + 1);
        }
    }

    fn do_merge(runtime: Runtime, src_index: i32, dest_index: i32) {
        // Iterate over every key in the source table.
        for (key, value) in runtime.iter(src_index) {
            runtime.state().push_value(key);

            if runtime.state().is_table(value) {
                runtime.state().new_table();

                // Merge the inner table recursively.
                do_merge(runtime.clone(), value, runtime.state().get_top());
            } else {
                runtime.state().push_value(value);
            }

            // t[k] = v
            runtime.state().set_table(dest_index);
        }
    }

    // If the input table is nil, just use the defaults table as the result.
    if runtime.state().is_nil(1) {
        runtime.state().push_value(2);
    } else {
        // Copy the input table
        runtime.state().push_value(1);
        // Create a table to be used as the input's metatable
        // Set __index in the metatable to be the defaults table
        runtime.state().push("__index");
        runtime.state().push_value(2);
        runtime.state().set_table(-3);
        runtime.state().set_metatable(-2);
    }

    Ok(1)
}

/// Prints text to the console output.
///
/// # Lua arguments
/// * `str: string` - The string to print.
fn print(runtime: Runtime) -> ScriptResult {
    for i in 1..runtime.state().get_top()+1 {
        let string = runtime.state().to_str(i).unwrap_or("").to_string();
        runtime.state().pop(1);

        let string = expand_string(&string, runtime.clone());
        println!("{}", string);
    }

    Ok(0)
}

/// Sets the default task.
///
/// # Lua arguments
/// * `name: string` - The name of the task to set as default.
fn set_default_task(runtime: Runtime) -> ScriptResult {
    // Get the task name as the first argument.
    let name = runtime.state().check_string(1).to_string();

    // Set the default task to the given name.
    runtime.environment().set_default_task(name);

    Ok(0)
}

/// Sets the description for the next task.
///
/// # Lua arguments
/// * `name: string` - The description.
fn set_description(runtime: Runtime) -> ScriptResult {
    let desc = runtime.state().check_string(1).to_string();
    runtime.state().push(desc);
    runtime.reg_set("rote.nextDescription");

    Ok(0)
}

/// Returns the current version of Rote as a string.
fn version(runtime: Runtime) -> ScriptResult {
    runtime.state().push_string(::ROTE_VERSION);
    Ok(1)
}


/// Makes the standard Rote module functions available in the runtime.
pub fn load(runtime: Runtime) {
    // Load the module functions.
    runtime.load_lib(&[
        ("change_dir", change_dir),
        ("create_rule", create_rule),
        ("create_task", create_task),
        ("current_dir", current_dir),
        ("env", env),
        ("execute", execute),
        ("expand", expand),
        ("export", export),
        ("glob", glob),
        ("merge", merge),
        ("pipe", pipe),
        ("print", print),
        ("set_default_task", set_default_task),
        ("version", version),
    ]);
    runtime.state().set_global("rote");

    // Define some global aliases.
    runtime.register_fn("default", set_default_task);
    runtime.register_fn("desc", set_description);
    runtime.register_fn("env", env);
    runtime.register_fn("exec", execute);
    runtime.register_fn("export", export);
    runtime.register_fn("glob", glob);
    runtime.register_fn("pipe", pipe);
    runtime.register_fn("print", print);
    runtime.register_fn("rule", create_rule);
    runtime.register_fn("task", create_task);

    // Set up reading global values to fallback to environment variables.
    runtime.state().push_global_table();
    runtime.state().new_table();
    runtime.state().push("__index");
    runtime.push_closure(Box::new(|runtime| {
        runtime.state().remove(1);
        env(runtime)
    }));
    runtime.state().set_table(-3);
    runtime.state().set_metatable(-2);
    runtime.state().pop(1);

    // Set up pipe to be a string method.
    runtime.state().get_global("string");
    runtime.state().push("pipe");
    runtime.push_fn(pipe);
    runtime.state().set_table(-3);
    runtime.state().pop(1);
}
