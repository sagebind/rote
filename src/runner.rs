use error::{Error, RoteError};
use lua;
use runtime;
use functions;
use std::cell::RefCell;
use std::collections::{HashMap, LinkedList};
use std::rc::{Rc, Weak};
use std::sync;


/// A single build task.
pub struct Task {
    /// The name of the task.
    pub name: String,

    /// A list of task names that must be ran before this task.
    pub deps: Vec<String>,

    /// A reference to this task's callback function.
    pub func: lua::Reference,
}

pub struct Runner {
    /// A map of all defined tasks.
    pub tasks: HashMap<String, Rc<RefCell<Task>>>,

    /// The name of the default task to run.
    pub default_task: Option<String>,

    /// Task execution stack.
    pub stack: LinkedList<Weak<RefCell<Task>>>,

    runtime: sync::Mutex<Box<runtime::Runtime>>,
}

impl Runner {
    pub fn new() -> Result<Box<Runner>, Error> {
        let runner = Box::new(Runner {
            tasks: HashMap::new(),
            default_task: None,
            stack: LinkedList::new(),
            runtime: sync::Mutex::new(try!(runtime::Runtime::new())),
        });

        {
            let mut runtime = runner.runtime.lock().unwrap();

            // Register core functions.
            let ptr = &*runner as *const Runner as usize;
            runtime.register_fn("task", functions::task, Some(ptr));
            runtime.register_fn("default", functions::default, Some(ptr));
            runtime.register_fn("print", functions::print, Some(ptr));
            runtime.register_fn("glob", functions::glob, None);
            runtime.register_fn("export", functions::export, None);

            // Load the core Lua module.
            try!(runtime.eval("require 'core'"));
        }

        Ok(runner)
    }

    /// Creates a new task.
    pub fn create_task(&mut self, name: String, deps: Vec<String>, func: lua::Reference) {
        // Create a task object.
        let task = Task {
            name: name,
            deps: deps,
            func: func,
        };

        // Add it to the master list of tasks.
        self.tasks.insert(task.name.clone(), Rc::new(RefCell::new(task)));
    }

    pub fn load(&self, filename: &str) -> Result<(), Error> {
        self.runtime.lock().unwrap().load(filename)
    }

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

        // Run all dependencies first.
        for dep_name in &task.borrow().deps {
            try!(self.run(&dep_name, Vec::new()));
        }

        // Call the task itself.
        // Push the task function onto the Lua stack.
        self.runtime.lock().unwrap().state().raw_geti(lua::REGISTRYINDEX, task.borrow().func.value() as i64);

        // Push the given task arguments onto the stack.
        for string in &args {
            self.runtime.lock().unwrap().state().push_string(&string);
        }

        // Invoke the task function.
        if self.runtime.lock().unwrap().state().pcall(args.len() as i32, 0, 0).is_err() {
            return Err(self.runtime.lock().unwrap().get_last_error().unwrap());
        }
        self.runtime.lock().unwrap().state().pop(1);

        // Pop the task off the call stack.
        self.stack.pop_front();

        Ok(())
    }

    pub fn close(self) {
        self.runtime.into_inner().unwrap().close();
    }
}
