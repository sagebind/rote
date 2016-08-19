use std::cmp::Ordering;
use std::error::Error;
use std::hash::{Hash, Hasher};


/// A single task that can be run.
///
/// A task represents a single unit of work. Tasks are created after all rules and named tasks are
/// already defined.
pub trait Task {
    /// Gets the synthesized name of the task.
    fn name<'a>(&'a self) -> &'a str;

    /// Checks if the task is satisfied.
    ///
    /// A task is not satisfied when its conditions are not met and its action must be run to
    /// create the desired output and move to the satisfied state. Task implementations should make
    /// sure that this always returns `true` after the action is run.
    fn satisfied(&self) -> bool;

    /// Gets an array of task names that this task depends on.
    fn dependencies(&self) -> &[String];

    /// Runs the task.
    fn run(&self) -> Result<(), Box<Error>>;
}

/// A single named build task.
pub struct NamedTask {
    /// The name of the task.
    pub name: String,

    /// The task description.
    pub description: Option<String>,

    /// A list of tasks that must be ran before this task.
    pub dependencies: Vec<String>,

    /// Rule action.
    action: Option<Box<Fn() -> Result<(), Box<Error>>>>,
}

impl NamedTask {
    pub fn new<S: Into<String>>(name: S, description: Option<S>, dependencies: Vec<String>, action: Option<Box<Fn() -> Result<(), Box<Error>>>>) -> NamedTask {
        NamedTask {
            name: name.into(),
            description: description.map(|s| s.into()),
            dependencies: dependencies,
            action: action,
        }
    }

    pub fn description<'a>(&'a self) -> Option<&'a str> {
        match self.description {
            Some(ref description) => Some(description),
            None => None,
        }
    }
}

impl Task for NamedTask {
    fn name<'a>(&'a self) -> &'a str {
        &self.name
    }

    // Named tasks should always be run.
    fn satisfied(&self) -> bool {
        false
    }

    fn dependencies(&self) -> &[String] {
        &self.dependencies
    }

    fn run(&self) -> Result<(), Box<Error>> {
        if let Some(ref action) = self.action {
            action()
        } else {
            Ok(())
        }
    }
}

// Implement ordering and comparison for all task types.
impl Eq for Task {}

impl PartialEq for Task {
    fn eq(&self, other: &Task) -> bool {
        self.name() == other.name()
    }
}

impl Ord for Task {
    fn cmp(&self, other: &Task) -> Ordering {
        self.name().cmp(other.name())
    }
}

impl PartialOrd for Task {
    fn partial_cmp(&self, other: &Task) -> Option<Ordering> {
        self.name().partial_cmp(other.name())
    }
}

impl Hash for Task {
    fn hash<H>(&self, state: &mut H) where H: Hasher {
        self.name().hash(state)
    }
}
