use error::{Error, RoteError};
use glob;
use lua;
use runtime::Runtime;
use std::cell::RefCell;
use std::collections::{HashMap, LinkedList};
use std::env;
use std::rc::{Rc, Weak};
use term;

/// A single build task.
pub struct Task {
    /// The name of the task.
    pub name: String,

    /// The task description.
    pub description: Option<String>,

    /// A list of task names that must be ran before this task.
    pub deps: Vec<String>,

    /// A closure for invoking the task.
    closure: Box<Fn(Vec<String>) -> Result<(), Error>>,
}

/// A task runner object that holds the state for defined tasks, dependencies, and the scripting
/// runtime.
pub struct Runner {
    /// A map of all defined tasks.
    pub tasks: HashMap<String, Rc<RefCell<Task>>>,

    /// The name of the default task to run.
    default_task: Option<String>,

    /// Task execution stack.
    stack: LinkedList<Weak<RefCell<Task>>>,

    /// The set description for the next defined task.
    next_description: Option<String>,

    /// The scripting runtime.
    runtime: Rc<RefCell<Box<Runtime>>>,
}

impl Runner {
    /// Creates a new runner instance.
    ///
    /// The instance is placed inside a box to ensure the runner has a constant location in memory
    /// so that it can be referenced by native closures in the runtime.
    pub fn new() -> Result<Box<Runner>, Error> {
        let runner = Box::new(Runner {
            tasks: HashMap::new(),
            default_task: None,
            stack: LinkedList::new(),
            next_description: None,
            runtime: Rc::new(RefCell::new(try!(Runtime::new()))),
        });

        {
            let mut runtime = runner.runtime.borrow_mut();

            // Register core functions.
            let ptr = &*runner as *const Runner as usize;
            runtime.register_fn("task", task, Some(ptr));
            runtime.register_fn("desc", desc, Some(ptr));
            runtime.register_fn("default", default, Some(ptr));
            runtime.register_fn("print", print, Some(ptr));
            runtime.register_fn("glob", glob, None);
            runtime.register_fn("export", export, None);

            // Load the core Lua module.
            try!(runtime.eval("require 'core'"));
        }

        Ok(runner)
    }

    /// Creates a new task.
    pub fn create_task(&mut self, name: String, description: Option<String>, deps: Vec<String>, func: lua::Reference) {
        let runtime = self.runtime.clone();

        // Create a task object.
        let task = Task {
            name: name,
            description: description,
            deps: deps,
            closure: Box::new(move |args: Vec<String>| -> Result<(), Error> {
                // Get the function reference onto the Lua stack.
                runtime.borrow_mut().state().raw_geti(lua::REGISTRYINDEX, func.value() as i64);

                // Push the given task arguments onto the stack.
                for string in &args {
                    runtime.borrow_mut().state().push_string(string);
                }

                // Invoke the task function.
                if runtime.borrow_mut().state().pcall(args.len() as i32, 0, 0).is_err() {
                    return Err(runtime.borrow_mut().get_last_error().unwrap());
                }

                runtime.borrow_mut().state().pop(1);

                Ok(())
            })
        };

        // Add it to the master list of tasks.
        self.tasks.insert(task.name.clone(), Rc::new(RefCell::new(task)));
    }

    /// Gets the default task to run, if any.
    pub fn default_task(&self) -> Option<Rc<RefCell<Task>>> {
        self.default_task
            .as_ref()
            .and_then(|name| self.tasks.get(name))
            .map(|rc| rc.clone())
        // ^ Look at that snazzy functional code.
    }

    /// Loads a build file script.
    pub fn load(&mut self, filename: &str) -> Result<(), Error> {
        self.runtime.borrow_mut().load(filename)
    }

    /// Runs a task with a specified name.
    pub fn run(&mut self, name: &str, args: Vec<String>) -> Result<(), Error> {
        // Determine the name of the task to run.
        let task_name = if name == "default" {
            if let Some(ref default_name) = self.default_task {
                default_name.clone()
            } else {
                return Err(Error::new(RoteError::TaskNotFound, "no default task defined"));
            }
        } else {
            name.to_string()
        };

        // Get the task object from the given name.
        let task = if let Some(task) = self.tasks.get(&task_name) {
            task.clone()
        } else {
            return Err(Error::new(RoteError::TaskNotFound, &format!("no such task \"{}\"", name)));
        };

        // Set the active task.
        self.stack.push_front(Rc::downgrade(&task));

        // Run all dependencies first concurrently in individual threads. We will use a mutex
        // around a finish count to alert the parent when all the dependencies have finished.
        for dep_name in &task.borrow().deps {
            try!(self.run(dep_name, Vec::new()));
        }

        // Call the task itself.
        try!((task.borrow().closure)(args));

        // Pop the task off the call stack.
        self.stack.pop_front();

        Ok(())
    }
}

/// Defines a new task.
///
/// # Lua arguments
/// * `name: string`         - The name of the task.
/// * `dependencies: table`  - A list of task names that the task depends on. (Optional)
/// * `func: function`       - A function that should be called when the task is run.
fn task(runtime: &mut Runtime, data: Option<usize>) -> i32 {
    let runner = unsafe { &mut *(data.unwrap() as *mut Runner) };

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
            let dep = runtime.state().to_str(-1).unwrap().to_string();
            runtime.state().pop(2);

            deps.push(dep);
        }

        arg_index += 1;
    }

    // Third argument is the task function.
    runtime.state().check_type(arg_index, lua::Type::Function);

    // Get a portable reference to the task function.
    let func = runtime.state().reference(lua::REGISTRYINDEX);

    // Create the task.
    let desc = runner.next_description.as_ref().map(|desc| desc.clone());
    runner.next_description = None;
    runner.create_task(name.to_string(), desc, deps, func);

    0
}

/// Sets the description of the next task.
fn desc(runtime: &mut Runtime, data: Option<usize>) -> i32 {
    let runner = unsafe { &mut *(data.unwrap() as *mut Runner) };

    let description = runtime.state().check_string(1).to_string();
    runner.next_description = Some(description);

    0
}

/// Sets the default task.
///
/// # Lua arguments
/// * `name: string` - The name of the task to set as default.
fn default(runtime: &mut Runtime, data: Option<usize>) -> i32 {
    let runner = unsafe { &mut *(data.unwrap() as *mut Runner) };

    // Get the task name as the first argument.
    let name = runtime.state().check_string(1).to_string();

    // Set the default task to the given name.
    runner.default_task = Some(name);

    0
}

/// Prints text to the console output.
///
/// # Lua arguments
/// * `str: string` - The string to print.
fn print(runtime: &mut Runtime, data: Option<usize>) -> i32 {
    let runner = unsafe { &mut *(data.unwrap() as *mut Runner) };
    let mut out = term::stdout().unwrap();

    if !runner.stack.is_empty() {
        let cell = runner.stack.front().unwrap().upgrade().unwrap();
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
fn glob(runtime: &mut Runtime, _: Option<usize>) -> i32 {
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
fn export(runtime: &mut Runtime, _: Option<usize>) -> i32 {
    let key = runtime.state().check_string(1).to_string();
    let value = runtime.state().check_string(2).to_string();

    env::set_var(key, value);

    0
}
