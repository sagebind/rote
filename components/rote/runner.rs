use graph::Graph;
use runtime::Runtime;
use runtime::task::{Task, NamedTask};
use runtime::rule::Rule;
use std::borrow::Cow;
use std::collections::{HashMap, LinkedList};
use std::env;
use std::error::Error;
use std::path::Path;
use std::rc::Rc;
use stdlib;


/// A task runner object that holds the state for defined tasks, dependencies, and the scripting
/// runtime.
pub struct Runner {
    /// A map of all defined tasks.
    pub tasks: HashMap<String, Rc<NamedTask>>,

    /// A vector of all defined file rules.
    pub rules: Vec<Rc<Rule>>,

    /// The name of the default task to run.
    pub default_task: Option<String>,

    /// Task execution stack.
    pub stack: LinkedList<String>,

    graph: Graph,

    /// The scripting runtime.
    runtime: Runtime,
}

impl Runner {
    /// Creates a new runner instance.
    ///
    /// The instance is placed inside a box to ensure the runner has a constant location in memory
    /// so that it can be referenced by native closures in the runtime.
    pub fn new() -> Result<Box<Runner>, Box<Error>> {
        let mut runner = Box::new(Runner {
            tasks: HashMap::new(),
            rules: Vec::new(),
            default_task: None,
            stack: LinkedList::new(),
            graph: Graph::new(),
            runtime: Runtime::new(),
        });

        // Initialize the standard Lua libraries.
        runner.runtime.state().open_libs();

        // Set up the environment.
        runner.runtime.add_path("./components/?.lua");
        runner.runtime.add_cpath("/usr/lib/rote/plugins/?.so");

        if let Some(mut path) = env::current_exe().ok()
            .and_then(|path| path.parent()
                .map(|p| p.to_path_buf())
            ) {
            path.push("lib?.so");
            runner.runtime.add_cpath(path.to_str().unwrap());
        }

        // Set a pointer we can use to fetch the runner from within the runtime.
        unsafe {
            runner.runtime.clone().state().push_light_userdata(&*runner as *const Runner as *mut Runner);
        }
        runner.runtime.clone().reg_set("rote.runner");

        runner.runtime.state().push_string(if cfg!(windows) {
            "windows"
        } else {
            "unix"
        });
        runner.runtime.state().set_global("OS");

        trace!("opening standard module...");
        stdlib::open_lib(runner.runtime.clone());

        Ok(runner)
    }

    /// Gets a reference to the runner that belongs to a given runtime.
    pub fn from_runtime<'a>(runtime: &mut Runtime) -> Option<&'a mut Self> {
        trace!("fetching pointer to runner from registry");
        runtime.reg_get("rote.runner");

        if !runtime.state().is_userdata(-1) {
            return None;
        }

        unsafe {
            let pointer = runtime.state().to_userdata(-1);
            runtime.state().pop(1);
            Some(&mut *(pointer as *mut Runner))
        }
    }

    /// Creates a new task.
    pub fn add_task(&mut self, task: Rc<NamedTask>) {
        // Add it to the master list of tasks.
        self.tasks.insert(task.name().into(), task.clone());
    }

    /// Creates a new rule.
    pub fn add_rule(&mut self, rule: Rc<Rule>) {
        self.rules.push(rule);
    }

    /// Gets the default task to run, if any.
    pub fn default_task(&self) -> Option<Rc<NamedTask>> {
        self.default_task
            .as_ref()
            .and_then(|name| self.tasks.get(name))
            .map(|rc| rc.clone())
        // ^ Look at that snazzy functional code.
    }

    /// Loads a build file script.
    pub fn load(&mut self, path: &Path) -> Result<(), Box<Error>> {
        self.runtime.load(path)
    }

    pub fn run_default(&mut self) -> Result<(), Box<Error>> {
        if self.default_task.is_none() {
            return Err("no default task defined".into());
        }

        let name = self.default_task.as_ref().unwrap().clone();
        self.run(&name)
    }

    pub fn run(&mut self, name: &str) -> Result<(), Box<Error>> {
        try!(self.resolve_task(name));

        let schedule = try!(self.graph.solve());
        debug!("need to run {} task(s)", schedule.len());

        for (i, task) in schedule.iter().enumerate() {
            println!("[{}/{}] {}", i + 1, schedule.len(), task.name());
            try!(task.run());
        }

        Ok(())
    }

    fn resolve_task(&mut self, name: &str) -> Result<(), Box<Error>> {
        if !self.graph.contains(name) {
            // Lookup the task to run.
            if let Some(task) = self.tasks.get(name) {
                debug!("task '{}' matches named task", name);
                self.graph.insert(task.clone());
            }

            // Find a rule that matches the task name.
            else if let Some(rule) = self.rules.iter().find(|rule| rule.matches(name)) {
                debug!("task '{}' matches rule '{}'", name, rule.pattern);
                // Create a task for the rule and insert it in the graph.
                self.graph.insert(Rc::new(rule.create_task(name).unwrap()));
            }

            // No matching task.
            else {
                return Err(format!("no matching task or rule for '{}'", name).into());
            }
        }

        for dependency in self.graph.get(name).unwrap().dependencies() {
            if !self.graph.contains(dependency) {
                try!(self.resolve_task(dependency));
            }
        }

        Ok(())
    }
}
