//use runner::Runner;
use glob;
use regex::{Captures, Regex};
use script::{Environment, ScriptResult};
use script::lua;
use script::rule::Rule;
use script::task::NamedTask;
use std::env;
use std::io::prelude::*;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::str;
use std::rc::Rc;
use term;


/// Expands global and environment variables inside a given string.
pub fn expand_string(input: &str, environment: Environment) -> String {
    // Replace anything that looks like a variable expansion.
    let pattern = Regex::new(r"\$(\w+)").unwrap();

    pattern.replace_all(input, |caps: &Captures| {
        let name = caps.at(1).unwrap_or("");

        // Attempt to match a variable definition, or fallback to the original string.
        environment.var(name).unwrap_or("".to_string())
    })
}

fn get_next_description(environment: Environment) -> Option<String> {
    environment.reg_get("rote.nextDescription");

    let result = if environment.state().is_string(-1) {
        Some(environment.state().check_string(-1).to_string())
    } else {
        None
    };

    environment.state().pop(1);
    environment.state().push_nil();
    environment.reg_set("rote.nextDescription");

    result
}


/// Sets the current working directory.
fn change_dir(environment: Environment) -> ScriptResult {
    let path = environment.state().check_string(1).to_string();

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
fn create_rule(environment: Environment) -> ScriptResult {
    let pattern = environment.state().check_string(1).to_string();
    let mut func_index = 3;

    // Get the list of dependencies if given.
    let deps = if environment.state().type_of(2) == Some(lua::Type::Table) {
        environment.iter(2)
            .map(|mut item| item.value().unwrap())
            .collect()
    } else {
        func_index -= 1;
        Vec::new()
    };

    // Get the task function if given.
    environment.state().push_value(func_index);
    let func = if environment.state().type_of(-1) == Some(lua::Type::Function) {
        // Get a portable reference to the task function.
        Some(environment.state().reference(lua::REGISTRYINDEX))
    } else {
        environment.state().pop(1);
        None
    };

    let closure_env = environment.clone();
    let callback = Rc::new(move |name: &str| {
        if let Some(func) = func {
            // Get the function reference onto the Lua stack.
            closure_env.state().raw_geti(lua::REGISTRYINDEX, func.value() as i64);

            // Push the synthesized name onto the stack.
            closure_env.state().push(name);

            // Invoke the task function.
            closure_env.call(1, 0, 0).map(|_| ()).map_err(|e| e.into())
        } else {
            Ok(())
        }
    });

    environment.create_rule(Rc::new(Rule::new(pattern, deps, Some(callback))));
    Ok(0)
}

/// Creates a new task.
///
/// # Lua arguments
/// * `name: string`         - The name of the task.
/// * `description: string`  - A description of the task. (Optional)
/// * `dependencies: table`  - A list of task names that the task depends on. (Optional)
/// * `func: function`       - A function that should be called when the task is run.
fn create_task(environment: Environment) -> ScriptResult {
    let name = environment.state().check_string(1).to_string();
    let desc = get_next_description(environment.clone());
    let mut func_index = 3;

    // Get the list of dependencies if given.
    let deps = if environment.state().type_of(2) == Some(lua::Type::Table) {
        environment.iter(2)
            .map(|mut item| item.value().unwrap())
            .collect()
    } else {
        func_index -= 1;
        Vec::new()
    };

    // Get the task function if given.
    environment.state().push_value(func_index);
    let func = if environment.state().type_of(-1) == Some(lua::Type::Function) {
        // Get a portable reference to the task function.
        Some(environment.state().reference(lua::REGISTRYINDEX))
    } else {
        environment.state().pop(1);
        None
    };

    let closure_env = environment.clone();
    let callback = Box::new(move || {
        if let Some(func) = func {
            // Get the function reference onto the Lua stack.
            closure_env.state().raw_geti(lua::REGISTRYINDEX, func.value() as i64);

            // Invoke the task function.
            closure_env.call(0, 0, 0).map(|_| ()).map_err(|e| e.into())
        } else {
            Ok(())
        }
    });

    environment.create_task(Rc::new(NamedTask::new(name, desc, deps, Some(callback))));
    Ok(0)
}

/// Gets the current working directory.
fn current_dir(environment: Environment) -> ScriptResult {
    Ok(env::current_dir()
        .map(|dir| {
            environment.state().push(dir.to_str());
            1
        })
        .unwrap_or(0))
}

/// Executes a shell command with a given list of arguments.
fn execute(environment: Environment) -> ScriptResult {
    // Create a command for the given program name.
    let mut command = Command::new(environment.state().check_string(1));

    // Set the current directory.
    if let Ok(dir) = env::current_dir() {
        command.current_dir(dir);
    }

    // For each other parameter given, add it as a shell argument.
    for i in 2..environment.state().get_top()+1 {
        // Expand each argument as we go.
        command.arg(expand_string(environment.state().check_string(i), environment.clone()));
    }

    // Execute the command and capture the exit status.
    command.status().map_err(|e| {
        format!("failed to execute process: {}", e).into()
    }).and_then(|status| {
        environment.state().push_number(status.code().unwrap_or(1) as f64);
        Ok(1)
    })
}

/// Executes a shell command with a given list of arguments.
fn pipe(environment: Environment) -> ScriptResult {
    // Create a command for the given program name.
    let mut command = Command::new(environment.state().check_string(2));

    // Set the current directory.
    if let Ok(dir) = env::current_dir() {
        command.current_dir(dir);
    }

    // For each other parameter given, add it as a shell argument.
    for i in 3..environment.state().get_top()+1 {
        // Expand each argument as we go.
        command.arg(expand_string(environment.state().check_string(i), environment.clone()));
    }

    // Get the input buffer string, if given.
    let input = if environment.state().type_of(1) == Some(lua::Type::Nil) {
        command.stdin(Stdio::null());
        None
    } else {
        command.stdin(Stdio::piped());
        Some(environment.state().check_string(1).to_string())
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
            environment.state().push_string(str::from_utf8_unchecked(&output.stdout));
            environment.state().push_string(str::from_utf8_unchecked(&output.stderr));
        }
        environment.state().push_number(output.status.code().unwrap_or(1) as f64);

        Ok(3)
    })
}

/// Expands global and environment variables inside a given string.
fn expand(environment: Environment) -> ScriptResult {
    // Get the input string.
    let input = environment.state().check_string(1).to_string();

    // Expand the string.
    let result = expand_string(&input, environment.clone());

    // Return the result.
    environment.state().push_string(&result);
    Ok(1)
}

/// Exports an environment variable.
///
/// # Lua arguments
/// * `key: string` - The variable name.
/// * `value: string` - The value to set.
fn export(environment: Environment) -> ScriptResult {
    let key = environment.state().check_string(1).to_string();
    let value = environment.state().check_string(2).to_string();
    let expanded = expand_string(&value, environment.clone());

    env::set_var(key, expanded);
    Ok(0)
}

/// Searches for paths matching a pattern.
///
/// # Lua arguments
/// * `pattern: string` - The glob pattern to match.
fn glob(environment: Environment) -> ScriptResult {
    // Get the pattern as the first argument.
    let pattern = environment.state().check_string(1).to_string();

    // Make the pattern absolute if it isn't already.
    let mut full_path = PathBuf::from(pattern);
    if full_path.is_relative() {
        let path = full_path;
        full_path = env::current_dir().unwrap();
        full_path.push(&path);
    }

    // Create a table of values to return.
    environment.state().new_table();

    // Get an iterator for the glob result and return a Lua iterator.
    if let Ok(mut iter) = glob::glob(&full_path.to_str().unwrap()) {
        environment.push_closure(Box::new(move |environment: Environment| {
            loop {
                match iter.next() {
                    Some(Ok(path)) => {
                        // Turn the path into a string.
                        if let Some(path) = path.to_str() {
                            // Push the path onto the return value list.
                            environment.state().push(path);
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
                        environment.state().push_nil();
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

/// Parses an input table of options and merges it with a table of default values.
fn options(environment: Environment) -> ScriptResult {
    // If the input table is nil, just use the defaults table as the result.
    if environment.state().is_nil(1) {
        environment.state().push_value(2);
    } else {
        // Copy the input table
        environment.state().push_value(1);
        // Create a table to be used as the input's metatable
        environment.state().new_table();
        // Set __index in the metatable to be the defaults table
        environment.state().push("__index");
        environment.state().push_value(2);
        environment.state().set_table(-3);
        environment.state().set_metatable(-2);
    }

    Ok(1)
}

/// Prints text to the console output.
///
/// # Lua arguments
/// * `str: string` - The string to print.
fn print(environment: Environment) -> ScriptResult {
    let string = environment.state().check_string(1).to_string();
    let string = expand_string(&string, environment);
    let mut out = term::stdout().unwrap();

    if false {
        /*for line in string.lines() {
            let name = environment.stack.front().unwrap();
            out.fg(term::color::GREEN).unwrap();
            write!(out, "{:9} ", format!("[{}]", name)).unwrap();
            out.reset().unwrap();

            writeln!(out, "{}", line).unwrap();
        }*/
    } else {
        writeln!(out, "{}", string).unwrap();
    }

    Ok(0)
}

/// Sets the default task.
///
/// # Lua arguments
/// * `name: string` - The name of the task to set as default.
fn set_default_task(environment: Environment) -> ScriptResult {
    // Get the task name as the first argument.
    let name = environment.state().check_string(1).to_string();

    // Set the default task to the given name.
    environment.set_default_task(name);

    Ok(0)
}

/// Sets the description for the next task.
///
/// # Lua arguments
/// * `name: string` - The description.
fn set_description(environment: Environment) -> ScriptResult {
    let desc = environment.state().check_string(1).to_string();
    environment.state().push(desc);
    environment.reg_set("rote.nextDescription");

    Ok(0)
}

/// Returns the current version of Rote as a string.
fn version(environment: Environment) -> ScriptResult {
    environment.state().push_string(::ROTE_VERSION);
    Ok(1)
}


/// Makes the standard Rote module functions available in the environment.
pub fn open_lib(environment: Environment) {
    // Load the module functions.
    environment.register_lib(&[
        ("change_dir", change_dir),
        ("create_rule", create_rule),
        ("create_task", create_task),
        ("current_dir", current_dir),
        ("execute", execute),
        ("expand", expand),
        ("export", export),
        ("glob", glob),
        ("options", options),
        ("pipe", pipe),
        ("print", print),
        ("set_default_task", set_default_task),
        ("version", version),
    ]);
    environment.state().set_global("rote");

    // Define some global aliases.
    environment.register_fn("default", set_default_task);
    environment.register_fn("desc", set_description);
    environment.register_fn("exec", execute);
    environment.register_fn("export", export);
    environment.register_fn("glob", glob);
    environment.register_fn("pipe", pipe);
    environment.register_fn("print", print);
    environment.register_fn("rule", create_rule);
    environment.register_fn("task", create_task);

    // Set up pipe to be a string method.
    environment.state().get_global("string");
    environment.state().push("pipe");
    environment.push_fn(pipe);
    environment.state().set_table(-3);
    environment.state().pop(1);
}
