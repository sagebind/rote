use graph::Graph;
use script::Environment;
use script::rule::Rule;
use script::task::{Task, NamedTask};
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
    /// The script environment.
    environment: Environment,

    graph: Graph,
}

impl Runner {
    /// Creates a new runner instance.
    pub fn new(environment: Environment) -> Runner {
        Runner {
            environment: environment,
            graph: Graph::new(),
        }
    }

    pub fn run_default(&mut self) -> Result<(), Box<Error>> {
        if let Some(ref name) = self.environment.default_task() {
            self.run(name)
        } else {
            Err("no default task defined".into())
        }
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
            if let Some(task) = self.environment.get_task(name) {
                debug!("task '{}' matches named task", name);
                self.graph.insert(task.clone());
            }

            // Find a rule that matches the task name.
            else if let Some(rule) = self.environment.rules().iter().find(|rule| rule.matches(name)) {
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
