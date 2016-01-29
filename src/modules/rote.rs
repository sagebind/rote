use lua;
use modules::ModuleTable;
use runtime::Runtime;
use runner::Runner;


pub const MTABLE: ModuleTable = ModuleTable(&[
    ("create_task", create_task),
    ("create_rule", create_rule),
    ("set_default_task", set_default_task),
]);

/// Creates a new task.
///
/// # Lua arguments
/// * `name: string`         - The name of the task.
/// * `description: string`  - A description of the task. (Optional)
/// * `dependencies: table`  - A list of task names that the task depends on. (Optional)
/// * `func: function`       - A function that should be called when the task is run.
fn create_task(runtime: &mut Runtime) -> i32 {
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
    0
}

/// Defines a new rule.
///
/// # Lua arguments
/// * `pattern: string`      - The name of the task.
/// * `description: string`  - A description of the task. (Optional)
/// * `dependencies: table`  - A list of task names that the rule depends on. (Optional)
/// * `func: function`       - A function that should be called when the rule is run. (Optional)
fn create_rule(runtime: &mut Runtime) -> i32 {
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
    0
}

/// Sets the default task.
///
/// # Lua arguments
/// * `name: string` - The name of the task to set as default.
fn set_default_task(runtime: &mut Runtime) -> i32 {
    let runner: &mut Runner = runtime.reg_get("runner").unwrap();

    // Get the task name as the first argument.
    let name = runtime.state().check_string(1).to_string();

    // Set the default task to the given name.
    runner.default_task = Some(name);

    0
}
