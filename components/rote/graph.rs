use script::task::Task;
use std::collections::{HashMap, HashSet, VecDeque};
use std::error::Error;
use std::rc::Rc;


/// Stores a directional, sparse graph of tasks and their dependencies.
pub struct Graph {
    tasks: HashMap<String, Rc<Task>>,
}

impl Graph {
    /// Creates a new graph.
    pub fn new() -> Graph {
        Graph {
            tasks: HashMap::new(),
        }
    }

    pub fn contains<S: AsRef<str>>(&self, name: S) -> bool {
        self.tasks.contains_key(name.as_ref())
    }

    /// Gets a task by name.
    pub fn get<S: AsRef<str>>(&self, name: S) -> Result<Rc<Task>, Box<Error>> {
        if let Some(task) = self.tasks.get(name.as_ref()) {
            Ok(task.clone())
        } else {
            Err(format!("task '{}' not found", name.as_ref()).into())
        }
    }

    /// Adds a task to the graph.
    pub fn insert(&mut self, rule: Rc<Task>) {
        self.tasks.insert(rule.name().into(), rule);
    }

    /// Produces a queue of tasks to run in order to satisfy all task dependencies.
    ///
    /// Dependency solving is done by performing a topological sort of the entire graph using a
    /// depth-first search-based algorithm.
    pub fn solve(&self, skip_satisfied_tasks: bool) -> Result<VecDeque<Rc<Task>>, Box<Error>> {
        Solver::new(&self, skip_satisfied_tasks).solve()
    }
}

struct Solver<'a> {
    graph: &'a Graph,
    // Set of tasks that have already been resolved.
    resolved: HashSet<Rc<Task>>,
    // Set of tasks that have been visited but not resolved.
    unresolved: HashSet<Rc<Task>>,
    // Resulting queue of tasks in solved order.
    schedule: VecDeque<Rc<Task>>,
    // Skip satisfied tasks?
    skip_satisfied_tasks: bool,
}

impl<'a> Solver<'a> {
    fn new<'b>(graph: &'b Graph, skip_satisfied_tasks: bool) -> Solver<'b> {
        Solver {
            graph: graph,
            resolved: HashSet::new(),
            unresolved: HashSet::new(),
            schedule: VecDeque::new(),
            skip_satisfied_tasks: skip_satisfied_tasks,
        }
    }

    fn solve(mut self) -> Result<VecDeque<Rc<Task>>, Box<Error>> {
        // Loop over each task in the graph.
        for task in self.graph.tasks.values() {
            // If this task has not already been visited, search its dependencies to verify that it
            // can be satisfied.
            if !self.resolved.contains(task) {
                try!(self.resolve(task.clone()));
            }
        }

        Ok(self.schedule)
    }

    fn resolve(&mut self, task: Rc<Task>) -> Result<(), Box<Error>> {
        // First, check if the task is already satisfied. If it is, it and its dependencies do not
        // need to run and we can skip this task in the schedule.
        if self.skip_satisfied_tasks && try!(self.satisfied(task.clone())) {
            info!("task '{}' is up to date", task.name());
            self.resolved.insert(task.clone());
            return Ok(());
        }

        // Mark this task as unresolved.
        self.unresolved.insert(task.clone());

        // Resolve each dependency.
        for dependency in task.dependencies() {
            trace!("task '{}' depends on '{}'", task.name(), dependency);

            // Lookup the dependency in the graph.
            let dependency = try!(self.graph.get(dependency));

            if !self.resolved.contains(&dependency) {
                if self.unresolved.contains(&dependency) {
                    return Err(format!("circular dependency detected: {} -> {}", task.name(), dependency.name()).into());
                }

                try!(self.resolve(dependency.clone()));
            }
        }

        // The task is now resolved.
        trace!("task '{}' resolved", task.name());
        self.unresolved.remove(&task);
        self.resolved.insert(task.clone());
        self.schedule.push_back(task.clone());

        Ok(())
    }

    /// Determines recursively if a task is satisfied. For a task to be satisfied, its dependencies
    /// must also be satisfied.
    fn satisfied(&self, task: Rc<Task>) -> Result<bool, Box<Error>> {
        if !task.satisfied() {
            return Ok(false);
        }

        for dependency in task.dependencies() {
            let dependency = try!(self.graph.get(dependency));

            if !try!(self.satisfied(dependency)) {
                return Ok(false);
            }
        }

        Ok(true)
    }
}
