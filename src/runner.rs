use error::Error;
use filetime::FileTime;
use glob;
use lua;
use runtime::Runtime;
use std::cell::RefCell;
use std::collections::{HashMap, LinkedList};
use std::env;
use std::fs;
use std::rc::Rc;
use term;


/// A single named build task.
pub struct Task {
    /// The name of the task.
    pub name: String,

    /// The task description.
    pub description: Option<String>,

    /// A list of tasks that must be ran before this task.
    pub deps: Vec<String>,

    /// Indicates if the task has been run.
    satisfied: bool,

    /// A reference to the Lua callback.
    func: lua::Reference,

    /// The runner the rule belongs to.
    runner: *mut Runner,
}

impl Task {
    pub fn is_satisfied(&self) -> bool {
        self.satisfied
    }

    pub fn run(&mut self, args: Vec<String>) -> Result<(), Error> {
        let runner = unsafe { &*self.runner };

        // Get the function reference onto the Lua stack.
        runner.runtime.borrow_mut().state().raw_geti(lua::REGISTRYINDEX, self.func.value() as i64);

        // Push the given task arguments onto the stack.
        for string in &args {
            runner.runtime.borrow_mut().state().push_string(string);
        }

        // Invoke the task function.
        if runner.runtime.borrow_mut().state().pcall(args.len() as i32, 0, 0).is_err() {
            throw!(runner.runtime.borrow_mut().get_last_error().unwrap());
        }

        Ok(())
    }
}


/// A rule task that matches against files.
pub struct Rule {
    /// The file pattern to match.
    pub pattern: String,

    /// The rule description.
    pub description: Option<String>,

    /// A list of tasks that must be ran before this task.
    pub deps: Vec<String>,

    /// A reference to the Lua callback.
    func: Option<lua::Reference>,

    /// The runner the rule belongs to.
    runner: *mut Runner,
}

impl Rule {
    pub fn matches(&self, file_name: &str) -> bool {
        if let Some(index) = self.pattern.find("%") {
            let (prefix, suffix) = self.pattern.split_at(index);
            let suffix = &suffix[1..];

            file_name.starts_with(prefix) && file_name.ends_with(suffix)
        } else {
            file_name.ends_with(&self.pattern)
        }
    }

    pub fn is_satisfied(&self, file_name: &str) -> bool {
        let runner = unsafe { &*self.runner };

        if let Some(age) = Self::get_age(file_name) {
            for dependency in &self.deps {
                if let Some(dep_age) = runner.match_rule(dependency)
                                             .and(Self::get_age(dependency)) {
                    if dep_age > age {
                        return false;
                    }
                } else {
                    return false;
                }
            }

            true
        } else {
            false
        }
    }

    /// Runs the rule with a specified matching file name.
    pub fn run(&self, file_name: &str) -> Result<(), Error> {
        if self.func.is_none() {
            return Ok(());
        }

        let runner = unsafe { &*self.runner };

        // Get the function reference onto the Lua stack.
        runner.runtime
              .borrow_mut()
              .state()
              .raw_geti(lua::REGISTRYINDEX, self.func.unwrap().value() as i64);

        // Pass the file name in as the only argument.
        runner.runtime.borrow_mut().state().push_string(file_name);

        // Invoke the task function.
        if runner.runtime.borrow_mut().state().pcall(1, 0, 0).is_err() {
            throw!(runner.runtime.borrow_mut().get_last_error().unwrap());
        }

        Ok(())
    }

    fn get_age(file_name: &str) -> Option<FileTime> {
        fs::metadata(file_name)
            .map(|metadata| FileTime::from_last_modification_time(&metadata))
            .ok()
    }
}


/// A task runner object that holds the state for defined tasks, dependencies, and the scripting
/// runtime.
pub struct Runner {
    /// A map of all defined tasks.
    pub tasks: HashMap<String, Rc<RefCell<Task>>>,

    /// A vector of all defined file rules.
    pub rules: Vec<Rc<Rule>>,

    /// The name of the default task to run.
    pub default_task: Option<String>,

    /// Task execution stack.
    stack: LinkedList<String>,

    /// Indicates if actually running tasks should be skipped.
    dry_run: bool,

    /// The scripting runtime.
    runtime: RefCell<Runtime>,
}

impl Runner {
    /// Creates a new runner instance.
    ///
    /// The instance is placed inside a box to ensure the runner has a constant location in memory
    /// so that it can be referenced by native closures in the runtime.
    pub fn new(dry_run: bool) -> Result<Box<Runner>, Error> {
        let runner = Box::new(Runner {
            tasks: HashMap::new(),
            rules: Vec::new(),
            default_task: None,
            stack: LinkedList::new(),
            dry_run: dry_run,
            runtime: RefCell::new(Runtime::new()),
        });

        runner.runtime.borrow_mut().init();

        // Set up the environment.
        runner.set_environment();

        // Register core functions.
        runner.runtime.borrow_mut().register_fn("print", print);
        runner.runtime.borrow_mut().register_fn("glob", glob);
        runner.runtime.borrow_mut().register_fn("current_dir", current_dir);
        runner.runtime.borrow_mut().register_fn("change_dir", change_dir);
        runner.runtime.borrow_mut().register_fn("export", export);

        // Load the core Lua module.
        try!(runner.runtime.borrow_mut().eval("require 'dsl'"));
        try!(runner.runtime.borrow_mut().eval("require 'core'"));

        Ok(runner)
    }

    /// Creates a new task.
    pub fn create_task(&mut self,
                       name: String,
                       description: Option<String>,
                       deps: Vec<String>,
                       func: lua::Reference) {
        let task = Task {
            name: name,
            description: description,
            deps: deps,
            satisfied: false,
            func: func,
            runner: self,
        };

        // Add it to the master list of tasks.
        self.tasks.insert(task.name.clone(), Rc::new(RefCell::new(task)));
    }

    /// Creates a new rule.
    pub fn create_rule(&mut self,
                       pattern: String,
                       description: Option<String>,
                       deps: Vec<String>,
                       func: Option<lua::Reference>) {
        let rule = Rule {
            pattern: pattern,
            description: description,
            deps: deps,
            func: func,
            runner: self,
        };

        self.rules.push(Rc::new(rule));
    }

    /// Gets the default task to run, if any.
    pub fn default_task(&self) -> Option<Rc<RefCell<Task>>> {
        self.default_task
            .as_ref()
            .and_then(|name| self.tasks.get(name))
            .map(|rc| rc.clone())
        // ^ Look at that snazzy functional code.
    }

    /// Gets a task or a matching file rule by name.
    pub fn get_task(&self, name: &str) -> Option<Rc<RefCell<Task>>> {
        self.tasks
            .get(name)
            .map(|rc| rc.clone())
    }

    /// Gets a rule for a given file name, if any defined rules match.
    pub fn match_rule(&self, file_name: &str) -> Option<Rc<Rule>> {
        if self.tasks.contains_key(file_name) {
            return None;
        }

        for rule in &self.rules {
            if rule.matches(file_name) {
                return Some(rule.clone());
            }
        }

        None
    }

    /// Loads a build file script.
    pub fn load(&mut self, filename: &str) -> Result<(), Error> {
        self.runtime.borrow_mut().load(filename)
    }

    /// Runs a task with a specified name.
    pub fn run(&mut self, name: &str, args: Vec<String>) -> Result<(), Error> {
        // If the default task is requested, fetch it.
        let name = if name == "default" {
            if let Some(ref task) = self.default_task {
                task.clone()
            } else {
                throw!(Error::TaskNotFound(name.to_string()));
            }
        } else {
            name.to_string()
        };

        // Push the task onto the call stack.
        self.stack.push_front(name.clone());

        // If the name is a task, run it first.
        if let Some(task) = self.get_task(&name) {
            let satisfied = task.borrow().is_satisfied();
            if !satisfied {
                try!(self.resolve_dependencies(&task.borrow().deps));

                if !self.dry_run {
                    try!(task.borrow_mut().run(args));
                } else {
                    println!("Would run task \"{}\".", &name);
                }
            } else {
                println!("Nothing to be done for task \"{}\".", &name);
            }
        } else if let Some(rule) = self.match_rule(&name) {
            if !rule.is_satisfied(&name) {
                try!(self.resolve_dependencies(&rule.deps));

                if !self.dry_run {
                    try!(rule.run(&name));
                } else {
                    println!("Would run rule \"{}\" for file \"{}\".",
                             &rule.pattern,
                             &name);
                }
            } else {
                println!("Nothing to be done for file \"{}\".", &name);
            }
        } else {
            throw!(Error::TaskNotFound(name));
        }

        // Pop the task off the call stack.
        self.stack.pop_front();

        Ok(())
    }

    fn resolve_dependencies(&mut self, dependencies: &Vec<String>) -> Result<(), Error> {
        for dependency in dependencies {
            try!(self.run(dependency, Vec::new()));
        }

        Ok(())
    }

    fn set_environment(&self) {
        // Set a pointer we can use to fetch the runner from within the runtime.
        self.runtime.borrow_mut().reg_set("runner", self as *const Runner as *mut usize);

        self.runtime.borrow_mut().state().push_string(if cfg!(windows) {
            "windows"
        } else {
            "unix"
        });
        self.runtime.borrow_mut().state().set_global("OS");
    }
}

/// Prints text to the console output.
///
/// # Lua arguments
/// * `str: string` - The string to print.
fn print(runtime: &mut Runtime) -> i32 {
    let runner: &mut Runner = runtime.reg_get("runner").unwrap();
    let string = runtime.state().check_string(1).to_string();
    let mut out = term::stdout().unwrap();

    for line in string.lines() {
        if !runner.stack.is_empty() {
            let name = runner.stack.front().unwrap();
            out.fg(term::color::GREEN).unwrap();
            write!(out, "{:9} ", format!("[{}]", name)).unwrap();
            out.reset().unwrap();
        }

        writeln!(out, "{}", line).unwrap();
    }

    0
}

/// Searches for paths matching a pattern.
///
/// # Lua arguments
/// * `pattern: string` - The glob pattern to match.
fn glob(runtime: &mut Runtime) -> i32 {
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

    1
}

/// Gets the current working directory.
fn current_dir(runtime: &mut Runtime) -> i32 {
    env::current_dir()
        .map(|dir| {
            runtime.state().push(dir.to_str());
            1
        })
        .unwrap_or(0)
}

/// Sets the current working directory.
fn change_dir(runtime: &mut Runtime) -> i32 {
    let path = runtime.state().check_string(1).to_string();

    if env::set_current_dir(path).is_err() {
        runtime.throw_error("failed to change directory");
    }

    0
}

/// Exports an environment variable.
///
/// # Lua arguments
/// * `key: string` - The variable name.
/// * `value: string` - The value to set.
fn export(runtime: &mut Runtime) -> i32 {
    let key = runtime.state().check_string(1).to_string();
    let value = runtime.state().check_string(2).to_string();

    env::set_var(key, value);

    0
}
