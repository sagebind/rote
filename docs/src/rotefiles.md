# Rotefiles

A *Rotefile* is a Lua script that contains all of the available [tasks](tasks.md) for a project. Typically in the root directory of a project, it is usually a file named `Rotefile` and should be committed with your project's source code.

An example can speak a thousand words, so let's start by dissecting a basic Rotefile:

```lua
default "debug"

desc("Build a debug binary")
task("debug", function()
    exec("cargo", "build")
end)

desc("Build a release binary")
task("release", function()
    exec("cargo", "build", "--release")
end)
```

Here we define two tasks called "debug" and "release".

Now to execute the `debug` task, we can run `rote debug`. Rote will look for the `Rotefile` in the current directory. If the file is in a different directory or has a different name, you can use the `-f` flag to specify a different location:

```sh
$ rote -f my/Rotefile debug
```

See the `default "debug"` at the start of the file? That sets the default task to `debug`. When `rote` is run without a task name, it assumes the "default" task should be run. To run the "debug" task then, we can just run

```sh
$ rote
```

## Alternate file names

In larger projects, it may be necessary to have multiple Rotefiles. You can run tasks contained in other files using the `--file` command line option:

```sh
$ rote --file rotefiles/Rotefile1 my-task
```
