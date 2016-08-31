# Writing tasks

Tasks are the basic building blocks of a robust build script. A task represents a single unit of work that can be performed as part of a build process.

To define a task, you can use the aptly-named `task()` function:

```lua
task("release", {"build"}, function()
    exec("tar", "-czvf", "build/")
end)
```

```lua
task("release"):depends("build"):does(function()
    exec("tar", "-czvf", "build/")
end)
```

```lua
cpp.binary "foo" {
    srcs = glob("src/*.cpp"),
    debug = false,
}

task "release" {
    depends = {"build"},
    function()
        exec("tar", "-czvf", "build/")
    end
}
```

The `task()` function takes up to three arguments. The first one is required, and must be a string defining the full name of the task. The second argument is an optional table that lists all of the task's *dependencies*, and the last is a function that contains all of the task's logic.

## Running commands

## Task dependencies

## Parallel execution

By default, Rote will attempt to run your tasks in parallel threads if possible to speed up overall execution time. Unlike some tools, you do not have to write your tasks in any special kind of way to use parallel execution; Rote will happily run any of your normal tasks in parallel for you. This makes parallelization much more useful, but it is not without caveats, so it is only fair for us to tell you about them ahead of time to save you potential headache later.

Rote implements parallel execution using multithreading, which we believe provides the best guarantees for correct and reproducible task execution. Unfortunately, multithreading is hard, so Rote makes some design choices that can affect how you write your tasks in order to ensure complete thread safety and to guarantee that your scripts cannot cause deadlock or race conditions.

The first thing you should know is that Rotefile scripts are initialized once for every thread that Rote creates. Normally, you shouldn't need to run anything substantial outside of a task besides set up some global variables, but this may cause a problem if you are working with the file system, for example. Let's take a look at the following Rotefile:

```lua
global_value = 42
```

If Rote is started with two task threads, the value of `global_value`

The second, and more important thing to know is that _global variables are thread-local_. Updating the value of a global variable from within a task **only updates it for the current thread** and the new value _is not guaranteed_ to be available to the next task to run. Below is an example of how _not_ to write your tasks:

```lua
fs = require "fs"
global_value = 42


task("mutator", function()
    global_value = 32
end)

task("my-task", {"mutator"}, function()
    fs.put(("build/%d.txt"):format(global_value))
end)
```

Tasks should be self-contained and should not expect global values to change after initialization. Shared mutable state tends to cause more problems than it solves, so Rote takes the safe road and keeps all global state in a per-thread basis.



[rust]: https://www.rust-lang.org
