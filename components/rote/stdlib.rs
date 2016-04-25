use runner::Runner;
use glob;
use regex::{Captures, Regex};
use runtime::lua;
use runtime::{Runtime, RuntimeResult};
use std::env;
use std::process::{Command, Stdio};
use term;


/// Expands global and environment variables inside a given string.
pub fn expand_string(input: &str, runtime: Runtime) -> String {
    // Replace anything that looks like a variable expansion.
    let pattern = Regex::new(r"\$\((\w+)\)").unwrap();

    pattern.replace_all(input, |caps: &Captures| {
        let name = caps.at(1).unwrap_or("");

        // Attempt to match a variable definition, or fallback to the original string.
        if let Ok(value) = env::var(name) {
            value.to_string()
        } else {
            let value = if runtime.state().get_global(name) != lua::Type::Nil {
                runtime.state().check_string(-1).to_string()
            } else {
                caps.at(0).unwrap_or("").to_string()
            };

            runtime.state().pop(1);
            value
        }
    })
}


/// Sets the current working directory.
fn change_dir(runtime: Runtime) -> RuntimeResult {
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
fn create_rule(mut runtime: Runtime) -> RuntimeResult {
    let runner: &mut Runner = runtime.reg_get("runner").unwrap();

    let pattern = runtime.state().check_string(1).to_string();
    let desc = if runtime.state().is_string(2) {
        Some(runtime.state().check_string(2).to_string())
    } else {
        None
    };

    // Read the dependency table.
    runtime.state().check_type(3, lua::Type::Table);
    let deps: Vec<String> = runtime.iter(3)
        .map(|item| item.value().unwrap())
        .collect();

    // Third argument is the task function.
    let func = runtime.state().type_of(4).and_then(|t| {
        if t == lua::Type::Function {
            // Get a portable reference to the task function.
            Some(runtime.state().reference(lua::REGISTRYINDEX))
        } else {
            None
        }
    });

    runner.create_rule(pattern, desc, deps, func);
    Ok(0)
}

/// Creates a new task.
///
/// # Lua arguments
/// * `name: string`         - The name of the task.
/// * `description: string`  - A description of the task. (Optional)
/// * `dependencies: table`  - A list of task names that the task depends on. (Optional)
/// * `func: function`       - A function that should be called when the task is run.
fn create_task(mut runtime: Runtime) -> RuntimeResult {
    let runner: &mut Runner = runtime.reg_get("runner").unwrap();

    let name = runtime.state().check_string(1).to_string();
    let desc = if runtime.state().is_string(2) {
        Some(runtime.state().check_string(2).to_string())
    } else {
        None
    };

    // Read the dependency table.
    runtime.state().check_type(3, lua::Type::Table);
    let deps: Vec<String> = runtime.iter(3)
        .map(|item| item.value().unwrap())
        .collect();

    // Get a portable reference to the task function.
    runtime.state().check_type(4, lua::Type::Function);
    let func = runtime.state().reference(lua::REGISTRYINDEX);

    runner.create_task(name, desc, deps, func);
    Ok(0)
}

/// Gets the current working directory.
fn current_dir(runtime: Runtime) -> RuntimeResult {
    Ok(env::current_dir()
        .map(|dir| {
            runtime.state().push(dir.to_str());
            1
        })
        .unwrap_or(0))
}

/// Executes a shell command with a given list of arguments.
fn execute(runtime: Runtime) -> RuntimeResult {
    //let runner = Runner::from_runtime(runtime.clone());

    // Create a command for the given program name.
    let mut command = Command::new(runtime.state().check_string(1));

    // Set the current directory.
    if let Ok(dir) = env::current_dir() {
        command.current_dir(dir);
    }

    // For each other parameter given, add it as a shell argument.
    for i in 2..runtime.state().get_top() {
        // Expand each argument as we go.
        command.arg(expand_string(runtime.state().check_string(i), runtime.clone()));
    }

    // If we're not inside a task, just inherit standard output streams.
    command.stdout(Stdio::inherit());
    command.stderr(Stdio::inherit());

    // Run the command, piping output to stdout and returning the exit code.
    command.output().map_err(|e| {
        format!("failed to execute process: {}", e).into()
    }).and_then(|output| {
        runtime.state().push_number(output.status.code().unwrap_or(1) as f64);
        Ok(1)
    })
}

/// Expands global and environment variables inside a given string.
fn expand(runtime: Runtime) -> RuntimeResult {
    // Get the input string.
    let input = runtime.state().check_string(1).to_string();

    // Expand the string.
    let result = expand_string(&input, runtime.clone());

    // Return the result.
    runtime.state().push_string(&result);
    Ok(1)
}

/// Exports an environment variable.
///
/// # Lua arguments
/// * `key: string` - The variable name.
/// * `value: string` - The value to set.
fn export(runtime: Runtime) -> RuntimeResult {
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
fn glob(runtime: Runtime) -> RuntimeResult {
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
                runtime.state().push(path.to_str().unwrap());
                runtime.state().raw_seti(2, index);
            }

            // if the path matched but was unreadable,
            // thereby preventing its contents from matching
            Err(_) => {}
        }

        index += 1;
    }

    Ok(1)
}

/// Parses an input table of options and merges it with a table of default values.
fn options(runtime: Runtime) -> RuntimeResult {
    // If the input table is nil, just use the defaults table as the result.
    if runtime.state().is_nil(1) {
        runtime.state().push_value(2);
    } else {
        // Copy the input table
        runtime.state().push_value(1);
        // Create a table to be used as the input's metatable
        runtime.state().new_table();
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
fn print(runtime: Runtime) -> RuntimeResult {
    let runner = Runner::from_runtime(runtime.clone());
    let string = runtime.state().check_string(1).to_string();
    let string = expand_string(&string, runtime.clone());
    let mut out = term::stdout().unwrap();

    if !runner.stack.is_empty() {
        for line in string.lines() {
            let name = runner.stack.front().unwrap();
            out.fg(term::color::GREEN).unwrap();
            write!(out, "{:9} ", format!("[{}]", name)).unwrap();
            out.reset().unwrap();

            writeln!(out, "{}", line).unwrap();
        }
    } else {
        writeln!(out, "{}", string).unwrap();
    }

    Ok(0)
}

/// Sets the default task.
///
/// # Lua arguments
/// * `name: string` - The name of the task to set as default.
fn set_default_task(mut runtime: Runtime) -> RuntimeResult {
    let runner: &mut Runner = runtime.reg_get("runner").unwrap();

    // Get the task name as the first argument.
    let name = runtime.state().check_string(1).to_string();

    // Set the default task to the given name.
    runner.default_task = Some(name);

    Ok(0)
}

/// Returns the current version of Rote as a string.
fn version(runtime: Runtime) -> RuntimeResult {
    runtime.state().push_string(env!("CARGO_PKG_VERSION"));
    Ok(1)
}


/// Makes the standard Rote module functions available in the runtime.
pub fn open_lib(mut runtime: Runtime) {
    // Load the module functions.
    runtime.register_lib(&[
        ("change_dir", change_dir),
        ("create_rule", create_rule),
        ("create_task", create_task),
        ("current_dir", current_dir),
        ("execute", execute),
        ("expand", expand),
        ("export", export),
        ("glob", glob),
        ("options", options),
        ("print", print),
        ("set_default_task", set_default_task),
        ("version", version),
    ]);
    runtime.state().set_global("rote");

    // Define some global aliases.
    runtime.register_fn("exec", execute);
    runtime.register_fn("export", export);
    runtime.register_fn("print", print);
}
