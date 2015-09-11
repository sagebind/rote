extern crate getopts;
extern crate lua;
extern crate term;
mod error;
mod project;

use getopts::Options;
use std::env;
use std::process;

/// Prints the program usage to the console.
fn print_usage(options: Options) {
    let brief = "Usage: rote [options] [task] [args]";
    print!("{}", options.usage(brief));
}

/// Parses command-line options and runs retest.
fn main() {
    let args: Vec<String> = env::args().collect();

    let mut options = Options::new();
    options.optflag("h", "help", "Print this help menu and exit.");
    options.optflag("v", "version", "Print the program version and exit.");
    options.optopt("f", "file", "Specify a Rotefile to read.", "FILE");

    let opt_matches = match options.parse(&args[1..]) {
        Ok(matches) => { matches }
        Err(err) => {
            println!("rote: error: {}", err);
            process::exit(1);
        }
    };

    // If the help flag is present or the user forgot to specify a pattern, show
    // the usage message.
    if opt_matches.opt_present("h") {
        print_usage(options);
        return;
    }

    // Get the file name of the Rotefile if given.
    let filename = opt_matches.opt_str("f").unwrap_or("Rotefile".to_string());

    // Get all of the task arguments.
    let mut args = opt_matches.free.clone();

    // Get the name of the task to run.
    let task_name = if args.is_empty() {
        "default".to_string()
    } else {
        args.remove(0)
    };

    let mut project = project::Project::new();
    if let Err(e) = project.load(&filename) {
        e.die();
    }

    // Run the default task.
    if let Err(e) = project.run_task(&task_name, args) {
        e.die();
    }

    project.close();
}
