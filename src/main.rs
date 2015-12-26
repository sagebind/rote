extern crate filetime;
extern crate getopts;
extern crate glob;
extern crate lazysort;
extern crate lua;
extern crate term;

use getopts::Options;
use lazysort::SortedBy;
use std::env;
use std::fs;
use std::path;
use std::process;

mod error;
mod modules;
mod runner;
mod runtime;


/// Prints the program usage to the console.
fn print_usage(options: Options) {
    let brief = "Rote multilingual build tool\r\n\r\nUsage: rote [options] [task] [args]";
    print!("{}", options.usage(brief));
}

fn print_task_list(runner: &runner::Runner) {
    let mut out = term::stdout().unwrap();

    println!("Available tasks:");

    for task in runner.tasks.iter().sorted_by(|a, b| {
        a.0.cmp(b.0)
    }) {
        out.fg(term::color::GREEN).unwrap();
        write!(out, "  {:16}", task.0).unwrap();
        out.reset().unwrap();

        if let Some(ref description) = task.1.borrow().description {
            write!(out, "{}", description).unwrap();
        }

        writeln!(out, "").unwrap();
    }

    if let Some(ref default) = runner.default_task() {
        println!("");
        println!("Default task: {}", default.borrow().name);
    }
}

/// Parses command-line options and runs retest.
fn main() {
    let args: Vec<String> = env::args().collect();

    // Parse command-line flags.
    let mut options = Options::new();
    options.optflag("d", "dry-run", "Don't actually perform any action.");
    options.optflag("h", "help", "Print this help menu and exit.");
    options.optflag("l", "list", "List available tasks.");
    options.optflag("v", "version", "Print the program version and exit.");
    options.optopt("C", "directory", "Change to DIRECTORY before running tasks.", "DIRECTORY");
    options.optopt("f", "file", "Read FILE as the Rotefile.", "FILE");

    let opt_matches = match options.parse(&args[1..]) {
        Ok(matches) => { matches }
        Err(err) => {
            println!("error: {}", err);
            process::exit(1);
        }
    };

    // If the help flag is present show the usage message.
    if opt_matches.opt_present("h") {
        print_usage(options);
        return;
    }

    // Get the file name of the Rotefile if given.
    let filename = opt_matches.opt_str("f").unwrap_or("Rotefile".to_string());
    let path = fs::canonicalize(path::Path::new(&filename))
        .ok()
        .and_then(|path_buf| path_buf.to_str()
            .map(|path| path.to_string())
        )
        .unwrap_or_else(|| {
            println!("error: the path {} is not a file or is not readable", filename);
            process::exit(1);
        });

    // If the directory flag is present, change directories first.
    if let Some(directory) = opt_matches.opt_str("C") {
        if env::set_current_dir(&directory).is_err() {
            println!("error: failed to change directory to '{}'", &directory);
            process::exit(1);
        }
    }

    // Get all of the task arguments.
    let mut args = opt_matches.free.clone();

    // Get the name of the task to run.
    let task_name = if args.is_empty() {
        "default".to_string()
    } else {
        args.remove(0)
    };

    println!("Build file: {}\r\n", &path);

    // Create a new script runtime.
    let mut runner = runner::Runner::new().unwrap();
    if let Err(e) = runner.load(&path) {
        e.die();
    }

    // List all tasks instead of running one.
    if opt_matches.opt_present("l") {
        print_task_list(&runner);
        return;
    }

    // Run the specified task.
    if let Err(e) = runner.run(&task_name, args) {
        e.die();
    }
}
