extern crate getopts;
extern crate glob;
extern crate hyper;
extern crate json;
#[macro_use]
extern crate log;
extern crate lua;
extern crate num_cpus;
extern crate regex;
extern crate term;

use getopts::Options;
use runner::Runner;
use std::env;
use std::path;
use std::process;

mod graph;
mod logger;
mod modules;
mod rule;
mod runner;
mod runtime;
mod task;


const ROTE_VERSION: &'static str = env!("CARGO_PKG_VERSION");


/// Prints the program usage to the console.
fn print_usage(options: Options) {
    let short_usage = options.short_usage("rote");

    println!("
   ▄████████  ▄██████▄      ███        ▄████████
  ███    ███ ███    ███ ▀█████████▄   ███    ███
  ███    ███ ███    ███    ▀███▀▀██   ███    █▀
 ▄███▄▄▄▄██▀ ███    ███     ███   ▀  ▄███▄▄▄
▀▀███▀▀▀▀▀   ███    ███     ███     ▀▀███▀▀▀
▀███████████ ███    ███     ███       ███    █▄
  ███    ███ ███    ███     ███       ███    ███
  ███    ███  ▀██████▀     ▄████▀     ██████████
  ███    ███

{}
Report issues at <https://github.com/sagebind/rote/issues>.
Rote home page: <https://github.com/sagebind/rote>"
    , options.usage(&short_usage));
}

/// Parses command-line options and runs retest.
fn main() {
    let args: Vec<String> = env::args().collect();

    // Parse command-line flags.
    let mut options = Options::new();
    options.optflag("B", "run-all", "Unconditionally run all tasks, including those up-to-date.");
    options.optopt("C", "directory", "Change to DIRECTORY before running tasks.", "DIRECTORY");
    options.optmulti("D", "var", "Override a variable value.", "NAME=VALUE");
    options.optopt("f", "file", "Read FILE as the Rotefile.", "FILE");
    options.optflag("h", "help", "Print this help message and exit.");
    options.optmulti("I", "include-path", "Include PATH in the search path for modules.", "PATH");
    options.optopt("j", "jobs", "The number of jobs to run simultaneously.", "N");
    options.optflag("k", "keep-going", "Keep going if some tasks fail.");
    options.optflag("l", "list", "List all tasks and exit.");
    options.optflag("n", "dry-run", "Simulate running tasks without executing them.");
    options.optflag("q", "quiet", "Supress all non-task output.");
    options.optflagmulti("v", "verbose", "Enable verbose logging.");
    options.optflag("V", "version", "Print the program version and exit.");

    let matches = options.parse(&args[1..]).unwrap_or_else(|err| {
        logger::init(logger::Filter::Error).unwrap();
        error!("{}", err);
        process::exit(2);
    });

    // Set the logging verbosity level.
    logger::init(if matches.opt_present("quiet") {
        logger::Filter::Error
    } else {
        match matches.opt_count("verbose") {
            0 => logger::Filter::Info,
            1 => logger::Filter::Debug,
            _ => logger::Filter::Trace,
        }
    }).unwrap();

    // Notify the user if higher vebosity has been achieved.
    debug!("debug messages turned on");
    trace!("trace messages turned on");

    // If the help flag is present show the usage message.
    if matches.opt_present("help") {
        print_usage(options);
        return;
    }

    // If the version flag is present, show the program version.
    if matches.opt_present("version") {
        println!("Rote {}", ROTE_VERSION);
        return;
    }

    // If the directory flag is present, change directories first.
    if let Some(directory) = matches.opt_str("directory") {
        if env::set_current_dir(&directory).is_err() {
            error!("failed to change directory to '{}'", &directory);
            process::exit(1);
        }
    }

    // Get the full path of the Rotefile to run.
    let filename = matches.opt_str("file").unwrap_or("Rotefile".to_string());
    let path = path::Path::new(&filename)
        .canonicalize()
        .unwrap_or_else(|_| {
            error!("the path '{}' is not a file or is not readable", filename);
            process::exit(1);
        });

    // Create a new task runner.
    let mut runner = Runner::new(path).unwrap_or_else(|e| {
        error!("{}", e);
        process::exit(1);
    });

    info!("build file: {}", runner.path().to_string_lossy());

    // Set the new current directory to the directory containing the Rotefile.
    if env::set_current_dir(runner.directory()).is_err() {
        error!("failed to change directory to '{}'", runner.directory().to_string_lossy());
        process::exit(1);
    }

    // Set project-local and global include path.
    runner.include_path("./rote");
    runner.include_path("/usr/lib/rote/plugins");

    // User-specified paths.
    for value in matches.opt_strs("include-path") {
        runner.include_path(value);
    }

    // Set environment variables.
    for value in matches.opt_strs("var") {
        let parts: Vec<_> = value.split('=').collect();

        if parts.len() != 2 {
            warn!("invalid variable syntax: '{}'", value);
        } else {
            runner.set_var(parts[0], parts[1]);
        }
    }

    // Toggle dry run.
    if matches.opt_present("dry-run") {
        info!("dry run is enabled; no task actions will be run");
        runner.dry_run();
    }

    // Toggle always run.
    if matches.opt_present("run-all") {
        info!("running all tasks unconditionally");
        runner.always_run();
    }

    // Toggle keep going.
    if matches.opt_present("keep-going") {
        info!("errors will be ignored");
        runner.keep_going();
    }

    // Set number of jobs.
    if let Some(jobs) = matches.opt_str("jobs") {
        if let Ok(jobs) = jobs.parse::<usize>() {
            runner.jobs(jobs);
        } else {
            warn!("invalid number of jobs");
        }
    }

    // Load the script.
    if let Err(e) = runner.load() {
        error!("{}", e);
        process::exit(1);
    }

    // List all tasks instead of running one.
    if matches.opt_present("list") {
        runner.print_task_list();
        return;
    }

    // Get all of the tasks to run.
    let tasks = matches.free;

    // Run the specified task, or the default if none is specified.
    if let Err(e) = {
        if tasks.is_empty() {
            runner.run_default()
        } else {
            // Run the specified tasks.
            runner.run(&tasks)
        }
    } {
        error!("{}", e);
        process::exit(1);
    }
}
