use graph::Graph;
use num_cpus;
use script::Environment;
use script::task::Task;
use std::cmp;
use std::error::Error;
use std::rc::Rc;


/// A task runner object that holds the state for defined tasks, dependencies, and the scripting
/// runtime.
pub struct Runner {
    /// The script environment.
    environment: Environment,

    /// The current DAG for tasks.
    graph: Graph,

    /// Indicates if actually running tasks should be skipped.
    dry_run: bool,

    /// Indicates if up-to-date tasks should be run anyway.
    always_run: bool,

    /// The number of threads to use.
    jobs: u32,
}

impl Runner {
    /// Creates a new runner instance.
    pub fn new(environment: Environment) -> Runner {
        // By default, set the number of jobs to be one less than the number of available CPU cores.
        let jobs = cmp::max(1, num_cpus::get() - 1);

        Runner {
            environment: environment,
            graph: Graph::new(),
            dry_run: false,
            always_run: false,
            jobs: jobs as u32,
        }
    }

    /// Sets the runner to "dry run" mode.
    ///
    /// When in "dry run" mode, running tasks will operate as normal, except that no task's actions
    /// will be actually run.
    pub fn dry_run(&mut self) {
        self.dry_run = true;
    }

    /// Run all tasks even if they are up-to-date.
    pub fn always_run(&mut self) {
        self.always_run = true;
    }

    /// Sets the number of threads to use to run tasks.
    pub fn jobs(&mut self, jobs: u32) {
        self.jobs = jobs;
    }

    /// Run the default task.
    pub fn run_default(&mut self) -> Result<(), Box<Error>> {
        if let Some(ref name) = self.environment.default_task() {
            let tasks = vec![name];
            self.run(&tasks)
        } else {
            Err("no default task defined".into())
        }
    }

    /// Runs the specified list of tasks.
    ///
    /// Tasks are run in parallel when possible during execution. The maximum number of parallel
    /// jobs can be set with the `jobs()` method.
    pub fn run<S: AsRef<str>>(&mut self, tasks: &[S]) -> Result<(), Box<Error>> {
        // Resolve all tasks given.
        for task in tasks {
            try!(self.resolve_task(task));
        }

        // Determine the schedule of tasks to execute.
        let schedule = try!(self.graph.solve(!self.always_run));
        debug!("need to run {} task(s)", schedule.len());

        for (i, task) in schedule.iter().enumerate() {
            println!("[{}/{}] {}", i + 1, schedule.len(), task.name());

            // Check for dry run.
            if !self.dry_run {
                try!(task.run());
            } else {
                info!("would run task '{}'", task.name());
            }
        }

        Ok(())
    }

    fn resolve_task<S: AsRef<str>>(&mut self, name: S) -> Result<(), Box<Error>> {
        if !self.graph.contains(&name) {
            // Lookup the task to run.
            if let Some(task) = self.environment.get_task(&name) {
                debug!("task '{}' matches named task", name.as_ref());
                self.graph.insert(task.clone());
            }

            // Find a rule that matches the task name.
            else if let Some(rule) = self.environment.rules().iter().find(|rule| rule.matches(&name)) {
                debug!("task '{}' matches rule '{}'", name.as_ref(), rule.pattern);
                // Create a task for the rule and insert it in the graph.
                self.graph.insert(Rc::new(rule.create_task(name.as_ref()).unwrap()));
            }

            // No matching task.
            else {
                return Err(format!("no matching task or rule for '{}'", name.as_ref()).into());
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
