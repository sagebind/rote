use rule::Rule;
use std::cell::RefCell;
use std::clone::Clone;
use std::collections::HashMap;
use std::error::Error;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use task::{Task, NamedTask};


/// Stores the state of an entire task execution environment.
pub struct Environment {
    /// A map of all named tasks.
    tasks: RefCell<HashMap<String, Rc<NamedTask>>>,

    /// A vector of all defined file rules.
    rules: RefCell<Vec<Rc<Rule>>>,

    /// The default task to run.
    default_task: RefCell<Option<String>>,

    path: PathBuf,

    directory: PathBuf,
}

impl Environment {
    /// Creates a new environment for a given script file.
    ///
    /// The instance is placed inside a box to ensure the runner has a constant location in memory
    /// so that it can be referenced by native closures in the runtime.
    pub fn new<P: Into<PathBuf>>(script: P) -> Result<Environment, Box<Error>> {
        let script = script.into();
        let directory = match script.parent() {
            Some(path) => path.into(),
            None => {
                return Err("failed to parse script directory".into());
            }
        };

        Ok(Environment {
            tasks: RefCell::new(HashMap::new()),
            rules: RefCell::new(Vec::new()),
            default_task: RefCell::new(None),
            path: script,
            directory: directory,
        })
    }

    /// Gets the full path of the script file.
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Gets the full path of the directory containing the script file.
    pub fn directory(&self) -> &Path {
        &self.directory
    }

    /// Gets a list of all registered tasks.
    pub fn tasks(&self) -> Vec<Rc<NamedTask>> {
        self.tasks.borrow().values().map(|rc| rc.clone()).collect()
    }

    /// Gets a list of all registered rules.
    pub fn rules(&self) -> Vec<Rc<Rule>> {
        self.rules.borrow().iter().map(|rc| rc.clone()).collect()
    }

    /// Creates a new task.
    pub fn create_task(&self, task: NamedTask) {
        // Add it to the master list of tasks.
        self.tasks.borrow_mut().insert(task.name().into(), Rc::new(task));
    }

    /// Creates a new rule.
    pub fn create_rule(&self, rule: Rule) {
        self.rules.borrow_mut().push(Rc::new(rule));
    }

    /// Gets a task by name.
    pub fn get_task<S: AsRef<str>>(&self, name: S) -> Option<Rc<NamedTask>> {
        self.tasks.borrow().get(name.as_ref()).map(|rc| rc.clone())
    }

    /// Gets the default task to run.
    pub fn default_task<'a>(&'a self) -> Option<String> {
        match *self.default_task.borrow() {
            Some(ref task) => Some(task.clone()),
            None => None,
        }
    }

    /// Sets the default task.
    pub fn set_default_task<S: Into<String>>(&self, name: S) {
        *self.default_task.borrow_mut() = Some(name.into());
    }
}
