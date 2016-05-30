# Documentation

## Overview


## Installing
The Rote command-line tool is a self-contained binary program that has very minimal system dependencies, so Rote can be run just about anywhere.

```sh
$ git clone https://github.com/coderstephen/rote.git
$ cd rote
$ ./install.sh
```

### Compiling it yourself
If you are developing Rote or need to run it on an unsupported operating system, you can compile the Rote executable yourself. The CLI is written in [Rust](https://www.rust-lang.org), so you will need to set up a Rust compilation environment first. You will also need Cargo, the Rust dependency manager. See the [Rust documentation](https://doc.rust-lang.org/stable/book/installing-rust.html) for more on installing Rust.

Once you have `rustc` and `cargo` installed, you can compile Rote with

```sh
$ cargo build --release
```


## The Rotefile
The *Rotefile* is a Lua script that contains all of the available tasks for a project. Typically in the root directory of a project, it is usually a file named `Rotefile` and should be committed with your project's source code.

A Rotefile is a plain Lua script that has access to some built-in functions provided by Rote for writing tasks. You can write any valid Lua code in your Rotefile, and even include any third-party Lua modules you like.

Below is an example `Rotefile`:

```lua
require "cargo"
default "debug"

task("debug", function()
    cargo.build()
end)

task("release", function()
    cargo.build {
        release = true
    }
end)

task("clean", function()
    cargo.clean()
end)
```

Now to execute the `debug` task, we can run `rote debug`. Rote will look for the `Rotefile` in the current directory. If the file is in a different directory or has a different name, you can use the `-f` flag to specify a different location:

    rote -f my/Rotefile debug

See the `default "debug"` at the start of the file? That sets the default task to `debug`. When `rote` is run without a task name, it assumes the "default" task should be run. To run the "debug" task then, we can just run

    rote

Tasks can also take arguments:

```lua
task("echo", function(message)
    echo message
end)
```

Running the `echo` task like this:

    rote echo "Hello, future!"

Will output:

    Hello, future!


## Writing Tasks
A Rote script is composed of a collection of *tasks*. A task is a series of steps to take in order to perform some sort of commom action.

### Rules
*Rules* are a special, but powerful, type of task. A rule is a task that forms a relation between a pattern of input files and output files in the file system.



## Modules and functions

### Custom modules
While Rote has a "batteries included" attitude about common actions, we can't cover every possible use case.
