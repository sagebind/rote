use graph::Graph;
use num_cpus;
use script::Environment;
use script::task::Task;
use std::cmp;
use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::rc::Rc;
use std::sync::mpsc::channel;
use std::thread;
use stdlib;


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
    jobs: usize,
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
            jobs: jobs as usize,
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
    pub fn jobs(&mut self, jobs: usize) {
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
        let mut schedule = try!(self.graph.solve(!self.always_run));
        let task_count = schedule.len();
        let thread_count = cmp::min(self.jobs, task_count);

        debug!("running {} task(s) across {} thread(s)", task_count, thread_count);

        // Spawn one thread for each job.
        let mut threads = Vec::new();
        let mut free_threads: HashSet<usize> = HashSet::new();
        let mut channels = Vec::new();
        let (sender, receiver) = channel::<usize>();

        // Spawn `jobs` number of threads (but no more than the task count!).
        for thread_id in 0..thread_count {
            let path = self.environment.path().to_path_buf();
            let dry_run = self.dry_run;
            let thread_sender = sender.clone();

            let (parent_sender, thread_receiver) = channel::<(String, usize)>();
            channels.push(parent_sender);

            free_threads.insert(thread_id);
            threads.push(thread::spawn(move || {
                // Prepare a new environment.
                let environment = Environment::new(path).unwrap_or_else(|e| {
                    error!("{}", e);
                    panic!();
                });

                // Open standard library functions.
                stdlib::open_lib(environment.clone());

                // Load the script.
                if let Err(e) = environment.load() {
                    error!("{}", e);
                    panic!();
                }

                if thread_sender.send(thread_id).is_err() {
                    trace!("thread {} failed to send channel", thread_id);
                }

                // Begin executing tasks!
                while let Ok((name, task_id)) = thread_receiver.recv() {
                    println!("[{}/{}] {}", task_id, task_count, name);

                    // Lookup the task to run.
                    let task = {
                        // Lookup the task to run.
                        if let Some(task) = environment.get_task(&name) {
                            task as Rc<Task>
                        }

                        // Find a rule that matches the task name.
                        else if let Some(rule) = environment.rules().iter().find(|rule| rule.matches(&name)) {
                            Rc::new(rule.create_task(name).unwrap()) as Rc<Task>
                        }

                        // No matching task.
                        else {
                            panic!("no matching task or rule for '{}'", name);
                        }
                    };

                    // Check for dry run.
                    if !dry_run {
                        if let Err(e) = task.run() {
                            error!("{}", e);
                            panic!();
                        }
                    } else {
                        info!("would run task '{}'", task.name());
                    }

                    if thread_sender.send(thread_id).is_err() {
                        trace!("thread {} failed to send channel", thread_id);
                        break;
                    }
                }
            }))
        }

        drop(sender);

        // Keep track of tasks completed and tasks in progress.
        let mut completed_tasks: HashSet<String> = HashSet::new();
        let mut current_tasks: HashMap<usize, String> = HashMap::new();
        let all_tasks: HashSet<String> = schedule.iter().map(|s| s.name().to_string()).collect();

        while !schedule.is_empty() {
            // Wait for a thread to request a task.
            let thread_id = receiver.recv().unwrap();
            free_threads.insert(thread_id);
            trace!("thread {} is idle", thread_id);

            // If the thread was previously running a task, mark it as completed.
            if let Some(task) = current_tasks.remove(&thread_id) {
                trace!("task {} completed", task);
                completed_tasks.insert(task);
            }

            // Attempt to schedule more tasks to run. The most we can schedule is the number of free
            // threads, but it is limited by the number of tasks that have their dependencies already
            // finished.
            'schedule: for _ in 0..free_threads.len() {
                // If the schedule is empty, we are done.
                if schedule.is_empty() {
                    break;
                }

                // Check the next task in the queue. If any of its dependencies have not yet been
                // completed, we cannot schedule it yet.
                for dependency in schedule.front().unwrap().dependencies() {
                    // Check that the dependency needs scheduled at all (some are already satisfied),
                    // and that it hasn't already finished.
                    if all_tasks.contains(dependency) && !completed_tasks.contains(dependency) {
                        // We can't run the next task, so we're done scheduling for now until another
                        // thread finishes.
                        break 'schedule;
                    }
                }

                // Pop the available task from the queue.
                let task = schedule.pop_front().unwrap();

                // Pick a free thread to run the task in.
                if let Some(thread_id) = free_threads.iter().next().map(|t| *t) {
                    trace!("scheduling task '{}' on thread {}", task.name(), thread_id);
                    let data = (task.name().to_string(), task_count - schedule.len());

                    // Send the task name.
                    if channels[thread_id].send(data).is_ok() {
                        current_tasks.insert(thread_id, task.name().to_string());
                        free_threads.remove(&thread_id);
                    } else {
                        trace!("failed to send channel to thread {}", thread_id);
                    }
                } else {
                    // We can schedule now, but there aren't any free threads. 😢
                    break;
                }
            }
        }

        // Close the input and wait for any remaining threads to finish.
        drop(channels);
        for (thread_id, thread) in threads.into_iter().enumerate() {
            if let Err(e) = thread.join() {
                trace!("thread {} closed with panic: {:?}", thread_id, e);
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
