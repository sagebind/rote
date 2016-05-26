use rule::Rule;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use task::NamedTask;


/// Stores the state of an entire task execution environment.
pub struct Environment {
    /// A map of all named tasks.
    pub tasks: HashMap<String, Rc<NamedTask>>,

    /// A vector of all defined file rules.
    pub rules: Vec<Rc<Rule>>,

    /// The default task to run.
    pub default_task: Option<String>,

    script: PathBuf,

    directory: PathBuf,

    /// Indicates if actually running tasks should be skipped.
    dry_run: bool,
}

impl Environment {
    pub fn new<P: Into<PathBuf>>(script: P, dry_run: bool) -> Environment {
        let script = script.into();
        let directory = script.parent().expect("failed to parse script directory").into();

        Environment {
            tasks: HashMap::new(),
            rules: Vec::new(),
            default_task: None,
            script: script,
            directory: directory,
            dry_run: dry_run,
        }
    }

    pub fn path(&self) -> &Path {
        &self.script
    }
}
