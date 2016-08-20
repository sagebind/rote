use filetime::FileTime;
use std::error::Error;
use std::fs;
use std::rc::Rc;
use task;


type ActionFn = Fn(&str) -> Result<(), Box<Error>>;

/// A rule task that matches against files. Rules are used to generate tasks from file name
/// patterns.
pub struct Rule {
    /// The file pattern to match.
    pub pattern: String,

    /// A list of tasks that must be ran before this task.
    dependencies: Vec<String>,

    /// Rule action.
    action: Option<Rc<ActionFn>>,
}

impl Rule {
    pub fn new<S, V, F>(pattern: S, dependencies: V, action: Option<F>) -> Rule
        where S: Into<String>,
              V: Into<Vec<String>>,
              F: Fn(&str) -> Result<(), Box<Error>> + 'static,
    {
        Rule {
            pattern: pattern.into(),
            dependencies: dependencies.into(),
            action: action.map(|a| Rc::new(a) as Rc<ActionFn>),
        }
    }

    /// Checks if a file name matches the rule.
    pub fn matches<S: AsRef<str>>(&self, name: S) -> bool {
        if let Some(index) = self.pattern.find("%") {
            let (prefix, suffix) = self.pattern.split_at(index);
            let suffix = &suffix[1..];

            name.as_ref().starts_with(prefix) && name.as_ref().ends_with(suffix)
        } else {
            &self.pattern == name.as_ref()
        }
    }

    /// Creates a task for a given file based on the rule.
    pub fn create_task<S: Into<String>>(&self, name: S) -> Option<FileTask> {
        let name = name.into();

        // First, check if the given filename matches.
        if !self.matches(&name) {
            return None;
        }

        // Clone the input files (dependencies).
        let mut inputs = self.dependencies.clone();

        // If the rule name is a pattern, determine the value of the replacement character "%".
        if let Some(index) = self.pattern.find("%") {
            let end = index + 1 + name.len() - self.pattern.len();
            let replacement = &name[index..end];

            // Expand the inputs with the corresponding names that match the output name.
            inputs = inputs.into_iter().map(|input| {
                input.replace("%", replacement)
            }).collect();
        }

        Some(FileTask {
            inputs: inputs,
            output: name,
            action: self.action.clone(),
        })
    }
}

pub struct FileTask {
    pub inputs: Vec<String>,
    pub output: String,
    action: Option<Rc<ActionFn>>,
}

impl task::Task for FileTask {
    fn name<'a>(&'a self) -> &'a str {
        &self.output
    }

    /// Checks if the task is dirty by comparing the file modification time of the input and output
    /// files. If any of the input files are newer than the output file, then the task is dirty.
    fn satisfied(&self) -> bool {
        get_file_time(&self.output)
            .map(|time| {
                self.inputs
                    .iter()
                    .all(|input| get_file_time(input).map(|t| t <= time).unwrap_or(true))
            })
            .unwrap_or(false)
    }

    fn dependencies(&self) -> &[String] {
        &self.inputs
    }

    fn run(&self) -> Result<(), Box<Error>> {
        if let Some(ref action) = self.action {
            action(&self.output)
        } else {
            Ok(())
        }
    }
}

/// Gets the modified time of a file, if it exists.
fn get_file_time(file_name: &str) -> Option<FileTime> {
    fs::metadata(file_name)
        .map(|metadata| FileTime::from_last_modification_time(&metadata))
        .ok()
}
